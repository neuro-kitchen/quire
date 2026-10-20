//! A paper's content in any format: rendering, text layer and conversion to the reading model.
//!
//! Formats are fixed layout (PDF: pages, coordinates, bitmaps) or reflowable (EPUB, HTML, JATS,
//! plain text: structure, no pages). Every format becomes the reader's Markdown
//! ([`paper_markdown`]), read as blocks and tabs by [`reading`].
//!
//! - [`pdf`]: checking PDFs, PDF → reader Markdown, and the reader's pages (hayro).
//! - [`epub`]: EPUB books → reader Markdown, chapter after chapter.
//! - [`html`]: arXiv's HTML, Europe PMC's JATS XML and plain XHTML → reader Markdown.
//! - [`paper_markdown`]: the reader Markdown format every converter writes.
//! - [`reading`]: a paper's stored text as blocks and reader tabs (terminal reader and apps).
//! - [`abstracts`]: a paper's abstract read from its text.
//! - [`error`]: the error type, [`DocumentError`].

pub mod abstracts;
pub mod epub;
pub mod error;
pub mod html;
pub mod paper_markdown;
pub mod pdf;
pub mod reading;

pub use error::{DocumentError, Result};
