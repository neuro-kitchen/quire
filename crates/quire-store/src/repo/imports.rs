//! Queries on `imported` (another program's key → the reference made from it).

use quire_core::Result;
use quire_core::ports::ImportStore;
use quire_model::{ReferenceId, Timestamp};
use rusqlite::params;

use super::SqliteLibrary;
use super::codec::{id_at, id_blob};
use super::core::first;
use crate::error::OrCore;

impl ImportStore for SqliteLibrary<'_> {
    fn imported(&self, source: &str, key: &str) -> Result<Option<ReferenceId>> {
        let sql = "SELECT reference_id FROM imported WHERE source = ?1 AND key = ?2";
        first(self.conn, sql, [source, key], |row| id_at(row, "reference_id"))
    }

    fn record_import(&mut self, source: &str, key: &str, reference: ReferenceId) -> Result<()> {
        let sql = "INSERT OR REPLACE INTO imported (source, key, reference_id, imported_at) VALUES (?1, ?2, ?3, ?4)";
        self.conn.execute(sql, params![source, key, id_blob(reference), Timestamp::now().as_millis()]).or_core()?;
        Ok(())
    }
}
