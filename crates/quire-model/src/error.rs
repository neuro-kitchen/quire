//! The one error type of `quire-model`: values that fail validation when parsed or built.

/// A value that does not satisfy the rules of its type.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    /// A TypeID string with the wrong prefix or a malformed suffix, or bytes that are not a UUIDv7.
    #[error("invalid {kind} ID `{input}`: {reason}")]
    InvalidId { kind: &'static str, input: String, reason: String },

    /// A DOI, PMID, PMCID or arXiv ID that does not follow its syntax.
    #[error("invalid {kind} `{input}`: {reason}")]
    InvalidIdentifier { kind: &'static str, input: String, reason: &'static str },

    /// Text that is none of the identifier kinds quire knows.
    #[error("unrecognised identifier `{0}` (expected a DOI, PMID, PMCID or arXiv ID)")]
    UnknownIdentifier(String),

    /// A CSL type name that is not in CSL 1.0.2.
    #[error("unknown entry type `{0}`")]
    UnknownEntryKind(String),

    /// A CSL name variable that is not in CSL 1.0.2.
    #[error("unknown creator role `{0}`")]
    UnknownCreatorRole(String),

    /// A field name quire does not store as a text field.
    #[error("unknown field `{0}`")]
    UnknownField(String),

    #[error("invalid date `{input}`: {reason}")]
    InvalidDate { input: String, reason: &'static str },

    #[error("invalid project slug `{input}`: {reason}")]
    InvalidSlug { input: String, reason: &'static str },

    #[error("invalid cite key `{input}`: {reason}")]
    InvalidCiteKey { input: String, reason: &'static str },
}

pub type Result<T> = std::result::Result<T, ModelError>;
