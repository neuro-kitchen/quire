//! Moving references in and out: Zotero import, exports (once or kept up to date), highlights and
//! notes as Markdown, and checking a manuscript's cite keys against the library.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use quire_engine::{AutoExportOutcome, ExportFormat, ExportRequest};

use crate::{QuireError, QuireLibrary, Result, query};

/// What a Zotero import brought in.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ZoteroImportRecord {
    pub added: u32,
    /// Already in the library (same identifier).
    pub existing: u32,
    pub projects: u32,
    pub notes: u32,
    pub pdfs: u32,
    pub highlights: u32,
    /// Items not imported, with why.
    pub skipped: Vec<String>,
}

/// What to export: a project (with subprojects if `recursive`) or the library; a profile from the
/// config and/or a format (`biblatex`, `bibtex`, `csl-json`, `hayagriva`).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ExportSelection {
    pub project: Option<String>,
    pub recursive: bool,
    pub profile: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ExportedText {
    pub text: String,
    pub count: u32,
}

/// A file rewritten after every change to the library.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AutoExportRecord {
    pub path: String,
    pub project: Option<String>,
    pub recursive: bool,
    pub profile: Option<String>,
    pub format: Option<String>,
    /// When it was last written (`YYYY-MM-DD`).
    pub written: Option<String>,
}

/// One automatic export's run: `written` (with `count`), `unchanged` or `failed` (with `error`).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AutoExportRun {
    pub path: String,
    pub outcome: String,
    pub count: u32,
    pub error: Option<String>,
}

/// A paper's highlights and notes as a Markdown file's text.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AnnotationsRecord {
    pub id: String,
    pub cite_key: String,
    pub markdown: String,
}

/// Where a manuscript cites a key: file, line and column (from 1).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct CiteUseRecord {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// A cite key the library does not have, with keys it may mean.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct UnknownKeyRecord {
    pub key: String,
    pub uses: Vec<CiteUseRecord>,
    pub suggestions: Vec<String>,
    /// It names a reference in the trash.
    pub trashed: bool,
}

/// A manuscript checked against the library (or a project).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ManuscriptRecord {
    pub files: Vec<String>,
    pub cited: u32,
    pub unknown: Vec<UnknownKeyRecord>,
    /// Cite keys of references cited but outside the project.
    pub outside: Vec<String>,
    /// Cite keys of the project's references not cited.
    pub uncited: Vec<String>,
}

/// The export formats, by name.
#[uniffi::export]
pub fn export_formats() -> Vec<String> {
    ExportFormat::ALL.iter().map(|f| f.as_str().to_string()).collect()
}

#[uniffi::export]
impl QuireLibrary {
    /// Imports the Zotero library in the data folder `dir` (collections under `project` if given;
    /// `base` the folder relative attachment paths start from).
    pub async fn import_zotero(
        &self,
        dir: String,
        project: Option<String>,
        base: Option<String>,
    ) -> Result<ZoteroImportRecord> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let base = base.map(PathBuf::from);
            let import = engine.import_zotero(Path::new(&dir), project.as_ref(), base.as_deref()).await?;
            Ok(ZoteroImportRecord {
                added: import.added.len() as u32,
                existing: import.existing as u32,
                projects: import.projects as u32,
                notes: import.notes as u32,
                pdfs: import.pdfs as u32,
                highlights: import.highlights as u32,
                skipped: import.skipped,
            })
        })
        .await
    }

    /// The selected references written as the profile and format say.
    pub async fn export(&self, selection: ExportSelection) -> Result<ExportedText> {
        self.with(|engine| async move {
            let (text, count) = engine.export(&request(&selection)?).await?;
            Ok(ExportedText { text, count: count as u32 })
        })
        .await
    }

    /// Writes the selection to `path` now and after every change; returns the file and the count.
    pub async fn keep_updated(&self, selection: ExportSelection, path: String) -> Result<ExportedText> {
        self.with(|engine| async move {
            let (path, count) = engine.keep_updated(&request(&selection)?, Path::new(&path)).await?;
            Ok(ExportedText { text: path.display().to_string(), count: count as u32 })
        })
        .await
    }

    pub async fn auto_exports(&self) -> Result<Vec<AutoExportRecord>> {
        self.with(|engine| async move {
            Ok(engine
                .auto_exports()
                .await?
                .into_iter()
                .map(|info| AutoExportRecord {
                    path: info.export.path,
                    project: info.project.map(|p| p.to_string()),
                    recursive: info.export.recursive,
                    profile: info.export.profile,
                    format: info.export.format,
                    written: info.export.written_at.map(|t| t.date().to_string()),
                })
                .collect())
        })
        .await
    }

    /// Stops rewriting `path` (the file stays); `false` if it was not an automatic export.
    pub async fn stop_auto_export(&self, path: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.stop_auto_export(Path::new(&path)).await?) }).await
    }

    /// Rewrites the automatic exports whose content changed (every one with `force`).
    pub async fn run_auto_exports(&self, force: bool) -> Result<Vec<AutoExportRun>> {
        self.with(|engine| async move {
            Ok(engine
                .run_auto_exports(force)
                .await?
                .into_iter()
                .map(|(path, outcome)| match outcome {
                    AutoExportOutcome::Written(count) => {
                        AutoExportRun { path, outcome: "written".into(), count: count as u32, error: None }
                    }
                    AutoExportOutcome::Unchanged => {
                        AutoExportRun { path, outcome: "unchanged".into(), count: 0, error: None }
                    }
                    AutoExportOutcome::Failed(error) => {
                        AutoExportRun { path, outcome: "failed".into(), count: 0, error: Some(error) }
                    }
                })
                .collect())
        })
        .await
    }

    /// Each paper with highlights or notes in `project` (or the library), as Markdown.
    pub async fn annotations_markdown(
        &self,
        project: Option<String>,
        recursive: bool,
    ) -> Result<Vec<AnnotationsRecord>> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            Ok(engine
                .annotations_markdown(project.as_ref(), recursive)
                .await?
                .into_iter()
                .map(|(reference, markdown)| AnnotationsRecord {
                    id: reference.id.to_string(),
                    cite_key: reference.cite_key.to_string(),
                    markdown,
                })
                .collect())
        })
        .await
    }

    /// Checks the cite keys of `files` (`.typ`, `.tex`, `.md`; folders are searched) against the
    /// library, or `project` (with subprojects if `recursive`).
    pub async fn check_manuscript(
        &self,
        files: Vec<String>,
        project: Option<String>,
        recursive: bool,
    ) -> Result<ManuscriptRecord> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let files: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
            let check = engine.check_manuscript(&files, project, recursive).await?;
            let uses = |uses: Vec<quire_engine::CiteUse>| {
                uses.into_iter()
                    .map(|u| CiteUseRecord {
                        file: u.file.display().to_string(),
                        line: u.line as u32,
                        column: u.column as u32,
                    })
                    .collect()
            };
            Ok(ManuscriptRecord {
                files: check.files.iter().map(|f| f.display().to_string()).collect(),
                cited: check.cited as u32,
                unknown: check
                    .unknown
                    .into_iter()
                    .map(|k| UnknownKeyRecord {
                        key: k.key,
                        uses: uses(k.uses),
                        suggestions: k.suggestions,
                        trashed: k.trashed,
                    })
                    .collect(),
                outside: check.outside.into_iter().map(|(r, _)| r.cite_key.to_string()).collect(),
                uncited: check.uncited.into_iter().map(|r| r.cite_key.to_string()).collect(),
            })
        })
        .await
    }
}

fn request(selection: &ExportSelection) -> Result<ExportRequest> {
    Ok(ExportRequest {
        project: selection.project.as_deref().map(query::slug_path).transpose()?,
        recursive: selection.recursive,
        profile: selection.profile.clone(),
        format: selection
            .format
            .as_deref()
            .map(|f| {
                ExportFormat::from_str(f)
                    .map_err(|_| QuireError::Invalid(format!("`{f}`: {}", export_formats().join(", "))))
            })
            .transpose()?,
    })
}
