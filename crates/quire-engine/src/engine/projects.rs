//! Project operations of [`Engine`], and export.

use quire_core::library::projects;
use quire_core::{Library, ReferenceFilter};
use quire_io::BibFormat;
use quire_model::{Project, Reference, SlugPath};

use super::Engine;
use crate::error::Result;

/// Output format of [`Engine::export`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    Biblatex,
    Bibtex,
    CslJson,
}

impl Engine {
    /// Creates a project named `name` under `parent` (top level if `None`); returns its path.
    pub async fn create_project(&self, name: &str, parent: Option<&SlugPath>) -> Result<(SlugPath, Project)> {
        let (name, parent) = (name.to_string(), parent.cloned());
        self.run(move |lib| projects::create(lib, &name, parent.as_ref())).await
    }

    /// Every project with its path, as a depth-first tree.
    pub async fn projects(&self) -> Result<Vec<(SlugPath, Project)>> {
        self.run(|lib| projects::tree(lib)).await
    }

    /// Puts the reference `query` into the project at `path`; `false` if it already was.
    pub async fn link(&self, query: &str, path: &SlugPath) -> Result<(Reference, bool)> {
        let (query, path) = (query.to_string(), path.clone());
        self.run(move |lib| projects::link(lib, &query, &path)).await
    }

    /// Takes the reference `query` out of the project at `path` (it stays in the library).
    pub async fn unlink(&self, query: &str, path: &SlugPath) -> Result<(Reference, bool)> {
        let (query, path) = (query.to_string(), path.clone());
        self.run(move |lib| projects::unlink(lib, &query, &path)).await
    }

    /// The references of `project` (and its subprojects if `recursive`) or of the whole library,
    /// written in `format`; also returns how many were written.
    pub async fn export(&self, project: Option<&SlugPath>, recursive: bool, format: ExportFormat) -> Result<(String, usize)> {
        let project = project.cloned();
        let references = self
            .run(move |lib| match &project {
                Some(path) => projects::references(lib, path, recursive),
                None => lib.references(&ReferenceFilter::default()),
            })
            .await?;
        let text = match format {
            ExportFormat::Biblatex => quire_io::to_bib(&references, BibFormat::Biblatex)?,
            ExportFormat::Bibtex => quire_io::to_bib(&references, BibFormat::Bibtex)?,
            ExportFormat::CslJson => quire_io::to_csl_json(&references),
        };
        Ok((text, references.len()))
    }
}
