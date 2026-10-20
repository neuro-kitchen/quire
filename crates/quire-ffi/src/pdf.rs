//! PDF pages for the reader, through PDFium (`pdfium-render`). The app ships the PDFium library
//! and passes its folder to [`pdf_init`]. Each call opens the file, does its work and closes it:
//! nothing PDFium holds crosses the boundary, and the app caches what it rendered.
//!
//! Positions are in PDF points (1/72 inch) from the page's top-left corner.

use std::sync::OnceLock;

use pdfium_render::prelude::*;

use crate::{QuireError, Result, on_runtime};

static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

/// A rendered page: `width` × `height` pixels, RGBA, row by row.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct RenderedPage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct PageRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Where one of the searched quotes is: its index in the request, its page, its line boxes.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct QuoteMatch {
    pub quote: u32,
    pub page: u32,
    pub rects: Vec<PageRect>,
}

/// Loads PDFium from `library_dir` (the folder holding `libpdfium.so`, `libpdfium.dylib` or
/// `pdfium.dll`), or from the system's library path for `None`. Later calls do nothing.
#[uniffi::export]
pub fn pdf_init(library_dir: Option<String>) -> Result<()> {
    if PDFIUM.get().is_some() {
        return Ok(());
    }
    let bindings = match library_dir {
        Some(dir) => Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir)),
        None => Pdfium::bind_to_system_library(),
    }
    .map_err(|e| QuireError::Invalid(format!("PDFium could not be loaded: {e}")))?;
    let _ = PDFIUM.set(Pdfium::new(bindings));
    Ok(())
}

/// The size of every page.
#[uniffi::export]
pub async fn pdf_page_sizes(path: String) -> Result<Vec<PageSize>> {
    blocking(move |pdfium| {
        let document = open(pdfium, &path)?;
        Ok(document.pages().iter().map(|p| PageSize { width: p.width().value, height: p.height().value }).collect())
    })
    .await
}

/// Page `index` (from 0) rendered `width` pixels wide, with its annotations and form fields.
#[uniffi::export]
pub async fn pdf_render_page(path: String, index: u32, width: u32) -> Result<RenderedPage> {
    blocking(move |pdfium| {
        let document = open(pdfium, &path)?;
        let index = PdfPageIndex::try_from(index).map_err(|_| QuireError::Invalid(format!("no page {index}")))?;
        let page = document.pages().get(index).map_err(pdf_error)?;
        let config = PdfRenderConfig::new()
            .set_target_width(width.clamp(16, 8192) as Pixels)
            .render_form_data(true)
            .render_annotations(true);
        let bitmap = page.render_with_config(&config).map_err(pdf_error)?;
        Ok(RenderedPage {
            width: bitmap.width() as u32,
            height: bitmap.height() as u32,
            pixels: bitmap.as_rgba_bytes(),
        })
    })
    .await
}

/// Where each of `quotes` appears (first match per quote), for drawing highlights on pages.
/// Quotes split across lines are matched by their first words when the whole quote is not found.
#[uniffi::export]
pub async fn pdf_find(path: String, quotes: Vec<String>) -> Result<Vec<QuoteMatch>> {
    blocking(move |pdfium| {
        let document = open(pdfium, &path)?;
        let mut found = Vec::new();
        'quotes: for (i, quote) in quotes.iter().enumerate() {
            for attempt in search_texts(quote) {
                for (page_index, page) in document.pages().iter().enumerate() {
                    let height = page.height().value;
                    let text = page.text().map_err(pdf_error)?;
                    let search = text.search(&attempt, &PdfSearchOptions::new()).map_err(pdf_error)?;
                    if let Some(segments) = search.find_next() {
                        let rects = segments
                            .iter()
                            .map(|segment| {
                                let b = segment.bounds();
                                PageRect {
                                    left: b.left().value,
                                    top: height - b.top().value,
                                    right: b.right().value,
                                    bottom: height - b.bottom().value,
                                }
                            })
                            .collect();
                        found.push(QuoteMatch { quote: i as u32, page: page_index as u32, rects });
                        continue 'quotes;
                    }
                }
            }
        }
        Ok(found)
    })
    .await
}

/// The quote, then (if long) its first eight words: PDF text breaks lines and hyphenates.
fn search_texts(quote: &str) -> Vec<String> {
    let words: Vec<&str> = quote.split_whitespace().collect();
    let mut texts = vec![words.join(" ")];
    if words.len() > 8 {
        texts.push(words[..8].join(" "));
    }
    texts.retain(|t| !t.is_empty());
    texts
}

async fn blocking<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&'static Pdfium) -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    let pdfium = PDFIUM.get().ok_or_else(|| QuireError::Invalid("PDFium is not loaded (pdf_init)".into()))?;
    on_runtime(async move {
        tokio::task::spawn_blocking(move || f(pdfium))
            .await
            .map_err(|e| QuireError::Invalid(format!("PDF task failed: {e}")))?
    })
    .await
}

fn open<'a>(pdfium: &'a Pdfium, path: &str) -> Result<PdfDocument<'a>> {
    pdfium.load_pdf_from_file(path, None).map_err(|e| QuireError::Invalid(format!("{path}: {e}")))
}

fn pdf_error(e: PdfiumError) -> QuireError {
    QuireError::Invalid(format!("PDF: {e}"))
}
