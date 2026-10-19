//! Queries on `project_reference`. Unlinking keeps the row with `deleted_at` set (tombstone);
//! linking again clears it.

use quire_core::Result;
use quire_model::{DeviceId, ProjectId, ReferenceId, Timestamp};
use rusqlite::{Connection, OptionalExtension, params};

use super::codec::{id_at, id_blob};
use crate::error::OrCore;

pub fn link(conn: &Connection, reference: ReferenceId, project: ProjectId, device: DeviceId) -> Result<bool> {
    let now = Timestamp::now().as_millis();
    let state: Option<Option<i64>> = conn
        .query_row(
            "SELECT deleted_at FROM project_reference WHERE project_id = ?1 AND reference_id = ?2",
            [id_blob(project), id_blob(reference)],
            |row| row.get(0),
        )
        .optional()
        .or_core()?;
    match state {
        Some(None) => Ok(false),
        Some(Some(_)) => {
            let sql = "UPDATE project_reference SET deleted_at = NULL, updated_at = ?3, updated_by = ?4
                       WHERE project_id = ?1 AND reference_id = ?2";
            conn.execute(sql, params![id_blob(project), id_blob(reference), now, id_blob(device)]).or_core()?;
            Ok(true)
        }
        None => {
            let sql = "INSERT INTO project_reference (project_id, reference_id, created_at, updated_at, updated_by)
                       VALUES (?1, ?2, ?3, ?3, ?4)";
            conn.execute(sql, params![id_blob(project), id_blob(reference), now, id_blob(device)]).or_core()?;
            Ok(true)
        }
    }
}

pub fn unlink(conn: &Connection, reference: ReferenceId, project: ProjectId, device: DeviceId) -> Result<bool> {
    let now = Timestamp::now().as_millis();
    let sql = "UPDATE project_reference SET deleted_at = ?3, updated_at = ?3, updated_by = ?4
               WHERE project_id = ?1 AND reference_id = ?2 AND deleted_at IS NULL";
    let changed = conn.execute(sql, params![id_blob(project), id_blob(reference), now, id_blob(device)]).or_core()?;
    Ok(changed > 0)
}

pub fn projects_of(conn: &Connection, reference: ReferenceId) -> Result<Vec<ProjectId>> {
    let sql = "SELECT project_id FROM project_reference
               WHERE reference_id = ?1 AND deleted_at IS NULL
                 AND project_id IN (SELECT id FROM project WHERE deleted_at IS NULL)";
    let mut statement = conn.prepare(sql).or_core()?;
    let rows = statement.query_map([id_blob(reference)], |row| id_at(row, "project_id")).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}
