//! The one error type of `quire-document`.

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Bytes that are not a readable PDF (e.g. an HTML page sent instead).
    #[error("not a PDF: {0}")]
    NotPdf(String),

    /// Bytes that are not a readable EPUB (no zip, no container, no chapters).
    #[error("not an EPUB: {0}")]
    NotEpub(String),

    /// A PDF the renderer could not parse.
    #[error("failed to load PDF: {0}")]
    Load(String),

    /// A render request superseded by a newer one for the same page before it started.
    #[error("render of page {0} cancelled (a newer request for it came first)")]
    Cancelled(u32),

    #[error("page index {0} out of bounds")]
    PageNotFound(u32),

    #[error("rendering failed: {0}")]
    RenderFailed(String),

    #[error("invalid dimensions: {0}")]
    InvalidDimensions(String),
}

pub type Result<T> = std::result::Result<T, DocumentError>;
