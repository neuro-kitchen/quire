//! Queries on `project_reference` (a reference in a project). Unlinking keeps the row with
//! `deleted_at` set (tombstone); linking again clears it.

use quire_core::Result;
use quire_model::{DeviceId, ProjectId, ReferenceId};
use rusqlite::Connection;

use super::codec::{id_at, id_blob};
use super::core::{LinkTable, all};

const MEMBERS: LinkTable = LinkTable { table: "project_reference", keys: ["project_id", "reference_id"] };

pub(super) fn link(conn: &Connection, reference: ReferenceId, project: ProjectId, device: DeviceId) -> Result<bool> {
    MEMBERS.set(conn, [id_blob(project), id_blob(reference)], None, device)
}

pub(super) fn unlink(conn: &Connection, reference: ReferenceId, project: ProjectId, device: DeviceId) -> Result<bool> {
    MEMBERS.clear(conn, [id_blob(project), id_blob(reference)], device)
}

impl super::SqliteLibrary<'_> {
    /// Every live reference in a live project, as (reference, project): one query for what each
    /// reference is in and how many each project holds.
    pub fn memberships(&self) -> Result<Vec<(ReferenceId, ProjectId)>> {
        let sql = "SELECT pr.reference_id, pr.project_id FROM project_reference pr
                   JOIN reference r ON r.id = pr.reference_id
                   JOIN project p ON p.id = pr.project_id
                   WHERE pr.deleted_at IS NULL AND r.deleted_at IS NULL AND p.deleted_at IS NULL";
        all(self.conn, sql, [], |row| Ok((id_at(row, "reference_id")?, id_at(row, "project_id")?)))
    }
}

pub(super) fn projects_of(conn: &Connection, reference: ReferenceId) -> Result<Vec<ProjectId>> {
    let sql = "SELECT project_id FROM project_reference
               WHERE reference_id = ?1 AND deleted_at IS NULL
                 AND project_id IN (SELECT id FROM project WHERE deleted_at IS NULL)";
    all(conn, sql, [id_blob(reference)], |row| id_at(row, "project_id"))
}
