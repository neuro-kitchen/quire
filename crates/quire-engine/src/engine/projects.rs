//! Project operations of [`Engine`].

use quire_core::library::projects;
use std::collections::HashMap;

use quire_model::{Project, ProjectId, Reference, ReferenceId, SlugPath};

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
            let mut counts: HashMap<ProjectId, usize> = HashMap::new();
            for (_, project) in lib.memberships()? {
                *counts.entry(project).or_default() += 1;
            }
            let tree = projects::tree(lib)?;
            Ok(tree
                .into_iter()
                .map(|(path, project)| {
                    let references = counts.get(&project.id).copied().unwrap_or(0);
                    ProjectSummary { path, project, references }
                })
                .collect())
        })
        .await
    }

    /// The projects each live reference is in, by path (references in none are left out).
    pub async fn memberships(&self) -> Result<HashMap<ReferenceId, Vec<SlugPath>>> {
        self.run(|lib| {
            let paths: HashMap<ProjectId, SlugPath> =
                projects::tree(lib)?.into_iter().map(|(path, project)| (project.id, path)).collect();
            let mut out: HashMap<ReferenceId, Vec<SlugPath>> = HashMap::new();
            for (reference, project) in lib.memberships()? {
                if let Some(path) = paths.get(&project) {
                    out.entry(reference).or_default().push(path.clone());
                }
            }
            Ok(out)
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
