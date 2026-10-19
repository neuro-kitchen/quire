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

    /// The service asks us to slow down (HTTP 429) and did not accept one retry.
    #[error("{url} is rate limiting requests; try again later")]
    RateLimited { url: String },

    /// The site answered with a challenge page ("are you human") instead of the content.
    #[error("{url} is blocked by an \"are you human\" check")]
    Blocked { url: String },

    /// The response could not be read as the expected format.
    #[error("unreadable response from {url}: {message}")]
    Parse { url: String, message: String },

    /// A link that should give a PDF gave something else (and not a challenge page).
    #[error("{0} did not return a PDF")]
    NotPdf(String),

    /// The institutional proxy answered with its login: no session, or it expired.
    #[error("the proxy session has expired or is not logged in: run `quire access login`")]
    ProxyLogin,

    /// No source offered a PDF (no open-access copy, no link on the publisher's page).
    #[error("no PDF found: {0}")]
    NoPdf(String),

    /// A web page without usable bibliographic metadata.
    #[error("no bibliographic metadata found at {0}")]
    NoMetadata(String),

    #[error("invalid URL `{0}`")]
    InvalidUrl(String),

    /// What the user typed is none of the inputs quire looks up.
    #[error("`{0}` is not a DOI, arXiv ID, PMID, PMCID or http(s) URL")]
    Unrecognised(String),

    /// Every source of a fallback chain failed; each with its error, most authoritative first.
    #[error("{}", .0.iter().map(|(source, error)| format!("{source}: {error}")).collect::<Vec<_>>().join("; "))]
    AllFailed(Vec<(String, FetchError)>),
}

impl FetchError {
    /// Whether trying another source for the same thing is pointless: the identifier does not
    /// exist (the DOI resolver said so) or the input is not a URL.
    pub fn is_final(&self) -> bool {
        matches!(self, Self::NotFound(_) | Self::InvalidUrl(_) | Self::Unrecognised(_))
    }

    /// Whether a site's bot check stopped us (here or in any source of a chain).
    pub fn is_blocked(&self) -> bool {
        match self {
            Self::Blocked { .. } => true,
            Self::AllFailed(failures) => failures.iter().any(|(_, e)| e.is_blocked()),
            _ => false,
        }
    }

    /// Whether the institutional proxy asked for a new login (here or in any source of a chain).
    pub fn is_proxy_login(&self) -> bool {
        match self {
            Self::ProxyLogin => true,
            Self::AllFailed(failures) => failures.iter().any(|(_, e)| e.is_proxy_login()),
            _ => false,
        }
    }

    /// Whether only the network or a service failed (worth retrying later), not "no PDF here".
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Http(_) | Self::RateLimited { .. } | Self::Status { .. } => true,
            Self::AllFailed(failures) => failures.iter().all(|(_, e)| e.is_transient()),
            _ => false,
        }
    }

    pub(crate) fn parse(url: impl ToString, message: impl ToString) -> Self {
        Self::Parse { url: url.to_string(), message: message.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, FetchError>;
