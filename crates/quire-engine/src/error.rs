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
    Format(#[from] quire_io::IoError),

    #[error("database: {0}")]
    Database(#[from] tokio_rusqlite::Error),

    #[error("database: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("{}: {source}", path.display())]
    File { path: PathBuf, source: std::io::Error },

    #[error("configuration: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

impl EngineError {
    pub(crate) fn file(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Self {
        let path = path.into();
        move |source| Self::File { path, source }
    }
}
