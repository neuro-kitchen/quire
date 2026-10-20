//! Queries on `alert` and `alert_seen`.

use quire_core::Result;
use quire_core::ports::AlertStore;
use quire_model::{Alert, AlertId, Timestamp};
use rusqlite::{Row, params};

use super::SqliteLibrary;
use super::codec::{id_at, id_blob, meta_at, parsed_at};
use super::core::{all, first};
use crate::error::OrCore;

const COLUMNS: &str = "id, name, source, query, checked_at, created_at, updated_at, updated_by, deleted_at";

impl AlertStore for SqliteLibrary<'_> {
    fn alerts(&self) -> Result<Vec<Alert>> {
        all(self.conn, &format!("SELECT {COLUMNS} FROM alert WHERE deleted_at IS NULL ORDER BY key"), [], from_row)
    }

    fn alert_by_key(&self, key: &str) -> Result<Option<Alert>> {
        let sql = format!("SELECT {COLUMNS} FROM alert WHERE key = ?1 AND deleted_at IS NULL");
        first(self.conn, &sql, [key], from_row)
    }

    fn insert_alert(&mut self, a: &Alert) -> Result<()> {
        let sql = "INSERT INTO alert (id, name, key, source, query, checked_at, created_at, updated_at, updated_by, deleted_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";
        let m = &a.meta;
        self.conn
            .execute(
                sql,
                params![
                    id_blob(a.id),
                    a.name,
                    a.key(),
                    a.source.as_str(),
                    a.query,
                    a.checked_at.map(Timestamp::as_millis),
                    m.created_at.as_millis(),
                    m.updated_at.as_millis(),
                    id_blob(m.updated_by),
                    m.deleted_at.map(Timestamp::as_millis),
                ],
            )
            .or_core()?;
        Ok(())
    }

    fn update_alert(&mut self, a: &Alert) -> Result<()> {
        let sql = "UPDATE alert SET name = ?2, key = ?3, source = ?4, query = ?5, checked_at = ?6, updated_at = ?7,
                   updated_by = ?8, deleted_at = ?9 WHERE id = ?1";
        let m = &a.meta;
        self.conn
            .execute(
                sql,
                params![
                    id_blob(a.id),
                    a.name,
                    a.key(),
                    a.source.as_str(),
                    a.query,
                    a.checked_at.map(Timestamp::as_millis),
                    m.updated_at.as_millis(),
                    id_blob(m.updated_by),
                    m.deleted_at.map(Timestamp::as_millis),
                ],
            )
            .or_core()?;
        Ok(())
    }

    fn seen(&self, alert: AlertId, works: &[String]) -> Result<Vec<String>> {
        let mut found = Vec::new();
        for work in works {
            let sql = "SELECT 1 FROM alert_seen WHERE alert_id = ?1 AND work = ?2";
            if first(self.conn, sql, params![id_blob(alert), work], |_| Ok(()))?.is_some() {
                found.push(work.clone());
            }
        }
        Ok(found)
    }

    fn mark_seen(&mut self, alert: AlertId, works: &[String]) -> Result<()> {
        let now = Timestamp::now().as_millis();
        for work in works {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO alert_seen (alert_id, work, seen_at) VALUES (?1, ?2, ?3)",
                    params![id_blob(alert), work, now],
                )
                .or_core()?;
        }
        Ok(())
    }
}

fn from_row(row: &Row) -> rusqlite::Result<Alert> {
    Ok(Alert {
        id: id_at(row, "id")?,
        name: row.get("name")?,
        source: parsed_at(row, "source")?,
        query: row.get("query")?,
        checked_at: row.get::<_, Option<i64>>("checked_at")?.map(Timestamp::from_millis),
        meta: meta_at(row)?,
    })
}
