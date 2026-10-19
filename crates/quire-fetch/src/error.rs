//! The one error type of `quire-fetch`.

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("network: {0}")]
    Http(#[from] reqwest::Error),

    /// The service does not know the identifier (HTTP 404).
    #[error("not found: {0}")]
    NotFound(String),

    /// Any other unsuccessful HTTP status.
    #[error("{url} answered HTTP {status}")]
    Status { url: String, status: u16 },

    /// The response could not be read as the expected format.
    #[error("unreadable response from {url}: {source}")]
    Parse { url: String, source: quire_io::IoError },

    /// A web page without usable bibliographic metadata.
    #[error("no bibliographic metadata found at {0}")]
    NoMetadata(String),

    #[error("invalid URL `{0}`")]
    InvalidUrl(String),
}

pub type Result<T> = std::result::Result<T, FetchError>;
