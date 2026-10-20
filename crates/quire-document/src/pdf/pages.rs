//! The reader's pages: each call takes the file's path; the parsed document and its text layers
//! stay in [`global_cache`] until the file changes.
//!
//! Positions are in PDF points (1/72 inch) from the page's top-left corner.

use std::path::Path;
use std::sync::Arc;

use super::cache::global_cache;
use super::find;
use super::render;
use super::schedule;
use super::types::{PageRect, PageSize, PageText, QuoteMatch, RenderedPage, TextMatch, WordBox};
use crate::error::DocumentError;

/// Returns the dimensions (width and height in points) for all pages in `path`.
pub fn page_sizes(path: impl AsRef<Path>) -> Result<Vec<PageSize>, DocumentError> {
    let pdf = global_cache().get_pdf(path.as_ref())?;
    let sizes = pdf
        .pages()
        .iter()
        .map(|page| {
            let (w, h) = page.render_dimensions();
            PageSize { width: w, height: h }
        })
        .collect();
    Ok(sizes)
}

/// Renders page `page_index` to an RGBA bitmap at target `width` in pixels.
/// Runs on a render worker, newest request first; [`DocumentError::Cancelled`] if a newer request
/// for the same page arrives meanwhile (see `schedule`).
pub fn render_page_at(path: impl AsRef<Path>, page_index: u32, width: u32) -> Result<RenderedPage, DocumentError> {
    let (key, path) = (path.as_ref().to_path_buf(), path.as_ref().to_path_buf());
    schedule::global().run(&key, page_index, move || {
        let pdf = global_cache().get_pdf(&path)?;
        let page = pdf.pages().get(page_index as usize).ok_or(DocumentError::PageNotFound(page_index))?;
        render::render_page(page, width)
    })
}

/// Returns the extracted text layer for page `page_index`.
pub fn page_text(path: impl AsRef<Path>, page_index: u32) -> Result<Arc<PageText>, DocumentError> {
    global_cache().get_page_text(path.as_ref(), page_index)
}

/// Locates highlights on the pages where their quotes appear.
pub fn locate(path: impl AsRef<Path>, quotes: &[String]) -> Result<Vec<QuoteMatch>, DocumentError> {
    find::locate_quotes(global_cache(), path.as_ref(), quotes)
}

/// Every place `query` appears (the reader's find), in page order, at most `limit`.
pub fn search(path: impl AsRef<Path>, query: &str, limit: usize) -> Result<Vec<TextMatch>, DocumentError> {
    let path = path.as_ref();
    let pages = global_cache().get_pdf(path)?.pages().len() as u32;
    find::search_in(pages, |page| global_cache().get_page_text(path, page), query, limit)
}

/// Finds the word at point `(x, y)` on page `page_index`.
pub fn word_at(path: impl AsRef<Path>, page_index: u32, x: f32, y: f32) -> Result<Option<WordBox>, DocumentError> {
    let pt = page_text(path, page_index)?;
    Ok(pt.word_at(x, y).cloned())
}

/// Finds the index of the nearest glyph to point `(x, y)` on page `page_index`.
pub fn nearest_glyph(path: impl AsRef<Path>, page_index: u32, x: f32, y: f32) -> Result<Option<usize>, DocumentError> {
    let pt = page_text(path, page_index)?;
    Ok(pt.nearest_glyph(x, y))
}

/// Returns text and selection rects for glyph range `[start, end)` on page `page_index`.
pub fn text_range(
    path: impl AsRef<Path>,
    page_index: u32,
    start: usize,
    end: usize,
) -> Result<(String, Vec<PageRect>), DocumentError> {
    let pt = page_text(path, page_index)?;
    Ok(pt.range_rects(start, end))
}
