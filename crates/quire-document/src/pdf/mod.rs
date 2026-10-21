//! PDF, the fixed-layout format.
//!
//! - Checking a downloaded or imported PDF with Firecrawl's `pdf-inspector` (MIT): that it is a
//!   PDF at all (a site may send an HTML page named `.pdf`), whether it has a text layer or is
//!   scanned, and which DOIs or arXiv IDs its first pages print, to confirm it is the expected
//!   paper (arXiv PDFs print `arXiv:1706.03762v7` in the margin, not their DOI).
//! - [`to_markdown`]: the paper as reader Markdown: pdf-inspector's, or the hayro layer's when that
//!   reads too little (`structure`); then [`tidy`] ([`plain`] for poppler's text).
//! - [`pages`]: the reader's pages with hayro (pure Rust): rendering ([`render`]), the text layer
//!   ([`layer`], ordered by [`layout`]), hit-testing and quotes ([`find`]), over a document cache
//!   ([`cache`]).

pub mod cache;
pub mod figures;
pub mod find;
pub mod layer;
pub mod layout;
mod ocr_layer;
pub mod pages;
pub mod plain;
pub mod render;
mod schedule;
mod structure;
pub mod tidy;
pub mod types;

pub use cache::{DocumentCache, global_cache};
pub use figures::{FigureRegion, figures};
pub use ocr_layer::is_sparse;
pub use pages::{locate, nearest_glyph, page_sizes, page_text, render_page_at, search, text_range, word_at};
pub use types::{Glyph, GlyphBox, Line, PageRect, PageSize, PageText, QuoteMatch, RenderedPage, TextMatch, WordBox};

use quire_model::{Doi, Identifier, PdfKind};

use crate::error::{DocumentError, Result};

/// The converter behind [`to_markdown`], written into cached reader text so a newer one rebuilds it.
/// Change it together with the `pdf-inspector` version, and when quire's own processing changes.
pub const READER_CONVERTER: &str = "pdf-inspector 1.25.2 + quire 6 (hayro layer fallback)";

/// Word agreement between pdf-inspector's text and the layer's under which pdf-inspector's is taken
/// as partial. Measured on the owner's library (`.knowledge/pdf-engine.md`, T04 step 3): below 0.7 the
/// layer was better in every paper (pdf-inspector's word F1 0.14–0.74, the layer's 0.87–0.98); at 0.8
/// one paper got worse.
const PARTIAL: f64 = 0.7;

/// The whole paper as Markdown in reading order (two-column pages become one column; headings,
/// lists and tables kept), for reader mode.
pub fn to_markdown(bytes: &[u8]) -> Result<String> {
    if !looks_like_pdf(bytes) {
        return Err(DocumentError::NotPdf("the file does not start with %PDF-".into()));
    }
    let converted = convert(bytes);
    let (mut markdown, pages) = match &converted {
        Ok((markdown, pages)) => (markdown.clone(), *pages),
        Err(_) => (None, 0),
    };
    let words = |text: Option<&str>| text.map_or(0, |t| t.split_whitespace().count());
    // Only stamps read: a scan whose OCR layer is invisible ([`ocr_layer`]). What the reveal step
    // reads still goes through the layer check below (it can be partial too).
    if markdown.as_deref().is_none_or(|m| ocr_layer::is_sparse(m, pages))
        && let Some(revealed) = ocr_layer::reveal(bytes)
        && let Ok((Some(fuller), _)) = convert(&revealed)
        && words(Some(&fuller)) > words(markdown.as_deref())
    {
        markdown = Some(fuller);
    }
    match choose(markdown, pages, layer_markdown(bytes)) {
        Some(markdown) => Ok(tidy::tidy(&markdown)),
        None => converted.and(Err(DocumentError::NotPdf("no text layer (a scanned PDF needs OCR)".into()))),
    }
}

/// pdf-inspector's text (`markdown`, of a `pages`-page PDF), or the hayro layer's (`layer`, with its
/// page count) when the layer reads more words and pdf-inspector read too little or only part of
/// the paper (the two share less than [`PARTIAL`] of their words). `None` if neither has text.
fn choose(markdown: Option<String>, pages: u32, layer: Option<(String, u32)>) -> Option<String> {
    let words = |text: &str| text.split_whitespace().count();
    let markdown = markdown.filter(|m| !m.trim().is_empty());
    if let Some((layer, layer_pages)) = layer
        && !ocr_layer::is_sparse(&layer, pages.max(layer_pages))
        && words(&layer) > markdown.as_deref().map_or(0, words)
        && markdown
            .as_deref()
            .is_none_or(|m| ocr_layer::is_sparse(m, pages) || find::word_agreement(m, &layer) < PARTIAL)
    {
        return Some(layer);
    }
    markdown
}

/// The paper as reader Markdown from the hayro text layer alone (structured, then tidied): what
/// [`to_markdown`] falls back to.
pub fn layer_to_markdown(bytes: &[u8]) -> Result<String> {
    let (markdown, _) = layer_markdown(bytes).ok_or_else(|| DocumentError::NotPdf("hayro cannot read it".into()))?;
    Ok(tidy::tidy(&markdown))
}

/// Every page's hayro text layer as reader Markdown (untidied), and the page count; `None` if hayro
/// cannot read the file (or panics on it).
fn layer_markdown(bytes: &[u8]) -> Option<(String, u32)> {
    std::panic::catch_unwind(|| {
        let pdf = hayro_syntax::Pdf::new(bytes.to_vec()).ok()?;
        let pages: Vec<PageText> =
            pdf.pages().iter().filter_map(|page| layer::TextCollector::extract(page).ok()).collect();
        Some((structure::markdown(&pages), pages.len() as u32))
    })
    .ok()
    .flatten()
}

/// `pdf-inspector`'s Markdown (if any text) and page count.
fn convert(bytes: &[u8]) -> Result<(Option<String>, u32)> {
    let parsed = std::panic::catch_unwind(|| pdf_inspector::process_pdf_mem(bytes))
        .map_err(|_| DocumentError::NotPdf("the PDF could not be read (malformed)".into()))?;
    let result = parsed.map_err(|e| not_pdf(&e))?;
    let markdown = result.markdown.filter(|m| !m.trim().is_empty());
    Ok((markdown, result.page_count))
}

/// Characters of the first pages kept for comparing titles.
const OPENING_CHARS: usize = 6000;

/// Pages read for DOIs: the article's first page carries it.
const PAGES_READ: [u32; 2] = [1, 2];

/// What a PDF is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfInfo {
    pub kind: PdfKind,
    pub pages: u32,
    /// The document's `/Title`, if set.
    pub title: Option<String>,
    /// DOIs and arXiv IDs printed on the first pages, in order of appearance.
    pub identifiers: Vec<Identifier>,
    /// The start of the first pages' text, for comparing titles.
    pub opening: String,
}

/// Whether a PDF is the paper a reference describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfMatch {
    /// It prints one of the reference's identifiers.
    Identifier,
    /// Most of the reference's title is on its first pages.
    Title,
    /// Nothing to compare (the reference has no identifier or title the PDF could show).
    Unknown,
    /// It prints other identifiers and not the reference's title: another paper.
    Different(Vec<Identifier>),
    /// It prints no identifier and not the reference's title.
    Unlikely,
}

impl PdfMatch {
    /// Whether an automatic download may be attached without asking.
    pub fn accepted(&self) -> bool {
        matches!(self, Self::Identifier | Self::Title | Self::Unknown)
    }

    /// Why it was not accepted, for messages.
    pub fn reason(&self) -> Option<String> {
        match self {
            Self::Different(ids) => Some(format!(
                "it shows {}, not this reference",
                ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
            )),
            Self::Unlikely => Some("its first pages show neither this reference's DOI nor its title".into()),
            _ => None,
        }
    }
}

/// Letters below which a PDF counts as having no text layer to compare.
const MIN_TEXT: usize = 200;

/// How much of the title must appear on the first pages.
const TITLE_SHARE: f64 = 0.6;

/// Whether the PDF `info` is the work with `identifiers` and `title`.
pub fn verdict(info: &PdfInfo, identifiers: &[Identifier], title: Option<&str>) -> PdfMatch {
    let ours = |id: &Identifier| identifiers.iter().any(|o| o.kind() == id.kind() && o.key() == id.key());
    if info.identifiers.iter().any(ours) {
        return PdfMatch::Identifier;
    }
    // A scanned PDF (old papers) has no text to compare: no reason to refuse it.
    if info.opening.chars().filter(|c| c.is_alphabetic()).count() < MIN_TEXT {
        return PdfMatch::Unknown;
    }
    let words = title.map(significant_words).unwrap_or_default();
    if words.len() >= 2 {
        // Spaces around, so the first and last words match as whole words too.
        let haystack = normalise(&format!(" {} {} ", info.title.as_deref().unwrap_or_default(), info.opening));
        let found = words.iter().filter(|w| haystack.contains(w.as_str())).count();
        if found as f64 / words.len() as f64 >= TITLE_SHARE {
            return PdfMatch::Title;
        }
    }
    match (info.identifiers.is_empty(), words.len() >= 3) {
        (false, _) if !identifiers.is_empty() || words.len() >= 3 => PdfMatch::Different(info.identifiers.clone()),
        (true, true) => PdfMatch::Unlikely,
        _ => PdfMatch::Unknown,
    }
}

/// Lowercase letters and digits, everything else a space (so hyphens and line breaks do not matter).
fn normalise(text: &str) -> String {
    text.chars().map(|c| if c.is_alphanumeric() { c.to_lowercase().next().unwrap_or(c) } else { ' ' }).collect()
}

/// A title's words of four or more letters (short words are too common to tell papers apart).
fn significant_words(title: &str) -> Vec<String> {
    normalise(title).split_whitespace().filter(|w| w.chars().count() >= 4).map(|w| format!(" {w} ")).collect()
}

/// Whether `bytes` start like a PDF file.
pub(crate) fn looks_like_pdf(bytes: &[u8]) -> bool {
    bytes.trim_ascii_start().starts_with(b"%PDF-")
}

/// Inspects `bytes`; an error if they are not a readable PDF. Downloaded and imported files are
/// untrusted: a panic in the parser is an error here (as in [`to_markdown`]), not a crash.
pub fn inspect(bytes: &[u8]) -> Result<PdfInfo> {
    std::panic::catch_unwind(|| inspect_unguarded(bytes))
        .unwrap_or_else(|_| Err(DocumentError::NotPdf("the PDF could not be read (malformed)".into())))
}

fn inspect_unguarded(bytes: &[u8]) -> Result<PdfInfo> {
    if !looks_like_pdf(bytes) {
        return Err(DocumentError::NotPdf("the file does not start with %PDF-".into()));
    }
    let options = pdf_inspector::PdfOptions::new().pages(PAGES_READ);
    let result = pdf_inspector::process_pdf_mem_with_options(bytes, options).map_err(|e| not_pdf(&e))?;
    let kind = match result.pdf_type {
        pdf_inspector::PdfType::TextBased => PdfKind::Text,
        pdf_inspector::PdfType::Scanned | pdf_inspector::PdfType::ImageBased => PdfKind::Scanned,
        pdf_inspector::PdfType::Mixed => PdfKind::Mixed,
    };
    let mut markdown = result.markdown.unwrap_or_default();
    // A scan with only its stamp read: its opening from the invisible OCR layer ([`ocr_layer`]).
    if ocr_layer::is_sparse(&markdown, PAGES_READ.len() as u32)
        && let Some(revealed) = ocr_layer::reveal(bytes)
        && let Ok(fuller) =
            pdf_inspector::process_pdf_mem_with_options(&revealed, pdf_inspector::PdfOptions::new().pages(PAGES_READ))
        && let Some(text) = fuller.markdown.filter(|m| m.len() > markdown.len())
    {
        markdown = text;
    }
    let identifiers = find_identifiers(&markdown);
    let opening: String = markdown.chars().take(OPENING_CHARS).collect();
    Ok(PdfInfo {
        kind,
        pages: result.page_count,
        title: result.title.filter(|t| !t.trim().is_empty()),
        identifiers,
        opening,
    })
}

/// DOIs (`10.<registrant>/<suffix>`) and arXiv IDs (`arXiv:<id>`) in `text`, without trailing
/// punctuation; an arXiv DOI counts as the arXiv ID.
pub(crate) fn find_identifiers(text: &str) -> Vec<Identifier> {
    let mut found: Vec<Identifier> = Vec::new();
    // The extracted text is Markdown: identifiers also sit inside `[text](url)` and `<url>`.
    // Parentheses do not split words: old DOIs have them (`10.1016/0006-8993(71)90358-1`).
    let words = text.split(|c: char| c.is_whitespace() || matches!(c, '[' | ']' | '<' | '>' | '"' | '|'));
    for word in words.filter(|w| !w.is_empty()) {
        let mut candidate = match (word.find("10."), word.to_ascii_lowercase().find("arxiv:")) {
            (_, Some(start)) => &word[start..],
            (Some(start), None) => &word[start..],
            (None, None) => continue,
        };
        // Trailing punctuation, and a `)` only while unbalanced (the end of `(doi:…)` or of a link).
        loop {
            let trimmed = candidate.trim_end_matches(['.', ',', ';', ':', ']', '}', '>', '"', '\'', '*', '_']);
            let trimmed = match trimmed.strip_suffix(')') {
                Some(rest) if trimmed.matches('(').count() < trimmed.matches(')').count() => rest,
                _ => trimmed,
            };
            if trimmed.len() == candidate.len() {
                break;
            }
            candidate = trimmed;
        }
        let parsed = match Doi::parse(candidate) {
            Ok(doi) => Some(doi.arxiv_id().map_or(Identifier::Doi(doi), Identifier::Arxiv)),
            Err(_) => Identifier::parse(candidate).ok().filter(|id| matches!(id, Identifier::Arxiv(_))),
        };
        if let Some(id) = parsed
            && !found.iter().any(|known| known.kind() == id.kind() && known.key() == id.key())
        {
            found.push(id);
        }
    }
    found
}

/// pdf-inspector's error, without its own "Not a PDF:" (ours says it).
fn not_pdf(error: &impl std::fmt::Display) -> DocumentError {
    let text = error.to_string();
    let detail = text.strip_prefix("Not a PDF:").map_or(text.as_str(), str::trim_start);
    DocumentError::NotPdf(detail.to_string())
}

#[cfg(test)]
mod tests {
    use quire_testkit::tiny_pdf;

    use super::*;

    /// A scan's invisible OCR text on a page with an inline image (`BI … ID … EI`, how scanners
    /// store strips): pdf-inspector skips invisible text, and still reads too little from the copy
    /// the reveal step writes, so only the hayro layer reads it (3 papers of the owner's library came
    /// out empty before).
    #[test]
    fn the_hayro_layer_reads_what_pdf_inspector_skips() {
        let line = "Place cells fire when the animal is in one place and grid cells fire across the room";
        let text: String =
            (0..30).map(|i| format!("BT 3 Tr /F1 10 Tf 72 {} Td ({line}) Tj ET ", 740 - i * 14)).collect();
        let content = format!("q 100 0 0 10 72 60 cm BI /W 4 /H 1 /BPC 8 /CS /G ID \u{1}\u{2}\u{3}\u{4} EI Q {text}");
        let bytes = quire_testkit::pdf_page(&content, "");
        let (inspected, _) = convert(&bytes).unwrap();
        assert!(inspected.is_none_or(|m| m.split_whitespace().count() < 10), "pdf-inspector reads it after all");
        let revealed = ocr_layer::reveal(&bytes).map(|copy| convert(&copy).map(|(m, _)| m));
        let revealed_words = revealed.and_then(Result::ok).flatten().map_or(0, |m| m.split_whitespace().count());
        assert!(revealed_words < 10, "the reveal step gets it after all ({revealed_words} words)");
        let markdown = to_markdown(&bytes).unwrap();
        assert!(markdown.contains("Place cells fire when the animal is in one place"), "{markdown}");
    }

    #[test]
    fn inspects_a_text_pdf() {
        let info = inspect(&tiny_pdf("Nature 500, 54 (2013) doi:10.1038/nature12373.")).unwrap();
        assert_eq!((info.kind, info.pages), (PdfKind::Text, 1));
        assert_eq!(info.identifiers.iter().map(|d| d.key()).collect::<Vec<_>>(), ["10.1038/NATURE12373"]);
    }

    /// A page pdf-inspector reads only in part (a visible header over a scan's invisible text, with
    /// an inline image), though not too little to be sparse: the layer's text, which has it all.
    #[test]
    fn a_partly_read_paper_takes_the_layers_text() {
        let visible: String = (0..5)
            .map(|i| {
                format!(
                    "BT /F1 10 Tf 72 {} Td (Journal of Physiology volume twelve issue three pages one to ten) Tj ET ",
                    770 - i * 12
                )
            })
            .collect();
        let line = "Place cells fire when the animal is in one place and grid cells fire across the room";
        let hidden: String =
            (0..20).map(|i| format!("BT 3 Tr /F1 10 Tf 72 {} Td ({line}) Tj ET ", 680 - i * 14)).collect();
        let content =
            format!("q 100 0 0 10 72 60 cm BI /W 4 /H 1 /BPC 8 /CS /G ID \u{1}\u{2}\u{3}\u{4} EI Q {visible}{hidden}");
        let bytes = quire_testkit::pdf_page(&content, "");
        let (inspected, pages) = convert(&bytes).unwrap();
        let inspected = inspected.unwrap_or_default();
        assert!(!ocr_layer::is_sparse(&inspected, pages), "pdf-inspector's text should not be sparse here");
        assert!(!inspected.contains("Place cells"), "pdf-inspector reads the hidden text after all");
        let markdown = to_markdown(&bytes).unwrap();
        assert!(markdown.contains("Place cells fire when the animal is in one place"), "{markdown}");
    }

    /// Two scanned pages (invisible OCR text), one with an inline image: pdf-inspector reads
    /// neither, nor the revealed copy; both pages come through the layer.
    #[test]
    fn two_scanned_pages_both_come_through_the_layer() {
        let lines = |words: &str| -> String {
            (0..25).map(|i| format!("BT 3 Tr /F1 10 Tf 72 {} Td ({words}) Tj ET ", 740 - i * 14)).collect()
        };
        let first = lines("Grid cells fire on a lattice across the whole room as the animal runs");
        let second = format!(
            "q 100 0 0 10 72 60 cm BI /W 4 /H 1 /BPC 8 /CS /G ID \u{1}\u{2}\u{3}\u{4} EI Q {}",
            lines("Place cells fire when the animal is in one place of the arena")
        );
        let bytes = quire_testkit::pdf_pages(&[&first, &second]);
        let markdown = to_markdown(&bytes).unwrap();
        assert!(markdown.contains("Grid cells fire on a lattice"), "{markdown}");
        assert!(markdown.contains("Place cells fire when the animal"), "the second page is missing");
    }

    /// The choice between pdf-inspector's text (whatever produced it: directly, or the reveal step)
    /// and the layer's.
    #[test]
    fn choosing_between_pdf_inspector_and_the_layer() {
        let paper = "Place cells fire when the animal is in one place and grid cells fire across the room. ".repeat(20);
        let partial = format!("{} {}", "Journal of Physiology volume twelve issue three.".repeat(10), &paper[..200]);
        let layer = Some((paper.clone(), 1));
        // Partial (most words not the layer's): the layer.
        assert_eq!(choose(Some(partial.clone()), 1, layer.clone()), Some(paper.clone()));
        // Nothing read: the layer.
        assert_eq!(choose(None, 1, layer.clone()), Some(paper.clone()));
        // Nearly the same words, a few fewer than the layer's: pdf-inspector's stays.
        let close = paper.replacen("grid ", "", 3);
        assert_eq!(choose(Some(close.clone()), 1, layer.clone()), Some(close));
        // A sparse layer is never chosen.
        assert_eq!(choose(Some(partial.clone()), 30, layer), Some(partial));
        assert_eq!(choose(None, 1, None), None);
    }

    #[test]
    fn whole_paper_as_markdown() {
        let markdown = to_markdown(&tiny_pdf("Attention Is All You Need")).unwrap();
        assert!(markdown.contains("Attention Is All You Need"), "{markdown}");
    }

    #[test]
    fn verdicts() {
        let doi = Identifier::parse("10.1038/nature12373").unwrap();
        let info = |ids: Vec<Identifier>, text: &str| PdfInfo {
            kind: PdfKind::Text,
            pages: 1,
            title: None,
            identifiers: ids,
            // A real first page has plenty of text besides the title.
            opening: if text.is_empty() {
                String::new()
            } else {
                format!("{text}{}", " results discussion methods introduction conclusion".repeat(8))
            },
        };
        let title = Some("Nanometre-scale thermometry in a living cell");
        assert_eq!(verdict(&info(vec![doi.clone()], ""), std::slice::from_ref(&doi), title), PdfMatch::Identifier);
        let page = "Letter\nNanometre-scale thermo-\nmetry in a living cell\nG. Kucsko";
        assert_eq!(
            verdict(&info(vec![], "Nanometre scale thermometry in a living cell"), std::slice::from_ref(&doi), title),
            PdfMatch::Title
        );
        assert_ne!(verdict(&info(vec![], page), std::slice::from_ref(&doi), title), PdfMatch::Identifier);
        let other = Identifier::parse("10.1000/other").unwrap();
        assert!(matches!(
            verdict(&info(vec![other], "Deep residual learning"), std::slice::from_ref(&doi), title),
            PdfMatch::Different(_)
        ));
        assert_eq!(
            verdict(&info(vec![], "Deep residual learning for image recognition"), std::slice::from_ref(&doi), title),
            PdfMatch::Unlikely
        );
        assert_eq!(verdict(&info(vec![], "anything"), &[], None), PdfMatch::Unknown);
        assert_eq!(
            verdict(&info(vec![], ""), std::slice::from_ref(&doi), title),
            PdfMatch::Unknown,
            "scanned: nothing to compare"
        );
    }

    #[test]
    fn html_is_not_a_pdf() {
        assert!(matches!(
            inspect(b"<!DOCTYPE html><title>Preparing to download ...</title>"),
            Err(DocumentError::NotPdf(_))
        ));
    }

    #[test]
    fn finds_identifiers_in_text() {
        let found = find_identifiers(
            "See https://doi.org/10.1371/journal.pone.0173392, and (doi:10.1000/xyz). arXiv:1706.03762v7 [cs.CL] \
             [10.1038/s41467-019-11731-7](https://doi.org/10.1038/s41467-019-11731-7)",
        );
        let keys: Vec<String> = found.iter().map(ToString::to_string).collect();
        assert_eq!(
            keys,
            [
                "doi:10.1371/journal.pone.0173392",
                "doi:10.1000/xyz",
                "arXiv:1706.03762v7",
                "doi:10.1038/s41467-019-11731-7"
            ]
        );
    }

    /// Old DOIs keep their parentheses, even at the end; an enclosing one is not part of them.
    #[test]
    fn dois_with_parentheses_are_whole() {
        let found = find_identifiers(
            "Brain Res. doi:10.1016/0006-8993(71)90358-1. Also (see 10.1002/(SICI)1097-4636(199601)) and \
             [link](https://doi.org/10.1016/S0006-8993(97)00002-1)",
        );
        let keys: Vec<String> = found.iter().map(ToString::to_string).collect();
        assert_eq!(
            keys,
            [
                "doi:10.1016/0006-8993(71)90358-1",
                "doi:10.1002/(SICI)1097-4636(199601)",
                "doi:10.1016/S0006-8993(97)00002-1"
            ]
        );
    }
}
