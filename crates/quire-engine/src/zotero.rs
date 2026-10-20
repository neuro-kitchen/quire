//! Reading a Zotero data folder (`zotero.sqlite` + `storage/`) into [`ZoteroLibrary`]. Zotero
//! keeps its database locked while it runs, so a copy is read (with its journal files). Only
//! "My Library" is read (not group libraries); items and collections in the trash are left out.
//! Table names follow Zotero's schema (`resource/schema/userdata.sql`, version 130); the
//! `…Combined` views (with custom fields and types) are used when they exist.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use quire_io::zotero::{
    AnnotationKind, ZoteroAnnotation, ZoteroAttachment, ZoteroCollection, ZoteroCreator, ZoteroItem, ZoteroLibrary,
    ZoteroNote,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};

use crate::error::{EngineError, Result};

/// The database file in a Zotero data folder.
pub const DATABASE: &str = "zotero.sqlite";

/// Reads the Zotero library in the data folder `dir` (`~/Zotero` by default).
pub fn read(dir: &Path) -> Result<ZoteroLibrary> {
    let database = dir.join(DATABASE);
    if !database.is_file() {
        return Err(EngineError::Zotero(format!("no {DATABASE} in {}", dir.display())));
    }
    let copy = tempfile::tempdir().map_err(EngineError::file(dir))?;
    for suffix in ["", "-wal", "-journal"] {
        let from = dir.join(format!("{DATABASE}{suffix}"));
        if from.is_file() {
            let to = copy.path().join(format!("{DATABASE}{suffix}"));
            std::fs::copy(&from, &to).map_err(EngineError::file(&from))?;
        }
    }
    let conn = Connection::open_with_flags(copy.path().join(DATABASE), OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(zotero_error)?;
    read_connection(&conn).map_err(zotero_error)
}

/// Where an attachment's file is, if it is a file: `storage:<name>` under `storage/<key>/`, an
/// absolute linked path, or `attachments:<relative>` under `base` (Zotero's linked-files folder).
pub fn file_of(dir: &Path, attachment: &ZoteroAttachment, base: Option<&Path>) -> Option<PathBuf> {
    let path = attachment.path.as_deref()?;
    if let Some(name) = path.strip_prefix("storage:") {
        return Some(dir.join("storage").join(&attachment.key).join(name));
    }
    if let Some(relative) = path.strip_prefix("attachments:") {
        return base.map(|base| base.join(relative));
    }
    Some(PathBuf::from(path)).filter(|p| p.is_absolute())
}

fn zotero_error(error: rusqlite::Error) -> EngineError {
    EngineError::Zotero(format!("reading the Zotero database: {error}"))
}

fn read_connection(conn: &Connection) -> rusqlite::Result<ZoteroLibrary> {
    let library: i64 = conn.query_row("SELECT libraryID FROM libraries WHERE type = 'user'", [], |row| row.get(0))?;
    let fields = combined(conn, "fields")?;
    let types = combined(conn, "itemTypes")?;
    let deleted = "SELECT itemID FROM deletedItems";

    // Regular items, keyed by item ID.
    let mut items: Vec<ZoteroItem> = Vec::new();
    let mut index: HashMap<i64, usize> = HashMap::new();
    let sql = format!(
        "SELECT i.itemID, i.key, t.typeName FROM items i JOIN {types} t ON t.itemTypeID = i.itemTypeID
         WHERE i.libraryID = ?1 AND t.typeName NOT IN ('note', 'attachment', 'annotation')
           AND i.itemID NOT IN ({deleted}) ORDER BY i.itemID"
    );
    for row in rows(conn, &sql, [library], |r| Ok((r.get::<_, i64>(0)?, r.get(1)?, r.get(2)?)))? {
        let (id, key, item_type) = row;
        index.insert(id, items.len());
        items.push(ZoteroItem { key, item_type, ..ZoteroItem::default() });
    }
    let keys: HashMap<i64, String> = all_keys(conn, library)?;

    let sql = format!(
        "SELECT d.itemID, f.fieldName, v.value FROM itemData d
         JOIN {fields} f ON f.fieldID = d.fieldID JOIN itemDataValues v ON v.valueID = d.valueID"
    );
    for (id, name, value) in rows(conn, &sql, [], |r| Ok((r.get::<_, i64>(0)?, r.get(1)?, text(r, 2)?)))? {
        if let Some(&i) = index.get(&id) {
            items[i].fields.insert(name, value);
        }
    }

    let sql = "SELECT ic.itemID, ct.creatorType, c.firstName, c.lastName, c.fieldMode FROM itemCreators ic
               JOIN creators c ON c.creatorID = ic.creatorID JOIN creatorTypes ct ON ct.creatorTypeID = ic.creatorTypeID
               ORDER BY ic.itemID, ic.orderIndex";
    for (id, role, first, last, mode) in rows(conn, sql, [], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<i64>>(4)?,
        ))
    })? {
        if let Some(&i) = index.get(&id) {
            items[i].creators.push(ZoteroCreator {
                role,
                first: first.unwrap_or_default(),
                last: last.unwrap_or_default(),
                single_field: mode == Some(1),
            });
        }
    }

    let sql = "SELECT it.itemID, t.name, it.type FROM itemTags it JOIN tags t ON t.tagID = it.tagID";
    for (id, name, kind) in rows(conn, sql, [], |r| Ok((r.get::<_, i64>(0)?, r.get(1)?, r.get::<_, i64>(2)?)))? {
        if let Some(&i) = index.get(&id) {
            items[i].tags.push((name, kind == 1));
        }
    }

    let sql = "SELECT r.itemID, r.object FROM itemRelations r
               JOIN relationPredicates p ON p.predicateID = r.predicateID WHERE p.predicate = 'dc:relation'";
    for (id, object) in rows(conn, sql, [], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        // `http://zotero.org/users/<id or local/…>/items/<KEY>`
        if let (Some(&i), Some(key)) = (index.get(&id), object.rsplit('/').next()) {
            items[i].related.push(key.to_string());
        }
    }

    let collections = collections(conn, library, &keys, &index)?;

    let sql = format!(
        "SELECT n.parentItemID, n.note FROM itemNotes n JOIN items i ON i.itemID = n.itemID
         WHERE i.libraryID = ?1 AND n.itemID NOT IN ({deleted}) ORDER BY n.itemID"
    );
    let notes = rows(conn, &sql, [library], |r| Ok((r.get::<_, Option<i64>>(0)?, text(r, 1)?)))?
        .into_iter()
        .filter(|(parent, html)| !html.trim().is_empty() && parent.is_none_or(|p| index.contains_key(&p)))
        .map(|(parent, html)| ZoteroNote { parent: parent.and_then(|p| keys.get(&p).cloned()), html })
        .collect();

    let sql = format!(
        "SELECT a.itemID, a.parentItemID, a.contentType, a.path FROM itemAttachments a JOIN items i ON i.itemID = a.itemID
         WHERE i.libraryID = ?1 AND a.itemID NOT IN ({deleted}) ORDER BY a.itemID"
    );
    let attachments: Vec<ZoteroAttachment> =
        rows(conn, &sql, [library], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?, r.get(2)?, r.get(3)?)))?
            .into_iter()
            .filter_map(|(id, parent, content_type, path)| {
                Some(ZoteroAttachment {
                    key: keys.get(&id)?.clone(),
                    parent: parent.and_then(|p| keys.get(&p).cloned()),
                    content_type,
                    path,
                })
            })
            .collect();

    let annotations = if exists(conn, "itemAnnotations")? {
        let sql = format!(
            "SELECT a.parentItemID, a.type, a.text, a.comment, a.pageLabel FROM itemAnnotations a
             JOIN items i ON i.itemID = a.itemID WHERE i.libraryID = ?1 AND a.itemID NOT IN ({deleted})
             ORDER BY a.parentItemID, a.sortIndex"
        );
        rows(conn, &sql, [library], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .into_iter()
        .filter_map(|(attachment, kind, text, comment, page_label)| {
            Some(ZoteroAnnotation {
                attachment: keys.get(&attachment)?.clone(),
                kind: AnnotationKind::from_code(kind),
                text,
                comment,
                page_label,
            })
        })
        .collect()
    } else {
        Vec::new()
    };

    Ok(ZoteroLibrary { items, collections, notes, attachments, annotations })
}

/// Collections not in the trash, parents first, with the keys of their (live, regular) items.
fn collections(
    conn: &Connection,
    library: i64,
    keys: &HashMap<i64, String>,
    index: &HashMap<i64, usize>,
) -> rusqlite::Result<Vec<ZoteroCollection>> {
    let deleted =
        if exists(conn, "deletedCollections")? { "SELECT collectionID FROM deletedCollections" } else { "SELECT NULL" };
    let sql = format!(
        "SELECT collectionID, key, collectionName, parentCollectionID FROM collections
         WHERE libraryID = ?1 AND collectionID NOT IN ({deleted})"
    );
    let found = rows(conn, &sql, [library], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<i64>>(3)?))
    })?;
    let key_of: HashMap<i64, String> = found.iter().map(|(id, key, _, _)| (*id, key.clone())).collect();
    let mut members: HashMap<i64, Vec<String>> = HashMap::new();
    let sql = "SELECT collectionID, itemID FROM collectionItems ORDER BY collectionID, orderIndex";
    for (collection, item) in rows(conn, sql, [], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))? {
        if index.contains_key(&item)
            && let Some(key) = keys.get(&item)
        {
            members.entry(collection).or_default().push(key.clone());
        }
    }
    // Parents first; a collection whose parent is in the trash is left out with it.
    let mut out: Vec<ZoteroCollection> = Vec::new();
    let mut pending = found;
    while !pending.is_empty() {
        let before = pending.len();
        let mut waiting = Vec::new();
        for (id, key, name, parent) in pending {
            match parent {
                Some(p) if !out.iter().any(|c| Some(&c.key) == key_of.get(&p)) => waiting.push((id, key, name, parent)),
                _ => out.push(ZoteroCollection {
                    key,
                    name,
                    parent: parent.and_then(|p| key_of.get(&p).cloned()),
                    items: members.remove(&id).unwrap_or_default(),
                }),
            }
        }
        if waiting.len() == before {
            break;
        }
        pending = waiting;
    }
    Ok(out)
}

/// Every item's key (notes and attachments included), by item ID.
fn all_keys(conn: &Connection, library: i64) -> rusqlite::Result<HashMap<i64, String>> {
    let sql = "SELECT itemID, key FROM items WHERE libraryID = ?1";
    Ok(rows(conn, sql, [library], |r| Ok((r.get(0)?, r.get(1)?)))?.into_iter().collect())
}

/// `<name>Combined` if Zotero created it (custom fields or types), else `<name>`.
fn combined(conn: &Connection, name: &str) -> rusqlite::Result<String> {
    let view = format!("{name}Combined");
    Ok(if exists(conn, &view)? { view } else { name.to_string() })
}

fn exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    conn.query_row("SELECT 1 FROM sqlite_master WHERE name = ?1", [name], |_| Ok(())).optional().map(|r| r.is_some())
}

/// A value stored as text or as a number (Zotero's `itemDataValues.value` has no type).
fn text(row: &rusqlite::Row, index: usize) -> rusqlite::Result<String> {
    use rusqlite::types::ValueRef;
    Ok(match row.get_ref(index)? {
        ValueRef::Text(bytes) | ValueRef::Blob(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        ValueRef::Integer(n) => n.to_string(),
        ValueRef::Real(x) => x.to_string(),
        ValueRef::Null => String::new(),
    })
}

fn rows<T>(
    conn: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
    read: impl FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
) -> rusqlite::Result<Vec<T>> {
    let mut statement = conn.prepare(sql)?;
    statement.query_map(params, read)?.collect()
}

#[cfg(test)]
pub(crate) mod fixture {
    //! A small Zotero data folder for tests (`tests/fixtures/zotero.sql`).

    use std::path::Path;

    use rusqlite::Connection;

    const SCHEMA: &str = include_str!("../tests/fixtures/zotero.sql");

    /// Writes `zotero.sqlite` and `storage/PDF44444/kucsko.pdf` (`pdf` bytes) into `dir`.
    pub fn write(dir: &Path, pdf: &[u8]) {
        let conn = Connection::open(dir.join(super::DATABASE)).unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        let storage = dir.join("storage").join("PDF44444");
        std::fs::create_dir_all(&storage).unwrap();
        std::fs::write(storage.join("kucsko.pdf"), pdf).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_my_library() {
        let dir = tempfile::tempdir().unwrap();
        fixture::write(dir.path(), b"%PDF-1.4");
        let library = read(dir.path()).unwrap();

        let keys: Vec<&str> = library.items.iter().map(|i| i.key.as_str()).collect();
        assert_eq!(keys, ["AAAA1111", "BBBB2222"], "trash and group items left out");
        let first = &library.items[0];
        assert_eq!(first.fields["DOI"], "10.1038/nature12373");
        assert_eq!(first.creators[0].last, "Kucsko");
        assert_eq!(first.tags, [("to-read".to_string(), false), ("Thermometry".to_string(), true)]);
        assert_eq!(first.related, ["BBBB2222"]);
        assert!(library.items[1].creators[0].single_field);

        let collections: Vec<(&str, Option<&str>, usize)> =
            library.collections.iter().map(|c| (c.name.as_str(), c.parent.as_deref(), c.items.len())).collect();
        assert_eq!(collections, [("Thesis", None, 1), ("Chapter 2", Some("COLL0001"), 1)]);

        assert_eq!(library.notes.len(), 2);
        assert_eq!(library.notes[0].parent.as_deref(), Some("AAAA1111"));
        assert_eq!(library.notes[1].parent, None);
        let attachment = &library.attachments[0];
        assert_eq!(attachment.parent.as_deref(), Some("AAAA1111"));
        let file = file_of(dir.path(), attachment, None).unwrap();
        assert_eq!(std::fs::read(file).unwrap(), b"%PDF-1.4");
        assert_eq!(library.annotations.len(), 2);
        assert_eq!(library.annotations[0].kind, AnnotationKind::Highlight);
        assert_eq!(library.annotations[0].attachment, "PDF44444");
    }

    #[test]
    fn missing_folder_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(read(dir.path()), Err(EngineError::Zotero(_))));
    }
}
