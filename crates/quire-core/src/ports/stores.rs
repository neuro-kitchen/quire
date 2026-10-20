//! The store traits, one per group of tables. Reads skip records in the trash unless stated
//! otherwise.

use quire_model::{
    Alert, AlertId, Attachment, AttachmentId, CiteKey, DeviceId, Highlight, HighlightId, Identifier, Issue, Note,
    NoteId, PdfAttempt, Project, ProjectId, ProjectMember, Reading, Reference, ReferenceId, SavedSearch, SavedSearchId,
    Slug, Tag, TagId, TagKind, Timestamp, UserId,
};

use super::records::{AutoExport, FailedInput, RawRecord, ReferenceFilter, SearchHit};
use crate::error::Result;

/// The library copy itself.
pub trait Device {
    /// The device this library copy belongs to (recorded on every change).
    fn device(&self) -> DeviceId;
}

pub trait ReferenceStore {
    /// By ID, including references in the trash.
    fn reference(&self, id: ReferenceId) -> Result<Option<Reference>>;
    /// By cite key, including the trash (keys are unique across it).
    fn reference_by_cite_key(&self, key: &CiteKey) -> Result<Option<Reference>>;
    /// The reference holding an identifier with the same kind and key, including the trash.
    fn reference_by_identifier(&self, identifier: &Identifier) -> Result<Option<Reference>>;
    /// Whether any reference (also in the trash) uses `key`.
    fn cite_key_taken(&self, key: &str) -> Result<bool>;
    fn insert_reference(&mut self, reference: &Reference, raw: Option<&RawRecord>) -> Result<()>;
    fn update_reference(&mut self, reference: &Reference) -> Result<()>;
    /// Keeps another record received for `reference` (e.g. on refetch).
    fn add_raw_record(&mut self, reference: ReferenceId, raw: &RawRecord) -> Result<()>;
    /// Ordered by cite key.
    fn references(&self, filter: &ReferenceFilter) -> Result<Vec<Reference>>;
}

/// Projects and their members (a reference in a project).
pub trait ProjectStore {
    fn project(&self, id: ProjectId) -> Result<Option<Project>>;
    /// The project named `slug` under `parent` (top level if `None`).
    fn child_project(&self, parent: Option<ProjectId>, slug: &Slug) -> Result<Option<Project>>;
    /// Projects directly under `parent`, ordered by slug.
    fn child_projects(&self, parent: Option<ProjectId>) -> Result<Vec<Project>>;
    fn insert_project(&mut self, project: &Project) -> Result<()>;
    fn update_project(&mut self, project: &Project) -> Result<()>;
    /// Links them; `false` if they already were.
    fn link(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool>;
    /// Unlinks them; `false` if they were not linked.
    fn unlink(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool>;
    /// Projects (not in the trash) `reference` is linked to.
    fn projects_of(&self, reference: ReferenceId) -> Result<Vec<ProjectId>>;
}

/// Issues recorded when a reference was fetched (lints are computed, not stored), and inputs
/// whose lookup failed.
pub trait IssueStore {
    fn recorded_issues(&self, reference: ReferenceId) -> Result<Vec<Issue>>;
    /// Replaces every recorded issue of `reference` with `issues`.
    fn set_recorded_issues(&mut self, reference: ReferenceId, issues: &[Issue]) -> Result<()>;
    /// Records a failure for `input` (counting attempts if it failed before).
    fn record_failure(&mut self, input: &str, project: Option<ProjectId>, error: &str) -> Result<()>;
    /// Forgets `input`'s failures (it was added); `false` if there were none.
    fn clear_failure(&mut self, input: &str) -> Result<bool>;
    /// Every failed input, oldest first.
    fn failures(&self) -> Result<Vec<FailedInput>>;
}

/// Files rewritten after every change.
pub trait ExportStore {
    fn auto_exports(&self) -> Result<Vec<AutoExport>>;
    /// Registers `export` (replacing one for the same path).
    fn set_auto_export(&mut self, export: &AutoExport) -> Result<()>;
    /// `false` if no export was registered for `path`.
    fn remove_auto_export(&mut self, path: &str) -> Result<bool>;
    fn mark_auto_export_written(&mut self, path: &str, at: Timestamp) -> Result<()>;
}

/// Attachments (PDFs) and failed downloads.
pub trait AttachmentStore {
    /// Live attachments of `reference`, oldest first.
    fn attachments_of(&self, reference: ReferenceId) -> Result<Vec<Attachment>>;
    fn attachment(&self, id: AttachmentId) -> Result<Option<Attachment>>;
    fn insert_attachment(&mut self, attachment: &Attachment) -> Result<()>;
    /// Writes `attachment`'s changes (e.g. soft delete).
    fn update_attachment(&mut self, attachment: &Attachment) -> Result<()>;
    /// References (by ID) that have at least one live attachment.
    fn references_with_attachments(&self) -> Result<Vec<ReferenceId>>;
    /// Records the last failed `pdf get` for `reference` (`None` clears it).
    fn set_pdf_attempt(&mut self, reference: ReferenceId, attempt: Option<(PdfAttempt, &str)>) -> Result<()>;
    /// Every recorded failed attempt.
    fn pdf_attempts(&self) -> Result<Vec<(ReferenceId, PdfAttempt)>>;
}

pub trait TagStore {
    /// Live tags: colored first by position, then by name.
    fn tags(&self) -> Result<Vec<Tag>>;
    /// The live tag whose lowercase name is `key`.
    fn tag_by_key(&self, key: &str) -> Result<Option<Tag>>;
    fn insert_tag(&mut self, tag: &Tag) -> Result<()>;
    /// Writes a tag's changes (name, color, position, soft delete).
    fn update_tag(&mut self, tag: &Tag) -> Result<()>;
    /// Puts `tag` on `reference`; `false` if it already was.
    fn tag_reference(&mut self, reference: ReferenceId, tag: TagId, kind: TagKind) -> Result<bool>;
    /// Takes `tag` off `reference`; `false` if it was not on.
    fn untag_reference(&mut self, reference: ReferenceId, tag: TagId) -> Result<bool>;
    /// Live tags of `reference`, with how each was added.
    fn tags_of(&self, reference: ReferenceId) -> Result<Vec<(Tag, TagKind)>>;
    /// References carrying `tag` (also those in the trash).
    fn tagged(&self, tag: TagId) -> Result<Vec<ReferenceId>>;
}

pub trait NoteStore {
    /// Live notes of `reference`, or standalone notes for `None`; oldest first.
    fn notes_of(&self, reference: Option<ReferenceId>) -> Result<Vec<Note>>;
    fn note(&self, id: NoteId) -> Result<Option<Note>>;
    fn insert_note(&mut self, note: &Note) -> Result<()>;
    fn update_note(&mut self, note: &Note) -> Result<()>;
    /// How many live notes each reference has (references without notes left out).
    fn note_counts(&self) -> Result<Vec<(ReferenceId, usize)>>;
}

/// Pairs of references: related (symmetric), and pairs the user said are not duplicates.
pub trait RelationStore {
    /// Relates them; `false` if they already were.
    fn relate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool>;
    /// `false` if they were not related.
    fn unrelate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool>;
    fn related(&self, reference: ReferenceId) -> Result<Vec<ReferenceId>>;
    /// Records that `a` and `b` are distinct works; `false` if already recorded.
    fn dismiss_duplicate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool>;
    fn dismissed_duplicates(&self) -> Result<Vec<(ReferenceId, ReferenceId)>>;
}

/// Passages marked in the reader.
pub trait HighlightStore {
    /// Live highlights of `reference`, oldest first.
    fn highlights_of(&self, reference: ReferenceId) -> Result<Vec<Highlight>>;
    fn highlight(&self, id: HighlightId) -> Result<Option<Highlight>>;
    fn insert_highlight(&mut self, highlight: &Highlight) -> Result<()>;
    /// Writes a highlight's changes (comment, reference, soft delete).
    fn update_highlight(&mut self, highlight: &Highlight) -> Result<()>;
}

/// Where the user is with each paper (personal).
pub trait ReadingStore {
    /// `reference`'s reading status, if it was ever set.
    fn reading(&self, reference: ReferenceId) -> Result<Option<Reading>>;
    /// Writes `reading` (new or changed).
    fn set_reading(&mut self, reading: &Reading) -> Result<()>;
    /// Every status set (unread ones included).
    fn readings(&self) -> Result<Vec<Reading>>;
}

/// Searches kept by name (personal).
pub trait SavedSearchStore {
    /// Live saved searches, by name.
    fn saved_searches(&self) -> Result<Vec<SavedSearch>>;
    /// The live search whose lowercase name is `key`.
    fn saved_search_by_key(&self, key: &str) -> Result<Option<SavedSearch>>;
    fn saved_search(&self, id: SavedSearchId) -> Result<Option<SavedSearch>>;
    fn insert_saved_search(&mut self, search: &SavedSearch) -> Result<()>;
    /// Writes its changes (name, query, soft delete).
    fn update_saved_search(&mut self, search: &SavedSearch) -> Result<()>;
}

/// Searches of arXiv, PubMed or Crossref checked for new papers (kept on this device).
pub trait AlertStore {
    /// Live alerts, by name.
    fn alerts(&self) -> Result<Vec<Alert>>;
    /// The live alert whose lowercase name is `key`.
    fn alert_by_key(&self, key: &str) -> Result<Option<Alert>>;
    fn insert_alert(&mut self, alert: &Alert) -> Result<()>;
    /// Writes its changes (query, last check, soft delete).
    fn update_alert(&mut self, alert: &Alert) -> Result<()>;
    /// Which of `works` (identifier keys) `alert` has shown before.
    fn seen(&self, alert: AlertId, works: &[String]) -> Result<Vec<String>>;
    fn mark_seen(&mut self, alert: AlertId, works: &[String]) -> Result<()>;
}

/// Members of shared projects.
pub trait MemberStore {
    /// The members of `project` (not its ancestors'), removed ones included.
    fn members_of(&self, project: ProjectId) -> Result<Vec<ProjectMember>>;
    /// Every membership `user` has (removed ones included).
    fn memberships(&self, user: &UserId) -> Result<Vec<ProjectMember>>;
    /// Writes `member` (new or changed).
    fn set_member(&mut self, member: &ProjectMember) -> Result<()>;
}

/// Records imported from another program (e.g. Zotero), by that program's key.
pub trait ImportStore {
    /// The reference imported from `source` under `key`, if any.
    fn imported(&self, source: &str, key: &str) -> Result<Option<ReferenceId>>;
    /// Records that `key` of `source` became `reference`.
    fn record_import(&mut self, source: &str, key: &str, reference: ReferenceId) -> Result<()>;
}

/// Full-text search.
pub trait SearchStore {
    /// References matching the FTS5 `query` (see `library::search::fts_query`), best first; with
    /// `in_text`, papers' text too.
    fn search(&self, query: &str, in_text: bool, limit: usize) -> Result<Vec<SearchHit>>;
    /// Replaces the indexed text of `reference`'s paper (`None` removes it).
    fn index_paper_text(&mut self, reference: ReferenceId, text: Option<&str>) -> Result<()>;
}
