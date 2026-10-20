//! Metadata every stored record carries from day one, so sync can be added without a migration
//! (`.knowledge/data-foundations.md`, 2–3): creation and change times, the device that changed it
//! last, and a soft-delete time (tombstone).

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::id::DeviceId;

/// Milliseconds since the Unix epoch (UTC), from a hybrid logical clock (in milliseconds):
/// [`now`](Self::now) never goes back and is later than every stamp [`observe`](Self::observe)d
/// from another device, so a change made after seeing another always sorts after it, whatever
/// the two devices' clocks say. Values stay readable as dates (`.knowledge/sync.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

/// The clock: the last stamp handed out or observed.
static CLOCK: AtomicI64 = AtomicI64::new(0);

/// How far ahead of this device's time another device's stamp may be: further means its clock is
/// wrong, and taking the stamp would let its changes win over every later one.
pub const MAX_DRIFT_MILLIS: i64 = 24 * 3600 * 1000;

fn wall_millis() -> i64 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock after 1970");
    i64::try_from(since_epoch.as_millis()).expect("milliseconds fit in i64")
}

impl Timestamp {
    /// The current time, later than any stamp given or observed before (by at least 1 ms).
    pub fn now() -> Self {
        let wall = wall_millis();
        let mut last = CLOCK.load(Ordering::Relaxed);
        loop {
            let next = wall.max(last + 1);
            match CLOCK.compare_exchange_weak(last, next, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => return Self(next),
                Err(seen) => last = seen,
            }
        }
    }

    /// Takes in a stamp from elsewhere (another device, the stored library): later
    /// [`now`](Self::now)s are after it. `false`, and nothing changes, when it is more than
    /// [`MAX_DRIFT_MILLIS`] ahead of this device's time.
    pub fn observe(self) -> bool {
        if self.0 > wall_millis() + MAX_DRIFT_MILLIS {
            return false;
        }
        CLOCK.fetch_max(self.0, Ordering::Relaxed);
        true
    }

    pub fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    pub fn as_millis(self) -> i64 {
        self.0
    }

    /// The UTC calendar day, e.g. for "new since" queries and showing when something happened.
    pub fn date(self) -> Date {
        let (year, month, day) = civil_from_days(self.0.div_euclid(MILLIS_PER_DAY));
        Date::new(year, Some(month), Some(day)).expect("a valid calendar day")
    }

    /// `days` whole days earlier.
    pub fn days_before(self, days: i64) -> Self {
        Self(self.0 - days * MILLIS_PER_DAY)
    }
}

const MILLIS_PER_DAY: i64 = 86_400_000;

/// Year, month, day of the day `days` after 1970-01-01 (Howard Hinnant's `civil_from_days`,
/// https://howardhinnant.github.io/date_algorithms.html).
fn civil_from_days(days: i64) -> (u16, u8, u8) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year as u16, month as u8, day as u8)
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
    fn clock_never_goes_back() {
        let first = Timestamp::now();
        let second = Timestamp::now();
        assert!(second > first, "two stamps in the same millisecond still differ");
        let ahead = Timestamp::from_millis(second.as_millis() + 60_000);
        assert!(ahead.observe());
        assert!(Timestamp::now() > ahead, "after a stamp observed from another device");
        let broken = Timestamp::from_millis(wall_millis() + 2 * MAX_DRIFT_MILLIS);
        assert!(!broken.observe());
        assert!(Timestamp::now() < broken);
    }

    #[test]
    fn calendar_days() {
        let day = |millis| {
            let d = Timestamp::from_millis(millis).date();
            (d.get_year(), d.month(), d.day())
        };
        assert_eq!(day(0), (1970, Some(1), Some(1)));
        assert_eq!(day(951_782_400_000), (2000, Some(2), Some(29)));
        assert_eq!(day(1_792_454_400_000), (2026, Some(10), Some(20)));
        assert_eq!(Timestamp::from_millis(MILLIS_PER_DAY).days_before(1), Timestamp::from_millis(0));
    }

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
