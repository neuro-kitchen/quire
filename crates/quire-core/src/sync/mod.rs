//! What devices exchange to stay in sync (`.knowledge/sync.md`): the current state of every
//! record that changed, as [`Row`]s. The store keeps a change log (which rows changed, with a
//! local sequence number) and merges incoming rows, newest `updated_at` first ([`newer`]); the
//! transport (QUIC, `quire-sync`) only carries them. References merge part by part
//! ([`merge_reference`]), so edits of different fields on two devices both survive.
//!
//! Synced: references, projects and memberships, tags and taggings, notes, highlights,
//! relations, attachments (their files go separately, by hash), "not duplicates" pairs, and the
//! personal records: reading status and saved searches ([`Row::is_personal`]). Kept on each
//! device: fetch issues, failed inputs, PDF download attempts, raw records, imports, automatic
//! exports, settings.

mod fields;

pub use fields::{merge_reference, stamp_changes};

use quire_model::{
    Attachment, DeviceId, Highlight, Note, Project, ProjectId, ProjectMember, Reading, Reference, ReferenceId,
    SavedSearch, Tag, TagId, TagKind, Timestamp,
};
use serde::{Deserialize, Serialize};

/// One record as it is now (tombstones included: a deletion is a row with `deleted_at`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "table", rename_all = "snake_case")]
pub enum Row {
    Reference(Reference),
    Project(Project),
    Membership(Membership),
    Tag(Tag),
    Tagging(Tagging),
    Note(Note),
    Highlight(Highlight),
    Relation(Relation),
    Attachment(Attachment),
    Distinct(Distinct),
    Reading(Reading),
    SavedSearch(SavedSearch),
    /// Someone's role on a shared project.
    Member(ProjectMember),
}

impl Row {
    /// When this version was written and on which device (what [`newer`] compares).
    pub fn version(&self) -> (Timestamp, DeviceId) {
        match self {
            Self::Reference(r) => (r.meta.updated_at, r.meta.updated_by),
            Self::Project(p) => (p.meta.updated_at, p.meta.updated_by),
            Self::Tag(t) => (t.meta.updated_at, t.meta.updated_by),
            Self::Note(n) => (n.meta.updated_at, n.meta.updated_by),
            Self::Highlight(h) => (h.meta.updated_at, h.meta.updated_by),
            Self::Attachment(a) => (a.meta.updated_at, a.meta.updated_by),
            Self::Reading(r) => (r.meta.updated_at, r.meta.updated_by),
            Self::SavedSearch(s) => (s.meta.updated_at, s.meta.updated_by),
            Self::Member(m) => (m.meta.updated_at, m.meta.updated_by),
            Self::Membership(Membership { stamp, .. })
            | Self::Tagging(Tagging { stamp, .. })
            | Self::Relation(Relation { stamp, .. }) => (stamp.updated_at, stamp.updated_by),
            Self::Distinct(d) => (d.created_at, d.by),
        }
    }

    /// The user's own records: they go to the user's devices only, never to collaborators.
    pub fn is_personal(&self) -> bool {
        matches!(self, Self::Reading(_) | Self::SavedSearch(_))
    }
}

/// A reference in a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Membership {
    pub project: ProjectId,
    pub reference: ReferenceId,
    pub stamp: Stamp,
}

/// A tag on a reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tagging {
    pub reference: ReferenceId,
    pub tag: TagId,
    pub kind: TagKind,
    pub stamp: Stamp,
}

/// Two related references (stored once, `a` < `b`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    pub a: ReferenceId,
    pub b: ReferenceId,
    pub stamp: Stamp,
}

/// Two references the user said are distinct works (never undone).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Distinct {
    pub a: ReferenceId,
    pub b: ReferenceId,
    pub created_at: Timestamp,
    pub by: DeviceId,
}

/// When a link row changed and by whom (the link tables have no `RecordMeta`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamp {
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub updated_by: DeviceId,
    pub deleted_at: Option<Timestamp>,
}

/// Whether a change made at `incoming` wins over the stored one made at `stored`: the later
/// write wins; on a tie, the larger device ID (the same choice on every device).
pub fn newer(incoming: (Timestamp, DeviceId), stored: (Timestamp, DeviceId)) -> bool {
    incoming > stored
}

/// What merging a batch of rows did.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeSummary {
    /// Rows written (new or newer).
    pub applied: usize,
    /// Rows older than (or the same as) what was here.
    pub kept: usize,
    /// Rows refused: stamped more than a day ahead of this device's clock (the sender's clock is
    /// wrong; taking them would let them win over every later change).
    pub refused: usize,
    /// References given another cite key (theirs was taken here by another paper).
    pub renamed: Vec<(String, String)>,
    /// Identifiers left off an incoming reference (another reference here has them): possible
    /// duplicates to merge.
    pub dropped_identifiers: Vec<(String, String)>,
}

/// Several deliveries' summaries as one (a hub session merges per delivery).
impl std::ops::AddAssign for MergeSummary {
    fn add_assign(&mut self, other: Self) {
        self.applied += other.applied;
        self.kept += other.kept;
        self.refused += other.refused;
        self.renamed.extend(other.renamed);
        self.dropped_identifiers.extend(other.dropped_identifiers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn later_write_wins_ties_by_device() {
        let (a, b) = (DeviceId::new(), DeviceId::new());
        let (low, high) = if a < b { (a, b) } else { (b, a) };
        let t = Timestamp::now();
        assert!(newer((Timestamp::from_millis(t.as_millis() + 1), low), (t, high)));
        assert!(newer((t, high), (t, low)));
        assert!(!newer((t, low), (t, low)), "the same change is not newer");
    }

    #[test]
    fn rows_round_trip_as_json() {
        let stamp = Stamp {
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            updated_by: DeviceId::new(),
            deleted_at: None,
        };
        let row = Row::Membership(Membership { project: ProjectId::new(), reference: ReferenceId::new(), stamp });
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.starts_with(r#"{"table":"membership""#), "{json}");
        assert_eq!(serde_json::from_str::<Row>(&json).unwrap(), row);
    }
}
