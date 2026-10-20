//! Queries on `attachment`.

use quire_core::Result;
use quire_core::ports::AttachmentStore;
use quire_model::{Attachment, AttachmentId, PdfAttempt, PdfKind, ReferenceId, Timestamp};
use rusqlite::{Connection, Row, params};

use super::SqliteLibrary;
use super::codec::{id_at, id_blob, meta_at, parsed_at};
use super::core::{all, first};
use crate::error::OrCore;

impl AttachmentStore for SqliteLibrary<'_> {
    fn attachments_of(&self, reference: ReferenceId) -> Result<Vec<Attachment>> {
        of(self.conn, reference)
    }

    fn attachment(&self, id: AttachmentId) -> Result<Option<Attachment>> {
        by_id(self.conn, id)
    }

    fn insert_attachment(&mut self, attachment: &Attachment) -> Result<()> {
        insert(self.conn, attachment)
    }

    fn update_attachment(&mut self, attachment: &Attachment) -> Result<()> {
        update(self.conn, attachment)
    }

    fn references_with_attachments(&self) -> Result<Vec<ReferenceId>> {
        let sql = "SELECT DISTINCT reference_id FROM attachment WHERE deleted_at IS NULL";
        all(self.conn, sql, [], |row| id_at(row, "reference_id"))
    }

    fn set_pdf_attempt(&mut self, reference: ReferenceId, attempt: Option<(PdfAttempt, &str)>) -> Result<()> {
        match attempt {
            Some((outcome, detail)) => {
                let sql = "INSERT OR REPLACE INTO pdf_attempt (reference_id, outcome, detail, attempted_at)
                           VALUES (?1, ?2, ?3, ?4)";
                let values = params![id_blob(reference), outcome.as_str(), detail, Timestamp::now().as_millis()];
                self.conn.execute(sql, values).or_core()?;
            }
            None => {
                self.conn.execute("DELETE FROM pdf_attempt WHERE reference_id = ?1", [id_blob(reference)]).or_core()?;
            }
        }
        Ok(())
    }

    fn pdf_attempts(&self) -> Result<Vec<(ReferenceId, PdfAttempt)>> {
        let sql = "SELECT reference_id, outcome FROM pdf_attempt";
        all(self.conn, sql, [], |row| Ok((id_at(row, "reference_id")?, parsed_at(row, "outcome")?)))
    }
}

const COLUMNS: &str = "id, reference_id, hash, filename, content_type, size, source, pdf_kind, pages, created_at, updated_at, updated_by, deleted_at";

fn from_row(row: &Row) -> rusqlite::Result<Attachment> {
    let pdf_kind: Option<String> = row.get("pdf_kind")?;
    Ok(Attachment {
        id: id_at(row, "id")?,
        reference: id_at(row, "reference_id")?,
        hash: row.get("hash")?,
        filename: row.get("filename")?,
        content_type: row.get("content_type")?,
        size: row.get::<_, i64>("size")? as u64,
        source: row.get("source")?,
        pdf_kind: pdf_kind.as_deref().and_then(kind_from),
        pages: row.get("pages")?,
        meta: meta_at(row)?,
    })
}

fn kind_name(kind: PdfKind) -> &'static str {
    match kind {
        PdfKind::Text => "text",
        PdfKind::Scanned => "scanned",
        PdfKind::Mixed => "mixed",
    }
}

fn kind_from(name: &str) -> Option<PdfKind> {
    Some(match name {
        "text" => PdfKind::Text,
        "scanned" => PdfKind::Scanned,
        "mixed" => PdfKind::Mixed,
        _ => return None,
    })
}

pub(super) fn of(conn: &Connection, reference: ReferenceId) -> Result<Vec<Attachment>> {
    let sql =
        format!("SELECT {COLUMNS} FROM attachment WHERE reference_id = ?1 AND deleted_at IS NULL ORDER BY created_at");
    all(conn, &sql, [id_blob(reference)], from_row)
}

pub(super) fn by_id(conn: &Connection, id: AttachmentId) -> Result<Option<Attachment>> {
    let sql = format!("SELECT {COLUMNS} FROM attachment WHERE id = ?1");
    first(conn, &sql, [id_blob(id)], from_row)
}

pub(super) fn insert(conn: &Connection, a: &Attachment) -> Result<()> {
    let sql =
        format!("INSERT INTO attachment ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)");
    conn.execute(
        &sql,
        params![
            id_blob(a.id),
            id_blob(a.reference),
            a.hash,
            a.filename,
            a.content_type,
            a.size as i64,
            a.source,
            a.pdf_kind.map(kind_name),
            a.pages,
            a.meta.created_at.as_millis(),
            a.meta.updated_at.as_millis(),
            id_blob(a.meta.updated_by),
            a.meta.deleted_at.map(|t| t.as_millis()),
        ],
    )
    .or_core()?;
    Ok(())
}

pub(super) fn update(conn: &Connection, a: &Attachment) -> Result<()> {
    // `reference_id` too: merging duplicates moves attachments to the kept reference.
    let sql = "UPDATE attachment SET filename = ?2, pdf_kind = ?3, pages = ?4, updated_at = ?5, updated_by = ?6,
               deleted_at = ?7, reference_id = ?8 WHERE id = ?1";
    conn.execute(
        sql,
        params![
            id_blob(a.id),
            a.filename,
            a.pdf_kind.map(kind_name),
            a.pages,
            a.meta.updated_at.as_millis(),
            id_blob(a.meta.updated_by),
            a.meta.deleted_at.map(|t| t.as_millis()),
            id_blob(a.reference),
        ],
    )
    .or_core()?;
    Ok(())
}
