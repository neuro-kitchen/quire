//! Queries on `tag` and `reference_tag` (a tag on a reference; removing it keeps a tombstone).

use quire_core::Result;
use quire_core::ports::TagStore;
use quire_model::{ReferenceId, Tag, TagId, TagKind};
use rusqlite::{Connection, Row, params};

use super::SqliteLibrary;
use super::codec::{id_at, id_blob, meta_at, parsed_at};
use super::core::{LinkTable, all, first};
use crate::error::OrCore;

const TAGGINGS: LinkTable = LinkTable { table: "reference_tag", keys: ["reference_id", "tag_id"] };

impl TagStore for SqliteLibrary<'_> {
    fn tags(&self) -> Result<Vec<Tag>> {
        let sql =
            format!("SELECT {TAG_COLUMNS} FROM tag WHERE deleted_at IS NULL ORDER BY position IS NULL, position, key");
        all(self.conn, &sql, [], tag_from_row)
    }

    fn tag_by_key(&self, key: &str) -> Result<Option<Tag>> {
        by_key(self.conn, key)
    }

    fn insert_tag(&mut self, tag: &Tag) -> Result<()> {
        insert(self.conn, tag)
    }

    fn update_tag(&mut self, tag: &Tag) -> Result<()> {
        update(self.conn, tag)
    }

    fn tag_reference(&mut self, reference: ReferenceId, tag: TagId, kind: TagKind) -> Result<bool> {
        TAGGINGS.set(self.conn, [id_blob(reference), id_blob(tag)], Some(("kind", &kind.as_str())), self.device)
    }

    fn untag_reference(&mut self, reference: ReferenceId, tag: TagId) -> Result<bool> {
        TAGGINGS.clear(self.conn, [id_blob(reference), id_blob(tag)], self.device)
    }

    fn tags_of(&self, reference: ReferenceId) -> Result<Vec<(Tag, TagKind)>> {
        let sql =
            "SELECT t.id, t.name, t.color, t.position, t.created_at, t.updated_at, t.updated_by, t.deleted_at, rt.kind
             FROM reference_tag rt JOIN tag t ON t.id = rt.tag_id
             WHERE rt.reference_id = ?1 AND rt.deleted_at IS NULL AND t.deleted_at IS NULL
             ORDER BY t.position IS NULL, t.position, t.key";
        all(self.conn, sql, [id_blob(reference)], |row| Ok((tag_from_row(row)?, parsed_at(row, "kind")?)))
    }

    fn tagged(&self, tag: TagId) -> Result<Vec<ReferenceId>> {
        let sql = "SELECT reference_id FROM reference_tag WHERE tag_id = ?1 AND deleted_at IS NULL";
        all(self.conn, sql, [id_blob(tag)], |row| id_at(row, "reference_id"))
    }
}

const TAG_COLUMNS: &str = "id, name, color, position, created_at, updated_at, updated_by, deleted_at";

fn tag_from_row(row: &Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: id_at(row, "id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        position: row.get::<_, Option<i64>>("position")?.map(|p| p as u8),
        meta: meta_at(row)?,
    })
}

/// By ID, including a tag in the trash.
pub fn by_id(conn: &Connection, id: TagId) -> Result<Option<Tag>> {
    first(conn, &format!("SELECT {TAG_COLUMNS} FROM tag WHERE id = ?1"), [id_blob(id)], tag_from_row)
}

pub fn by_key(conn: &Connection, key: &str) -> Result<Option<Tag>> {
    first(conn, &format!("SELECT {TAG_COLUMNS} FROM tag WHERE key = ?1 AND deleted_at IS NULL"), [key], tag_from_row)
}

pub fn insert(conn: &Connection, tag: &Tag) -> Result<()> {
    let sql = "INSERT INTO tag (id, name, key, color, position, created_at, updated_at, updated_by, deleted_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";
    let m = &tag.meta;
    conn.execute(
        sql,
        params![
            id_blob(tag.id),
            tag.name,
            tag.key(),
            tag.color,
            tag.position,
            m.created_at.as_millis(),
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}

pub fn update(conn: &Connection, tag: &Tag) -> Result<()> {
    let sql = "UPDATE tag SET name = ?2, key = ?3, color = ?4, position = ?5, updated_at = ?6, updated_by = ?7, deleted_at = ?8
               WHERE id = ?1";
    let m = &tag.meta;
    conn.execute(
        sql,
        params![
            id_blob(tag.id),
            tag.name,
            tag.key(),
            tag.color,
            tag.position,
            m.updated_at.as_millis(),
            id_blob(m.updated_by),
            m.deleted_at.map(|t| t.as_millis())
        ],
    )
    .or_core()?;
    Ok(())
}
