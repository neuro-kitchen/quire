//! Merging rows from another device. A row is written when it is new here or newer than what is
//! here ([`newer`]: the later `updated_at`, ties by device ID), so every device ends with the
//! same state whatever the order of syncs. What two devices created apart but must stay unique:
//!
//! - one cite key for two papers: the one with the larger ID gets a free key (on every device the
//!   same one), stamped as a new change so every device takes it;
//! - an identifier another reference has here: left off the incoming copy (a possible duplicate
//!   the duplicates view offers to merge), not stamped, so the other device keeps it;
//! - a project with the same name under the same parent, a tag with the same name: the same
//!   thing; their ID is recorded as an alias of ours (`sync_alias`) and rows naming it follow;
//! - the same file attached to the same reference: kept once.
//!
//! Rows go in an order that satisfies foreign keys: references, projects (parents first), tags,
//! then the rows pointing at them. A row whose reference or project is still missing is left for
//! a later sync (it stays in the sender's log).

use quire_core::Result;
use quire_core::citekey::free_key;
use quire_core::sync::{Distinct, Membership, Merged, Relation, Row, Stamp, Tagging, newer};
use quire_model::{Attachment, DeviceId, Highlight, Id, IdKind, Note, Project, Reference, Tag, Timestamp};
use rusqlite::{Connection, OptionalExtension, params};

use crate::error::OrCore;
use crate::repo::codec::id_blob;
use crate::repo::{attachments, highlights, projects, references, tags};

/// Merges `rows`; also returns the references this merge renamed (their change must reach every
/// device, the sender too).
pub fn merge(conn: &Connection, device: DeviceId, rows: Vec<Row>) -> Result<(Merged, Vec<quire_model::ReferenceId>)> {
    let mut merged = Merged::default();
    let mut renamed = Vec::new();
    let (mut projects_in, mut tags_in, mut rest) = (Vec::new(), Vec::new(), Vec::new());
    for row in rows {
        match row {
            Row::Reference(r) => reference(conn, device, r, &mut merged, &mut renamed)?,
            Row::Project(p) => projects_in.push(p),
            Row::Tag(t) => tags_in.push(t),
            other => rest.push(other),
        }
    }
    // Parents first: repeat while some project's parent arrived in an earlier pass.
    while !projects_in.is_empty() {
        let before = projects_in.len();
        let mut waiting = Vec::new();
        for project in projects_in {
            let parent = project.parent.map(|p| alias(conn, p)).transpose()?;
            match parent {
                Some(parent) if projects::by_id(conn, parent)?.is_none() => waiting.push(project),
                _ => self::project(conn, project, &mut merged)?,
            }
        }
        if waiting.len() == before {
            merged.kept += waiting.len(); // parent not here yet: next sync
            break;
        }
        projects_in = waiting;
    }
    for tag in tags_in {
        self::tag(conn, tag, &mut merged)?;
    }
    rest.sort_by_key(order);
    for row in rest {
        match row {
            Row::Membership(m) => membership(conn, m, &mut merged)?,
            Row::Tagging(t) => tagging(conn, t, &mut merged)?,
            Row::Note(n) => note(conn, n, &mut merged)?,
            Row::Highlight(h) => highlight(conn, h, &mut merged)?,
            Row::Relation(r) => relation(conn, r, &mut merged)?,
            Row::Attachment(a) => attachment(conn, a, &mut merged)?,
            Row::Distinct(d) => distinct(conn, d, &mut merged)?,
            Row::Reference(_) | Row::Project(_) | Row::Tag(_) => unreachable!("applied above"),
        }
    }
    Ok((merged, renamed))
}

fn order(row: &Row) -> u8 {
    match row {
        Row::Membership(_) => 0,
        Row::Tagging(_) => 1,
        Row::Note(_) => 2,
        Row::Highlight(_) => 3,
        Row::Relation(_) => 4,
        Row::Attachment(_) => 5,
        _ => 6,
    }
}

/// Our ID for `id` (another device's project or tag that is the same as ours), else `id`.
fn alias<K: IdKind>(conn: &Connection, id: Id<K>) -> Result<Id<K>> {
    let local: Option<[u8; 16]> = conn
        .query_row("SELECT local FROM sync_alias WHERE remote = ?1", [id_blob(id)], |row| row.get(0))
        .optional()
        .or_core()?;
    Ok(match local {
        Some(bytes) => Id::from_bytes(bytes)?,
        None => id,
    })
}

fn set_alias<K: IdKind>(conn: &Connection, remote: Id<K>, local: Id<K>) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO sync_alias (remote, local) VALUES (?1, ?2)",
        params![id_blob(remote), id_blob(local)],
    )
    .or_core()?;
    Ok(())
}

fn exists(conn: &Connection, sql: &str, id: [u8; 16]) -> Result<bool> {
    Ok(conn.query_row(sql, [id], |_| Ok(())).optional().or_core()?.is_some())
}

fn reference_exists(conn: &Connection, id: quire_model::ReferenceId) -> Result<bool> {
    exists(conn, "SELECT 1 FROM reference WHERE id = ?1", id_blob(id))
}

fn reference(
    conn: &Connection,
    device: DeviceId,
    mut r: Reference,
    merged: &mut Merged,
    renamed: &mut Vec<quire_model::ReferenceId>,
) -> Result<()> {
    let local = references::by_id(conn, r.id)?;
    if let Some(local) = &local
        && !newer((r.meta.updated_at, r.meta.updated_by), (local.meta.updated_at, local.meta.updated_by))
    {
        merged.kept += 1;
        return Ok(());
    }
    if let Some(mut other) = references::by_cite_key(conn, &r.cite_key)?
        && other.id != r.id
    {
        // Two papers, one key: the one with the larger ID takes a new key, on every device
        // (renaming whichever arrives would make devices rename each other back and forth).
        let local_loses = other.id > r.id;
        let loser = if local_loses { &mut other } else { &mut r };
        let old = loser.cite_key.to_string();
        loser.cite_key = free_key(&old, |key| references::cite_key_taken(conn, key))?;
        loser.meta.touch(device);
        merged.renamed.push((old, loser.cite_key.to_string()));
        renamed.push(loser.id);
        if local_loses {
            references::update(conn, &other)?;
        }
    }
    let mut kept_ids = Vec::new();
    for identifier in std::mem::take(&mut r.data.identifiers) {
        match references::by_identifier(conn, &identifier)? {
            Some(other) if other.id != r.id => {
                merged.dropped_identifiers.push((r.cite_key.to_string(), identifier.to_string()))
            }
            _ => kept_ids.push(identifier),
        }
    }
    r.data.identifiers = kept_ids;
    match local {
        Some(_) => references::update(conn, &r)?,
        None => references::insert(conn, &r, None)?,
    }
    merged.applied += 1;
    Ok(())
}

fn project(conn: &Connection, mut p: Project, merged: &mut Merged) -> Result<()> {
    p.id = alias(conn, p.id)?;
    p.parent = p.parent.map(|parent| alias(conn, parent)).transpose()?;
    let local = projects::by_id(conn, p.id)?;
    if local.is_none()
        && !p.meta.is_deleted()
        && let Some(same) = projects::child(conn, p.parent, &p.slug)?
    {
        // Created on both devices: one project.
        set_alias(conn, p.id, same.id)?;
        merged.kept += 1;
        return Ok(());
    }
    match local {
        Some(local)
            if !newer((p.meta.updated_at, p.meta.updated_by), (local.meta.updated_at, local.meta.updated_by)) =>
        {
            merged.kept += 1
        }
        Some(_) => {
            projects::update(conn, &p)?;
            merged.applied += 1;
        }
        None => {
            projects::insert(conn, &p)?;
            merged.applied += 1;
        }
    }
    Ok(())
}

fn tag(conn: &Connection, mut t: Tag, merged: &mut Merged) -> Result<()> {
    t.id = alias(conn, t.id)?;
    let local = super::rows::tag(conn, t.id)?;
    if local.is_none()
        && !t.meta.is_deleted()
        && let Some(same) = tags::by_key(conn, &t.key())?
    {
        set_alias(conn, t.id, same.id)?;
        merged.kept += 1;
        return Ok(());
    }
    if let Some(local) = &local
        && !newer((t.meta.updated_at, t.meta.updated_by), (local.meta.updated_at, local.meta.updated_by))
    {
        merged.kept += 1;
        return Ok(());
    }
    // Its number key may be another tag's here: the tag comes without it.
    if let Some(position) = t.position {
        let taken: Option<[u8; 16]> = conn
            .query_row(
                "SELECT id FROM tag WHERE position = ?1 AND deleted_at IS NULL AND id != ?2",
                params![position, id_blob(t.id)],
                |row| row.get(0),
            )
            .optional()
            .or_core()?;
        if taken.is_some() {
            t.position = None;
        }
    }
    match local {
        Some(_) => tags::update(conn, &t)?,
        None => tags::insert(conn, &t)?,
    }
    merged.applied += 1;
    Ok(())
}

/// The stored stamp of a link row, if there is one.
fn stored_stamp(conn: &Connection, sql: &str, keys: [[u8; 16]; 2]) -> Result<Option<Stamp>> {
    conn.query_row(sql, keys, super::rows::stamp).optional().or_core()
}

fn wins(incoming: &Stamp, stored: Option<Stamp>) -> bool {
    stored.is_none_or(|s| newer((incoming.updated_at, incoming.updated_by), (s.updated_at, s.updated_by)))
}

fn membership(conn: &Connection, mut m: Membership, merged: &mut Merged) -> Result<()> {
    m.project = alias(conn, m.project)?;
    if projects::by_id(conn, m.project)?.is_none() || !reference_exists(conn, m.reference)? {
        merged.kept += 1;
        return Ok(());
    }
    let keys = [id_blob(m.project), id_blob(m.reference)];
    let sql = "SELECT created_at, updated_at, updated_by, deleted_at FROM project_reference
               WHERE project_id = ?1 AND reference_id = ?2";
    if !wins(&m.stamp, stored_stamp(conn, sql, keys)?) {
        merged.kept += 1;
        return Ok(());
    }
    let s = &m.stamp;
    conn.execute(
        "INSERT INTO project_reference (project_id, reference_id, created_at, updated_at, updated_by, deleted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (project_id, reference_id) DO UPDATE SET
             updated_at = excluded.updated_at, updated_by = excluded.updated_by, deleted_at = excluded.deleted_at",
        params![
            keys[0],
            keys[1],
            s.created_at.as_millis(),
            s.updated_at.as_millis(),
            id_blob(s.updated_by),
            millis(s.deleted_at)
        ],
    )
    .or_core()?;
    merged.applied += 1;
    Ok(())
}

fn tagging(conn: &Connection, mut t: Tagging, merged: &mut Merged) -> Result<()> {
    t.tag = alias(conn, t.tag)?;
    if super::rows::tag(conn, t.tag)?.is_none() || !reference_exists(conn, t.reference)? {
        merged.kept += 1;
        return Ok(());
    }
    let keys = [id_blob(t.reference), id_blob(t.tag)];
    let sql = "SELECT created_at, updated_at, updated_by, deleted_at FROM reference_tag WHERE reference_id = ?1 AND tag_id = ?2";
    if !wins(&t.stamp, stored_stamp(conn, sql, keys)?) {
        merged.kept += 1;
        return Ok(());
    }
    let s = &t.stamp;
    conn.execute(
        "INSERT INTO reference_tag (reference_id, tag_id, kind, created_at, updated_at, updated_by, deleted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (reference_id, tag_id) DO UPDATE SET kind = excluded.kind,
             updated_at = excluded.updated_at, updated_by = excluded.updated_by, deleted_at = excluded.deleted_at",
        params![
            keys[0],
            keys[1],
            t.kind.as_str(),
            s.created_at.as_millis(),
            s.updated_at.as_millis(),
            id_blob(s.updated_by),
            millis(s.deleted_at)
        ],
    )
    .or_core()?;
    merged.applied += 1;
    Ok(())
}

fn relation(conn: &Connection, r: Relation, merged: &mut Merged) -> Result<()> {
    if !reference_exists(conn, r.a)? || !reference_exists(conn, r.b)? {
        merged.kept += 1;
        return Ok(());
    }
    let keys = [id_blob(r.a), id_blob(r.b)];
    let sql = "SELECT created_at, updated_at, updated_by, deleted_at FROM reference_relation WHERE a = ?1 AND b = ?2";
    if !wins(&r.stamp, stored_stamp(conn, sql, keys)?) {
        merged.kept += 1;
        return Ok(());
    }
    let s = &r.stamp;
    conn.execute(
        "INSERT INTO reference_relation (a, b, created_at, updated_at, updated_by, deleted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (a, b) DO UPDATE SET
             updated_at = excluded.updated_at, updated_by = excluded.updated_by, deleted_at = excluded.deleted_at",
        params![keys[0], keys[1], s.created_at.as_millis(), s.updated_at.as_millis(), id_blob(s.updated_by), millis(s.deleted_at)],
    )
    .or_core()?;
    merged.applied += 1;
    Ok(())
}

fn note(conn: &Connection, n: Note, merged: &mut Merged) -> Result<()> {
    if let Some(reference) = n.reference
        && !reference_exists(conn, reference)?
    {
        merged.kept += 1;
        return Ok(());
    }
    match tags::note(conn, n.id)? {
        Some(local)
            if !newer((n.meta.updated_at, n.meta.updated_by), (local.meta.updated_at, local.meta.updated_by)) =>
        {
            merged.kept += 1
        }
        Some(_) => {
            tags::update_note(conn, &n)?;
            merged.applied += 1;
        }
        None => {
            tags::insert_note(conn, &n)?;
            merged.applied += 1;
        }
    }
    Ok(())
}

fn highlight(conn: &Connection, h: Highlight, merged: &mut Merged) -> Result<()> {
    if !reference_exists(conn, h.reference)? {
        merged.kept += 1;
        return Ok(());
    }
    match highlights::by_id(conn, h.id)? {
        Some(local)
            if !newer((h.meta.updated_at, h.meta.updated_by), (local.meta.updated_at, local.meta.updated_by)) =>
        {
            merged.kept += 1
        }
        Some(_) => {
            highlights::update(conn, &h)?;
            merged.applied += 1;
        }
        None => {
            highlights::insert(conn, &h)?;
            merged.applied += 1;
        }
    }
    Ok(())
}

fn attachment(conn: &Connection, a: Attachment, merged: &mut Merged) -> Result<()> {
    if !reference_exists(conn, a.reference)? {
        merged.kept += 1;
        return Ok(());
    }
    let local = attachments::by_id(conn, a.id)?;
    if local.is_none() && !a.meta.is_deleted() {
        // The same file on the same reference, attached on both devices: kept once.
        let same = attachments::of(conn, a.reference)?.into_iter().any(|l| l.hash == a.hash);
        if same {
            merged.kept += 1;
            return Ok(());
        }
    }
    match local {
        Some(local)
            if !newer((a.meta.updated_at, a.meta.updated_by), (local.meta.updated_at, local.meta.updated_by)) =>
        {
            merged.kept += 1
        }
        Some(_) => {
            attachments::update(conn, &a)?;
            merged.applied += 1;
        }
        None => {
            attachments::insert(conn, &a)?;
            merged.applied += 1;
        }
    }
    Ok(())
}

fn distinct(conn: &Connection, d: Distinct, merged: &mut Merged) -> Result<()> {
    if !reference_exists(conn, d.a)? || !reference_exists(conn, d.b)? {
        merged.kept += 1;
        return Ok(());
    }
    let changed = conn
        .execute(
            "INSERT OR IGNORE INTO duplicate_dismissal (a, b, created_at, updated_by) VALUES (?1, ?2, ?3, ?4)",
            params![id_blob(d.a), id_blob(d.b), d.created_at.as_millis(), id_blob(d.by)],
        )
        .or_core()?;
    if changed > 0 {
        merged.applied += 1
    } else {
        merged.kept += 1
    }
    Ok(())
}

fn millis(t: Option<Timestamp>) -> Option<i64> {
    t.map(Timestamp::as_millis)
}
