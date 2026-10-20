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

    /// Removing a project that has subprojects without asking for them too.
    #[error("project `{0}` has subprojects; remove them first or use --recursive")]
    ProjectHasSubprojects(String),

    /// Nothing matches what the user typed (ID, cite key or identifier).
    #[error("no reference `{0}`")]
    ReferenceNotFound(String),

    /// An identifier typed by the user belongs to another reference.
    #[error("{identifier} already belongs to `{cite_key}`")]
    IdentifierTaken { identifier: String, cite_key: String },

    #[error("no tag `{0}`")]
    TagNotFound(String),

    /// A colored tag's position must be 1–9 (0: all nine are taken).
    #[error("tag position must be 1 to 9 (0 means all nine colored positions are taken)")]
    TagPosition(u8),

    #[error("no note `{0}`")]
    NoteNotFound(String),

    #[error("a note needs some text")]
    EmptyNote,

    #[error("no highlight `{0}`")]
    HighlightNotFound(String),

    /// A search filter with a value it does not take.
    #[error("search filter {0}")]
    InvalidFilter(String),

    #[error("no saved search `{0}`")]
    SavedSearchNotFound(String),

    #[error("a saved search `{0}` already exists")]
    SavedSearchExists(String),

    #[error("no alert `{0}`")]
    AlertNotFound(String),

    #[error("no attachment `{0}`")]
    AttachmentNotFound(String),

    /// Sharing a project in a way the user's role does not allow.
    #[error("{0}")]
    Sharing(String),

    #[error("a highlight needs some text")]
    EmptyHighlight,

    #[error("`{0}` cannot be related to itself")]
    SelfRelation(String),

    /// Merging or separating a reference with itself.
    #[error("`{0}` and `{1}` are the same reference")]
    SameReference(String, String),

    /// A cite key chosen by hand is used by another reference.
    #[error("cite key `{0}` is already used")]
    CiteKeyTaken(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
