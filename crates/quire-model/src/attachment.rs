//! [`Attachment`]: a file (a PDF) belonging to a reference. The file itself is stored once by
//! the BLAKE3 hash of its content (`.knowledge/storage.md`); the attachment records which
//! reference it belongs to, where it came from and what it is.

use serde::{Deserialize, Serialize};

use crate::id::{AttachmentId, DeviceId, ReferenceId};
use crate::record::RecordMeta;

/// What a PDF turned out to be when checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PdfKind {
    /// Has a text layer (searchable).
    Text,
    /// Images only: needs OCR to be searched.
    Scanned,
    /// Some pages text, some images.
    Mixed,
}

/// Whether `text` is a content hash as the file store names files: BLAKE3, 64 lowercase hex
/// digits. Attachments arrive from other devices; a hash is used to build a file's path, so
/// anything else (`../..`) is refused.
pub fn is_content_hash(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub reference: ReferenceId,
    /// BLAKE3 hash of the content, hex: the file's name in the library's file store.
    pub hash: String,
    /// A readable name for opening and exporting, e.g. `vaswaniAttentionAllYou2017.pdf`.
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    /// Where it came from: a URL, `browser`, or `file:<name>`.
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_kind: Option<PdfKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pages: Option<u32>,
    pub meta: RecordMeta,
}

impl Attachment {
    pub fn new(
        reference: ReferenceId,
        hash: String,
        filename: String,
        content_type: String,
        size: u64,
        source: String,
        device: DeviceId,
    ) -> Self {
        Self {
            id: AttachmentId::new(),
            reference,
            hash,
            filename,
            content_type,
            size,
            source,
            pdf_kind: None,
            pages: None,
            meta: RecordMeta::new(device),
        }
    }
}

name_enum! {
    /// Why the last `pdf get` for a reference without a PDF found none.
    pub enum PdfAttempt (unknown: crate::error::ModelError::UnknownIssue) {
        /// A site refused scripts, showed a bot check or asked for a login.
        Blocked => "blocked",
        /// Every source was tried; none had a PDF.
        NotFound => "not-found",
        /// No DOI or arXiv ID to search by.
        NoIdentifier => "no-identifier",
        /// A network or service error.
        Error => "error",
    }
}
