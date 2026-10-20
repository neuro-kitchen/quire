//! Queries on `reading` (a paper's reading status) and `saved_search`.

use quire_core::Result;
use quire_core::ports::{ReadingStore, SavedSearchStore};
use quire_model::{Reading, ReferenceId, SavedSearch, SavedSearchId, Timestamp};
use rusqlite::{Connection, Row, params};

use super::SqliteLibrary;
use super::codec::{id_at, id_blob, meta_at, parsed_at};
use super::core::{all, first};
use crate::error::OrCore;

impl ReadingStore for SqliteLibrary<'_> {
    fn reading(&self, reference: ReferenceId) -> Result<Option<Reading>> {
        reading_by_id(self.conn, reference)
    }

    fn set_reading(&mut self, reading: &Reading) -> Result<()> {
        set_reading(self.conn, reading)
    }

    fn readings(&self) -> Result<Vec<Reading>> {
        all(self.conn, &format!("SELECT {READING_COLUMNS} FROM reading"), [], reading_from_row)
    }
}

impl SavedSearchStore for SqliteLibrary<'_> {
    fn saved_searches(&self) -> Result<Vec<SavedSearch>> {
        let sql = format!("SELECT {SEARCH_COLUMNS} FROM saved_search WHERE deleted_at IS NULL ORDER BY key");
        all(self.conn, &sql, [], search_from_row)
    }

    fn saved_search_by_key(&self, key: &str) -> Result<Option<SavedSearch>> {
        let sql = format!("SELECT {SEARCH_COLUMNS} FROM saved_search WHERE key = ?1 AND deleted_at IS NULL");
        first(self.conn, &sql, [key], search_from_row)
    }

    fn saved_search(&self, id: SavedSearchId) -> Result<Option<SavedSearch>> {
        search_by_id(self.conn, id)
    }

    fn insert_saved_search(&mut self, search: &SavedSearch) -> Result<()> {
        insert_search(self.conn, search)
    }

    fn update_saved_search(&mut self, search: &SavedSearch) -> Result<()> {
        update_search(self.conn, search)
    }
}

const READING_COLUMNS: &str =
    "reference_id, status, started_at, finished_at, created_at, updated_at, updated_by, deleted_at";

fn reading_from_row(row: &Row) -> rusqlite::Result<Reading> {
    let time = |column: &str| row.get::<_, Option<i64>>(column).map(|t| t.map(Timestamp::from_millis));
    Ok(Reading {
        reference: id_at(row, "reference_id")?,
        status: parsed_at(row, "status")?,
        started_at: time("started_at")?,
        finished_at: time("finished_at")?,
        meta: meta_at(row)?,
    })
}

pub(super) fn reading_by_id(conn: &Connection, reference: ReferenceId) -> Result<Option<Reading>> {
    let sql = format!("SELECT {READING_COLUMNS} FROM reading WHERE reference_id = ?1");
    first(conn, &sql, [id_blob(reference)], reading_from_row)
}

/// Inserts or replaces the reading status of its reference.
pub(super) fn set_reading(conn: &Connection, r: &Reading) -> Result<()> {
    let sql = format!(
        "INSERT INTO reading ({READING_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (reference_id) DO UPDATE SET status = excluded.status, started_at = excluded.started_at,
             finished_at = excluded.finished_at, updated_at = excluded.updated_at, updated_by = excluded.updated_by,
             deleted_at = excluded.deleted_at"
    );
    let m = &r.meta;
    conn.execute(
        &sql,
        params![
            id_blob(r.reference),
            r.status.as_str(),
            r.started_at.map(Timestamp::as_millis),
            r.finished_at.map(Timestamp::as_millis),
            m.created_at.as_millis(),
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(Timestamp::as_millis),
        ],
    )
    .or_core()?;
    Ok(())
}

const SEARCH_COLUMNS: &str = "id, name, query, created_at, updated_at, updated_by, deleted_at";

fn search_from_row(row: &Row) -> rusqlite::Result<SavedSearch> {
    Ok(SavedSearch { id: id_at(row, "id")?, name: row.get("name")?, query: row.get("query")?, meta: meta_at(row)? })
}

pub(super) fn search_by_id(conn: &Connection, id: SavedSearchId) -> Result<Option<SavedSearch>> {
    first(conn, &format!("SELECT {SEARCH_COLUMNS} FROM saved_search WHERE id = ?1"), [id_blob(id)], search_from_row)
}

pub(super) fn insert_search(conn: &Connection, s: &SavedSearch) -> Result<()> {
    let sql = "INSERT INTO saved_search (id, name, key, query, created_at, updated_at, updated_by, deleted_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";
    let m = &s.meta;
    conn.execute(
        sql,
        params![
            id_blob(s.id),
            s.name,
            s.key(),
            s.query,
            m.created_at.as_millis(),
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(Timestamp::as_millis),
        ],
    )
    .or_core()?;
    Ok(())
}

pub(super) fn update_search(conn: &Connection, s: &SavedSearch) -> Result<()> {
    let sql =
        "UPDATE saved_search SET name = ?2, key = ?3, query = ?4, updated_at = ?5, updated_by = ?6, deleted_at = ?7
               WHERE id = ?1";
    let m = &s.meta;
    conn.execute(
        sql,
        params![
            id_blob(s.id),
            s.name,
            s.key(),
            s.query,
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(Timestamp::as_millis),
        ],
    )
    .or_core()?;
    Ok(())
}
