//! Import and export of bibliographic data. Pure conversions: text in, model out (or back); no
//! files or network.
//!
//! - [`abstracts`]: a paper's abstract read from its text.
//! - [`annotations`]: highlights and notes as a Markdown file for notes apps.
//! - [`csl`]: CSL-JSON (what DOI services return; Pandoc, Zotero).
//! - [`edit`]: a reference as an editable TOML document (`quire ref edit`).
//! - [`manuscript`]: the cite keys a Typst, LaTeX or Markdown document uses.
//! - [`export`]: every output format, with an export profile's settings (Hayagriva via Typst's crate).
//! - [`text`]: plain text from markup (tags, entities, whitespace).
//! - [`zotero`]: a Zotero library's items, notes and fields as quire's model.
//! - [`pdf`]: checking PDFs (is it one, text or scanned, which DOIs it prints).
//! - [`bib`]: BibTeX/BibLaTeX `.bib` files, via the Typst `biblatex` crate.
//! - [`error`]: the error type, [`IoError`].

pub mod abstracts;
pub mod annotations;
pub mod bib;
pub mod csl;
pub mod edit;
pub mod error;
pub mod export;
pub mod manuscript;
pub mod paper_markdown;
pub mod pdf;
pub mod text;
pub mod zotero;

pub use bib::{BibFormat, BibOptions, BibRecord, PreprintForm, from_bib, to_bib};
pub use csl::{from_csl_json, from_csl_value, to_csl_json};
pub use edit::{from_edit_toml, to_edit_toml};
pub use error::{IoError, Result};
pub use export::{ExportFormat, ExportOptions, export};
