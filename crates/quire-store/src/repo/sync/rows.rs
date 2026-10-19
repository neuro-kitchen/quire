//! Reading the rows the change log names, as [`Row`]s to send.

use quire_core::Result;
use quire_core::sync::{Distinct, Membership, Relation, Row, Stamp, Tagging};
use quire_model::{Id, IdKind, TagId, Timestamp};
use rusqlite::{Connection, OptionalExtension, Row as SqlRow};

use crate::error::OrCore;
use crate::repo::codec::{id_at, parsed_at};
use crate::repo::{attachments, highlights, projects, references, tags};

/// The row `table` holds at (`k1`, `k2`), if it still exists.
pub fn load(conn: &Connection, table: &str, k1: &[u8], k2: &[u8]) -> Result<Option<Row>> {
    Ok(match table {
        "reference" => references::by_id(conn, id(k1)?)?.map(Row::Reference),
        "project" => projects::by_id(conn, id(k1)?)?.map(Row::Project),
        "tag" => tag(conn, id(k1)?)?.map(Row::Tag),
        "note" => tags::note(conn, id(k1)?)?.map(Row::Note),
        "highlight" => highlights::by_id(conn, id(k1)?)?.map(Row::Highlight),
        "attachment" => attachments::by_id(conn, id(k1)?)?.map(Row::Attachment),
        "project_reference" => conn
            .query_row(
                "SELECT project_id, reference_id, created_at, updated_at, updated_by, deleted_at
                 FROM project_reference WHERE project_id = ?1 AND reference_id = ?2",
                [k1, k2],
                |row| Ok(Membership { project: id_at(row, "project_id")?, reference: id_at(row, "reference_id")?, stamp: stamp(row)? }),
            )
            .optional()
            .or_core()?
            .map(Row::Membership),
        "reference_tag" => conn
            .query_row(
                "SELECT reference_id, tag_id, kind, created_at, updated_at, updated_by, deleted_at
                 FROM reference_tag WHERE reference_id = ?1 AND tag_id = ?2",
                [k1, k2],
                |row| {
                    Ok(Tagging {
                        reference: id_at(row, "reference_id")?,
                        tag: id_at(row, "tag_id")?,
                        kind: parsed_at(row, "kind")?,
                        stamp: stamp(row)?,
                    })
                },
            )
            .optional()
            .or_core()?
            .map(Row::Tagging),
        "reference_relation" => conn
            .query_row(
                "SELECT a, b, created_at, updated_at, updated_by, deleted_at FROM reference_relation WHERE a = ?1 AND b = ?2",
                [k1, k2],
                |row| Ok(Relation { a: id_at(row, "a")?, b: id_at(row, "b")?, stamp: stamp(row)? }),
            )
            .optional()
            .or_core()?
            .map(Row::Relation),
        "duplicate_dismissal" => conn
            .query_row(
                "SELECT a, b, created_at, updated_by FROM duplicate_dismissal WHERE a = ?1 AND b = ?2",
                [k1, k2],
                |row| {
                    Ok(Distinct {
                        a: id_at(row, "a")?,
                        b: id_at(row, "b")?,
                        created_at: Timestamp::from_millis(row.get("created_at")?),
                        by: id_at(row, "updated_by")?,
                    })
                },
            )
            .optional()
            .or_core()?
            .map(Row::Distinct),
        _ => None,
    })
}

/// The ID in a change log key.
fn id<K: IdKind>(bytes: &[u8]) -> Result<Id<K>> {
    let bytes: [u8; 16] =
        bytes.try_into().map_err(|_| quire_core::CoreError::Storage("a change log key is not an ID".into()))?;
    Ok(Id::from_bytes(bytes)?)
}

pub(super) fn tag(conn: &Connection, id: TagId) -> Result<Option<quire_model::Tag>> {
    let sql = format!("SELECT {} FROM tag WHERE id = ?1", tags::TAG_COLUMNS);
    conn.query_row(&sql, [crate::repo::codec::id_blob(id)], tags::tag_from_row).optional().or_core()
}

/// The `created_at`, `updated_at`, `updated_by`, `deleted_at` columns of a link row.
pub fn stamp(row: &SqlRow) -> rusqlite::Result<Stamp> {
    Ok(Stamp {
        created_at: Timestamp::from_millis(row.get("created_at")?),
        updated_at: Timestamp::from_millis(row.get("updated_at")?),
        updated_by: id_at(row, "updated_by")?,
        deleted_at: row.get::<_, Option<i64>>("deleted_at")?.map(Timestamp::from_millis),
    })
}
