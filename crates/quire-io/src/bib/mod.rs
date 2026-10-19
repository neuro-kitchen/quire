//! BibTeX/BibLaTeX `.bib` files, read and written by the Typst `biblatex` crate. This module
//! only maps between quire's model and `biblatex` entries; the mapping follows citation-js
//! `@citation-js/plugin-bibtex` 0.9.0 (MIT, © Lars Willighagen), which JabRef also adopted
//! (`.knowledge/bibliography-formats.md`).
//!
//! - [`export`]: references → `.bib` text (BibLaTeX or BibTeX).
//! - [`import`]: `.bib` text → cite keys + [`ReferenceData`](quire_model::ReferenceData).

mod export;
mod fields;
mod import;
mod names;
mod types;

pub use export::{to_bib, to_entry, BibFormat};
pub use import::{from_bib, BibRecord};
