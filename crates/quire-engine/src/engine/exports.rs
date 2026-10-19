//! Exports of [`Engine`]: writing references through an export profile, and automatic exports
//! that are rewritten after every change (`.knowledge/roadmap.md`, M6).

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use quire_core::library::projects;
use quire_core::{AutoExport, Library, ReferenceFilter};
use quire_io::{ExportFormat, ExportOptions, PreprintForm};
use quire_model::{Reference, SlugPath, Timestamp};

use super::Engine;
use crate::error::{EngineError, Result};

/// The profile used when none is named, if the config defines it.
pub const DEFAULT_PROFILE: &str = "default";

/// What to export and how.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExportRequest {
    /// `None`: the whole library.
    pub project: Option<SlugPath>,
    pub recursive: bool,
    /// Export profile from the config; `None` uses `default` if defined.
    pub profile: Option<String>,
    /// Overrides the profile's format.
    pub format: Option<ExportFormat>,
}

/// An automatic export, with the path of its project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoExportInfo {
    pub export: AutoExport,
    /// `None`: the whole library (or a project removed since).
    pub project: Option<SlugPath>,
}

/// What happened to one automatic export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoExportOutcome {
    /// Rewritten with this many references.
    Written(usize),
    /// Content unchanged; the file was not touched.
    Unchanged,
    Failed(String),
}

impl Engine {
    /// The references `request` selects, written as its profile and format say; also returns
    /// how many were written.
    pub async fn export(&self, request: &ExportRequest) -> Result<(String, usize)> {
        let options = self.export_options(request.profile.as_deref(), request.format)?;
        let references = self.select(request.project.clone(), request.recursive).await?;
        Ok((quire_io::export(&references, &options)?, references.len()))
    }

    /// The options of profile `name` (or `default` if defined), with `format` overriding.
    pub fn export_options(&self, name: Option<&str>, format: Option<ExportFormat>) -> Result<ExportOptions> {
        let profiles = &self.config.export;
        let profile = match name {
            Some(name) => Some(profiles.get(name).ok_or_else(|| {
                let known: Vec<&str> = profiles.keys().map(String::as_str).collect();
                EngineError::Config(format!(
                    "no export profile `{name}` (known: {})",
                    if known.is_empty() { "none".into() } else { known.join(", ") }
                ))
            })?),
            None => profiles.get(DEFAULT_PROFILE),
        };
        let mut options = ExportOptions::default();
        if let Some(profile) = profile {
            if let Some(format) = &profile.format {
                options.format = format.parse()?;
            }
            options.omit = profile.omit.clone();
            options.ascii = profile.ascii;
            options.month_macros = profile.months.as_deref() == Some("macro");
            if profile.preprints.as_deref() == Some("article") {
                options.preprints = PreprintForm::Article;
            }
        }
        if let Some(format) = format {
            options.format = format;
        }
        Ok(options)
    }

    /// The references to export, each with its tags as keywords (`.bib` `keywords`, CSL
    /// `keyword`), unless it already has keywords from an imported file.
    async fn select(&self, project: Option<SlugPath>, recursive: bool) -> Result<Vec<Reference>> {
        self.run(move |lib| {
            let mut references = match &project {
                Some(path) => projects::references(lib, path, recursive)?,
                None => lib.references(&ReferenceFilter::default())?,
            };
            for reference in &mut references {
                let names: Vec<String> = lib.tags_of(reference.id)?.into_iter().map(|(tag, _)| tag.name).collect();
                if !names.is_empty() {
                    let joined = names.join(", ");
                    reference
                        .data
                        .extra
                        .entry(super::references::KEYWORDS_EXTRA.into())
                        .or_insert_with(|| joined.clone());
                    reference.data.extra.entry("keyword".into()).or_insert(joined);
                }
            }
            Ok(references)
        })
        .await
    }

    /// Writes `request` to `path` now and registers it to be rewritten after every change.
    pub async fn keep_updated(&self, request: &ExportRequest, path: &Path) -> Result<(PathBuf, usize)> {
        let path = std::path::absolute(path).map_err(EngineError::file(path))?;
        let (text, count) = self.export(request).await?;
        write_if_changed(&path, &text)?;
        let project = self.project_id(request.project.as_ref()).await?;
        let export = AutoExport {
            path: path.display().to_string(),
            project,
            recursive: request.recursive,
            profile: request.profile.clone(),
            format: request.format.map(|f| f.as_str().to_string()),
            written_at: Some(Timestamp::now()),
        };
        self.run(move |lib| lib.set_auto_export(&export)).await?;
        Ok((path, count))
    }

    /// Every automatic export of this library.
    pub async fn auto_exports(&self) -> Result<Vec<AutoExportInfo>> {
        self.run(|lib| {
            let exports = lib.auto_exports()?;
            Ok(exports
                .into_iter()
                .map(|export| {
                    let project = export.project.and_then(|id| projects::path_of(lib, id).ok());
                    AutoExportInfo { export, project }
                })
                .collect())
        })
        .await
    }

    /// Stops rewriting `path` (the file stays); `false` if it was not an automatic export.
    pub async fn stop_auto_export(&self, path: &Path) -> Result<bool> {
        let path = std::path::absolute(path).map_err(EngineError::file(path))?.display().to_string();
        self.run(move |lib| lib.remove_auto_export(&path)).await
    }

    /// Rewrites every automatic export whose content changed, if anything changed in the library
    /// since the last run (or `force`). A failing export does not stop the others.
    pub async fn run_auto_exports(&self, force: bool) -> Result<Vec<(String, AutoExportOutcome)>> {
        if !self.changed.swap(false, Ordering::SeqCst) && !force {
            return Ok(Vec::new());
        }
        let mut results = Vec::new();
        for info in self.auto_exports().await? {
            let export = info.export;
            let outcome = match self.write_auto_export(&export, info.project).await {
                Ok(outcome) => outcome,
                Err(error) => AutoExportOutcome::Failed(error.to_string()),
            };
            if matches!(outcome, AutoExportOutcome::Written(_)) {
                let path = export.path.clone();
                self.run(move |lib| lib.mark_auto_export_written(&path, Timestamp::now())).await?;
            }
            results.push((export.path, outcome));
        }
        Ok(results)
    }

    async fn write_auto_export(&self, export: &AutoExport, project: Option<SlugPath>) -> Result<AutoExportOutcome> {
        if export.project.is_some() && project.is_none() {
            return Ok(AutoExportOutcome::Failed("its project was removed".into()));
        }
        let format = export.format.as_deref().map(str::parse).transpose()?;
        let request = ExportRequest { project, recursive: export.recursive, profile: export.profile.clone(), format };
        let (text, count) = self.export(&request).await?;
        Ok(if write_if_changed(Path::new(&export.path), &text)? {
            AutoExportOutcome::Written(count)
        } else {
            AutoExportOutcome::Unchanged
        })
    }

    /// Notes that the library changed: automatic exports are rewritten, subscribers are told.
    pub(crate) fn mark_changed(&self) {
        self.changed.store(true, Ordering::SeqCst);
        // No subscriber is not an error.
        let _ = self.changes.send(super::Change { at: quire_model::Timestamp::now().as_millis() });
    }
}

/// Writes `text` to `path` unless the file already holds it (keeps its modification time for
/// `typst watch` / latexmk). Returns whether it wrote.
pub fn write_if_changed(path: &Path, text: &str) -> Result<bool> {
    if std::fs::read_to_string(path).is_ok_and(|current| current == text) {
        return Ok(false);
    }
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(EngineError::file(dir))?;
    }
    std::fs::write(path, text).map_err(EngineError::file(path))?;
    Ok(true)
}
