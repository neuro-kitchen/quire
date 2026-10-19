//! An in-memory [`Library`] for tests of the use cases.

use std::collections::{BTreeMap, BTreeSet};

use quire_model::{
    Attachment, AttachmentId, CiteKey, DeviceId, Highlight, HighlightId, Identifier, Issue, Note, NoteId, PdfAttempt,
    Project, ProjectId, Reference, ReferenceId, Slug, Tag, TagId, TagKind, Timestamp,
};

use crate::error::Result;
use crate::ports::{AutoExport, FailedInput, Library, RawRecord, ReferenceFilter, SearchHit};

pub struct MemoryLibrary {
    device: DeviceId,
    references: Vec<Reference>,
    projects: Vec<Project>,
    links: BTreeSet<(ReferenceId, ProjectId)>,
    issues: BTreeMap<ReferenceId, Vec<Issue>>,
    failures: Vec<FailedInput>,
    auto_exports: Vec<AutoExport>,
    attachments: Vec<Attachment>,
    pdf_attempts: BTreeMap<ReferenceId, PdfAttempt>,
    tags: Vec<Tag>,
    reference_tags: BTreeMap<(ReferenceId, TagId), TagKind>,
    notes: Vec<Note>,
    relations: BTreeSet<(ReferenceId, ReferenceId)>,
    dismissed: BTreeSet<(ReferenceId, ReferenceId)>,
    highlights: Vec<Highlight>,
}

impl Default for MemoryLibrary {
    fn default() -> Self {
        Self {
            device: DeviceId::new(),
            references: Vec::new(),
            projects: Vec::new(),
            links: BTreeSet::new(),
            issues: BTreeMap::new(),
            failures: Vec::new(),
            auto_exports: Vec::new(),
            attachments: Vec::new(),
            pdf_attempts: BTreeMap::new(),
            tags: Vec::new(),
            reference_tags: BTreeMap::new(),
            notes: Vec::new(),
            relations: BTreeSet::new(),
            dismissed: BTreeSet::new(),
            highlights: Vec::new(),
        }
    }
}

impl Library for MemoryLibrary {
    fn device(&self) -> DeviceId {
        self.device
    }

    fn reference(&self, id: ReferenceId) -> Result<Option<Reference>> {
        Ok(self.references.iter().find(|r| r.id == id).cloned())
    }

    fn reference_by_cite_key(&self, key: &CiteKey) -> Result<Option<Reference>> {
        Ok(self.references.iter().find(|r| &r.cite_key == key).cloned())
    }

    fn reference_by_identifier(&self, identifier: &Identifier) -> Result<Option<Reference>> {
        let same = |id: &Identifier| id.kind() == identifier.kind() && id.key() == identifier.key();
        Ok(self.references.iter().find(|r| r.data.identifiers.iter().any(same)).cloned())
    }

    fn cite_key_taken(&self, key: &str) -> Result<bool> {
        Ok(self.references.iter().any(|r| r.cite_key.as_str() == key))
    }

    fn insert_reference(&mut self, reference: &Reference, _raw: Option<&RawRecord>) -> Result<()> {
        self.references.push(reference.clone());
        Ok(())
    }

    fn update_reference(&mut self, reference: &Reference) -> Result<()> {
        if let Some(slot) = self.references.iter_mut().find(|r| r.id == reference.id) {
            *slot = reference.clone();
        }
        Ok(())
    }

    fn add_raw_record(&mut self, _reference: ReferenceId, _raw: &RawRecord) -> Result<()> {
        Ok(())
    }

    fn references(&self, filter: &ReferenceFilter) -> Result<Vec<Reference>> {
        let in_projects = |r: &Reference| match &filter.projects {
            Some(projects) => projects.iter().any(|p| self.links.contains(&(r.id, *p))),
            None => true,
        } && filter.tags.iter().all(|t| self.reference_tags.contains_key(&(r.id, *t)));
        let mut found: Vec<Reference> = self
            .references
            .iter()
            .filter(|r| (filter.include_deleted || !r.meta.is_deleted()) && in_projects(r))
            .cloned()
            .collect();
        found.sort_by(|a, b| a.cite_key.cmp(&b.cite_key));
        Ok(found)
    }

    fn project(&self, id: ProjectId) -> Result<Option<Project>> {
        Ok(self.projects.iter().find(|p| p.id == id).cloned())
    }

    fn child_project(&self, parent: Option<ProjectId>, slug: &Slug) -> Result<Option<Project>> {
        Ok(self.projects.iter().find(|p| p.parent == parent && &p.slug == slug && !p.meta.is_deleted()).cloned())
    }

    fn child_projects(&self, parent: Option<ProjectId>) -> Result<Vec<Project>> {
        let mut found: Vec<Project> =
            self.projects.iter().filter(|p| p.parent == parent && !p.meta.is_deleted()).cloned().collect();
        found.sort_by(|a, b| a.slug.cmp(&b.slug));
        Ok(found)
    }

    fn insert_project(&mut self, project: &Project) -> Result<()> {
        self.projects.push(project.clone());
        Ok(())
    }

    fn update_project(&mut self, project: &Project) -> Result<()> {
        if let Some(slot) = self.projects.iter_mut().find(|p| p.id == project.id) {
            *slot = project.clone();
        }
        Ok(())
    }

    fn link(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool> {
        Ok(self.links.insert((reference, project)))
    }

    fn unlink(&mut self, reference: ReferenceId, project: ProjectId) -> Result<bool> {
        Ok(self.links.remove(&(reference, project)))
    }

    fn projects_of(&self, reference: ReferenceId) -> Result<Vec<ProjectId>> {
        let live = |p: &ProjectId| self.projects.iter().any(|project| project.id == *p && !project.meta.is_deleted());
        Ok(self.links.iter().filter(|(r, p)| *r == reference && live(p)).map(|(_, p)| *p).collect())
    }

    fn recorded_issues(&self, reference: ReferenceId) -> Result<Vec<Issue>> {
        Ok(self.issues.get(&reference).cloned().unwrap_or_default())
    }

    fn set_recorded_issues(&mut self, reference: ReferenceId, issues: &[Issue]) -> Result<()> {
        self.issues.insert(reference, issues.to_vec());
        Ok(())
    }

    fn record_failure(&mut self, input: &str, project: Option<ProjectId>, error: &str) -> Result<()> {
        let now = Timestamp::now();
        match self.failures.iter_mut().find(|f| f.input == input) {
            Some(failure) => {
                failure.attempts += 1;
                failure.error = error.into();
                failure.project = project;
                failure.last_at = now;
            }
            None => self.failures.push(FailedInput {
                input: input.into(),
                project,
                error: error.into(),
                attempts: 1,
                first_at: now,
                last_at: now,
            }),
        }
        Ok(())
    }

    fn clear_failure(&mut self, input: &str) -> Result<bool> {
        let before = self.failures.len();
        self.failures.retain(|f| f.input != input);
        Ok(self.failures.len() < before)
    }

    fn failures(&self) -> Result<Vec<FailedInput>> {
        Ok(self.failures.clone())
    }

    fn auto_exports(&self) -> Result<Vec<AutoExport>> {
        Ok(self.auto_exports.clone())
    }

    fn set_auto_export(&mut self, export: &AutoExport) -> Result<()> {
        self.auto_exports.retain(|e| e.path != export.path);
        self.auto_exports.push(export.clone());
        Ok(())
    }

    fn remove_auto_export(&mut self, path: &str) -> Result<bool> {
        let before = self.auto_exports.len();
        self.auto_exports.retain(|e| e.path != path);
        Ok(self.auto_exports.len() < before)
    }

    fn mark_auto_export_written(&mut self, path: &str, at: Timestamp) -> Result<()> {
        if let Some(export) = self.auto_exports.iter_mut().find(|e| e.path == path) {
            export.written_at = Some(at);
        }
        Ok(())
    }

    fn attachments_of(&self, reference: ReferenceId) -> Result<Vec<Attachment>> {
        Ok(self.attachments.iter().filter(|a| a.reference == reference && !a.meta.is_deleted()).cloned().collect())
    }

    fn attachment(&self, id: AttachmentId) -> Result<Option<Attachment>> {
        Ok(self.attachments.iter().find(|a| a.id == id).cloned())
    }

    fn insert_attachment(&mut self, attachment: &Attachment) -> Result<()> {
        self.attachments.push(attachment.clone());
        Ok(())
    }

    fn update_attachment(&mut self, attachment: &Attachment) -> Result<()> {
        if let Some(slot) = self.attachments.iter_mut().find(|a| a.id == attachment.id) {
            *slot = attachment.clone();
        }
        Ok(())
    }

    fn references_with_attachments(&self) -> Result<Vec<ReferenceId>> {
        let mut ids: Vec<ReferenceId> =
            self.attachments.iter().filter(|a| !a.meta.is_deleted()).map(|a| a.reference).collect();
        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    fn set_pdf_attempt(&mut self, reference: ReferenceId, attempt: Option<(PdfAttempt, &str)>) -> Result<()> {
        match attempt {
            Some((outcome, _)) => self.pdf_attempts.insert(reference, outcome),
            None => self.pdf_attempts.remove(&reference),
        };
        Ok(())
    }

    fn pdf_attempts(&self) -> Result<Vec<(ReferenceId, PdfAttempt)>> {
        Ok(self.pdf_attempts.iter().map(|(r, a)| (*r, *a)).collect())
    }

    fn tags(&self) -> Result<Vec<Tag>> {
        let mut tags: Vec<Tag> = self.tags.iter().filter(|t| !t.meta.is_deleted()).cloned().collect();
        tags.sort_by_key(|t| (t.position.is_none(), t.position, t.key()));
        Ok(tags)
    }

    fn tag_by_key(&self, key: &str) -> Result<Option<Tag>> {
        Ok(self.tags.iter().find(|t| t.key() == key && !t.meta.is_deleted()).cloned())
    }

    fn insert_tag(&mut self, tag: &Tag) -> Result<()> {
        self.tags.push(tag.clone());
        Ok(())
    }

    fn update_tag(&mut self, tag: &Tag) -> Result<()> {
        if let Some(slot) = self.tags.iter_mut().find(|t| t.id == tag.id) {
            *slot = tag.clone();
        }
        Ok(())
    }

    fn tag_reference(&mut self, reference: ReferenceId, tag: TagId, kind: TagKind) -> Result<bool> {
        Ok(self.reference_tags.insert((reference, tag), kind).is_none())
    }

    fn untag_reference(&mut self, reference: ReferenceId, tag: TagId) -> Result<bool> {
        Ok(self.reference_tags.remove(&(reference, tag)).is_some())
    }

    fn tags_of(&self, reference: ReferenceId) -> Result<Vec<(Tag, TagKind)>> {
        let live = self.tags()?;
        Ok(live.into_iter().filter_map(|t| self.reference_tags.get(&(reference, t.id)).map(|k| (t, *k))).collect())
    }

    fn tagged(&self, tag: TagId) -> Result<Vec<ReferenceId>> {
        Ok(self.reference_tags.keys().filter(|(_, t)| *t == tag).map(|(r, _)| *r).collect())
    }

    fn notes_of(&self, reference: Option<ReferenceId>) -> Result<Vec<Note>> {
        Ok(self.notes.iter().filter(|n| n.reference == reference && !n.meta.is_deleted()).cloned().collect())
    }

    fn note(&self, id: NoteId) -> Result<Option<Note>> {
        Ok(self.notes.iter().find(|n| n.id == id).cloned())
    }

    fn insert_note(&mut self, note: &Note) -> Result<()> {
        self.notes.push(note.clone());
        Ok(())
    }

    fn update_note(&mut self, note: &Note) -> Result<()> {
        if let Some(slot) = self.notes.iter_mut().find(|n| n.id == note.id) {
            *slot = note.clone();
        }
        Ok(())
    }

    fn note_counts(&self) -> Result<Vec<(ReferenceId, usize)>> {
        let mut counts: BTreeMap<ReferenceId, usize> = BTreeMap::new();
        for note in self.notes.iter().filter(|n| !n.meta.is_deleted()) {
            if let Some(r) = note.reference {
                *counts.entry(r).or_default() += 1;
            }
        }
        Ok(counts.into_iter().collect())
    }

    fn relate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        Ok(self.relations.insert(if a < b { (a, b) } else { (b, a) }))
    }

    fn unrelate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        Ok(self.relations.remove(&if a < b { (a, b) } else { (b, a) }))
    }

    fn related(&self, reference: ReferenceId) -> Result<Vec<ReferenceId>> {
        Ok(self
            .relations
            .iter()
            .filter_map(|(a, b)| {
                if *a == reference {
                    Some(*b)
                } else if *b == reference {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect())
    }

    fn dismiss_duplicate(&mut self, a: ReferenceId, b: ReferenceId) -> Result<bool> {
        Ok(self.dismissed.insert(if a < b { (a, b) } else { (b, a) }))
    }

    fn dismissed_duplicates(&self) -> Result<Vec<(ReferenceId, ReferenceId)>> {
        Ok(self.dismissed.iter().copied().collect())
    }

    fn highlights_of(&self, reference: ReferenceId) -> Result<Vec<Highlight>> {
        Ok(self.highlights.iter().filter(|h| h.reference == reference && !h.meta.is_deleted()).cloned().collect())
    }

    fn highlight(&self, id: HighlightId) -> Result<Option<Highlight>> {
        Ok(self.highlights.iter().find(|h| h.id == id).cloned())
    }

    fn insert_highlight(&mut self, highlight: &Highlight) -> Result<()> {
        self.highlights.push(highlight.clone());
        Ok(())
    }

    fn update_highlight(&mut self, highlight: &Highlight) -> Result<()> {
        if let Some(slot) = self.highlights.iter_mut().find(|h| h.id == highlight.id) {
            *slot = highlight.clone();
        }
        Ok(())
    }

    /// Naive: every word of the query (field prefixes and `*` dropped) in the title, abstract or
    /// a creator's name. Enough for use-case tests; SQLite does the real ranking.
    fn search(&self, query: &str, _in_text: bool, limit: usize) -> Result<Vec<SearchHit>> {
        let words: Vec<String> = query
            .split_whitespace()
            .map(|w| w.rsplit(':').next().unwrap_or(w).trim_matches(['"', '*']).to_lowercase())
            .filter(|w| !w.is_empty() && w != "and" && w != "or")
            .collect();
        let hits = self
            .references
            .iter()
            .filter(|r| !r.meta.is_deleted())
            .filter(|r| {
                let names: Vec<String> = r.data.creators.iter().map(|c| c.sort_name()).collect();
                let text = format!(
                    "{} {} {}",
                    r.data.title().unwrap_or_default(),
                    r.data.get(quire_model::TextField::Abstract).unwrap_or_default(),
                    names.join(" ")
                )
                .to_lowercase();
                words.iter().all(|w| text.contains(w.as_str()))
            })
            .map(|r| SearchHit { reference: r.id, score: 1.0, snippet: String::new(), in_text: false })
            .take(limit)
            .collect();
        Ok(hits)
    }

    fn index_paper_text(&mut self, _reference: ReferenceId, _text: Option<&str>) -> Result<()> {
        Ok(())
    }
}
