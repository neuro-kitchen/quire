//! The error type of `quire-store`, and its conversion into `quire-core`'s storage error.

use quire_core::CoreError;
use quire_model::ModelError;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error("migrating the library database: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("stored JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// A stored value that fails validation (the database was changed outside quire).
    #[error("stored value: {0}")]
    Model(#[from] ModelError),
}

pub type Result<T> = std::result::Result<T, StoreError>;

impl From<StoreError> for CoreError {
    fn from(error: StoreError) -> Self {
        CoreError::Storage(Box::new(error))
    }
}

/// `?` from store results inside `Library` methods (which return `quire_core::Result`).
pub trait OrCore<T> {
    fn or_core(self) -> quire_core::Result<T>;
}

impl<T, E: Into<StoreError>> OrCore<T> for std::result::Result<T, E> {
    fn or_core(self) -> quire_core::Result<T> {
        self.map_err(|e| CoreError::from(e.into()))
    }
}
