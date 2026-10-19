//! [`Id<K>`]: a UUIDv7 tagged with its record kind. Stored as 16 bytes, written as a TypeID.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typeid_suffix::prelude::TypeIdSuffix;
use uuid::Uuid;

use super::IdKind;
use crate::error::{ModelError, Result};

/// A record ID of kind `K`. A `ProjectId` cannot be passed where a `ReferenceId` is expected,
/// and parsing `prj_…` as a reference ID fails.
pub struct Id<K> {
    uuid: Uuid,
    // `fn() -> K`: no ownership of a `K`, so `Id<K>` is Send + Sync whatever `K` is.
    kind: PhantomData<fn() -> K>,
}

impl<K: IdKind> Id<K> {
    /// A new ID from the current time (UUIDv7).
    pub fn new() -> Self {
        Self::wrap(Uuid::now_v7())
    }

    /// The ID stored as `bytes` (16-byte BLOB in the database). Fails unless they hold a UUIDv7.
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self> {
        Self::from_uuid(Uuid::from_bytes(bytes))
    }

    /// Fails unless `uuid` is a UUIDv7.
    pub fn from_uuid(uuid: Uuid) -> Result<Self> {
        if uuid.get_version_num() != 7 {
            return Err(Self::error(uuid.to_string(), format!("UUID version {} (expected 7)", uuid.get_version_num())));
        }
        Ok(Self::wrap(uuid))
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// The 16 bytes to store.
    pub fn to_bytes(&self) -> [u8; 16] {
        self.uuid.into_bytes()
    }

    fn wrap(uuid: Uuid) -> Self {
        Self { uuid, kind: PhantomData }
    }

    fn error(input: String, reason: String) -> ModelError {
        ModelError::InvalidId { kind: K::NAME, input, reason }
    }
}

impl<K: IdKind> Default for Id<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: IdKind> fmt::Display for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", K::PREFIX, TypeIdSuffix::from(self.uuid))
    }
}

impl<K: IdKind> fmt::Debug for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<K: IdKind> FromStr for Id<K> {
    type Err = ModelError;

    /// Parses `<prefix>_<26 Base32 chars>`; the prefix must be this kind's.
    fn from_str(s: &str) -> Result<Self> {
        let Some((prefix, suffix)) = s.rsplit_once('_') else {
            return Err(Self::error(s.into(), format!("expected `{}_<26 characters>`", K::PREFIX)));
        };
        if prefix != K::PREFIX {
            return Err(Self::error(s.into(), format!("prefix `{prefix}` (expected `{}`)", K::PREFIX)));
        }
        let suffix = TypeIdSuffix::from_str(suffix).map_err(|e| Self::error(s.into(), e.to_string()))?;
        Self::from_uuid(suffix.to_uuid()).map_err(|_| Self::error(s.into(), "not a UUIDv7".into()))
    }
}

// Manual impls: derives would require `K` itself to implement these traits.
impl<K> Clone for Id<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for Id<K> {}

impl<K> PartialEq for Id<K> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl<K> Eq for Id<K> {}

impl<K> Hash for Id<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl<K> PartialOrd for Id<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Creation-time order within one device; across devices only approximate (see `.knowledge/data-foundations.md`).
impl<K> Ord for Id<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl<K: IdKind> Serialize for Id<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de, K: IdKind> Deserialize<'de> for Id<K> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{ProjectId, ReferenceId};

    #[test]
    fn display_parse_round_trip() {
        let id = ReferenceId::new();
        let text = id.to_string();
        assert!(text.starts_with("ref_") && text.len() == 4 + 26, "{text}");
        assert_eq!(text.parse::<ReferenceId>().unwrap(), id);
    }

    #[test]
    fn bytes_round_trip() {
        let id = ProjectId::new();
        assert_eq!(ProjectId::from_bytes(id.to_bytes()).unwrap(), id);
    }

    #[test]
    fn wrong_prefix_is_rejected() {
        let text = ProjectId::new().to_string();
        let err = text.parse::<ReferenceId>().unwrap_err();
        assert!(err.to_string().contains("prefix `prj`"), "{err}");
    }

    #[test]
    fn non_v7_is_rejected() {
        assert!(ReferenceId::from_uuid(Uuid::nil()).is_err());
        assert!("ref_".parse::<ReferenceId>().is_err());
        assert!("ref01h455vb4pex5vsknk084sn02q".parse::<ReferenceId>().is_err());
    }

    #[test]
    fn later_ids_sort_after_earlier_ones() {
        let first = ReferenceId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        assert!(ReferenceId::new() > first);
    }

    #[test]
    fn serde_uses_the_typeid_text() {
        let id = ReferenceId::new();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{id}\""));
        assert_eq!(serde_json::from_str::<ReferenceId>(&json).unwrap(), id);
    }
}
