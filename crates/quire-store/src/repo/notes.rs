//! Queries on `note` (on a reference, or standalone).

use quire_core::Result;
use quire_core::ports::NoteStore;
use quire_model::{Note, NoteId, ReferenceId};
use rusqlite::{Connection, Row, params};

use super::SqliteLibrary;
use super::codec::{id_at, id_blob, meta_at, optional_id_at};
use super::core::{all, first};
use crate::error::OrCore;

impl NoteStore for SqliteLibrary<'_> {
    fn notes_of(&self, reference: Option<ReferenceId>) -> Result<Vec<Note>> {
        let sql =
            format!("SELECT {COLUMNS} FROM note WHERE reference_id IS ?1 AND deleted_at IS NULL ORDER BY created_at");
        all(self.conn, &sql, [reference.map(id_blob)], from_row)
    }

    fn note(&self, id: NoteId) -> Result<Option<Note>> {
        by_id(self.conn, id)
    }

    fn insert_note(&mut self, note: &Note) -> Result<()> {
        insert(self.conn, note)
    }

    fn update_note(&mut self, note: &Note) -> Result<()> {
        update(self.conn, note)
    }

    fn note_counts(&self) -> Result<Vec<(ReferenceId, usize)>> {
        let sql = "SELECT reference_id, COUNT(*) FROM note WHERE reference_id IS NOT NULL AND deleted_at IS NULL
                   GROUP BY reference_id";
        all(self.conn, sql, [], |row| Ok((id_at(row, "reference_id")?, row.get::<_, i64>(1)? as usize)))
    }
}

const COLUMNS: &str = "id, reference_id, body, created_at, updated_at, updated_by, deleted_at";

fn from_row(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: id_at(row, "id")?,
        reference: optional_id_at(row, "reference_id")?,
        body: row.get("body")?,
        meta: meta_at(row)?,
    })
}

pub fn by_id(conn: &Connection, id: NoteId) -> Result<Option<Note>> {
    first(conn, &format!("SELECT {COLUMNS} FROM note WHERE id = ?1"), [id_blob(id)], from_row)
}

pub fn insert(conn: &Connection, note: &Note) -> Result<()> {
    let sql = format!("INSERT INTO note ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)");
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

pub fn update(conn: &Connection, note: &Note) -> Result<()> {
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
