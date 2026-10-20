//! Queries on pairs of references: `reference_relation` (related; removing keeps a tombstone)
//! and `duplicate_dismissal` (not duplicates; never undone). Each pair is stored once, `a < b`.

use quire_core::Result;
use quire_core::ports::RelationStore;
use quire_model::{ReferenceId, Timestamp};
use rusqlite::params;

use super::SqliteLibrary;
use super::codec::{id_at, id_blob};
use super::core::{LinkTable, all};
use crate::error::OrCore;

const RELATIONS: LinkTable = LinkTable { table: "reference_relation", keys: ["a", "b"] };

impl RelationStore for SqliteLibrary<'_> {
    fn relate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        RELATIONS.set(self.conn, ordered(a, b), None, self.device)
    }

    fn unrelate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        RELATIONS.clear(self.conn, ordered(a, b), self.device)
    }

    fn related(&self, reference: ReferenceId) -> Result<Vec<ReferenceId>> {
        let sql = "SELECT CASE WHEN a = ?1 THEN b ELSE a END AS other FROM reference_relation
                   WHERE (a = ?1 OR b = ?1) AND deleted_at IS NULL";
        all(self.conn, sql, [id_blob(reference)], |row| id_at(row, "other"))
    }

    fn dismiss_duplicate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        let [a, b] = ordered(a, b);
        let sql = "INSERT OR IGNORE INTO duplicate_dismissal (a, b, created_at, updated_by) VALUES (?1, ?2, ?3, ?4)";
        let values = params![a, b, Timestamp::now().as_millis(), id_blob(self.device)];
        Ok(self.conn.execute(sql, values).or_core()? > 0)
    }

    fn dismissed_duplicates(&self) -> Result<Vec<(ReferenceId, ReferenceId)>> {
        all(self.conn, "SELECT a, b FROM duplicate_dismissal", [], |row| Ok((id_at(row, "a")?, id_at(row, "b")?)))
    }
}

/// The pair in stored order (`a < b` by bytes).
fn ordered(a: ReferenceId, b: ReferenceId) -> [[u8; 16]; 2] {
    let (a, b) = (id_blob(a), id_blob(b));
    if a < b { [a, b] } else { [b, a] }
}
