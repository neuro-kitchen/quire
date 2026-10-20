//! Sync on the store (`quire_core::sync`): the change log the triggers of migration 010 keep,
//! what each peer has received, the library's identity, and merging incoming rows ([`merge`]).

mod merge;
mod rows;
mod share;

use quire_core::Result;
use quire_core::sync::{MergeSummary, Row};
use rusqlite::{OptionalExtension, params};

use super::SqliteLibrary;
use crate::error::OrCore;

/// Changes sent at most in one batch (the rest follow in the next).
pub(super) const BATCH: usize = 500;

impl SqliteLibrary<'_> {
    /// The library's identity, shared by every device that syncs it.
    pub fn library_id(&self) -> Result<String> {
        self.conn.query_row("SELECT value FROM library_meta WHERE key = 'library_id'", [], |row| row.get(0)).or_core()
    }

    /// Takes another device's library identity (a new, empty copy joining it).
    pub fn adopt_library_id(&self, id: &str) -> Result<()> {
        self.conn
            .execute("INSERT OR REPLACE INTO library_meta (key, value) VALUES ('library_id', ?1)", [id])
            .or_core()?;
        Ok(())
    }

    /// Whether this copy has anything of its own yet (references or projects) or has synced.
    pub fn has_content(&self) -> Result<bool> {
        let sql = "SELECT EXISTS (SELECT 1 FROM reference) OR EXISTS (SELECT 1 FROM project)
                   OR EXISTS (SELECT 1 FROM sync_peer)";
        self.conn.query_row(sql, [], |row| row.get(0)).or_core()
    }

    /// Up to [`BATCH`] change-log entries after `seq`, as rows (oldest first), and the `seq` of the
    /// last entry (`seq` itself when nothing changed). Rows merged from `to` itself are left out:
    /// it has them.
    pub fn changes_since(&self, seq: i64, to: &str) -> Result<(Vec<Row>, i64)> {
        let mut statement = self
            .conn
            .prepare("SELECT seq, tbl, k1, k2, origin IS ?3 FROM change_log WHERE seq > ?1 ORDER BY seq LIMIT ?2")
            .or_core()?;
        let entries = statement
            .query_map(params![seq, BATCH as i64, to], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, bool>(4)?,
                ))
            })
            .or_core()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .or_core()?;
        let mut last = seq;
        let mut out = Vec::with_capacity(entries.len());
        for (entry, table, k1, k2, echo) in entries {
            last = entry;
            if !echo {
                out.extend(rows::load(self.conn, &table, &k1, &k2)?);
            }
        }
        Ok((out, last))
    }

    /// The highest `seq` received from `device` (0: never synced).
    pub fn received_from(&self, device: &str) -> Result<i64> {
        let sql = "SELECT received FROM sync_peer WHERE device = ?1";
        Ok(self.conn.query_row(sql, [device], |row| row.get(0)).optional().or_core()?.unwrap_or(0))
    }

    /// Records that `device`'s changes up to `seq` are merged here.
    pub fn record_received(&self, device: &str, seq: i64) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO sync_peer (device, received, synced_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT (device) DO UPDATE SET received = max(received, excluded.received), synced_at = excluded.synced_at",
                params![device, seq, quire_model::Timestamp::now().as_millis()],
            )
            .or_core()?;
        Ok(())
    }

    /// The file hashes the live attachments name (each once).
    pub fn attachment_hashes(&self) -> Result<Vec<String>> {
        let mut statement =
            self.conn.prepare("SELECT DISTINCT hash FROM attachment WHERE deleted_at IS NULL").or_core()?;
        let rows = statement.query_map([], |row| row.get(0)).or_core()?;
        rows.collect::<rusqlite::Result<Vec<String>>>().or_core()
    }

    /// Merges rows from the device `from` (`merge` has the rules). The changes it makes are
    /// marked as coming from `from`, so they are not sent back to it, except renames made here.
    pub fn merge(&mut self, rows: Vec<Row>, from: &str) -> Result<MergeSummary> {
        let conn = self.conn;
        conn.execute("INSERT OR REPLACE INTO sync_state (key, value) VALUES ('merging_from', ?1)", [from]).or_core()?;
        let result = merge::merge(conn, self.device, rows);
        conn.execute("DELETE FROM sync_state WHERE key = 'merging_from'", []).or_core()?;
        let (merged, relog) = result?;
        for id in relog {
            conn.execute(
                "UPDATE change_log SET origin = NULL WHERE tbl = 'reference' AND k1 = ?1",
                [crate::repo::codec::id_blob(id)],
            )
            .or_core()?;
        }
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use quire_core::AddRequest;
    use quire_core::library::{projects, references, tags};
    use quire_core::prelude::*;
    use quire_model::{EntryKind, Identifier, Origin, ReferenceData, TagKind, TextField};
    use rusqlite::Connection;

    use super::*;

    fn device(conn: &Connection) -> SqliteLibrary<'_> {
        SqliteLibrary::new(conn, crate::device_id(conn).unwrap())
    }

    fn paper(title: &str, doi: Option<&str>) -> AddRequest {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, title);
        data.creators.push(quire_model::Creator::person(quire_model::CreatorRole::Author, "Vaswani", None));
        data.issued = Some(quire_model::Date::year(2017));
        data.identifiers.extend(doi.map(|d| Identifier::parse(d).unwrap()));
        AddRequest::new(data, Origin::User)
    }

    /// Sends everything `from` has that `to` has not received, as a sync session would.
    fn sync(from: &SqliteLibrary, to: &mut SqliteLibrary) -> MergeSummary {
        let peer = from.device.to_string();
        let mut total = MergeSummary::default();
        loop {
            let since = to.received_from(&peer).unwrap();
            let (rows, last) = from.changes_since(since, &to.device.to_string()).unwrap();
            if last == since {
                break;
            }
            let merged = to.merge(rows, &peer).unwrap();
            total.applied += merged.applied;
            total.kept += merged.kept;
            total.renamed.extend(merged.renamed);
            total.dropped_identifiers.extend(merged.dropped_identifiers);
            to.record_received(&peer, last).unwrap();
        }
        total
    }

    #[test]
    fn two_devices_converge() {
        let (a_conn, b_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut a, mut b) = (device(&a_conn), device(&b_conn));
        assert_ne!(a.device, b.device);
        b.adopt_library_id(&a.library_id().unwrap()).unwrap();

        let key =
            references::add(&mut a, paper("Attention is all you need", Some("10.48550/arXiv.1706.03762"))).unwrap();
        let key = key.reference().cite_key.to_string();
        projects::create(&mut a, "Thesis", None).unwrap();
        projects::link(&mut a, &key, &"thesis".parse().unwrap()).unwrap();
        tags::add(&mut a, &key, "to-read", TagKind::Manual).unwrap();
        tags::add_note(&mut a, Some(&key), "key idea").unwrap();

        let merged = sync(&a, &mut b);
        assert_eq!(merged.kept, 0, "{merged:?}");
        let on_b = b.reference_by_cite_key(&key.parse().unwrap()).unwrap().unwrap();
        assert_eq!(b.projects_of(on_b.id).unwrap().len(), 1);
        assert_eq!(b.tags_of(on_b.id).unwrap().len(), 1);
        assert_eq!(b.notes_of(Some(on_b.id)).unwrap().len(), 1);

        // Back the other way: nothing is sent back to A (B merged those rows from A).
        let back = sync(&b, &mut a);
        assert_eq!((back.applied, back.kept), (0, 0), "{back:?}");

        // A later edit on B wins on A; a deletion travels as a tombstone.
        let mut edited = on_b.clone();
        edited.data.set(TextField::Title, "Attention Is All You Need");
        edited.meta.updated_at = quire_model::Timestamp::from_millis(edited.meta.updated_at.as_millis() + 10);
        edited.meta.updated_by = b.device;
        b.update_reference(&edited).unwrap();
        references::trash(&mut b, &key).unwrap();
        sync(&b, &mut a);
        let on_a = a.reference(on_b.id).unwrap().unwrap();
        assert_eq!(on_a.data.title(), Some("Attention Is All You Need"));
        assert!(on_a.meta.is_deleted());
    }

    #[test]
    fn things_created_apart() {
        let (a_conn, b_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut a, mut b) = (device(&a_conn), device(&b_conn));
        // The same project and tag made on both; two different papers with the same cite key;
        // the same DOI added on both.
        projects::create(&mut a, "Thesis", None).unwrap();
        projects::create(&mut b, "Thesis", None).unwrap();
        let on_a = references::add(&mut a, paper("Attention is all you need", None)).unwrap().reference().clone();
        let on_b = references::add(&mut b, paper("Attention is all you need", None)).unwrap().reference().clone();
        assert_eq!(on_a.cite_key, on_b.cite_key);
        tags::add(&mut a, on_a.cite_key.as_str(), "to-read", TagKind::Manual).unwrap();
        tags::add(&mut b, on_b.cite_key.as_str(), "to-read", TagKind::Manual).unwrap();
        let doi_on_a = references::add(&mut a, paper("Another paper on attention", Some("10.1/same"))).unwrap();
        let doi_on_b = references::add(&mut b, paper("Different study of transformers", Some("10.1/same"))).unwrap();

        let merged = sync(&a, &mut b);
        assert_eq!(merged.renamed.len(), 1, "{merged:?}");
        assert!(merged.dropped_identifiers.is_empty(), "{merged:?}");
        // The same DOI: one reference on B (A's ID is an alias of B's), merged part by part.
        let doi = Identifier::parse("10.1/same").unwrap();
        let same = b.reference_by_identifier(&doi).unwrap().unwrap();
        assert_eq!(same.id, doi_on_b.reference().id);
        assert!(b.reference(doi_on_a.reference().id).unwrap().is_none());
        assert_eq!(projects::tree(&b).unwrap().len(), 1, "one Thesis");
        assert_eq!(b.tags().unwrap().len(), 1, "one to-read tag");
        // Both papers kept, under different keys; tags follow B's to-read tag (alias).
        let (first, second) = (b.reference(on_a.id).unwrap().unwrap(), b.reference(on_b.id).unwrap().unwrap());
        assert_ne!(first.cite_key, second.cite_key);
        assert_eq!(b.tags_of(first.id).unwrap()[0].0.name, "to-read");
        // The rename is a new change: A takes it too, so each paper has one key everywhere.
        sync(&b, &mut a);
        sync(&a, &mut b);
        for id in [on_a.id, on_b.id] {
            assert_eq!(a.reference(id).unwrap().unwrap().cite_key, b.reference(id).unwrap().unwrap().cite_key);
        }
        // Each device keeps its own ID for the DOI's paper; the data is the same on both.
        let (on_a_doi, on_b_doi) =
            (a.reference_by_identifier(&doi).unwrap().unwrap(), b.reference_by_identifier(&doi).unwrap().unwrap());
        assert_eq!(on_a_doi.data, on_b_doi.data);
        assert_eq!(on_a_doi.cite_key, on_b_doi.cite_key);
    }

    #[test]
    fn personal_records_travel() {
        use quire_core::library::{reading, saved};
        let (a_conn, b_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut a, mut b) = (device(&a_conn), device(&b_conn));
        b.adopt_library_id(&a.library_id().unwrap()).unwrap();
        let key =
            references::add(&mut a, paper("Attention is all you need", None)).unwrap().reference().cite_key.to_string();
        reading::set(&mut a, &key, quire_model::ReadingStatus::Read).unwrap();
        saved::save(&mut a, "Queue", "status:unread").unwrap();
        saved::save(&mut b, "queue", "pdf:no").unwrap();

        sync(&a, &mut b);
        let on_b = b.reference_by_cite_key(&key.parse().unwrap()).unwrap().unwrap();
        assert_eq!(b.reading(on_b.id).unwrap().unwrap().status, quire_model::ReadingStatus::Read);
        let names: Vec<String> = b.saved_searches().unwrap().into_iter().map(|s| s.name).collect();
        assert_eq!(names.len(), 2, "both kept, one renamed: {names:?}");
    }

    /// Upserts on synced tables (reading status set twice; a membership changed on the other
    /// device) must not break the change log (migration 013).
    #[test]
    fn upserts_keep_the_change_log() {
        use quire_core::library::reading;
        let (a_conn, b_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut a, mut b) = (device(&a_conn), device(&b_conn));
        b.adopt_library_id(&a.library_id().unwrap()).unwrap();
        let key =
            references::add(&mut a, paper("Attention is all you need", None)).unwrap().reference().cite_key.to_string();
        reading::set(&mut a, &key, quire_model::ReadingStatus::Reading).unwrap();
        reading::set(&mut a, &key, quire_model::ReadingStatus::Read).unwrap();
        projects::create(&mut a, "Thesis", None).unwrap();
        projects::link(&mut a, &key, &"thesis".parse().unwrap()).unwrap();
        sync(&a, &mut b);
        // A unlinks: B's existing membership row is updated by the merge (an upsert).
        projects::unlink(&mut a, &key, &"thesis".parse().unwrap()).unwrap();
        let merged = sync(&a, &mut b);
        assert_eq!(merged.applied, 1, "{merged:?}");
        let on_b = b.reference_by_cite_key(&key.parse().unwrap()).unwrap().unwrap();
        assert!(b.projects_of(on_b.id).unwrap().is_empty());
    }

    #[test]
    fn a_broken_clock_cannot_win() {
        let (a_conn, b_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut a, mut b) = (device(&a_conn), device(&b_conn));
        b.adopt_library_id(&a.library_id().unwrap()).unwrap();
        let added = references::add(&mut a, paper("Attention is all you need", None)).unwrap();
        let mut future = added.reference().clone();
        future.meta.updated_at = quire_model::Timestamp::from_millis(
            quire_model::Timestamp::now().as_millis() + 2 * quire_model::record::MAX_DRIFT_MILLIS,
        );
        let merged = b.merge(vec![Row::Reference(future)], &a.device.to_string()).unwrap();
        assert_eq!((merged.applied, merged.refused), (0, 1));
        // An ordinary stamp from the other device moves this clock past it.
        let mut ahead = added.reference().clone();
        ahead.meta.updated_at = quire_model::Timestamp::from_millis(quire_model::Timestamp::now().as_millis() + 5_000);
        let stamp = ahead.meta.updated_at;
        b.merge(vec![Row::Reference(ahead)], &a.device.to_string()).unwrap();
        assert!(quire_model::Timestamp::now() > stamp);
    }

    #[test]
    fn edits_of_different_fields_meet() {
        let (a_conn, b_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut a, mut b) = (device(&a_conn), device(&b_conn));
        b.adopt_library_id(&a.library_id().unwrap()).unwrap();
        let key = references::add(&mut a, paper("attention is all you need", Some("10.1/x"))).unwrap();
        let id = key.reference().id;
        sync(&a, &mut b);
        // A fixes the title, B adds a volume, neither having seen the other.
        let mut on_a = a.reference(id).unwrap().unwrap();
        on_a.data.set(TextField::Title, "Attention Is All You Need");
        on_a.meta.touch(a.device);
        references::save(&mut a, &mut on_a).unwrap();
        let mut on_b = b.reference(id).unwrap().unwrap();
        on_b.data.set(TextField::Volume, "30");
        on_b.meta.touch(b.device);
        references::save(&mut b, &mut on_b).unwrap();
        sync(&a, &mut b);
        sync(&b, &mut a);
        for side in [&a, &b] {
            let r = side.reference(id).unwrap().unwrap();
            assert_eq!(r.data.title(), Some("Attention Is All You Need"));
            assert_eq!(r.data.get(TextField::Volume), Some("30"));
        }
        // Settled: nothing more to send either way.
        assert_eq!(sync(&a, &mut b).applied + sync(&b, &mut a).applied, 0);
    }
}
