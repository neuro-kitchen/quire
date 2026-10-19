//! quire's use cases, independent of storage and network: adding references (with
//! deduplication and Better BibTeX cite keys), projects, and choosing what to export.
//!
//! - [`ports`]: the [`Library`] trait a store implements, and [`RawRecord`].
//! - [`library`]: the use cases ([`library::references`], [`library::projects`]).
//! - [`check`]: lints and recorded issues of references.
//! - [`citekey`]: cite-key formula and uniqueness.
//! - [`error`]: the error type, [`CoreError`].

pub mod check;
pub mod citekey;
pub mod error;
pub mod library;
pub mod ports;

#[cfg(test)]
mod memory;

pub use check::Checked;
pub use error::{CoreError, Result, StorageError};
pub use library::references::{AddRequest, Added};
pub use ports::{AutoExport, FailedInput, Library, RawRecord, ReferenceFilter};
