//! Metadata every stored record carries from day one, so sync can be added without a migration
//! (`.knowledge/data-foundations.md`, 2–3): creation and change times, the device that changed it
//! last, and a soft-delete time (tombstone).

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::id::DeviceId;

/// Milliseconds since the Unix epoch (UTC). Becomes a hybrid logical clock value with sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn now() -> Self {
        let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock after 1970");
        Self(i64::try_from(since_epoch.as_millis()).expect("milliseconds fit in i64"))
    }

    pub fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    pub fn as_millis(self) -> i64 {
        self.0
    }
}

/// Times and device of a record's life.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordMeta {
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub updated_by: DeviceId,
    /// Set when the record is in the trash; the row stays as a tombstone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<Timestamp>,
}

impl RecordMeta {
    /// A record created now on `device`.
    pub fn new(device: DeviceId) -> Self {
        let now = Timestamp::now();
        Self { created_at: now, updated_at: now, updated_by: device, deleted_at: None }
    }

    /// Marks a change made now on `device`.
    pub fn touch(&mut self, device: DeviceId) {
        self.updated_at = Timestamp::now();
        self.updated_by = device;
    }

    /// Moves the record to the trash.
    pub fn delete(&mut self, device: DeviceId) {
        self.touch(device);
        self.deleted_at = Some(self.updated_at);
    }

    /// Takes the record out of the trash.
    pub fn restore(&mut self, device: DeviceId) {
        self.touch(device);
        self.deleted_at = None;
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_and_restore() {
        let device = DeviceId::new();
        let mut meta = RecordMeta::new(device);
        assert!(!meta.is_deleted());
        meta.delete(device);
        assert!(meta.is_deleted());
        assert_eq!(meta.deleted_at, Some(meta.updated_at));
        meta.restore(device);
        assert!(!meta.is_deleted());
    }
}
