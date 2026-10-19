//! Queries on `tag`, `reference_tag`, `note` and `reference_relation`.

use quire_core::Result;
use quire_model::{Note, NoteId, ReferenceId, Tag, TagId, TagKind, Timestamp};
use rusqlite::{Connection, OptionalExtension, Row, params};

use super::codec::{id_at, id_blob, meta_at, optional_id_at, parsed_at};
use crate::error::OrCore;

const TAG_COLUMNS: &str = "id, name, color, position, created_at, updated_at, updated_by, deleted_at";

fn tag_from_row(row: &Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: id_at(row, "id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        position: row.get::<_, Option<i64>>("position")?.map(|p| p as u8),
        meta: meta_at(row)?,
    })
}

pub fn all(conn: &Connection) -> Result<Vec<Tag>> {
    let sql =
        format!("SELECT {TAG_COLUMNS} FROM tag WHERE deleted_at IS NULL ORDER BY position IS NULL, position, key");
    let mut statement = conn.prepare(&sql).or_core()?;
    let rows = statement.query_map([], tag_from_row).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

pub fn by_key(conn: &Connection, key: &str) -> Result<Option<Tag>> {
    let sql = format!("SELECT {TAG_COLUMNS} FROM tag WHERE key = ?1 AND deleted_at IS NULL");
    conn.query_row(&sql, [key], tag_from_row).optional().or_core()
}

pub fn insert(conn: &Connection, tag: &Tag) -> Result<()> {
    let sql = "INSERT INTO tag (id, name, key, color, position, created_at, updated_at, updated_by, deleted_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";
    let m = &tag.meta;
    conn.execute(
        sql,
        params![
            id_blob(tag.id),
            tag.name,
            tag.key(),
            tag.color,
            tag.position,
            m.created_at.as_millis(),
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}

pub fn update(conn: &Connection, tag: &Tag) -> Result<()> {
    let sql = "UPDATE tag SET name = ?2, key = ?3, color = ?4, position = ?5, updated_at = ?6, updated_by = ?7, deleted_at = ?8
               WHERE id = ?1";
    let m = &tag.meta;
    conn.execute(
        sql,
        params![
            id_blob(tag.id),
            tag.name,
            tag.key(),
            tag.color,
            tag.position,
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}

pub fn tag_reference(
    conn: &Connection,
    reference: ReferenceId,
    tag: TagId,
    kind: TagKind,
    device: quire_model::DeviceId,
) -> Result<bool> {
    let now = Timestamp::now().as_millis();
    let state: Option<Option<i64>> = conn
        .query_row(
            "SELECT deleted_at FROM reference_tag WHERE reference_id = ?1 AND tag_id = ?2",
            [id_blob(reference), id_blob(tag)],
            |row| row.get(0),
        )
        .optional()
        .or_core()?;
    let sql = match state {
        Some(None) => return Ok(false),
        Some(Some(_)) => {
            "UPDATE reference_tag SET kind = ?3, deleted_at = NULL, updated_at = ?4, updated_by = ?5 WHERE reference_id = ?1 AND tag_id = ?2"
        }
        None => {
            "INSERT INTO reference_tag (reference_id, tag_id, kind, created_at, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4, ?4, ?5)"
        }
    };
    conn.execute(sql, params![id_blob(reference), id_blob(tag), kind.as_str(), now, id_blob(device)]).or_core()?;
    Ok(true)
}

pub fn untag_reference(
    conn: &Connection,
    reference: ReferenceId,
    tag: TagId,
    device: quire_model::DeviceId,
) -> Result<bool> {
    let now = Timestamp::now().as_millis();
    let sql = "UPDATE reference_tag SET deleted_at = ?3, updated_at = ?3, updated_by = ?4
               WHERE reference_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL";
    Ok(conn.execute(sql, params![id_blob(reference), id_blob(tag), now, id_blob(device)]).or_core()? > 0)
}

pub fn tags_of(conn: &Connection, reference: ReferenceId) -> Result<Vec<(Tag, TagKind)>> {
    let sql =
        "SELECT t.id, t.name, t.color, t.position, t.created_at, t.updated_at, t.updated_by, t.deleted_at, rt.kind
               FROM reference_tag rt JOIN tag t ON t.id = rt.tag_id
               WHERE rt.reference_id = ?1 AND rt.deleted_at IS NULL AND t.deleted_at IS NULL
               ORDER BY t.position IS NULL, t.position, t.key";
    let mut statement = conn.prepare(sql).or_core()?;
    let rows =
        statement.query_map([id_blob(reference)], |row| Ok((tag_from_row(row)?, parsed_at(row, "kind")?))).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

pub fn tagged(conn: &Connection, tag: TagId) -> Result<Vec<ReferenceId>> {
    let mut statement =
        conn.prepare("SELECT reference_id FROM reference_tag WHERE tag_id = ?1 AND deleted_at IS NULL").or_core()?;
    let rows = statement.query_map([id_blob(tag)], |row| id_at(row, "reference_id")).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

const NOTE_COLUMNS: &str = "id, reference_id, body, created_at, updated_at, updated_by, deleted_at";

fn note_from_row(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: id_at(row, "id")?,
        reference: optional_id_at(row, "reference_id")?,
        body: row.get("body")?,
        meta: meta_at(row)?,
    })
}

pub fn notes_of(conn: &Connection, reference: Option<ReferenceId>) -> Result<Vec<Note>> {
    let sql =
        format!("SELECT {NOTE_COLUMNS} FROM note WHERE reference_id IS ?1 AND deleted_at IS NULL ORDER BY created_at");
    let mut statement = conn.prepare(&sql).or_core()?;
    let rows = statement.query_map([reference.map(id_blob)], note_from_row).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

pub fn note(conn: &Connection, id: NoteId) -> Result<Option<Note>> {
    let sql = format!("SELECT {NOTE_COLUMNS} FROM note WHERE id = ?1");
    conn.query_row(&sql, [id_blob(id)], note_from_row).optional().or_core()
}

pub fn insert_note(conn: &Connection, note: &Note) -> Result<()> {
    let sql = format!("INSERT INTO note ({NOTE_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)");
    let m = &note.meta;
    conn.execute(
        &sql,
        params![
            id_blob(note.id),
            note.reference.map(id_blob),
            note.body,
            m.created_at.as_millis(),
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}

pub fn update_note(conn: &Connection, note: &Note) -> Result<()> {
    let sql =
        "UPDATE note SET reference_id = ?2, body = ?3, updated_at = ?4, updated_by = ?5, deleted_at = ?6 WHERE id = ?1";
    let m = &note.meta;
    conn.execute(
        sql,
        params![
            id_blob(note.id),
            note.reference.map(id_blob),
            note.body,
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}

pub fn note_counts(conn: &Connection) -> Result<Vec<(ReferenceId, usize)>> {
    let sql = "SELECT reference_id, COUNT(*) FROM note WHERE reference_id IS NOT NULL AND deleted_at IS NULL GROUP BY reference_id";
    let mut statement = conn.prepare(sql).or_core()?;
    let rows =
        statement.query_map([], |row| Ok((id_at(row, "reference_id")?, row.get::<_, i64>(1)? as usize))).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

/// The pair in stored order (`a < b` by bytes).
fn ordered(a: ReferenceId, b: ReferenceId) -> ([u8; 16], [u8; 16]) {
    let (a, b) = (id_blob(a), id_blob(b));
    if a < b { (a, b) } else { (b, a) }
}

pub fn relate(conn: &Connection, a: ReferenceId, b: ReferenceId, device: quire_model::DeviceId) -> Result<bool> {
    let (a, b) = ordered(a, b);
    let now = Timestamp::now().as_millis();
    let state: Option<Option<i64>> = conn
        .query_row("SELECT deleted_at FROM reference_relation WHERE a = ?1 AND b = ?2", [a, b], |row| row.get(0))
        .optional()
        .or_core()?;
    let sql = match state {
        Some(None) => return Ok(false),
        Some(Some(_)) => {
            "UPDATE reference_relation SET deleted_at = NULL, updated_at = ?3, updated_by = ?4 WHERE a = ?1 AND b = ?2"
        }
        None => "INSERT INTO reference_relation (a, b, created_at, updated_at, updated_by) VALUES (?1, ?2, ?3, ?3, ?4)",
    };
    conn.execute(sql, params![a, b, now, id_blob(device)]).or_core()?;
    Ok(true)
}

pub fn unrelate(conn: &Connection, a: ReferenceId, b: ReferenceId, device: quire_model::DeviceId) -> Result<bool> {
    let (a, b) = ordered(a, b);
    let now = Timestamp::now().as_millis();
    let sql = "UPDATE reference_relation SET deleted_at = ?3, updated_at = ?3, updated_by = ?4 WHERE a = ?1 AND b = ?2 AND deleted_at IS NULL";
    Ok(conn.execute(sql, params![a, b, now, id_blob(device)]).or_core()? > 0)
}

pub fn related(conn: &Connection, reference: ReferenceId) -> Result<Vec<ReferenceId>> {
    let sql = "SELECT CASE WHEN a = ?1 THEN b ELSE a END AS other FROM reference_relation
               WHERE (a = ?1 OR b = ?1) AND deleted_at IS NULL";
    let mut statement = conn.prepare(sql).or_core()?;
    let rows = statement.query_map([id_blob(reference)], |row| id_at(row, "other")).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

/// Records that `a` and `b` are not duplicates; `false` if already recorded.
pub fn dismiss_duplicate(
    conn: &Connection,
    a: ReferenceId,
    b: ReferenceId,
    device: quire_model::DeviceId,
) -> Result<bool> {
    let (a, b) = ordered(a, b);
    let sql = "INSERT OR IGNORE INTO duplicate_dismissal (a, b, created_at, updated_by) VALUES (?1, ?2, ?3, ?4)";
    Ok(conn.execute(sql, params![a, b, Timestamp::now().as_millis(), id_blob(device)]).or_core()? > 0)
}

pub fn dismissed_duplicates(conn: &Connection) -> Result<Vec<(ReferenceId, ReferenceId)>> {
    let mut statement = conn.prepare("SELECT a, b FROM duplicate_dismissal").or_core()?;
    let rows = statement.query_map([], |row| Ok((id_at(row, "a")?, id_at(row, "b")?))).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}
