//! PDF pages and text layer for the reader, powered by pure-Rust hayro (`quire-document`).
//!
//! Positions are in PDF points (1/72 inch) from the page's top-left corner.
//!
//! `PageSize`, `RenderedPage`, `PageRect`, `QuoteMatch` and `TextMatch` are `quire-document`'s own types, sent
//! as they are (UniFFI remote records: the compiler checks the field lists below against them).

pub use quire_document::pdf::{FigureRegion, PageRect, PageSize, QuoteMatch, RenderedPage, TextMatch};

use crate::{QuireError, Result, on_runtime};

#[uniffi::remote(Record)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

/// A rendered page: `width` x `height` pixels, RGBA, row by row.
#[uniffi::remote(Record)]
pub struct RenderedPage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[uniffi::remote(Record)]
pub struct PageRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Where one of the searched quotes is: its index in the request, its page, its line boxes.
#[uniffi::remote(Record)]
pub struct QuoteMatch {
    pub quote: u32,
    pub page: u32,
    pub rects: Vec<PageRect>,
}

/// One place [`pdf_search`] found: its page and line boxes.
#[uniffi::remote(Record)]
pub struct TextMatch {
    pub page: u32,
    pub rects: Vec<PageRect>,
}

/// Word bounding record for PDF text layer hit-testing.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PdfWordRecord {
    pub text: String,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Selected text and its union bounding boxes across lines.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct TextRangeRecord {
    pub text: String,
    pub rects: Vec<PageRect>,
}

/// A figure found on a page from its caption (`quire_document::pdf::figures`).
#[uniffi::remote(Record)]
pub struct FigureRegion {
    pub page: u32,
    pub label: String,
    pub caption: String,
    pub rect: PageRect,
}

/// The figures of the PDF, from their captions (for papers read from their PDF, whose text has
/// no images): each one's page and box, to crop from the rendered page.
#[uniffi::export]
pub async fn pdf_figures(path: String) -> Result<Vec<FigureRegion>> {
    blocking(move || quire_document::pdf::figures(&path).map_err(pdf_error)).await
}

/// The size of every page in points.
#[uniffi::export]
pub async fn pdf_page_sizes(path: String) -> Result<Vec<PageSize>> {
    blocking(move || quire_document::pdf::page_sizes(&path).map_err(pdf_error)).await
}

/// Page `index` (from 0) rendered `width` pixels wide.
#[uniffi::export]
pub async fn pdf_render_page(path: String, index: u32, width: u32) -> Result<RenderedPage> {
    blocking(move || quire_document::pdf::render_page_at(&path, index, width).map_err(pdf_error)).await
}

/// Where each of `quotes` appears (first match per quote), for drawing highlights on pages.
#[uniffi::export]
pub async fn pdf_find(path: String, quotes: Vec<String>) -> Result<Vec<QuoteMatch>> {
    blocking(move || quire_document::pdf::locate(&path, &quotes).map_err(pdf_error)).await
}

/// Every place `query` appears (the reader's find), in page order, at most `limit`: its words in
/// order, ignoring case and punctuation, the last one as the start of a word.
#[uniffi::export]
pub async fn pdf_search(path: String, query: String, limit: u32) -> Result<Vec<TextMatch>> {
    blocking(move || quire_document::pdf::search(&path, &query, limit as usize).map_err(pdf_error)).await
}

/// Finds the word at point `(x, y)` on page `page` (points from top-left).
#[uniffi::export]
pub async fn pdf_word_at(path: String, page: u32, x: f32, y: f32) -> Result<Option<PdfWordRecord>> {
    blocking(move || {
        let word = quire_document::pdf::word_at(&path, page, x, y).map_err(pdf_error)?;
        Ok(word.map(|w| PdfWordRecord {
            text: w.text,
            left: w.rect.left,
            top: w.rect.top,
            right: w.rect.right,
            bottom: w.rect.bottom,
        }))
    })
    .await
}

/// Finds the index of the nearest glyph to point `(x, y)` on page `page`.
#[uniffi::export]
pub async fn pdf_nearest_glyph(path: String, page: u32, x: f32, y: f32) -> Result<Option<u32>> {
    blocking(move || {
        let idx = quire_document::pdf::nearest_glyph(&path, page, x, y).map_err(pdf_error)?;
        Ok(idx.map(|i| i as u32))
    })
    .await
}

/// Returns text and selection rectangles for glyph range `[start, end)` on page `page`.
#[uniffi::export]
pub async fn pdf_text_range(path: String, page: u32, start: u32, end: u32) -> Result<TextRangeRecord> {
    blocking(move || {
        let (text, rects) =
            quire_document::pdf::text_range(&path, page, start as usize, end as usize).map_err(pdf_error)?;
        Ok(TextRangeRecord { text, rects })
    })
    .await
}

/// Returns the full plain text of page `page`.
#[uniffi::export]
pub async fn pdf_page_text(path: String, page: u32) -> Result<String> {
    blocking(move || Ok(quire_document::pdf::page_text(&path, page).map_err(pdf_error)?.text.clone())).await
}

async fn blocking<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    on_runtime(async move {
        tokio::task::spawn_blocking(f).await.map_err(|e| QuireError::Invalid(format!("PDF task failed: {e}")))?
    })
    .await
}

fn pdf_error(e: quire_document::DocumentError) -> QuireError {
    QuireError::Invalid(format!("PDF: {e}"))
}
