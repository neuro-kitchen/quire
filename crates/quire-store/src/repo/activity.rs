//! This device's activity log for its notifications pane (migration 019; never synced): what
//! syncs brought from which device, and security events (a device linked or unlinked, the key
//! changed, a wrong link code, a new device on the hub).

use quire_core::Result;
use rusqlite::{OptionalExtension, params};

use super::SqliteLibrary;
use crate::error::OrCore;

/// Entries kept: older ones go (the pane is for recent events).
const KEEP: i64 = 500;

/// One entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub id: i64,
    /// Milliseconds since 1970.
    pub at: i64,
    pub kind: String,
    /// The device it concerns, by name.
    pub source: Option<String>,
    /// `sync`: `kind=count,…`; other kinds: a number, or empty.
    pub detail: String,
    pub read: bool,
}

impl SqliteLibrary<'_> {
    /// Adds an entry (and drops the oldest past [`KEEP`]).
    pub fn add_activity(&self, at: i64, kind: &str, source: Option<&str>, detail: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO activity (at, kind, source, detail) VALUES (?1, ?2, ?3, ?4)",
                params![at, kind, source, detail],
            )
            .or_core()?;
        self.conn.execute("DELETE FROM activity WHERE id <= (SELECT MAX(id) FROM activity) - ?1", [KEEP]).or_core()?;
        Ok(())
    }

    /// The newest entries first, at most `limit`.
    pub fn activity(&self, limit: u32) -> Result<Vec<Activity>> {
        let mut statement = self
            .conn
            .prepare("SELECT id, at, kind, source, detail, read FROM activity ORDER BY id DESC LIMIT ?1")
            .or_core()?;
        statement
            .query_map([i64::from(limit)], |r| {
                Ok(Activity {
                    id: r.get(0)?,
                    at: r.get(1)?,
                    kind: r.get(2)?,
                    source: r.get(3)?,
                    detail: r.get(4)?,
                    read: r.get(5)?,
                })
            })
            .or_core()?
            .collect::<rusqlite::Result<_>>()
            .or_core()
    }

    pub fn unread_activity(&self) -> Result<u32> {
        let count: i64 =
            self.conn.query_row("SELECT COUNT(*) FROM activity WHERE read = 0", [], |r| r.get(0)).or_core()?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    pub fn mark_activity_read(&self) -> Result<()> {
        self.conn.execute("UPDATE activity SET read = 1 WHERE read = 0", []).or_core()?;
        Ok(())
    }

    /// Notes `device` as seen; `true` if it was not known yet.
    pub fn see_known_device(&self, device: &str, at: i64) -> Result<bool> {
        Ok(self
            .conn
            .execute("INSERT OR IGNORE INTO known_device (device, first_seen) VALUES (?1, ?2)", params![device, at])
            .or_core()?
            > 0)
    }

    /// Whether this device has seen any device of the person yet (the first list only teaches it).
    pub fn knows_devices(&self) -> Result<bool> {
        Ok(self
            .conn
            .query_row("SELECT 1 FROM known_device LIMIT 1", [], |r| r.get::<_, i64>(0))
            .optional()
            .or_core()?
            .is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_come_newest_first_and_are_marked_read() {
        let conn = crate::open_in_memory().unwrap();
        let lib = SqliteLibrary::new(&conn, quire_model::DeviceId::new());
        lib.add_activity(1, "sync", Some("Pixel"), "note=2").unwrap();
        lib.add_activity(2, "unlinked", Some("Old laptop"), "").unwrap();
        let all = lib.activity(10).unwrap();
        assert_eq!(all.iter().map(|a| a.kind.as_str()).collect::<Vec<_>>(), ["unlinked", "sync"]);
        assert_eq!(lib.unread_activity().unwrap(), 2);
        lib.mark_activity_read().unwrap();
        assert_eq!(lib.unread_activity().unwrap(), 0);
        assert!(!lib.knows_devices().unwrap());
        assert!(lib.see_known_device("dev", 1).unwrap());
        assert!(!lib.see_known_device("dev", 2).unwrap(), "known now");
    }
}
