//! Reference operations of [`Engine`].

use std::path::Path;

use quire_core::library::{projects, references};
use quire_core::{AddRequest, Added, Library, ReferenceFilter};
use quire_fetch::Lookup;
use quire_model::{EntryKind, Origin, ProjectId, Reference, ReferenceData, SlugPath};

use super::Engine;
use crate::error::{EngineError, Result};

impl Engine {
    /// Adds what `input` names (DOI, arXiv ID, PMID, PMCID, or a URL), linked to `project`. A
    /// work already in the library is not fetched again; it is only linked (and restored from
    /// the trash).
    pub async fn add(&self, input: &str, project: Option<&SlugPath>) -> Result<Added> {
        let lookup = Lookup::parse(input)?;
        let project = self.project_id(project).await?;
        if let Lookup::Identifier(identifier) = &lookup {
            let known = identifier.clone();
            if self.run(move |lib| lib.reference_by_identifier(&known)).await?.is_some() {
                let mut data = ReferenceData::new(EntryKind::Document);
                data.identifiers.push(identifier.clone());
                return self.store(AddRequest { project, ..AddRequest::new(data, Origin::User) }).await;
            }
        }
        let fetched = self.fetcher.fetch(&lookup).await?;
        let origin = Origin::Fetched { source: fetched.raw.source.clone() };
        let request = AddRequest { raw: Some(fetched.raw), project, ..AddRequest::new(fetched.data, origin) };
        self.store(request).await
    }

    /// Imports every entry of the `.bib` file at `path`, keeping its cite keys where valid and
    /// free; all or nothing (one transaction). Entries already in the library (same identifier)
    /// are linked, not duplicated.
    pub async fn import_bib(&self, path: &Path, project: Option<&SlugPath>) -> Result<Vec<Added>> {
        let text = std::fs::read_to_string(path).map_err(EngineError::file(path))?;
        let records = quire_io::from_bib(&text)?;
        let project = self.project_id(project).await?;
        let file = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
        self.run(move |lib| {
            let requests = records.into_iter().map(|record| AddRequest {
                project,
                preferred_key: Some(record.cite_key),
                ..AddRequest::new(record.data, Origin::Imported { file: file.clone() })
            });
            requests.map(|request| references::add(lib, request)).collect()
        })
        .await
    }

    /// References in `project` (and its subprojects if `recursive`), or the whole library; with
    /// `trash`, only references in the trash.
    pub async fn list(&self, project: Option<&SlugPath>, recursive: bool, trash: bool) -> Result<Vec<Reference>> {
        let project = project.cloned();
        self.run(move |lib| match &project {
            Some(path) if !trash => projects::references(lib, path, recursive),
            _ => {
                let filter = ReferenceFilter { projects: None, include_deleted: trash };
                let all = lib.references(&filter)?;
                Ok(if trash { all.into_iter().filter(|r| r.meta.is_deleted()).collect() } else { all })
            }
        })
        .await
    }

    /// The reference `query` names (ID, cite key or identifier) and the paths of its projects.
    pub async fn show(&self, query: &str) -> Result<(Reference, Vec<SlugPath>)> {
        let query = query.to_string();
        self.run(move |lib| {
            let reference = references::find(lib, &query)?;
            let ids = lib.projects_of(reference.id)?;
            let paths = ids.into_iter().map(|id| projects::path_of(lib, id)).collect::<quire_core::Result<_>>()?;
            Ok((reference, paths))
        })
        .await
    }

    /// Moves a reference to the trash.
    pub async fn trash(&self, query: &str) -> Result<Reference> {
        let query = query.to_string();
        self.run(move |lib| references::trash(lib, &query)).await
    }

    /// Takes a reference out of the trash.
    pub async fn restore(&self, query: &str) -> Result<Reference> {
        let query = query.to_string();
        self.run(move |lib| references::restore(lib, &query)).await
    }

    async fn store(&self, request: AddRequest) -> Result<Added> {
        self.run(move |lib| references::add(lib, request)).await
    }

    /// The ID of the project at `path`; checked before fetching, so a typo costs no request.
    pub(super) async fn project_id(&self, path: Option<&SlugPath>) -> Result<Option<ProjectId>> {
        let Some(path) = path.cloned() else { return Ok(None) };
        Ok(Some(self.run(move |lib| projects::resolve(lib, &path).map(|p| p.id)).await?))
    }
}
