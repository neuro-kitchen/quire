//! The storage port: what the use cases need from a library store. `quire-store` implements it
//! on SQLite; tests use the in-memory [`MemoryLibrary`](crate::memory::MemoryLibrary).
//!
//! Calls are synchronous: the engine runs them on the database thread, inside one transaction
//! per use case.

use quire_model::{
    Attachment, AttachmentId, CiteKey, DeviceId, Identifier, Issue, Note, NoteId, PdfAttempt, Project, ProjectId,
    Reference, ReferenceId, Slug, Tag, TagId, TagKind, Timestamp,
};

use crate::error::Result;

/// The record a reference was built from, as received (e.g. CSL-JSON from doi.org), kept so a
/// mapping fix can re-derive fields without fetching again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRecord {
    /// Where it came from, e.g. `doi.org`, `pubmed`, a file name.
    pub source: String,
    /// Format of `body`, e.g. `csl-json`, `bibtex`, `html-meta`.
    pub format: String,
    pub body: String,
    pub received_at: Timestamp,
}

/// An input (`quire add <input>`) whose lookup failed, kept so `add --retry` can try again and
/// `ref check` can list it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedInput {
    pub input: String,
    /// The project it was to be added to.
    pub project: Option<ProjectId>,
    /// The last error, as text.
    pub error: String,
    pub attempts: u32,
    pub first_at: Timestamp,
    pub last_at: Timestamp,
}

/// A file rewritten after every change (`quire export --keep-updated`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoExport {
    /// Absolute path of the file.
    pub path: String,
    /// The project exported; `None` for the whole library.
    pub project: Option<ProjectId>,
    pub recursive: bool,
    /// Export profile (config) to use, if any.
    pub profile: Option<String>,
    /// Format overriding the profile's, if any (`biblatex`, `hayagriva`, …).
    pub format: Option<String>,
    pub written_at: Option<Timestamp>,
}

/// Which references to list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceFilter {
    /// Only references linked to one of these projects (all references if `None`).
    pub projects: Option<Vec<ProjectId>>,
    /// Also references in the trash.
    pub include_deleted: bool,
    /// Only references carrying every one of these tags.
    pub tags: Vec<TagId>,
}

/// A library store. Reads skip records in the trash unless stated otherwise.
pub trait Library {
    /// The device this library copy belongs to (recorded on every change).
    fn device(&self) -> DeviceId;

    // References
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

    // Projects
    fn project(&self, id: ProjectId) -> Result<Option<Project>>;
    /// The project named `slug` under `parent` (top level if `None`).
    fn child_project(&self, parent: Option<ProjectId>, slug: &Slug) -> Result<Option<Project>>;
    /// Projects directly under `parent`, ordered by slug.
    fn child_projects(&self, parent: Option<ProjectId>) -> Result<Vec<Project>>;
    fn insert_project(&mut self, project: &Project) -> Result<()>;
    fn update_project(&mut self, project: &Project) -> Result<()>;

    // Membership (a reference in a project)
    /// Links them; `false` if they already were.
    fn link(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool>;
    /// Unlinks them; `false` if they were not linked.
    fn unlink(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool>;
    /// Projects (not in the trash) `reference` is linked to.
    fn projects_of(&self, reference: ReferenceId) -> Result<Vec<ProjectId>>;

    // Issues recorded when a reference was fetched (lints are computed, not stored)
    fn recorded_issues(&self, reference: ReferenceId) -> Result<Vec<Issue>>;
    /// Replaces every recorded issue of `reference` with `issues`.
    fn set_recorded_issues(&mut self, reference: ReferenceId, issues: &[Issue]) -> Result<()>;

    // Inputs whose lookup failed
    /// Records a failure for `input` (counting attempts if it failed before).
    fn record_failure(&mut self, input: &str, project: Option<ProjectId>, error: &str) -> Result<()>;
    /// Forgets `input`'s failures (it was added); `false` if there were none.
    fn clear_failure(&mut self, input: &str) -> Result<bool>;
    /// Every failed input, oldest first.
    fn failures(&self) -> Result<Vec<FailedInput>>;

    // Automatic exports
    fn auto_exports(&self) -> Result<Vec<AutoExport>>;
    /// Registers `export` (replacing one for the same path).
    fn set_auto_export(&mut self, export: &AutoExport) -> Result<()>;
    /// `false` if no export was registered for `path`.
    fn remove_auto_export(&mut self, path: &str) -> Result<bool>;
    fn mark_auto_export_written(&mut self, path: &str, at: Timestamp) -> Result<()>;

    // Attachments
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

    // Tags
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

    // Notes
    /// Live notes of `reference`, or standalone notes for `None`; oldest first.
    fn notes_of(&self, reference: Option<ReferenceId>) -> Result<Vec<Note>>;
    fn note(&self, id: NoteId) -> Result<Option<Note>>;
    fn insert_note(&mut self, note: &Note) -> Result<()>;
    fn update_note(&mut self, note: &Note) -> Result<()>;
    /// How many live notes each reference has (references without notes left out).
    fn note_counts(&self) -> Result<Vec<(ReferenceId, usize)>>;

    // Related references (symmetric)
    /// Relates them; `false` if they already were.
    fn relate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool>;
    /// `false` if they were not related.
    fn unrelate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool>;
    fn related(&self, reference: ReferenceId) -> Result<Vec<ReferenceId>>;
}
