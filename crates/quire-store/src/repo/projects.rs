//! Queries on `project`.

use quire_core::Result;
use quire_model::{Project, ProjectId, Slug};
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::codec::{id_at, id_blob, meta_at, optional_id_at, parsed_at};
use crate::error::OrCore;

const COLUMNS: &str = "id, parent_id, name, slug, created_at, updated_at, updated_by, deleted_at";

fn from_row(row: &Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: id_at(row, "id")?,
        parent: optional_id_at(row, "parent_id")?,
        name: row.get("name")?,
        slug: parsed_at::<Slug>(row, "slug")?,
        meta: meta_at(row)?,
    })
}

pub fn by_id(conn: &Connection, id: ProjectId) -> Result<Option<Project>> {
    let sql = format!("SELECT {COLUMNS} FROM project WHERE id = ?1");
    conn.query_row(&sql, [id_blob(id)], from_row).optional().or_core()
}

pub fn child(conn: &Connection, parent: Option<ProjectId>, slug: &Slug) -> Result<Option<Project>> {
    let sql = format!("SELECT {COLUMNS} FROM project WHERE parent_id IS ?1 AND slug = ?2 AND deleted_at IS NULL");
    conn.query_row(&sql, params![parent.map(id_blob), slug.as_str()], from_row).optional().or_core()
}

pub fn children(conn: &Connection, parent: Option<ProjectId>) -> Result<Vec<Project>> {
    let sql = format!("SELECT {COLUMNS} FROM project WHERE parent_id IS ?1 AND deleted_at IS NULL ORDER BY slug");
    let mut statement = conn.prepare(&sql).or_core()?;
    let rows = statement.query_map([parent.map(id_blob)], from_row).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}

pub fn insert(conn: &Connection, project: &Project) -> Result<()> {
    let sql = "INSERT INTO project (id, parent_id, name, slug, created_at, updated_at, updated_by, deleted_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";
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
        ],
    )
    .or_core()?;
    Ok(())
}

pub fn update(conn: &Connection, project: &Project) -> Result<()> {
    let sql = "UPDATE project SET parent_id = ?2, name = ?3, slug = ?4, updated_at = ?5, updated_by = ?6, deleted_at = ?7
               WHERE id = ?1";
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
        ],
    )
    .or_core()?;
    Ok(())
}
