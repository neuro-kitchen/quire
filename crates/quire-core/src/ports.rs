//! The storage port: what the use cases need from a library store. `quire-store` implements it
//! on SQLite; tests use the in-memory [`MemoryLibrary`](crate::memory::MemoryLibrary).
//!
//! Calls are synchronous: the engine runs them on the database thread, inside one transaction
//! per use case.

use quire_model::{CiteKey, DeviceId, Identifier, Project, ProjectId, Reference, ReferenceId, Slug, Timestamp};

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

/// Which references to list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceFilter {
    /// Only references linked to one of these projects (all references if `None`).
    pub projects: Option<Vec<ProjectId>>,
    /// Also references in the trash.
    pub include_deleted: bool,
}

/// A library store. Reads skip records in the trash unless stated otherwise.
pub trait Library {
    /// The device this library copy belongs to (recorded on every change).
    fn device(&self) -> DeviceId;

    // References
    /// By ID, including references in the trash.
    fn reference(&self, id: ReferenceId) -> Result<Option<Reference>>;
    fn reference_by_cite_key(&self, key: &CiteKey) -> Result<Option<Reference>>;
    /// The reference holding an identifier with the same kind and key, including the trash.
    fn reference_by_identifier(&self, identifier: &Identifier) -> Result<Option<Reference>>;
    /// Whether any reference (also in the trash) uses `key`.
    fn cite_key_taken(&self, key: &str) -> Result<bool>;
    fn insert_reference(&mut self, reference: &Reference, raw: Option<&RawRecord>) -> Result<()>;
    fn update_reference(&mut self, reference: &Reference) -> Result<()>;
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
    /// Projects `reference` is linked to.
    fn projects_of(&self, reference: ReferenceId) -> Result<Vec<ProjectId>>;
}
