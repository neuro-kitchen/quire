//! [`Engine`]: an open library. Every front end goes through it (`.knowledge/architecture.md`).
//! The database runs on its own thread (`tokio-rusqlite`); each operation is one transaction.
//!
//! - [`references`]: add (by identifier or URL), import `.bib`, list, show, copy, trash, restore.
//! - [`import`]: importing a Zotero library.
//! - [`projects`]: create, rename, remove, tree, link, unlink.
//! - [`abstracts`]: filling missing abstracts (Europe PMC, the PDF).
//! - [`exports`]: export through profiles; automatic exports.
//! - [`manuscript`]: a document's cite keys checked against the library.
//! - [`pdfs`]: find, download, attach and import PDFs.
//! - [`backup`]: the library as one archive, and restoring it.
//! - [`ocr`]: OCR of scanned PDFs (OCRmyPDF or Tesseract, when installed).
//! - [`update`]: refetch and edit references.
//! - [`alerts`]: new papers from arXiv, PubMed or Crossref searches.
//! - [`graph`]: the citation graph (OpenAlex).
//! - [`reading`]: reading status and saved searches.
//! - [`retractions`]: retractions and expressions of concern (Crossref).
//! - [`tags`]: tags, notes and related references.

mod abstracts;
mod alerts;
pub mod backup;
mod duplicates;
mod exports;
mod graph;
mod highlights;
mod import;
mod manuscript;
mod ocr;
mod pdfs;
mod projects;
mod reading;
mod references;
mod retractions;
mod search;
mod sync;
mod tags;
mod text;
mod update;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use quire_fetch::{Endpoints, Fetcher};
use quire_model::DeviceId;
use quire_store::SqliteLibrary;

pub use abstracts::AbstractOutcome;
pub use alerts::AlertCheck;
pub use duplicates::DuplicateGroup;
pub use exports::{AutoExportInfo, AutoExportOutcome, DEFAULT_PROFILE, ExportRequest, write_if_changed};
pub use graph::{GraphEntry, GraphKind};
pub use import::ZoteroImport;
pub use manuscript::{ManuscriptCheck, UnknownKey, Use as CiteUse, suggest_keys};
pub use ocr::OcrOutcome;
pub use pdfs::{Attached, PdfOutcome, PdfStatus};
pub use projects::ProjectSummary;
pub use retractions::RetractionCheck;
pub use search::DEFAULT_LIMIT as SEARCH_LIMIT;
pub use text::{PaperText, TextSource, online_candidate};

use crate::config::Config;
use crate::error::Result;
use crate::libraries::{self, LibraryChoice, LibraryInfo};
use crate::paths::{self, DATABASE_FILE};
use crate::private;

/// Which library to open and with which config; `None` fields use the defaults (`paths`).
#[derive(Debug, Clone, Default)]
pub struct OpenOptions {
    pub library: LibraryChoice,
    pub config_file: Option<PathBuf>,
    /// Service URLs; tests point them at a mock server.
    pub endpoints: Option<Endpoints>,
}

pub struct Engine {
    conn: tokio_rusqlite::Connection,
    device: DeviceId,
    config: Config,
    config_file: PathBuf,
    library: LibraryInfo,
    fetcher: Fetcher,
    /// Set by every change; automatic exports run when set.
    changed: AtomicBool,
    /// Tells front ends (TUI, desktop) that the library changed, e.g. by a background job.
    changes: tokio::sync::broadcast::Sender<Change>,
}

/// A change to the library, sent to [`Engine::subscribe`]rs. Front ends reload what they show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    /// Milliseconds since the epoch.
    pub at: i64,
}

/// Changes kept for a slow subscriber before older ones are dropped (it then reloads anyway).
const CHANGE_BUFFER: usize = 64;

impl Engine {
    /// Opens the library, creating its directory and database on first use.
    pub async fn open(options: OpenOptions) -> Result<Self> {
        let config_file = paths::config_file(options.config_file.as_deref())?;
        let config = Config::load(&config_file)?;
        let library = libraries::resolve(&config, &options.library)?;
        let library_dir = &library.path;
        // References, notes and PDFs are the user's only (`private`); also existing folders.
        if let Ok(data) = paths::data_dir()
            && library_dir.starts_with(&data)
        {
            private::dir(&data)?;
        }
        private::dir(library_dir)?;
        private::file(&config_file)?;

        let conn = tokio_rusqlite::Connection::open(library_dir.join(DATABASE_FILE)).await?;
        let device = conn
            .call_raw(|conn| -> quire_store::Result<DeviceId> {
                quire_store::prepare_in_place(conn)?;
                quire_store::device_id(conn)
            })
            .await??;
        let email = config.user.email.as_deref();
        let fetcher = Fetcher::with_endpoints(email, options.endpoints.unwrap_or_default())?;
        tracing::debug!(library = %library_dir.display(), %device, "library open");
        let (changes, _) = tokio::sync::broadcast::channel(CHANGE_BUFFER);
        Ok(Self { conn, device, config, config_file, library, fetcher, changed: AtomicBool::new(false), changes })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn library_dir(&self) -> &Path {
        &self.library.path
    }

    /// The open library: name (if registered), folder, default or not.
    pub fn library(&self) -> &LibraryInfo {
        &self.library
    }

    pub fn device(&self) -> DeviceId {
        self.device
    }

    /// Changes from now on: one message per change to the library.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Change> {
        self.changes.subscribe()
    }

    /// Runs `f` on the library in one transaction, on the database thread.
    async fn run<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut SqliteLibrary<'_>) -> quire_core::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let device = self.device;
        self.conn
            .call_raw(move |conn| -> Result<T> {
                let tx = conn.transaction()?;
                let out = f(&mut SqliteLibrary::new(&tx, device))?;
                tx.commit()?;
                Ok(out)
            })
            .await?
    }
}
