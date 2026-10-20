//! Records the store keeps that are not domain types: raw responses, failed inputs, automatic
//! exports, search hits, and the filter for listing references.

use quire_model::{ProjectId, ReferenceId, TagId, Timestamp};

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

/// A reference found by a full-text search.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub reference: ReferenceId,
    /// Higher is better.
    pub score: f64,
    /// The matching words in context, marked `«…»`.
    pub snippet: String,
    /// Found only in the paper's text (not its metadata, tags or notes).
    pub in_text: bool,
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
