//! Opening a library database: connection settings, migrations, and this copy's device ID.

use std::path::Path;

use quire_model::DeviceId;
use rusqlite::{Connection, OptionalExtension};

use crate::error::Result;
use crate::schema::migrations;

/// Waits this long for a lock held by another quire process before failing.
const BUSY_TIMEOUT_MS: u32 = 5_000;

/// Opens (creating if needed) the database at `path` and migrates it to the latest schema.
pub fn open(path: &Path) -> Result<Connection> {
    prepare(Connection::open(path)?)
}

/// A new in-memory database (tests).
pub fn open_in_memory() -> Result<Connection> {
    prepare(Connection::open_in_memory()?)
}

/// Settings and migrations for a fresh connection.
pub fn prepare(mut conn: Connection) -> Result<Connection> {
    prepare_in_place(&mut conn)?;
    Ok(conn)
}

/// [`prepare`] for a connection owned elsewhere (e.g. by `tokio-rusqlite`'s thread).
pub fn prepare_in_place(conn: &mut Connection) -> Result<()> {
    configure(conn)?;
    migrations().to_latest(conn)?;
    seed_clock(conn)
}

/// Tables whose rows carry `updated_at` (every synced record).
const STAMPED: &[&str] = &[
    "reference",
    "project",
    "project_reference",
    "tag",
    "reference_tag",
    "note",
    "highlight",
    "reference_relation",
    "attachment",
    "reading",
    "saved_search",
];

/// Starts the clock after the newest stamp stored (`Timestamp` is a hybrid logical clock): if
/// this device's time went back, or a peer's stamps were ahead, new changes still sort after.
fn seed_clock(conn: &Connection) -> Result<()> {
    let parts: Vec<String> = STAMPED.iter().map(|t| format!("SELECT max(updated_at) AS m FROM {t}")).collect();
    let sql = format!("SELECT max(m) FROM ({})", parts.join(" UNION ALL "));
    let newest: Option<i64> = conn.query_row(&sql, [], |row| row.get(0))?;
    if let Some(millis) = newest {
        quire_model::Timestamp::from_millis(millis).observe();
    }
    Ok(())
}

/// Foreign keys on (SQLite's default is off), WAL journal (readers do not block the writer),
/// busy timeout.
pub fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    // Returns the resulting mode (`memory` for in-memory databases), hence the `_and_check`.
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
    conn.pragma_update(None, "busy_timeout", BUSY_TIMEOUT_MS)?;
    Ok(())
}

/// This library copy's device ID, created on first use.
pub fn device_id(conn: &Connection) -> Result<DeviceId> {
    let stored: Option<String> =
        conn.query_row("SELECT value FROM library_meta WHERE key = 'device_id'", [], |row| row.get(0)).optional()?;
    if let Some(text) = stored {
        return Ok(text.parse()?);
    }
    let device = DeviceId::new();
    conn.execute("INSERT INTO library_meta (key, value) VALUES ('device_id', ?1)", [device.to_string()])?;
    Ok(device)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_id_is_created_once() {
        let conn = open_in_memory().unwrap();
        let first = device_id(&conn).unwrap();
        assert_eq!(device_id(&conn).unwrap(), first);
    }

    #[test]
    fn reopening_a_file_keeps_data() {
        let dir = std::env::temp_dir().join(format!("quire-store-test-{}", DeviceId::new()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quire.sqlite");
        let device = device_id(&open(&path).unwrap()).unwrap();
        assert_eq!(device_id(&open(&path).unwrap()).unwrap(), device);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
