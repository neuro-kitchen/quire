//! The one error type of `quire-core`.

use quire_model::ModelError;

/// A boxed error from a [`Library`](crate::Library) implementation (e.g. SQLite).
pub type StorageError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error(transparent)]
    Model(#[from] ModelError),

    #[error("storage: {0}")]
    Storage(StorageError),

    #[error("no project `{0}`")]
    ProjectNotFound(String),

    #[error("project `{0}` already exists")]
    ProjectExists(String),

    /// Nothing matches what the user typed (ID, cite key or identifier).
    #[error("no reference `{0}`")]
    ReferenceNotFound(String),

    /// A cite key chosen by hand is used by another reference.
    #[error("cite key `{0}` is already used")]
    CiteKeyTaken(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
