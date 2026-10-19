//! [`SqliteLibrary`]: the `quire-core` [`Library`] port on a SQLite connection (usually a
//! transaction, so one use case commits or rolls back as a whole).
//!
//! - [`codec`]: IDs, times and JSON ↔ SQLite values.
//! - [`references`], [`projects`], [`links`]: the queries, one module per table group.

mod codec;
mod links;
mod projects;
mod references;

use quire_core::{Library, RawRecord, ReferenceFilter, Result};
use quire_model::{CiteKey, DeviceId, Identifier, Project, ProjectId, Reference, ReferenceId, Slug};
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
}
