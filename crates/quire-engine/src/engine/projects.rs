//! Project operations of [`Engine`].

use quire_core::ReferenceFilter;
use quire_core::library::projects;
use quire_core::prelude::*;
use quire_model::{Project, Reference, SlugPath};

use super::Engine;
use crate::error::Result;

/// A project in [`Engine::project_summaries`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummary {
    pub path: SlugPath,
    pub project: Project,
    /// References linked to this project itself (not its subprojects).
    pub references: usize,
}

impl Engine {
    /// Creates a project named `name` under `parent` (top level if `None`); returns its path.
    pub async fn create_project(&self, name: &str, parent: Option<&SlugPath>) -> Result<(SlugPath, Project)> {
        let (name, parent) = (name.to_string(), parent.cloned());
        let created = self.run(move |lib| projects::create(lib, &name, parent.as_ref())).await?;
        self.mark_changed();
        Ok(created)
    }

    /// Every project with its path, as a depth-first tree.
    pub async fn projects(&self) -> Result<Vec<(SlugPath, Project)>> {
        self.run(|lib| projects::tree(lib)).await
    }

    /// Every project as a depth-first tree, with how many references it holds directly.
    pub async fn project_summaries(&self) -> Result<Vec<ProjectSummary>> {
        self.run(|lib| {
            let tree = projects::tree(lib)?;
            tree.into_iter()
                .map(|(path, project)| {
                    let filter = ReferenceFilter { projects: Some(vec![project.id]), ..ReferenceFilter::default() };
                    let references = lib.references(&filter)?.len();
                    Ok(ProjectSummary { path, project, references })
                })
                .collect()
        })
        .await
    }

    /// Renames the project at `path`; returns its new path.
    pub async fn rename_project(&self, path: &SlugPath, name: &str) -> Result<(SlugPath, Project)> {
        let (path, name) = (path.clone(), name.to_string());
        let renamed = self.run(move |lib| projects::rename(lib, &path, &name)).await?;
        self.mark_changed();
        Ok(renamed)
    }

    /// Moves the project at `path` (and its subprojects if `recursive`) to the trash; its
    /// references stay in the library. Returns how many projects were removed.
    pub async fn remove_project(&self, path: &SlugPath, recursive: bool) -> Result<usize> {
        let path = path.clone();
        let removed = self.run(move |lib| projects::remove(lib, &path, recursive)).await?;
        self.mark_changed();
        Ok(removed)
    }

    /// Puts the reference `query` into the project at `path`; `false` if it already was.
    pub async fn link(&self, query: &str, path: &SlugPath) -> Result<(Reference, bool)> {
        let (query, path) = (query.to_string(), path.clone());
        let linked = self.run(move |lib| projects::link(lib, &query, &path)).await?;
        self.mark_changed();
        Ok(linked)
    }

    /// Takes the reference `query` out of the project at `path` (it stays in the library).
    pub async fn unlink(&self, query: &str, path: &SlugPath) -> Result<(Reference, bool)> {
        let (query, path) = (query.to_string(), path.clone());
        let unlinked = self.run(move |lib| projects::unlink(lib, &query, &path)).await?;
        self.mark_changed();
        Ok(unlinked)
    }
}
