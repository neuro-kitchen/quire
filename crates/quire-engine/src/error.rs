//! The error type of `quire-engine`: every lower crate's error, plus configuration and files.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Core(#[from] quire_core::CoreError),

    #[error(transparent)]
    Model(#[from] quire_model::ModelError),

    #[error(transparent)]
    Store(#[from] quire_store::StoreError),

    #[error(transparent)]
    Fetch(#[from] quire_fetch::FetchError),

    #[error(transparent)]
    Sync(#[from] quire_sync::SyncError),

    #[error(transparent)]
    Format(#[from] quire_io::IoError),

    #[error("database: {0}")]
    Database(#[from] tokio_rusqlite::Error),

    #[error("database: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("{}: {source}", path.display())]
    File { path: PathBuf, source: std::io::Error },

    #[error("configuration: {0}")]
    Config(String),

    /// The user's editor could not run or failed.
    #[error("editor: {0}")]
    Editor(String),

    /// A downloaded PDF that is not the reference's paper.
    #[error("not this paper's PDF: {0}")]
    NotThisPaper(String),

    /// A PDF that cannot be used as asked.
    #[error("{0}")]
    Pdf(String),

    /// A reference with no identifier or URL to look it up by.
    #[error("`{0}` has no DOI, arXiv ID, PubMed ID or URL to look it up by")]
    Unrefetchable(String),

    /// A Zotero data folder that cannot be read.
    #[error("Zotero: {0}")]
    Zotero(String),

    /// An unknown, duplicate or invalid library name.
    #[error("{0}")]
    Library(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

impl EngineError {
    /// Whether a site's "are you human" check stopped a lookup.
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Fetch(error) if error.is_blocked())
    }

    pub(crate) fn file(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Self {
        let path = path.into();
        move |source| Self::File { path, source }
    }
}
