//! An in-memory [`Library`] for tests of the use cases.

use std::collections::BTreeSet;

use quire_model::{CiteKey, DeviceId, Identifier, Project, ProjectId, Reference, ReferenceId, Slug};

use crate::error::Result;
use crate::ports::{Library, RawRecord, ReferenceFilter};

pub struct MemoryLibrary {
    device: DeviceId,
    references: Vec<Reference>,
    projects: Vec<Project>,
    links: BTreeSet<(ReferenceId, ProjectId)>,
}

impl Default for MemoryLibrary {
    fn default() -> Self {
        Self { device: DeviceId::new(), references: Vec::new(), projects: Vec::new(), links: BTreeSet::new() }
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
        Ok(self.references.iter().find(|r| &r.cite_key == key && !r.meta.is_deleted()).cloned())
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

    fn references(&self, filter: &ReferenceFilter) -> Result<Vec<Reference>> {
        let in_projects = |r: &Reference| match &filter.projects {
            Some(projects) => projects.iter().any(|p| self.links.contains(&(r.id, *p))),
            None => true,
        };
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
        Ok(self.links.iter().filter(|(r, _)| *r == reference).map(|(_, p)| *p).collect())
    }
}
