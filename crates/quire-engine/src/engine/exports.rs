//! Exports of [`Engine`]: writing references through an export profile, and automatic exports
//! that are rewritten after every change (`.knowledge/roadmap.md`, M6).

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use quire_core::library::{projects, references};
use quire_core::prelude::*;
use quire_core::{AutoExport, ReferenceFilter};
use quire_io::{ExportFormat, ExportOptions, PreprintForm};
use quire_model::{Reference, SlugPath, Timestamp};

use super::Engine;
use crate::config::ExportProfile;
use crate::error::{EngineError, Result};

/// The profile used when none is named, if the config defines it.
pub const DEFAULT_PROFILE: &str = "default";

/// What to export and how.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExportRequest {
    /// `None`: the whole library.
    pub project: Option<SlugPath>,
    pub recursive: bool,
    /// These references (IDs or cite keys) instead of a project or the library.
    pub references: Vec<String>,
    /// Export profile from the config; `None` uses `default` if defined.
    pub profile: Option<String>,
    /// Options given directly (the app's export dialog), instead of a profile.
    pub options: Option<ExportProfile>,
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
        let options = self.request_options(request)?;
        let references = self.select(request).await?;
        Ok((quire_io::export(&references, &options)?, references.len()))
    }

    /// Writes `request` to `path` once, in its encoding, unless the file already holds it; returns
    /// the file, the count, and whether it wrote.
    pub async fn export_to(&self, request: &ExportRequest, path: &Path) -> Result<(PathBuf, usize, bool)> {
        let path = std::path::absolute(path).map_err(EngineError::file(path))?;
        let (bytes, count) = self.export_bytes(request).await?;
        let wrote = write_bytes_if_changed(&path, &bytes)?;
        Ok((path, count, wrote))
    }

    /// [`Self::export`] as the file's bytes.
    async fn export_bytes(&self, request: &ExportRequest) -> Result<(Vec<u8>, usize)> {
        let encoding = self.request_options(request)?.encoding;
        let (text, count) = self.export(request).await?;
        Ok((quire_io::encoding::encode(&text, encoding), count))
    }

    /// The options `request` asks for: given directly, or its profile's; its format overriding.
    fn request_options(&self, request: &ExportRequest) -> Result<ExportOptions> {
        match &request.options {
            Some(options) => options_of(Some(options), request.format),
            None => self.export_options(request.profile.as_deref(), request.format),
        }
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
        options_of(profile, format)
    }

    /// The references to export, each with its tags as keywords (`.bib` `keywords`, CSL
    /// `keyword`), unless it already has keywords from an imported file.
    async fn select(&self, request: &ExportRequest) -> Result<Vec<Reference>> {
        let (project, recursive, chosen) = (request.project.clone(), request.recursive, request.references.clone());
        self.run(move |lib| {
            let mut references = if !chosen.is_empty() {
                chosen.iter().map(|query| references::find(lib, query)).collect::<quire_core::Result<Vec<_>>>()?
            } else {
                match &project {
                    Some(path) => projects::references(lib, path, recursive)?,
                    None => lib.references(&ReferenceFilter::default())?,
                }
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
        if !request.references.is_empty() {
            return Err(EngineError::Library(
                "a selection cannot be kept updated; keep a project or the library updated".into(),
            ));
        }
        let path = std::path::absolute(path).map_err(EngineError::file(path))?;
        let (bytes, count) = self.export_bytes(request).await?;
        write_bytes_if_changed(&path, &bytes)?;
        let options = request.options.as_ref().map(serde_json::to_string).transpose().map_err(|e| EngineError::Library(e.to_string()))?;
        let project = self.project_id(request.project.as_ref()).await?;
        let export = AutoExport {
            path: path.display().to_string(),
            project,
            recursive: request.recursive,
            profile: request.profile.clone(),
            format: request.format.map(|f| f.as_str().to_string()),
            options,
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
        let options = export
            .options
            .as_deref()
            .map(serde_json::from_str::<ExportProfile>)
            .transpose()
            .map_err(|e| EngineError::Library(format!("its stored options cannot be read: {e}")))?;
        let request = ExportRequest {
            project,
            recursive: export.recursive,
            profile: export.profile.clone(),
            options,
            format,
            ..ExportRequest::default()
        };
        let (bytes, count) = self.export_bytes(&request).await?;
        Ok(if write_bytes_if_changed(Path::new(&export.path), &bytes)? {
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
    write_bytes_if_changed(path, text.as_bytes())
}

/// [`write_if_changed`] for a file's bytes (an export in its encoding).
pub fn write_bytes_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    if std::fs::read(path).is_ok_and(|current| current == bytes) {
        return Ok(false);
    }
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(EngineError::file(dir))?;
    }
    std::fs::write(path, bytes).map_err(EngineError::file(path))?;
    Ok(true)
}

/// The export options `profile` says (defaults without one), `format` overriding its format.
fn options_of(profile: Option<&ExportProfile>, format: Option<ExportFormat>) -> Result<ExportOptions> {
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
        if let Some(encoding) = &profile.encoding {
            options.encoding = encoding.parse()?;
        }
    }
    if let Some(format) = format {
        options.format = format;
    }
    Ok(options)
}
