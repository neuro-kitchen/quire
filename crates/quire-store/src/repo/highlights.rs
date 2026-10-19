//! Queries on `highlight`. Removing one keeps the row with `deleted_at` set (tombstone).

use quire_core::Result;
use quire_model::{Highlight, HighlightId, ReferenceId};
use rusqlite::{Connection, OptionalExtension, Row, params};

use super::codec::{id_at, id_blob, meta_at};
use crate::error::OrCore;

const COLUMNS: &str = "id, reference_id, quote, prefix, comment, created_at, updated_at, updated_by, deleted_at";

fn from_row(row: &Row) -> rusqlite::Result<Highlight> {
    Ok(Highlight {
        id: id_at(row, "id")?,
        reference: id_at(row, "reference_id")?,
        quote: row.get("quote")?,
        prefix: row.get("prefix")?,
        comment: row.get("comment")?,
        meta: meta_at(row)?,
    })
}

/// Live highlights of `reference`, oldest first.
pub fn of(conn: &Connection, reference: ReferenceId) -> Result<Vec<Highlight>> {
    let sql =
        format!("SELECT {COLUMNS} FROM highlight WHERE reference_id = ?1 AND deleted_at IS NULL ORDER BY created_at");
    let mut statement = conn.prepare(&sql).or_core()?;
    let rows = statement.query_map([id_blob(reference)], from_row).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

pub fn by_id(conn: &Connection, id: HighlightId) -> Result<Option<Highlight>> {
    let sql = format!("SELECT {COLUMNS} FROM highlight WHERE id = ?1");
    conn.query_row(&sql, [id_blob(id)], from_row).optional().or_core()
}

pub fn insert(conn: &Connection, h: &Highlight) -> Result<()> {
    let sql = format!("INSERT INTO highlight ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)");
    let m = &h.meta;
    conn.execute(
        &sql,
        params![
            id_blob(h.id),
            id_blob(h.reference),
            h.quote,
            h.prefix,
            h.comment,
            m.created_at.as_millis(),
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}

/// Writes a highlight's changes (comment, the reference after a merge, soft delete).
pub fn update(conn: &Connection, h: &Highlight) -> Result<()> {
    let sql = "UPDATE highlight SET reference_id = ?2, comment = ?3, updated_at = ?4, updated_by = ?5, deleted_at = ?6
               WHERE id = ?1";
    let m = &h.meta;
    conn.execute(
        sql,
        params![
            id_blob(h.id),
            id_blob(h.reference),
            h.comment,
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}
