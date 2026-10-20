//! quire's use cases, independent of storage and network: adding references (with
//! deduplication and Better BibTeX cite keys), projects, and choosing what to export.
//!
//! - [`ports`]: the store traits ([`Library`] is all of them), and [`RawRecord`].
//! - [`library`]: the use cases ([`library::references`], [`library::projects`]).
//! - [`check`]: lints and recorded issues of references.
//! - [`citekey`]: cite-key formula and uniqueness.
//! - [`sync`]: the rows devices exchange, and which change wins.
//! - [`error`]: the error type, [`CoreError`].

pub mod check;
pub mod citekey;
pub mod error;
pub mod library;
pub mod ports;
pub mod sync;

#[cfg(test)]
mod memory;

pub use check::Checked;
pub use error::{CoreError, Result, StorageError};
pub use library::references::{AddRequest, Added};
pub use ports::{AutoExport, FailedInput, Library, RawRecord, ReferenceFilter, SearchHit, prelude};
