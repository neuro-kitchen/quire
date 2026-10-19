//! Queries on `auto_export`.

use quire_core::{AutoExport, Result};
use quire_model::Timestamp;
use rusqlite::{Connection, params};

use super::codec::{id_blob, optional_id_at};
use crate::error::OrCore;

pub fn all(conn: &Connection) -> Result<Vec<AutoExport>> {
    let sql = "SELECT path, project_id, recursive, profile, format, written_at FROM auto_export ORDER BY path";
    let mut statement = conn.prepare(sql).or_core()?;
    let rows = statement
        .query_map([], |row| {
            Ok(AutoExport {
                path: row.get("path")?,
                project: optional_id_at(row, "project_id")?,
                recursive: row.get::<_, i64>("recursive")? != 0,
                profile: row.get("profile")?,
                format: row.get("format")?,
                written_at: row.get::<_, Option<i64>>("written_at")?.map(Timestamp::from_millis),
            })
        })
        .or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

pub fn set(conn: &Connection, export: &AutoExport) -> Result<()> {
    let sql =
        "INSERT OR REPLACE INTO auto_export (path, project_id, recursive, profile, format, created_at, written_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";
    let values = params![
        export.path,
        export.project.map(id_blob),
        i64::from(export.recursive),
        export.profile,
        export.format,
        Timestamp::now().as_millis(),
        export.written_at.map(|t| t.as_millis()),
    ];
    conn.execute(sql, values).or_core()?;
    Ok(())
}

pub fn remove(conn: &Connection, path: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM auto_export WHERE path = ?1", [path]).or_core()? > 0)
}

pub fn mark_written(conn: &Connection, path: &str, at: Timestamp) -> Result<()> {
    conn.execute("UPDATE auto_export SET written_at = ?2 WHERE path = ?1", params![path, at.as_millis()]).or_core()?;
    Ok(())
}
