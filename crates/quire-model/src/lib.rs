//! Domain types of quire, with no I/O: everything else (store, import/export, fetch, CLI) speaks
//! in these types.
//!
//! - [`id`]: record IDs (UUIDv7, written as TypeIDs such as `ref_01h…`), typed by record kind.
//! - [`identifier`]: external identifiers (DOI, PMID, PMCID, arXiv ID): parsing and normalisation.
//! - [`reference`]: bibliographic content ([`ReferenceData`], close to CSL-JSON) and stored
//!   [`Reference`]s with cite key and per-field provenance.
//! - [`attachment`]: files (PDFs) of a reference, stored by content hash.
//! - [`creator`]: authors, editors, … with names in parts.
//! - [`date`]: dates known to the year, month or day.
//! - [`issue`]: problems found in or recorded about a reference's metadata.
//! - [`project`]: nestable projects and their slugs; [`member`]: sharing them (users, roles).
//! - [`tag`], [`note`]: labels and notes on references (as in Zotero).
//! - [`reading`]: whether the user has read a paper; [`saved_search`]: searches kept by name.
//! - [`record`]: metadata every stored record carries (times, device, soft delete).
//! - [`error`]: [`ModelError`], returned when a value fails validation.

#[macro_use]
mod macros;

pub mod attachment;
pub mod creator;
pub mod date;
pub mod error;
pub mod highlight;
pub mod id;
pub mod identifier;
pub mod issue;
pub mod member;
pub mod note;
pub mod project;
pub mod reading;
pub mod record;
pub mod reference;
pub mod saved_search;
pub mod tag;

pub use attachment::{Attachment, PdfAttempt, PdfKind};
pub use creator::{Creator, CreatorRole, Name, PersonName};
pub use date::Date;
pub use error::{ModelError, Result};
pub use highlight::Highlight;
pub use id::{
    AlertId, AttachmentId, DeviceId, HighlightId, Id, IdKind, NoteId, ProjectId, ReferenceId, SavedSearchId, TagId,
};
pub use identifier::{ArxivId, Doi, Identifier, IdentifierKind, Pmcid, Pmid};
pub use issue::{Issue, IssueCode, Severity};
pub use member::{ProjectMember, Role, UserId};
pub use note::Note;
pub use project::{Project, Slug, SlugPath};
pub use reading::{Reading, ReadingStatus};
pub use record::{RecordMeta, Timestamp};
pub use reference::{CiteKey, EntryKind, Field, Origin, Provenance, Reference, ReferenceData, TextField};
pub use saved_search::{Alert, AlertSource, SavedSearch};
pub use tag::{Tag, TagKind};
