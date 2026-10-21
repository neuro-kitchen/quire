//! quire's engine for the Kotlin apps (`app/`), through UniFFI: the Gradle build compiles this
//! crate to a native library, and `uniffi-bindgen` generates its Kotlin API (`suspend` functions,
//! data classes). See `.knowledge/app.md`.
//!
//! The layer is thin on purpose: it opens an [`Engine`], turns its results into flat records the
//! screens use, and answers a [`Query`] (source, words, tags, sort). Every call runs on one Tokio
//! runtime owned here, so the Kotlin side needs no Rust executor.
//!
//! - [`records`]: the records and enums crossing the boundary.
//! - [`config`]: the settings screen (config keys, values, the raw file).
//! - [`reader`]: a paper's reader tabs, highlights, notes and reading status.
//! - [`attachments`]: a reference's PDFs: download, attach, import, status, OCR, the browser.
//! - [`discovery`]: search, saved searches, alerts, the citation graph.
//! - [`curation`]: duplicates, checks, retractions.
//! - [`exchange`]: Zotero import, exports, highlights as Markdown, manuscript checks.
//! - [`libraries`]: libraries in the config, backup, copying references between libraries.
//! - [`sync`]: syncing devices, sharing projects.
//! - [`references`]: adding references (identifier, page, typed in) and changing them.
//! - [`projects`]: renaming and removing projects, linking references to them.
//! - [`tags`]: tags beyond tagging, related references, standalone notes.
//! - [`pdf`]: PDF pages and the text layer (hayro, through `quire-document`), and where
//!   highlighted quotes are on them.
//! - [`query`]: answering a [`Query`] with engine calls.

mod attachments;
mod config;
mod curation;
mod discovery;
mod exchange;
mod hub;
mod libraries;
mod pdf;
mod projects;
mod query;
mod reader;
mod records;
mod references;
mod sync;
mod tags;

use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use quire_engine::{Engine, LibraryChoice, OpenOptions};
use tokio::runtime::Runtime;

pub use attachments::{AttachedRecord, ImportedPdf, PdfFetchRecord, PdfStatusRecord, VerifiedPdf};
pub use config::{
    ConfigEntry, ConfigKey, config_check, config_entries, config_keys, config_path, config_read, config_set,
    config_unset, config_write,
};
pub use curation::{
    CheckedRecord, DuplicateGroupRecord, DuplicateRecord, MergedRecord, NoticeRecord, RetractionRecord,
};
pub use discovery::{AlertCheckRecord, AlertRecord, GraphWorkRecord, SavedSearchRecord, SearchHitRecord};
pub use exchange::{
    AnnotationsRecord, AutoExportRecord, AutoExportRun, CiteUseRecord, ExportOptionsRecord, ExportProfileRecord,
    ExportSelection, ExportedText, ManuscriptRecord, UnknownKeyRecord, ZoteroImportRecord, export_encodings,
    export_formats,
};
pub use hub::{EvictionRecord, HubDeviceRecord, HubReportRecord, OpenedPdfRecord};
pub use libraries::{
    BackupRecord, LibraryEntryRecord, list_libraries, register_library, rename_library, set_default_library,
    unregister_library,
};
pub use pdf::{
    FigureRegion, PageRect, PageSize, PdfWordRecord, QuoteMatch, RenderedPage, TextMatch, TextRangeRecord, pdf_figures,
    pdf_find, pdf_nearest_glyph, pdf_page_sizes, pdf_page_text, pdf_render_page, pdf_search, pdf_text_range,
    pdf_word_at,
};
pub use reader::{BlockKind, BlockRecord, HighlightRecord, NoteRecord, PaperRecord, ReaderTab, WordRecord};
pub use records::{Creator, LibraryInfo, ProjectRecord, Query, ReferenceRecord, SortColumn, Source, TagRecord};
pub use references::{AddResult, CreatorInput, FailureRecord, FieldChangeRecord, IssueRecord, ReferenceFields};
pub use sync::{MemberRecord, SyncReportRecord};

uniffi::setup_scaffolding!();

/// Every error crosses as its message: the app shows it, it does not branch on it.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum QuireError {
    #[error("{0}")]
    Engine(#[from] quire_engine::EngineError),
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, QuireError>;

/// The runtime every engine call runs on (the engine needs Tokio for its network clients).
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("quire-engine")
        .enable_all()
        .build()
        .expect("the Tokio runtime starts")
});

/// Runs `future` on [`RUNTIME`]; the returned future can be polled by any executor (UniFFI's).
async fn on_runtime<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> T {
    match RUNTIME.spawn(future).await {
        Ok(value) => value,
        Err(error) => std::panic::resume_unwind(error.into_panic()),
    }
}

/// Opens a library: `name` a registered one (`None`: the default), `config_file` another config
/// than the usual one (tests, portable setups).
#[uniffi::export]
pub async fn open_library(name: Option<String>, config_file: Option<String>) -> Result<Arc<QuireLibrary>> {
    on_runtime(async move {
        let options = OpenOptions {
            library: name.map_or(LibraryChoice::Default, LibraryChoice::Named),
            config_file: config_file.map(PathBuf::from),
            endpoints: None,
        };
        let engine = Engine::open(options).await?;
        Ok(Arc::new(QuireLibrary { engine: Arc::new(engine) }))
    })
    .await
}

/// Opens the library in the folder `path`, registered or not.
#[uniffi::export]
pub async fn open_library_at(path: String, config_file: Option<String>) -> Result<Arc<QuireLibrary>> {
    on_runtime(async move {
        let options = OpenOptions {
            library: LibraryChoice::Path(PathBuf::from(path)),
            config_file: config_file.map(PathBuf::from),
            endpoints: None,
        };
        let engine = Engine::open(options).await?;
        Ok(Arc::new(QuireLibrary { engine: Arc::new(engine) }))
    })
    .await
}

/// Told when the library changes (an edit here, a sync, a background job): reload what is shown.
#[uniffi::export(with_foreign)]
pub trait ChangeListener: Send + Sync {
    fn on_change(&self);
}

/// An open library.
#[derive(uniffi::Object)]
pub struct QuireLibrary {
    engine: Arc<Engine>,
}

impl QuireLibrary {
    /// Runs `f` with the engine on [`RUNTIME`].
    async fn with<T, F, Fut>(&self, f: F) -> T
    where
        F: FnOnce(Arc<Engine>) -> Fut,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        on_runtime(f(self.engine.clone())).await
    }
}

#[uniffi::export]
impl QuireLibrary {
    pub fn info(&self) -> LibraryInfo {
        let library = self.engine.library();
        LibraryInfo {
            name: library.name.clone(),
            path: library.path.display().to_string(),
            config_file: self.engine.config_file().display().to_string(),
        }
    }

    /// The references answering `query`.
    pub async fn references(&self, query: Query) -> Result<Vec<ReferenceRecord>> {
        self.with(|engine| async move { query::references(&engine, &query).await }).await
    }

    /// One reference by ID or cite key, with its projects.
    pub async fn reference(&self, id: String) -> Result<ReferenceRecord> {
        self.with(|engine| async move { query::reference(&engine, &id).await }).await
    }

    /// Every project, parents before children.
    pub async fn projects(&self) -> Result<Vec<ProjectRecord>> {
        self.with(|engine| async move { query::projects(&engine).await }).await
    }

    /// Every tag with how many references carry it.
    pub async fn tags(&self) -> Result<Vec<TagRecord>> {
        self.with(|engine| async move {
            Ok(engine
                .tags()
                .await?
                .into_iter()
                .map(|(tag, count)| TagRecord { name: tag.name, color: tag.color, count: count as u64 })
                .collect())
        })
        .await
    }

    /// Creates a project named `name` under `parent` (a path; `None`: top level).
    pub async fn create_project(&self, name: String, parent: Option<String>) -> Result<ProjectRecord> {
        self.with(|engine| async move {
            let parent = parent.as_deref().map(query::slug_path).transpose()?;
            let (path, project) = engine.create_project(&name, parent.as_ref()).await?;
            Ok(ProjectRecord {
                path: path.to_string(),
                name: project.name,
                parent: path.parent().map(|p| p.to_string()),
                references: 0,
            })
        })
        .await
    }

    /// Imports a `.bib` file (into `project` if given); returns how many references were new.
    pub async fn import_bib(&self, path: String, project: Option<String>) -> Result<u32> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let added = engine.import_bib(std::path::Path::new(&path), project.as_ref()).await?;
            Ok(added.iter().filter(|a| matches!(a, quire_engine::Added::New(_))).count() as u32)
        })
        .await
    }

    /// Puts tags on a reference (ID or cite key).
    pub async fn tag(&self, id: String, names: Vec<String>) -> Result<()> {
        self.with(|engine| async move {
            engine.tag(&id, &names).await?;
            Ok(())
        })
        .await
    }

    /// The paper's reader tabs; its text is fetched and stored on first opening if needed.
    pub async fn paper(&self, id: String) -> Result<PaperRecord> {
        self.with(|engine| async move { reader::paper(&engine, &id).await }).await
    }

    /// The PDF files attached to a reference, oldest first.
    pub async fn pdf_files(&self, id: String) -> Result<Vec<String>> {
        self.with(|engine| async move {
            let (_, files) = engine.pdfs_of(&id).await?;
            Ok(files.into_iter().map(|(_, path)| path.display().to_string()).collect())
        })
        .await
    }

    /// Converts again the stored reader texts an older converter made (after an update), in the
    /// background; returns how many. Opening those papers then does not wait for it.
    pub async fn rebuild_stale_texts(&self) -> Result<u32> {
        self.with(|engine| async move { Ok(engine.rebuild_stale_texts().await? as u32) }).await
    }

    /// The local file of a figure image (`BlockKind::Image`), downloaded once.
    pub async fn figure(&self, url: String) -> Result<String> {
        self.with(|engine| async move { Ok(engine.figure(&url).await?.display().to_string()) }).await
    }

    /// Records that the paper was opened (unread becomes reading); returns its status.
    pub async fn opened(&self, id: String) -> Result<String> {
        self.with(|engine| async move {
            let status = engine.opened(reader::reference_id(&id)?).await?;
            Ok(status.as_str().to_string())
        })
        .await
    }

    /// Sets the reading status: `unread`, `reading` or `read`.
    pub async fn set_reading(&self, id: String, status: String) -> Result<()> {
        self.with(|engine| async move {
            engine.set_reading(&id, reader::reading_status(&status)?).await?;
            Ok(())
        })
        .await
    }

    pub async fn highlights(&self, id: String) -> Result<Vec<HighlightRecord>> {
        self.with(
            |engine| async move { Ok(engine.highlights(&id).await?.1.into_iter().map(reader::highlight).collect()) },
        )
        .await
    }

    /// Highlights `quote` (after `prefix`, which tells repeated quotes apart).
    pub async fn add_highlight(
        &self,
        id: String,
        quote: String,
        prefix: String,
        comment: Option<String>,
    ) -> Result<HighlightRecord> {
        self.with(|engine| async move {
            Ok(reader::highlight(engine.add_highlight(&id, &quote, &prefix, comment.as_deref()).await?))
        })
        .await
    }

    pub async fn comment_highlight(&self, highlight: String, comment: String) -> Result<HighlightRecord> {
        self.with(|engine| async move {
            let id = highlight.parse().map_err(|_| QuireError::Invalid(format!("`{highlight}` is not a highlight")))?;
            Ok(reader::highlight(engine.comment_highlight(id, &comment).await?))
        })
        .await
    }

    pub async fn remove_highlight(&self, highlight: String) -> Result<()> {
        self.with(|engine| async move {
            let id = highlight.parse().map_err(|_| QuireError::Invalid(format!("`{highlight}` is not a highlight")))?;
            engine.remove_highlight(id).await?;
            Ok(())
        })
        .await
    }

    /// Notes on a reference, oldest first.
    pub async fn notes(&self, id: String) -> Result<Vec<NoteRecord>> {
        self.with(|engine| async move { Ok(engine.notes(Some(&id)).await?.into_iter().map(reader::note).collect()) })
            .await
    }

    pub async fn add_note(&self, id: String, body: String) -> Result<NoteRecord> {
        self.with(|engine| async move { Ok(reader::note(engine.add_note(Some(&id), &body).await?)) }).await
    }

    pub async fn edit_note(&self, note: String, body: String) -> Result<NoteRecord> {
        self.with(|engine| async move {
            let id = note.parse().map_err(|_| QuireError::Invalid(format!("`{note}` is not a note")))?;
            Ok(reader::note(engine.edit_note(id, &body).await?))
        })
        .await
    }

    pub async fn remove_note(&self, note: String) -> Result<()> {
        self.with(|engine| async move {
            let id = note.parse().map_err(|_| QuireError::Invalid(format!("`{note}` is not a note")))?;
            engine.remove_note(id).await?;
            Ok(())
        })
        .await
    }

    /// Calls `listener` after every change until the returned subscription is cancelled.
    pub fn subscribe(&self, listener: Arc<dyn ChangeListener>) -> Arc<Subscription> {
        let mut changes = self.engine.subscribe();
        let task = RUNTIME.spawn(async move {
            use tokio::sync::broadcast::error::RecvError;
            // Missed changes (`Lagged`) still mean "reload"; `Closed` ends it.
            while let Ok(_) | Err(RecvError::Lagged(_)) = changes.recv().await {
                listener.on_change();
            }
        });
        Arc::new(Subscription { task })
    }
}

/// A [`QuireLibrary::subscribe`] in progress.
#[derive(uniffi::Object)]
pub struct Subscription {
    task: tokio::task::JoinHandle<()>,
}

#[uniffi::export]
impl Subscription {
    pub fn cancel(&self) {
        self.task.abort();
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.task.abort();
    }
}
