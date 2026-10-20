//! Importing a Zotero library (`quire import zotero`): items become references (a work already
//! here is linked, not duplicated), collections become projects, tags, notes and related items
//! follow, PDFs are stored and checked like any other, and highlights made in Zotero's reader
//! become highlights. Importing again adds only what is new: items are found by their Zotero
//! key, notes by their text, highlights by their passage.

use std::collections::HashMap;
use std::path::Path;

use quire_core::library::{highlights, projects, references, tags};
use quire_core::prelude::*;
use quire_core::{AddRequest, Added};
use quire_io::zotero::{self as format, AnnotationKind, ZoteroAnnotation, ZoteroLibrary};
use quire_model::{Note, Origin, ProjectId, Reference, ReferenceId, SlugPath, TagKind};
use quire_store::SqliteLibrary;

use super::Engine;
use crate::error::Result;
use crate::zotero;

/// The `imported.source` of Zotero items.
const SOURCE: &str = "zotero";

/// What [`Engine::import_zotero`] did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ZoteroImport {
    /// New references.
    pub added: Vec<Reference>,
    /// Items whose work was already in the library (same identifier, or imported before).
    pub existing: usize,
    pub projects: usize,
    pub notes: usize,
    pub pdfs: usize,
    pub highlights: usize,
    /// What was left out, and why (one line each).
    pub skipped: Vec<String>,
}

impl Engine {
    /// Imports the Zotero library in the data folder `dir`. Collections go under `project` when
    /// given (else at the top level). `base` is Zotero's folder for linked files (its "Linked
    /// Attachment Base Directory"), for attachments stored relative to it.
    pub async fn import_zotero(
        &self,
        dir: &Path,
        project: Option<&SlugPath>,
        base: Option<&Path>,
    ) -> Result<ZoteroImport> {
        let source = dir.to_path_buf();
        let library = tokio::task::spawn_blocking(move || zotero::read(&source))
            .await
            .map_err(|e| crate::error::EngineError::Zotero(e.to_string()))??;
        let parent = self.project_id(project).await?;
        let metadata = library.clone();
        let (mut report, ids) = self.run(move |lib| apply(lib, &metadata, parent)).await?;
        self.mark_changed();

        // PDFs: each file read, checked and stored as if attached by hand.
        let mut attached: HashMap<&str, ReferenceId> = HashMap::new();
        for attachment in &library.attachments {
            let Some(reference) = attachment.parent.as_ref().and_then(|p| ids.get(p)) else { continue };
            let is_pdf = attachment.content_type.as_deref() == Some("application/pdf");
            let Some(file) = zotero::file_of(dir, attachment, base).filter(|_| is_pdf) else {
                if is_pdf {
                    report.skipped.push(format!("PDF {}: linked to a folder quire was not given", attachment.key));
                }
                continue;
            };
            let found = self.run({
                let id = *reference;
                move |lib| lib.reference(id)
            });
            let Some(reference) = found.await? else { continue };
            match std::fs::read(&file) {
                Ok(bytes) => {
                    match self.attach_bytes(&reference, &bytes, &format!("zotero:{}", attachment.key), false).await {
                        Ok(done) => {
                            report.pdfs += usize::from(done.new);
                            attached.insert(attachment.key.as_str(), reference.id);
                        }
                        Err(error) => report.skipped.push(format!("PDF {}: {error}", file.display())),
                    }
                }
                Err(error) => report.skipped.push(format!("PDF {}: {error}", file.display())),
            }
        }

        // Highlights and comments from Zotero's reader, on the papers whose PDF came along.
        let marks: Vec<(ReferenceId, ZoteroAnnotation)> = library
            .annotations
            .iter()
            .filter_map(|a| Some((*attached.get(a.attachment.as_str())?, a.clone())))
            .collect();
        let skipped_marks = library.annotations.len() - marks.len();
        let (highlights, notes, other) = self.run(move |lib| apply_marks(lib, marks)).await?;
        report.highlights += highlights;
        report.notes += notes;
        if skipped_marks + other > 0 {
            report
                .skipped
                .push(format!("{} annotations (images, ink, or on PDFs not imported)", skipped_marks + other));
        }
        self.mark_changed();
        Ok(report)
    }
}

/// Everything but the files, in one transaction. Returns the report so far and each item's
/// reference by Zotero key.
fn apply(
    lib: &mut SqliteLibrary<'_>,
    library: &ZoteroLibrary,
    parent: Option<ProjectId>,
) -> quire_core::Result<(ZoteroImport, HashMap<String, ReferenceId>)> {
    let mut report = ZoteroImport::default();
    let mut ids: HashMap<String, ReferenceId> = HashMap::new();
    for item in &library.items {
        if let Some(known) = lib.imported(SOURCE, &item.key)?
            && lib.reference(known)?.is_some()
        {
            ids.insert(item.key.clone(), known);
            report.existing += 1;
            continue;
        }
        let record = format::to_record(item);
        let request = AddRequest {
            preferred_key: record.cite_key,
            ..AddRequest::new(record.data, Origin::Imported { file: SOURCE.into() })
        };
        let added = references::add(lib, request)?;
        let id = added.reference().id;
        lib.record_import(SOURCE, &item.key, id)?;
        ids.insert(item.key.clone(), id);
        match added {
            Added::New(reference) => report.added.push(reference),
            Added::Existing(_) => report.existing += 1,
        }
    }

    for item in &library.items {
        let id = ids[&item.key].to_string();
        for (name, automatic) in &item.tags {
            let kind = if *automatic { TagKind::Automatic } else { TagKind::Manual };
            if let Err(error) = tags::add(lib, &id, name, kind) {
                report.skipped.push(format!("tag `{name}`: {error}"));
            }
        }
        for other in item.related.iter().filter_map(|key| ids.get(key)) {
            tags::relate(lib, &id, &other.to_string())?;
        }
    }

    // Collections as projects (parents come first).
    let mut project_of: HashMap<&str, ProjectId> = HashMap::new();
    for collection in &library.collections {
        let under = match &collection.parent {
            Some(key) => match project_of.get(key.as_str()) {
                Some(id) => Some(*id),
                None => continue,
            },
            None => parent,
        };
        let before = lib.child_projects(under)?.len();
        let project = projects::ensure(lib, &collection.name, under)?;
        report.projects += usize::from(lib.child_projects(under)?.len() > before);
        project_of.insert(collection.key.as_str(), project.id);
        for key in &collection.items {
            if let Some(id) = ids.get(key) {
                lib.link(*id, project.id)?;
            }
        }
    }

    for note in &library.notes {
        let reference = match &note.parent {
            Some(key) => match ids.get(key) {
                Some(id) => Some(*id),
                None => continue,
            },
            None => None,
        };
        let body = format::note_text(&note.html);
        if add_note(lib, reference, &body)? {
            report.notes += 1;
        }
    }
    Ok((report, ids))
}

/// Adds a note unless the reference (or the standalone notes) already has one with this text.
fn add_note(lib: &mut SqliteLibrary<'_>, reference: Option<ReferenceId>, body: &str) -> quire_core::Result<bool> {
    if body.is_empty() || lib.notes_of(reference)?.iter().any(|n| n.body == body) {
        return Ok(false);
    }
    lib.insert_note(&Note::new(reference, body, lib.device()))?;
    Ok(true)
}

/// Highlights (and underlines) as highlights; comments without text as notes. Returns the
/// highlights and notes added, and the marks left out.
fn apply_marks(
    lib: &mut SqliteLibrary<'_>,
    marks: Vec<(ReferenceId, ZoteroAnnotation)>,
) -> quire_core::Result<(usize, usize, usize)> {
    let (mut added, mut notes, mut other) = (0, 0, 0);
    for (reference, mark) in marks {
        let text = mark.text.unwrap_or_default();
        let comment = mark.comment.filter(|c| !c.trim().is_empty());
        match mark.kind {
            AnnotationKind::Highlight | AnnotationKind::Underline if !text.trim().is_empty() => {
                let before = lib.highlights_of(reference)?.len();
                highlights::add(lib, &reference.to_string(), text.trim(), "", comment.as_deref())?;
                added += usize::from(lib.highlights_of(reference)?.len() > before);
            }
            AnnotationKind::Note | AnnotationKind::Text if comment.is_some() => {
                let body = match mark.page_label {
                    Some(page) => format!("{} (p. {page})", comment.unwrap_or_default()),
                    None => comment.unwrap_or_default(),
                };
                notes += usize::from(add_note(lib, Some(reference), &body)?);
            }
            _ => other += 1,
        }
    }
    Ok((added, notes, other))
}
