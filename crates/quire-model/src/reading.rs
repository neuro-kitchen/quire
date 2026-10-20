//! [`Reading`]: where the user is with a paper (unread, reading, read) and since when. Personal:
//! it travels to the user's own devices only, never to collaborators (`.knowledge/sync.md`).

use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::id::{DeviceId, ReferenceId};
use crate::record::{RecordMeta, Timestamp};

name_enum! {
    /// Where the user is with a paper.
    pub enum ReadingStatus (unknown: ModelError::UnknownReadingStatus) {
        Unread => "unread",
        Reading => "reading",
        Read => "read",
    }
}

/// A paper's reading status. A reference without one is unread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reading {
    pub reference: ReferenceId,
    pub status: ReadingStatus,
    /// When reading began (kept when it is finished).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<Timestamp>,
    /// When it was marked read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<Timestamp>,
    pub meta: RecordMeta,
}

impl Reading {
    /// `reference` as never opened.
    pub fn unread(reference: ReferenceId, device: DeviceId) -> Self {
        Self {
            reference,
            status: ReadingStatus::Unread,
            started_at: None,
            finished_at: None,
            meta: RecordMeta::new(device),
        }
    }

    /// Moves to `status` now: reading starts the clock (if not started), read stops it (and starts
    /// it if it never was), unread clears both. `false` if it already had that status.
    pub fn set(&mut self, status: ReadingStatus, device: DeviceId) -> bool {
        if self.status == status {
            return false;
        }
        self.meta.touch(device);
        let now = self.meta.updated_at;
        match status {
            ReadingStatus::Unread => (self.started_at, self.finished_at) = (None, None),
            ReadingStatus::Reading => {
                self.started_at.get_or_insert(now);
                self.finished_at = None;
            }
            ReadingStatus::Read => {
                self.started_at.get_or_insert(now);
                self.finished_at = Some(now);
            }
        }
        self.status = status;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_follow_the_status() {
        let device = DeviceId::new();
        let mut reading = Reading::unread(ReferenceId::new(), device);
        assert!(reading.set(ReadingStatus::Reading, device));
        let started = reading.started_at.unwrap();
        assert!(!reading.set(ReadingStatus::Reading, device));
        assert!(reading.set(ReadingStatus::Read, device));
        assert_eq!(reading.started_at, Some(started));
        assert!(reading.finished_at.is_some());
        reading.set(ReadingStatus::Unread, device);
        assert_eq!((reading.started_at, reading.finished_at), (None, None));
        assert_eq!("read".parse::<ReadingStatus>().unwrap(), ReadingStatus::Read);
    }
}
