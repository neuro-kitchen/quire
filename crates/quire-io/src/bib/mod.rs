//! BibTeX/BibLaTeX `.bib` files, read and written by the Typst `biblatex` crate. This module
//! only maps between quire's model and `biblatex` entries; the mapping follows citation-js
//! `@citation-js/plugin-bibtex` 0.9.0 (MIT, © Lars Willighagen), which JabRef also adopted
//! (`.knowledge/bibliography-formats.md`).
//!
//! - [`export`]: references → `.bib` text (BibLaTeX or BibTeX), with export options.
//! - [`ascii`]: non-ASCII text as LaTeX commands.
//! - [`import`]: `.bib` text → cite keys + [`ReferenceData`](quire_model::ReferenceData).

mod ascii;
mod export;
mod fields;
mod import;
mod names;
mod types;

pub use export::{BibFormat, BibOptions, PreprintForm, entries, to_bib, to_bib_with, to_entry};
pub use import::{BibRecord, from_bib};
