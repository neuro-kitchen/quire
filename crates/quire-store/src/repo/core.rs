//! What the query modules share: reading many rows, and link rows that are soft deleted (a
//! reference in a project, a tag on a reference, two related references).

use quire_core::Result;
use quire_model::{DeviceId, Timestamp};
use rusqlite::types::ToSql;
use rusqlite::{Connection, OptionalExtension, Params, Row, params_from_iter};

use super::codec::id_blob;
use crate::error::OrCore;

/// Every row `sql` returns, read with `read`.
pub(super) fn all<T>(
    conn: &Connection,
    sql: &str,
    params: impl Params,
    read: impl FnMut(&Row) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    let mut statement = conn.prepare(sql).or_core()?;
    let rows = statement.query_map(params, read).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

/// The first row `sql` returns, if any.
pub(super) fn first<T>(
    conn: &Connection,
    sql: &str,
    params: impl Params,
    read: impl FnOnce(&Row) -> rusqlite::Result<T>,
) -> Result<Option<T>> {
    conn.query_row(sql, params, read).optional().or_core()
}

/// A table of links between two records, keyed by two ID columns, with the `created_at`,
/// `updated_at`, `updated_by`, `deleted_at` columns. Removing a link keeps the row as a tombstone
/// (so sync carries the removal); linking again revives it.
pub(super) struct LinkTable {
    pub table: &'static str,
    pub keys: [&'static str; 2],
}

impl LinkTable {
    /// Links `keys`, also writing `extra` (column, value) when the row is created or revived;
    /// `false` if the link was already there.
    pub(super) fn set(
        &self,
        conn: &Connection,
        keys: [[u8; 16]; 2],
        extra: Option<(&str, &dyn ToSql)>,
        device: DeviceId,
    ) -> Result<bool> {
        let [k1, k2] = self.keys;
        let table = self.table;
        let state: Option<Option<i64>> =
            first(conn, &format!("SELECT deleted_at FROM {table} WHERE {k1} = ?1 AND {k2} = ?2"), keys, |row| {
                row.get(0)
            })?;
        let now = Timestamp::now().as_millis();
        let device = id_blob(device);
        let mut values: Vec<&dyn ToSql> = vec![&keys[0], &keys[1], &now, &device];
        let sql = match (state, extra) {
            (Some(None), _) => return Ok(false),
            (Some(Some(_)), None) => format!(
                "UPDATE {table} SET deleted_at = NULL, updated_at = ?3, updated_by = ?4 WHERE {k1} = ?1 AND {k2} = ?2"
            ),
            (Some(Some(_)), Some((column, value))) => {
                values.push(value);
                format!(
                    "UPDATE {table} SET {column} = ?5, deleted_at = NULL, updated_at = ?3, updated_by = ?4
                     WHERE {k1} = ?1 AND {k2} = ?2"
                )
            }
            (None, None) => format!(
                "INSERT INTO {table} ({k1}, {k2}, created_at, updated_at, updated_by) VALUES (?1, ?2, ?3, ?3, ?4)"
            ),
            (None, Some((column, value))) => {
                values.push(value);
                format!(
                    "INSERT INTO {table} ({k1}, {k2}, {column}, created_at, updated_at, updated_by)
                     VALUES (?1, ?2, ?5, ?3, ?3, ?4)"
                )
            }
        };
        conn.execute(&sql, params_from_iter(values)).or_core()?;
        Ok(true)
    }

    /// Removes the link (keeps a tombstone); `false` if there was none.
    pub(super) fn clear(&self, conn: &Connection, keys: [[u8; 16]; 2], device: DeviceId) -> Result<bool> {
        let [k1, k2] = self.keys;
        let sql = format!(
            "UPDATE {} SET deleted_at = ?3, updated_at = ?3, updated_by = ?4
             WHERE {k1} = ?1 AND {k2} = ?2 AND deleted_at IS NULL",
            self.table
        );
        let now = Timestamp::now().as_millis();
        let changed = conn.execute(&sql, rusqlite::params![keys[0], keys[1], now, id_blob(device)]).or_core()?;
        Ok(changed > 0)
    }
}
