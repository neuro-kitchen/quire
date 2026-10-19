//! [`Engine`]: an open library. Every front end goes through it (`.knowledge/architecture.md`).
//! The database runs on its own thread (`tokio-rusqlite`); each operation is one transaction.
//!
//! - [`references`]: add (by identifier or URL), import `.bib`, list, show, trash, restore.
//! - [`projects`]: create, tree, link, unlink, export.

mod projects;
mod references;

use std::path::{Path, PathBuf};

use quire_fetch::{Endpoints, Fetcher};
use quire_model::DeviceId;
use quire_store::SqliteLibrary;

pub use projects::ExportFormat;

use crate::config::Config;
use crate::error::{EngineError, Result};
use crate::paths::{self, DATABASE_FILE};

/// Where to open the library; `None` fields use the defaults (`paths`).
#[derive(Debug, Clone, Default)]
pub struct OpenOptions {
    pub library: Option<PathBuf>,
    pub config_file: Option<PathBuf>,
    /// Service URLs; tests point them at a mock server.
    pub endpoints: Option<Endpoints>,
}

pub struct Engine {
    conn: tokio_rusqlite::Connection,
    device: DeviceId,
    config: Config,
    config_file: PathBuf,
    library_dir: PathBuf,
    fetcher: Fetcher,
}

impl Engine {
    /// Opens the library, creating its directory and database on first use.
    pub async fn open(options: OpenOptions) -> Result<Self> {
        let config_file = paths::config_file(options.config_file.as_deref())?;
        let config = Config::load(&config_file)?;
        let library_dir = paths::library_dir(options.library.as_deref(), config.library.path.as_deref())?;
        std::fs::create_dir_all(&library_dir).map_err(EngineError::file(&library_dir))?;

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
        Ok(Self { conn, device, config, config_file, library_dir, fetcher })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn library_dir(&self) -> &Path {
        &self.library_dir
    }

    pub fn device(&self) -> DeviceId {
        self.device
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
