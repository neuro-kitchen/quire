//! Import and export of bibliographic data. Pure conversions: text in, model out (or back); no
//! files or network.
//!
//! - [`csl`]: CSL-JSON (what DOI services return; Pandoc, Zotero).
//! - [`bib`]: BibTeX/BibLaTeX `.bib` files, via the Typst `biblatex` crate.
//! - [`error`]: the error type, [`IoError`].

pub mod bib;
pub mod csl;
pub mod error;

pub use bib::{from_bib, to_bib, BibFormat, BibRecord};
pub use csl::{from_csl_json, from_csl_value, to_csl_json};
pub use error::{IoError, Result};
