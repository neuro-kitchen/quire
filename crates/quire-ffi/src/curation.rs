//! Keeping the library clean: duplicates (find, merge, dismiss), checks, retractions.

use quire_engine::{DuplicateReason, Engine, UpdateKind};

use crate::records::ReferenceRecord;
use crate::references::{IssueRecord, issue};
use crate::{QuireLibrary, Result, query};

/// A reference and those that look like the same work.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct DuplicateGroupRecord {
    pub reference: ReferenceRecord,
    pub duplicates: Vec<DuplicateRecord>,
}

/// `reason`: `identifier` (then `identifier` is the shared one) or `title`.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct DuplicateRecord {
    pub reference: ReferenceRecord,
    pub reason: String,
    pub identifier: Option<String>,
}

/// What a merge did: what moved from the removed reference into the kept one.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct MergedRecord {
    pub kept: ReferenceRecord,
    pub removed_id: String,
    pub fields: Vec<String>,
    pub projects: u32,
    pub tags: u32,
    pub notes: u32,
    pub attachments: u32,
    pub highlights: u32,
    pub relations: u32,
}

/// A checked reference with its issues (recorded first, then lints; worst first).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct CheckedRecord {
    pub id: String,
    pub cite_key: String,
    pub issues: Vec<IssueRecord>,
}

/// A reference's retraction and correction notices (Crossref, Retraction Watch).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct RetractionRecord {
    pub reference: ReferenceRecord,
    pub notices: Vec<NoticeRecord>,
    pub error: Option<String>,
}

/// `kind`: `retraction`, `concern` (expression of concern) or another update (`correction`, …).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct NoticeRecord {
    pub kind: String,
    pub notice: Option<String>,
    pub date: Option<String>,
    pub source: Option<String>,
}

#[uniffi::export]
impl QuireLibrary {
    /// Every group of references that look like the same work.
    pub async fn duplicates(&self) -> Result<Vec<DuplicateGroupRecord>> {
        self.with(|engine| async move {
            let mut groups = Vec::new();
            for group in engine.duplicates().await? {
                let mut duplicates = Vec::new();
                for (reference, reason) in group.duplicates {
                    let (reason, identifier) = match reason {
                        DuplicateReason::SameIdentifier(id) => ("identifier", Some(id)),
                        DuplicateReason::SameTitle => ("title", None),
                    };
                    duplicates.push(DuplicateRecord {
                        reference: record(&engine, &reference).await?,
                        reason: reason.into(),
                        identifier,
                    });
                }
                groups.push(DuplicateGroupRecord { reference: record(&engine, &group.reference).await?, duplicates });
            }
            Ok(groups)
        })
        .await
    }

    /// Merges `other` into `keep` (its fields, projects, tags, notes, PDFs, highlights move over).
    pub async fn merge(&self, keep: String, other: String) -> Result<MergedRecord> {
        self.with(|engine| async move {
            let merged = engine.merge(&keep, &other).await?;
            Ok(MergedRecord {
                kept: record(&engine, &merged.kept).await?,
                removed_id: merged.removed.id.to_string(),
                fields: merged.fields.iter().map(ToString::to_string).collect(),
                projects: merged.projects as u32,
                tags: merged.tags as u32,
                notes: merged.notes as u32,
                attachments: merged.attachments as u32,
                highlights: merged.highlights as u32,
                relations: merged.relations as u32,
            })
        })
        .await
    }

    /// Records that `a` and `b` are distinct works; `false` if already recorded.
    pub async fn dismiss_duplicate(&self, a: String, b: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.dismiss_duplicate(&a, &b).await?) }).await
    }

    /// The references in `project` (with subprojects if `recursive`), or the library, that have
    /// issues, with them.
    pub async fn check(&self, project: Option<String>, recursive: bool) -> Result<Vec<CheckedRecord>> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            Ok(engine
                .check(project.as_ref(), recursive)
                .await?
                .into_iter()
                .map(|checked| CheckedRecord {
                    id: checked.reference.id.to_string(),
                    cite_key: checked.reference.cite_key.to_string(),
                    issues: checked.issues.iter().map(issue).collect(),
                })
                .collect())
        })
        .await
    }

    /// Checks the references with a DOI in `project` (or the library) for retractions and notices.
    pub async fn check_retractions(&self, project: Option<String>, recursive: bool) -> Result<Vec<RetractionRecord>> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let mut records = Vec::new();
            for check in engine.check_retractions(project.as_ref(), recursive).await? {
                records.push(RetractionRecord {
                    reference: record(&engine, &check.reference).await?,
                    notices: check
                        .updates
                        .into_iter()
                        .map(|update| NoticeRecord {
                            kind: match update.kind {
                                UpdateKind::Retracted(_) => "retraction".into(),
                                UpdateKind::Concern => "concern".into(),
                                UpdateKind::Other(kind) => kind,
                            },
                            notice: update.notice,
                            date: update.date,
                            source: update.source,
                        })
                        .collect(),
                    error: check.error,
                });
            }
            Ok(records)
        })
        .await
    }
}

async fn record(engine: &Engine, reference: &quire_model::Reference) -> Result<ReferenceRecord> {
    query::reference(engine, &reference.id.to_string()).await
}
