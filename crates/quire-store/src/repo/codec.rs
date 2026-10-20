//! Conversions between model values and SQLite columns.

use quire_model::{Id, IdKind, RecordMeta, Timestamp};
use rusqlite::Row;
use rusqlite::types::Type;

/// The 16 bytes stored for `id`.
pub(crate) fn id_blob<K: IdKind>(id: Id<K>) -> [u8; 16] {
    id.to_bytes()
}

/// An ID from a BLOB column.
pub(crate) fn id_at<K: IdKind>(row: &Row, column: &str) -> rusqlite::Result<Id<K>> {
    let bytes: [u8; 16] = row.get(column)?;
    Id::from_bytes(bytes).map_err(|e| conversion(column, e))
}

/// An optional ID from a nullable BLOB column.
pub(crate) fn optional_id_at<K: IdKind>(row: &Row, column: &str) -> rusqlite::Result<Option<Id<K>>> {
    let bytes: Option<[u8; 16]> = row.get(column)?;
    bytes.map(|b| Id::from_bytes(b).map_err(|e| conversion(column, e))).transpose()
}

/// `created_at`, `updated_at`, `updated_by`, `deleted_at` of a record row.
pub(crate) fn meta_at(row: &Row) -> rusqlite::Result<RecordMeta> {
    Ok(RecordMeta {
        created_at: Timestamp::from_millis(row.get("created_at")?),
        updated_at: Timestamp::from_millis(row.get("updated_at")?),
        updated_by: id_at(row, "updated_by")?,
        deleted_at: row.get::<_, Option<i64>>("deleted_at")?.map(Timestamp::from_millis),
    })
}

/// A JSON column parsed into `T`.
pub(crate) fn json_at<T: serde::de::DeserializeOwned>(row: &Row, column: &str) -> rusqlite::Result<T> {
    let text: String = row.get(column)?;
    serde_json::from_str(&text).map_err(|e| conversion(column, e))
}

/// A value from a TEXT column parsed with `FromStr`.
pub(crate) fn parsed_at<T>(row: &Row, column: &str) -> rusqlite::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let text: String = row.get(column)?;
    text.parse().map_err(|e| conversion(column, e))
}

fn conversion(column: &str, error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    let index = 0; // rusqlite wants an index; the column name is in the message instead.
    rusqlite::Error::FromSqlConversionFailure(index, Type::Text, format!("{column}: {error}").into())
}
