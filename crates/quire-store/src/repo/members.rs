//! Queries on `project_member` (who a shared project is shared with, and their roles).

use quire_core::Result;
use quire_core::ports::MemberStore;
use quire_model::{ProjectId, ProjectMember, Timestamp, UserId};
use rusqlite::{Connection, Row, params};

use super::SqliteLibrary;
use super::codec::{id_at, id_blob, meta_at, parsed_at};
use super::core::{all, first};
use crate::error::OrCore;

const COLUMNS: &str = "project_id, user_id, name, role, granted_by, created_at, updated_at, updated_by, deleted_at";

impl MemberStore for SqliteLibrary<'_> {
    fn members_of(&self, project: ProjectId) -> Result<Vec<ProjectMember>> {
        let sql = format!("SELECT {COLUMNS} FROM project_member WHERE project_id = ?1 ORDER BY name");
        all(self.conn, &sql, [id_blob(project)], from_row)
    }

    fn memberships(&self, user: &UserId) -> Result<Vec<ProjectMember>> {
        all(self.conn, &format!("SELECT {COLUMNS} FROM project_member WHERE user_id = ?1"), [user.as_str()], from_row)
    }

    fn set_member(&mut self, member: &ProjectMember) -> Result<()> {
        set(self.conn, member)
    }
}

fn from_row(row: &Row) -> rusqlite::Result<ProjectMember> {
    Ok(ProjectMember {
        project: id_at(row, "project_id")?,
        user: parsed_at(row, "user_id")?,
        name: row.get("name")?,
        role: parsed_at(row, "role")?,
        granted_by: parsed_at(row, "granted_by")?,
        meta: meta_at(row)?,
    })
}

/// The membership of `user` in `project`, removed or not.
pub fn get(conn: &Connection, project: ProjectId, user: &str) -> Result<Option<ProjectMember>> {
    let sql = format!("SELECT {COLUMNS} FROM project_member WHERE project_id = ?1 AND user_id = ?2");
    first(conn, &sql, params![id_blob(project), user], from_row)
}

/// Inserts or replaces a membership.
pub fn set(conn: &Connection, m: &ProjectMember) -> Result<()> {
    let sql = format!(
        "INSERT INTO project_member ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT (project_id, user_id) DO UPDATE SET name = excluded.name, role = excluded.role,
             granted_by = excluded.granted_by, updated_at = excluded.updated_at, updated_by = excluded.updated_by,
             deleted_at = excluded.deleted_at"
    );
    conn.execute(
        &sql,
        params![
            id_blob(m.project),
            m.user.as_str(),
            m.name,
            m.role.as_str(),
            m.granted_by.as_str(),
            m.meta.created_at.as_millis(),
            m.meta.updated_at.as_millis(),
            id_blob(m.meta.updated_by),
            m.meta.deleted_at.map(Timestamp::as_millis),
        ],
    )
    .or_core()?;
    Ok(())
}
