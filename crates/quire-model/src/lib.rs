//! Domain types of quire, with no I/O: everything else (store, import/export, fetch, CLI) speaks
//! in these types.
//!
//! - [`id`]: record IDs (UUIDv7, written as TypeIDs such as `ref_01h…`), typed by record kind.
//! - [`identifier`]: external identifiers (DOI, PMID, PMCID, arXiv ID): parsing and normalisation.
//! - [`reference`]: bibliographic content ([`ReferenceData`], close to CSL-JSON) and stored
//!   [`Reference`]s with cite key and per-field provenance.
//! - [`creator`]: authors, editors, … with names in parts.
//! - [`date`]: dates known to the year, month or day.
//! - [`project`]: nestable projects and their slugs.
//! - [`record`]: metadata every stored record carries (times, device, soft delete).
//! - [`error`]: [`ModelError`], returned when a value fails validation.

#[macro_use]
mod macros;

pub mod creator;
pub mod date;
pub mod error;
pub mod id;
pub mod identifier;
pub mod project;
pub mod record;
pub mod reference;

pub use creator::{Creator, CreatorRole, Name, PersonName};
pub use date::Date;
pub use error::{ModelError, Result};
pub use id::{AttachmentId, DeviceId, Id, IdKind, ProjectId, ReferenceId, TagId};
pub use identifier::{ArxivId, Doi, Identifier, IdentifierKind, Pmcid, Pmid};
pub use project::{Project, Slug, SlugPath};
pub use record::{RecordMeta, Timestamp};
pub use reference::{CiteKey, EntryKind, Field, Origin, Provenance, Reference, ReferenceData, TextField};
