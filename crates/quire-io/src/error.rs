//! The one error type of `quire-io`.

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// CSL-JSON that is neither one item (object) nor a list of items.
    #[error("CSL-JSON must be an object or an array of objects")]
    NotCslItems,

    #[error("invalid .bib file: {0}")]
    BibParse(String),

    /// Two references with the same cite key in one export (`biblatex` would silently keep one).
    #[error("cite key `{0}` is used by more than one reference")]
    DuplicateCiteKey(String),

    /// An entry `biblatex` cannot write as BibTeX (e.g. a malformed date).
    #[error("cannot write `{key}` as BibTeX: {message}")]
    BibtexWrite { key: String, message: String },
}

pub type Result<T> = std::result::Result<T, IoError>;
