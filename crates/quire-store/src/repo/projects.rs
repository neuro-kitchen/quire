//! Queries on `project`; [`ProjectStore`] with the members in [`links`](super::links).

use quire_core::Result;
use quire_core::ports::ProjectStore;
use quire_model::{Project, ProjectId, ReferenceId, Slug};
use rusqlite::{Connection, Row, params};

use super::codec::{id_at, id_blob, meta_at, optional_id_at, parsed_at};
use super::core::{all, first};
use super::{SqliteLibrary, links};
use crate::error::OrCore;

impl ProjectStore for SqliteLibrary<'_> {
    fn project(&self, id: ProjectId) -> Result<Option<Project>> {
        by_id(self.conn, id)
    }

    fn child_project(&self, parent: Option<ProjectId>, slug: &Slug) -> Result<Option<Project>> {
        child(self.conn, parent, slug)
    }

    fn child_projects(&self, parent: Option<ProjectId>) -> Result<Vec<Project>> {
        let sql = format!("SELECT {COLUMNS} FROM project WHERE parent_id IS ?1 AND deleted_at IS NULL ORDER BY slug");
        all(self.conn, &sql, [parent.map(id_blob)], from_row)
    }

    fn insert_project(&mut self, project: &Project) -> Result<()> {
        insert(self.conn, project)
    }

    fn update_project(&mut self, project: &Project) -> Result<()> {
        update(self.conn, project)
    }

    fn link(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool> {
        links::link(self.conn, reference, project, self.device)
    }

    fn unlink(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool> {
        links::unlink(self.conn, reference, project, self.device)
    }

    fn projects_of(&self, reference: ReferenceId) -> Result<Vec<ProjectId>> {
        links::projects_of(self.conn, reference)
    }
}

const COLUMNS: &str = "id, parent_id, name, slug, owner, created_at, updated_at, updated_by, deleted_at";

fn from_row(row: &Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: id_at(row, "id")?,
        parent: optional_id_at(row, "parent_id")?,
        name: row.get("name")?,
        slug: parsed_at::<Slug>(row, "slug")?,
        owner: row.get::<_, Option<String>>("owner")?.and_then(|o| o.parse().ok()),
        meta: meta_at(row)?,
    })
}

pub fn by_id(conn: &Connection, id: ProjectId) -> Result<Option<Project>> {
    first(conn, &format!("SELECT {COLUMNS} FROM project WHERE id = ?1"), [id_blob(id)], from_row)
}

pub fn child(conn: &Connection, parent: Option<ProjectId>, slug: &Slug) -> Result<Option<Project>> {
    let sql = format!("SELECT {COLUMNS} FROM project WHERE parent_id IS ?1 AND slug = ?2 AND deleted_at IS NULL");
    first(conn, &sql, params![parent.map(id_blob), slug.as_str()], from_row)
}

pub fn insert(conn: &Connection, project: &Project) -> Result<()> {
    let sql = "INSERT INTO project (id, parent_id, name, slug, created_at, updated_at, updated_by, deleted_at, owner)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";
    let meta = &project.meta;
    conn.execute(
        sql,
        params![
            id_blob(project.id),
            project.parent.map(id_blob),
            project.name,
            project.slug.as_str(),
            meta.created_at.as_millis(),
            meta.updated_at.as_millis(),
            id_blob(meta.updated_by),
            meta.deleted_at.map(|t| t.as_millis()),
            project.owner.as_ref().map(|o| o.as_str()),
        ],
    )
    .or_core()?;
    Ok(())
}

pub fn update(conn: &Connection, project: &Project) -> Result<()> {
    let sql =
        "UPDATE project SET parent_id = ?2, name = ?3, slug = ?4, updated_at = ?5, updated_by = ?6, deleted_at = ?7,
               owner = ?8 WHERE id = ?1";
    let meta = &project.meta;
    conn.execute(
        sql,
        params![
            id_blob(project.id),
            project.parent.map(id_blob),
            project.name,
            project.slug.as_str(),
            meta.updated_at.as_millis(),
            id_blob(meta.updated_by),
            meta.deleted_at.map(|t| t.as_millis()),
            project.owner.as_ref().map(|o| o.as_str()),
        ],
    )
    .or_core()?;
    Ok(())
}
