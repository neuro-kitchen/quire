//! Queries on `reference`, `reference_identifier` and `raw_record`.

use quire_core::{RawRecord, ReferenceFilter, Result};
use quire_model::{CiteKey, Identifier, Reference, ReferenceId};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};

use super::codec::{id_at, id_blob, json_at, meta_at, parsed_at};
use crate::error::OrCore;

const COLUMNS: &str = "id, cite_key, data, provenance, created_at, updated_at, updated_by, deleted_at";

fn from_row(row: &Row) -> rusqlite::Result<Reference> {
    Ok(Reference {
        id: id_at(row, "id")?,
        cite_key: parsed_at::<CiteKey>(row, "cite_key")?,
        data: json_at(row, "data")?,
        provenance: json_at(row, "provenance")?,
        meta: meta_at(row)?,
    })
}

fn one(conn: &Connection, filter: &str, params: impl rusqlite::Params) -> Result<Option<Reference>> {
    let sql = format!("SELECT {COLUMNS} FROM reference WHERE {filter}");
    conn.query_row(&sql, params, from_row).optional().or_core()
}

pub fn by_id(conn: &Connection, id: ReferenceId) -> Result<Option<Reference>> {
    one(conn, "id = ?1", [id_blob(id)])
}

pub fn by_cite_key(conn: &Connection, key: &CiteKey) -> Result<Option<Reference>> {
    one(conn, "cite_key = ?1 AND deleted_at IS NULL", [key.as_str()])
}

pub fn by_identifier(conn: &Connection, identifier: &Identifier) -> Result<Option<Reference>> {
    let filter = "id = (SELECT reference_id FROM reference_identifier WHERE kind = ?1 AND key = ?2)";
    one(conn, filter, params![identifier.kind().as_str(), identifier.key()])
}

pub fn cite_key_taken(conn: &Connection, key: &str) -> Result<bool> {
    conn.query_row("SELECT EXISTS (SELECT 1 FROM reference WHERE cite_key = ?1)", [key], |row| row.get(0)).or_core()
}

pub fn insert(conn: &Connection, reference: &Reference, raw: Option<&RawRecord>) -> Result<()> {
    let sql = "INSERT INTO reference (id, cite_key, kind, title, year, data, provenance, created_at, updated_at, updated_by, deleted_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
    let meta = &reference.meta;
    conn.execute(
        sql,
        params![
            id_blob(reference.id),
            reference.cite_key.as_str(),
            reference.data.kind.as_str(),
            reference.data.title(),
            reference.data.issued.map(|d| d.get_year()),
            serde_json::to_string(&reference.data).or_core()?,
            serde_json::to_string(&reference.provenance).or_core()?,
            meta.created_at.as_millis(),
            meta.updated_at.as_millis(),
            id_blob(meta.updated_by),
            meta.deleted_at.map(|t| t.as_millis()),
        ],
    )
    .or_core()?;
    insert_identifiers(conn, reference)?;
    if let Some(raw) = raw {
        let sql = "INSERT INTO raw_record (reference_id, source, format, body, received_at) VALUES (?1, ?2, ?3, ?4, ?5)";
        conn.execute(sql, params![id_blob(reference.id), raw.source, raw.format, raw.body, raw.received_at.as_millis()])
            .or_core()?;
    }
    Ok(())
}

pub fn update(conn: &Connection, reference: &Reference) -> Result<()> {
    let sql = "UPDATE reference SET cite_key = ?2, kind = ?3, title = ?4, year = ?5, data = ?6, provenance = ?7,
               updated_at = ?8, updated_by = ?9, deleted_at = ?10 WHERE id = ?1";
    let meta = &reference.meta;
    conn.execute(
        sql,
        params![
            id_blob(reference.id),
            reference.cite_key.as_str(),
            reference.data.kind.as_str(),
            reference.data.title(),
            reference.data.issued.map(|d| d.get_year()),
            serde_json::to_string(&reference.data).or_core()?,
            serde_json::to_string(&reference.provenance).or_core()?,
            meta.updated_at.as_millis(),
            id_blob(meta.updated_by),
            meta.deleted_at.map(|t| t.as_millis()),
        ],
    )
    .or_core()?;
    conn.execute("DELETE FROM reference_identifier WHERE reference_id = ?1", [id_blob(reference.id)]).or_core()?;
    insert_identifiers(conn, reference)
}

fn insert_identifiers(conn: &Connection, reference: &Reference) -> Result<()> {
    let sql = "INSERT INTO reference_identifier (kind, key, value, reference_id) VALUES (?1, ?2, ?3, ?4)";
    for identifier in &reference.data.identifiers {
        let value = identifier.to_string();
        conn.execute(sql, params![identifier.kind().as_str(), identifier.key(), value, id_blob(reference.id)])
            .or_core()?;
    }
    Ok(())
}

pub fn list(conn: &Connection, filter: &ReferenceFilter) -> Result<Vec<Reference>> {
    let mut conditions = Vec::new();
    let mut values: Vec<[u8; 16]> = Vec::new();
    if !filter.include_deleted {
        conditions.push("deleted_at IS NULL".to_string());
    }
    if let Some(projects) = &filter.projects {
        let marks = vec!["?"; projects.len()].join(", ");
        conditions.push(format!(
            "id IN (SELECT reference_id FROM project_reference WHERE deleted_at IS NULL AND project_id IN ({marks}))"
        ));
        values.extend(projects.iter().map(|p| id_blob(*p)));
    }
    let filter = if conditions.is_empty() { "1".to_string() } else { conditions.join(" AND ") };
    let sql = format!("SELECT {COLUMNS} FROM reference WHERE {filter} ORDER BY cite_key");
    let mut statement = conn.prepare(&sql).or_core()?;
    let rows = statement.query_map(params_from_iter(values), from_row).or_core()?;
    rows.collect::<rusqlite::Result<Vec<_>>>().or_core()
}
