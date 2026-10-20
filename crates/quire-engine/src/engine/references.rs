//! Reference operations of [`Engine`].

use std::path::Path;

use quire_core::library::{projects, references, tags};
use quire_core::prelude::*;
use quire_core::{AddRequest, Added, Checked, FailedInput, ReferenceFilter, check};
use quire_fetch::Lookup;
use quire_model::{EntryKind, Issue, Origin, ProjectId, Reference, ReferenceData, SlugPath, TagKind};

use super::Engine;
use crate::error::{EngineError, Result};

impl Engine {
    /// Adds what `input` names (DOI, arXiv ID, PMID, PMCID, or a URL), linked to `project`. A
    /// work already in the library is not fetched again; it is only linked (and restored from
    /// the trash).
    ///
    /// A failed lookup is recorded (see [`Engine::failures`]) so it can be retried; a success
    /// clears it.
    pub async fn add(&self, input: &str, project: Option<&SlugPath>) -> Result<Added> {
        let lookup = Lookup::parse(input)?;
        let project = self.project_id(project).await?;
        if let Lookup::Identifier(identifier) = &lookup {
            let known = identifier.clone();
            if self.run(move |lib| lib.reference_by_identifier(&known)).await?.is_some() {
                let mut data = ReferenceData::new(EntryKind::Document);
                data.identifiers.push(identifier.clone());
                return self.store(input, AddRequest { project, ..AddRequest::new(data, Origin::User) }).await;
            }
        }
        let fetched = match self.fetcher.fetch(&lookup).await {
            Ok(fetched) => fetched,
            Err(error) => {
                let (input, message) = (input.to_string(), error.to_string());
                self.run(move |lib| lib.record_failure(&input, project, &message)).await?;
                return Err(error.into());
            }
        };
        let origin = Origin::Fetched { source: fetched.raw.source.clone() };
        let request = AddRequest {
            raw: Some(fetched.raw),
            project,
            issues: fetched.issues,
            ..AddRequest::new(fetched.data, origin)
        };
        self.store(input, request).await
    }

    /// What `input` names (DOI, arXiv ID, PMID, PMCID, or a URL), looked up but not stored: a
    /// preview before [`Engine::add`].
    pub async fn lookup(&self, input: &str) -> Result<ReferenceData> {
        let lookup = Lookup::parse(input)?;
        Ok(self.fetcher.fetch(&lookup).await?.data)
    }

    /// Adds a reference the user typed in, linked to `project`. As for any added reference, one
    /// with an identifier already here is only linked; look-alikes without one are left to
    /// [`Engine::duplicates`].
    pub async fn add_manual(&self, data: ReferenceData, project: Option<&SlugPath>) -> Result<Added> {
        let project = self.project_id(project).await?;
        let added = self
            .run(move |lib| references::add(lib, AddRequest { project, ..AddRequest::new(data, Origin::User) }))
            .await?;
        self.mark_changed();
        Ok(added)
    }

    /// Adds the work the page at `url` describes, from the page's `html` as the user's browser
    /// loaded it (no request to the page itself, so "are you human" checks do not apply). An
    /// identifier on the page is looked up as usual.
    pub async fn add_from_page(&self, url: &str, html: &str, project: Option<&SlugPath>) -> Result<Added> {
        let parsed = url::Url::parse(url).map_err(|_| quire_fetch::FetchError::InvalidUrl(url.into()))?;
        let page = quire_fetch::web::read(html, &parsed)?;
        let project = self.project_id(project).await?;
        if let quire_fetch::web::PageResult::Identifier { identifier, .. } = &page {
            let known = identifier.clone();
            if self.run(move |lib| lib.reference_by_identifier(&known)).await?.is_some() {
                let mut data = ReferenceData::new(EntryKind::Document);
                data.identifiers.push(identifier.clone());
                return self.store(url, AddRequest { project, ..AddRequest::new(data, Origin::User) }).await;
            }
        }
        let fetched = self.fetcher.from_page(page).await?;
        let origin = Origin::Fetched { source: fetched.raw.source.clone() };
        let request = AddRequest {
            raw: Some(fetched.raw),
            project,
            issues: fetched.issues,
            ..AddRequest::new(fetched.data, origin)
        };
        self.store(url, request).await
    }

    /// Inputs whose lookup failed, with the path of the project they were for.
    pub async fn failures(&self) -> Result<Vec<(FailedInput, Option<SlugPath>)>> {
        self.run(|lib| {
            let failures = lib.failures()?;
            failures
                .into_iter()
                .map(|failure| {
                    // A project removed since then is forgotten, not an error.
                    let path = failure.project.and_then(|id| projects::path_of(lib, id).ok());
                    Ok((failure, path))
                })
                .collect()
        })
        .await
    }

    /// Forgets a failed input (e.g. a typo the user will not retry); `false` if unknown.
    pub async fn forget_failure(&self, input: &str) -> Result<bool> {
        let input = input.to_string();
        self.run(move |lib| lib.clear_failure(&input)).await
    }

    /// The references in `project` (and its subprojects if `recursive`), or the whole library,
    /// that have issues (recorded or found by checking their fields).
    pub async fn check(&self, project: Option<&SlugPath>, recursive: bool) -> Result<Vec<Checked>> {
        let references = self.list(project, recursive, false).await?;
        self.run(move |lib| check::check(lib, references)).await
    }

    /// Like [`Engine::list`], each reference with its issues (for marking them in lists).
    pub async fn list_with_issues(
        &self,
        project: Option<&SlugPath>,
        recursive: bool,
        trash: bool,
        tags: &[String],
    ) -> Result<Vec<Checked>> {
        let references = self.list_tagged(project, recursive, trash, tags).await?;
        self.run(move |lib| check::with_issues(lib, references)).await
    }

    /// Recorded issues and lints of one reference.
    pub async fn issues(&self, reference: &Reference) -> Result<Vec<Issue>> {
        let reference = reference.clone();
        self.run(move |lib| check::issues(lib, &reference)).await
    }

    /// Imports every entry of the `.bib` file at `path`, keeping its cite keys where valid and
    /// free; all or nothing (one transaction). Entries already in the library (same identifier)
    /// are linked, not duplicated.
    pub async fn import_bib(&self, path: &Path, project: Option<&SlugPath>) -> Result<Vec<Added>> {
        let text = std::fs::read_to_string(path).map_err(EngineError::file(path))?;
        let file = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
        self.import_bib_text(&text, &file, project).await
    }

    /// [`Engine::import_bib`] for `.bib` text (pasted, say); `source` names where it came from in the
    /// references' provenance.
    pub async fn import_bib_text(&self, text: &str, source: &str, project: Option<&SlugPath>) -> Result<Vec<Added>> {
        let records = quire_io::from_bib(text)?;
        let project = self.project_id(project).await?;
        let file = source.to_string();
        let added = self
            .run(move |lib| {
                let mut added = Vec::with_capacity(records.len());
                for record in records {
                    // A `.bib` file's `keywords` become automatic tags (as Zotero imports them).
                    let keywords = keyword_list(record.data.extra.get(KEYWORDS_EXTRA).map(String::as_str));
                    let request = AddRequest {
                        project,
                        preferred_key: Some(record.cite_key),
                        ..AddRequest::new(record.data, Origin::Imported { file: file.clone() })
                    };
                    let result = references::add(lib, request)?;
                    for keyword in keywords {
                        tags::add(lib, &result.reference().id.to_string(), &keyword, TagKind::Automatic)?;
                    }
                    added.push(result);
                }
                Ok(added)
            })
            .await?;
        self.mark_changed();
        Ok(added)
    }

    /// References in `project` (and its subprojects if `recursive`), or the whole library; with
    /// `trash`, only references in the trash.
    pub async fn list(&self, project: Option<&SlugPath>, recursive: bool, trash: bool) -> Result<Vec<Reference>> {
        self.list_tagged(project, recursive, trash, &[]).await
    }

    /// [`Engine::list`] limited to references carrying every tag in `tags` (names, any case).
    pub async fn list_tagged(
        &self,
        project: Option<&SlugPath>,
        recursive: bool,
        trash: bool,
        tags: &[String],
    ) -> Result<Vec<Reference>> {
        let (project, tags) = (project.cloned(), tags.to_vec());
        self.run(move |lib| {
            let tag_ids = tags
                .iter()
                .map(|name| tags::find_tag(lib, name).map(|t| t.id))
                .collect::<quire_core::Result<Vec<_>>>()?;
            match &project {
                Some(path) if !trash => {
                    let mut found = projects::references(lib, path, recursive)?;
                    for tag in &tag_ids {
                        let carrying: std::collections::HashSet<_> = lib.tagged(*tag)?.into_iter().collect();
                        found.retain(|r| carrying.contains(&r.id));
                    }
                    Ok(found)
                }
                _ => {
                    let filter =
                        ReferenceFilter { include_deleted: trash, tags: tag_ids, ..ReferenceFilter::default() };
                    let all = lib.references(&filter)?;
                    Ok(if trash { all.into_iter().filter(|r| r.meta.is_deleted()).collect() } else { all })
                }
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

    /// Copies the reference `query` names into the library of `target` (keeping its ID), linked
    /// to `project` there. Already there (same ID or identifier): only linked.
    pub async fn copy_to(&self, query: &str, target: &Engine, project: Option<&SlugPath>) -> Result<Added> {
        if target.library_dir() == self.library_dir() {
            return Err(EngineError::Library("source and target are the same library".into()));
        }
        let (reference, _) = self.show(query).await?;
        let project = target.project_id(project).await?;
        let added = target.run(move |lib| references::copy_in(lib, reference, project)).await?;
        target.mark_changed();
        Ok(added)
    }

    /// Moves a reference to the trash.
    pub async fn trash(&self, query: &str) -> Result<Reference> {
        let query = query.to_string();
        let reference = self.run(move |lib| references::trash(lib, &query)).await?;
        self.mark_changed();
        Ok(reference)
    }

    /// Takes a reference out of the trash.
    pub async fn restore(&self, query: &str) -> Result<Reference> {
        let query = query.to_string();
        let reference = self.run(move |lib| references::restore(lib, &query)).await?;
        self.mark_changed();
        Ok(reference)
    }

    /// Adds `request` and clears any recorded failure of the `input` it came from.
    async fn store(&self, input: &str, request: AddRequest) -> Result<Added> {
        let input = input.to_string();
        let added = self
            .run(move |lib| {
                let added = references::add(lib, request)?;
                lib.clear_failure(&input)?;
                Ok(added)
            })
            .await?;
        self.mark_changed();
        Ok(added)
    }

    /// The ID of the project at `path`; checked before fetching, so a typo costs no request.
    pub(super) async fn project_id(&self, path: Option<&SlugPath>) -> Result<Option<ProjectId>> {
        let Some(path) = path.cloned() else { return Ok(None) };
        Ok(Some(self.run(move |lib| projects::resolve(lib, &path).map(|p| p.id)).await?))
    }
}

/// Where an imported `.bib` file's `keywords` are kept (`quire_io::bib` keeps unmodelled fields
/// under `biblatex.<name>`).
pub(super) const KEYWORDS_EXTRA: &str = "biblatex.keywords";

/// `a, b; c` → `[a, b, c]`: BibLaTeX separates keywords with commas, some tools with semicolons.
pub(super) fn keyword_list(text: Option<&str>) -> Vec<String> {
    text.map(|t| t.split([',', ';']).map(str::trim).filter(|k| !k.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}
