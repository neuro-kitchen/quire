//! [`SqliteLibrary`]: the `quire-core` [`Library`] port on a SQLite connection (usually a
//! transaction, so one use case commits or rolls back as a whole).
//!
//! - [`codec`]: IDs, times and JSON ↔ SQLite values.
//! - [`references`], [`projects`], [`links`], [`issues`], [`auto_export`], [`attachments`], [`tags`]: the
//!   queries, one module per table group.

mod attachments;
mod auto_export;
mod codec;
mod highlights;
mod issues;
mod links;
mod projects;
mod references;
mod tags;

use quire_core::{AutoExport, FailedInput, Library, RawRecord, ReferenceFilter, Result};
use quire_model::{
    Attachment, AttachmentId, CiteKey, DeviceId, Highlight, HighlightId, Identifier, Issue, Note, NoteId, PdfAttempt,
    Project, ProjectId, Reference, ReferenceId, Slug, Tag, TagId, TagKind, Timestamp,
};
use rusqlite::Connection;

/// A library on `conn`, recording changes as made by `device`.
pub struct SqliteLibrary<'c> {
    conn: &'c Connection,
    device: DeviceId,
}

impl<'c> SqliteLibrary<'c> {
    pub fn new(conn: &'c Connection, device: DeviceId) -> Self {
        Self { conn, device }
    }
}

impl Library for SqliteLibrary<'_> {
    fn device(&self) -> DeviceId {
        self.device
    }

    fn reference(&self, id: ReferenceId) -> Result<Option<Reference>> {
        references::by_id(self.conn, id)
    }

    fn reference_by_cite_key(&self, key: &CiteKey) -> Result<Option<Reference>> {
        references::by_cite_key(self.conn, key)
    }

    fn reference_by_identifier(&self, identifier: &Identifier) -> Result<Option<Reference>> {
        references::by_identifier(self.conn, identifier)
    }

    fn cite_key_taken(&self, key: &str) -> Result<bool> {
        references::cite_key_taken(self.conn, key)
    }

    fn insert_reference(&mut self, reference: &Reference, raw: Option<&RawRecord>) -> Result<()> {
        references::insert(self.conn, reference, raw)
    }

    fn update_reference(&mut self, reference: &Reference) -> Result<()> {
        references::update(self.conn, reference)
    }

    fn add_raw_record(&mut self, reference: ReferenceId, raw: &RawRecord) -> Result<()> {
        references::insert_raw(self.conn, reference, raw)
    }

    fn references(&self, filter: &ReferenceFilter) -> Result<Vec<Reference>> {
        references::list(self.conn, filter)
    }

    fn project(&self, id: ProjectId) -> Result<Option<Project>> {
        projects::by_id(self.conn, id)
    }

    fn child_project(&self, parent: Option<ProjectId>, slug: &Slug) -> Result<Option<Project>> {
        projects::child(self.conn, parent, slug)
    }

    fn child_projects(&self, parent: Option<ProjectId>) -> Result<Vec<Project>> {
        projects::children(self.conn, parent)
    }

    fn insert_project(&mut self, project: &Project) -> Result<()> {
        projects::insert(self.conn, project)
    }

    fn update_project(&mut self, project: &Project) -> Result<()> {
        projects::update(self.conn, project)
    }

    fn link(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool> {
        links::link(self.conn, reference, project, self.device)
    }

    fn unlink(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool> {
        links::unlink(self.conn, reference, project, self.device)
    }

    fn projects_of(&self, reference: ReferenceId) -> Result<Vec<ProjectId>> {
        links::projects_of(self.conn, reference)
    }

    fn recorded_issues(&self, reference: ReferenceId) -> Result<Vec<Issue>> {
        issues::recorded(self.conn, reference)
    }

    fn set_recorded_issues(&mut self, reference: ReferenceId, list: &[Issue]) -> Result<()> {
        issues::set_recorded(self.conn, reference, list)
    }

    fn record_failure(&mut self, input: &str, project: Option<ProjectId>, error: &str) -> Result<()> {
        issues::record_failure(self.conn, input, project, error)
    }

    fn clear_failure(&mut self, input: &str) -> Result<bool> {
        issues::clear_failure(self.conn, input)
    }

    fn failures(&self) -> Result<Vec<FailedInput>> {
        issues::failures(self.conn)
    }

    fn auto_exports(&self) -> Result<Vec<AutoExport>> {
        auto_export::all(self.conn)
    }

    fn set_auto_export(&mut self, export: &AutoExport) -> Result<()> {
        auto_export::set(self.conn, export)
    }

    fn remove_auto_export(&mut self, path: &str) -> Result<bool> {
        auto_export::remove(self.conn, path)
    }

    fn mark_auto_export_written(&mut self, path: &str, at: Timestamp) -> Result<()> {
        auto_export::mark_written(self.conn, path, at)
    }

    fn attachments_of(&self, reference: ReferenceId) -> Result<Vec<Attachment>> {
        attachments::of(self.conn, reference)
    }

    fn attachment(&self, id: AttachmentId) -> Result<Option<Attachment>> {
        attachments::by_id(self.conn, id)
    }

    fn insert_attachment(&mut self, attachment: &Attachment) -> Result<()> {
        attachments::insert(self.conn, attachment)
    }

    fn update_attachment(&mut self, attachment: &Attachment) -> Result<()> {
        attachments::update(self.conn, attachment)
    }

    fn references_with_attachments(&self) -> Result<Vec<ReferenceId>> {
        attachments::references_with(self.conn)
    }

    fn set_pdf_attempt(&mut self, reference: ReferenceId, attempt: Option<(PdfAttempt, &str)>) -> Result<()> {
        attachments::set_attempt(self.conn, reference, attempt)
    }

    fn pdf_attempts(&self) -> Result<Vec<(ReferenceId, PdfAttempt)>> {
        attachments::attempts(self.conn)
    }

    fn tags(&self) -> Result<Vec<Tag>> {
        tags::all(self.conn)
    }

    fn tag_by_key(&self, key: &str) -> Result<Option<Tag>> {
        tags::by_key(self.conn, key)
    }

    fn insert_tag(&mut self, tag: &Tag) -> Result<()> {
        tags::insert(self.conn, tag)
    }

    fn update_tag(&mut self, tag: &Tag) -> Result<()> {
        tags::update(self.conn, tag)
    }

    fn tag_reference(&mut self, reference: ReferenceId, tag: TagId, kind: TagKind) -> Result<bool> {
        tags::tag_reference(self.conn, reference, tag, kind, self.device)
    }

    fn untag_reference(&mut self, reference: ReferenceId, tag: TagId) -> Result<bool> {
        tags::untag_reference(self.conn, reference, tag, self.device)
    }

    fn tags_of(&self, reference: ReferenceId) -> Result<Vec<(Tag, TagKind)>> {
        tags::tags_of(self.conn, reference)
    }

    fn tagged(&self, tag: TagId) -> Result<Vec<ReferenceId>> {
        tags::tagged(self.conn, tag)
    }

    fn notes_of(&self, reference: Option<ReferenceId>) -> Result<Vec<Note>> {
        tags::notes_of(self.conn, reference)
    }

    fn note(&self, id: NoteId) -> Result<Option<Note>> {
        tags::note(self.conn, id)
    }

    fn insert_note(&mut self, note: &Note) -> Result<()> {
        tags::insert_note(self.conn, note)
    }

    fn update_note(&mut self, note: &Note) -> Result<()> {
        tags::update_note(self.conn, note)
    }

    fn note_counts(&self) -> Result<Vec<(ReferenceId, usize)>> {
        tags::note_counts(self.conn)
    }

    fn relate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        tags::relate(self.conn, a, b, self.device)
    }

    fn unrelate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        tags::unrelate(self.conn, a, b, self.device)
    }

    fn related(&self, reference: ReferenceId) -> Result<Vec<ReferenceId>> {
        tags::related(self.conn, reference)
    }

    fn dismiss_duplicate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        tags::dismiss_duplicate(self.conn, a, b, self.device)
    }

    fn dismissed_duplicates(&self) -> Result<Vec<(ReferenceId, ReferenceId)>> {
        tags::dismissed_duplicates(self.conn)
    }

    fn highlights_of(&self, reference: ReferenceId) -> Result<Vec<Highlight>> {
        highlights::of(self.conn, reference)
    }

    fn highlight(&self, id: HighlightId) -> Result<Option<Highlight>> {
        highlights::by_id(self.conn, id)
    }

    fn insert_highlight(&mut self, highlight: &Highlight) -> Result<()> {
        highlights::insert(self.conn, highlight)
    }

    fn update_highlight(&mut self, highlight: &Highlight) -> Result<()> {
        highlights::update(self.conn, highlight)
    }
}
