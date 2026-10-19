//! Checking a downloaded or imported PDF with Firecrawl's `pdf-inspector` (MIT): that it is a
//! PDF at all (a site may send an HTML page named `.pdf`), whether it has a text layer or is
//! scanned, and which DOIs or arXiv IDs its first pages print, to confirm it is the expected
//! paper (arXiv PDFs print `arXiv:1706.03762v7` in the margin, not their DOI).

mod ocr_layer;
pub mod plain;
pub mod tidy;

pub use ocr_layer::is_sparse;

use quire_model::{Doi, Identifier, PdfKind};

use crate::error::{IoError, Result};

/// The converter behind [`to_markdown`], written into cached reader text so a newer one rebuilds it.
/// Change it together with the `pdf-inspector` version, and when quire's own processing changes.
pub const READER_CONVERTER: &str = "pdf-inspector 1.25.2 + quire 4";

/// The whole paper as Markdown in reading order (two-column pages become one column; headings,
/// lists and tables kept), for reader mode.
pub fn to_markdown(bytes: &[u8]) -> Result<String> {
    if !looks_like_pdf(bytes) {
        return Err(IoError::NotPdf("the file does not start with %PDF-".into()));
    }
    let (markdown, pages) = convert(bytes)?;
    // Only stamps read: a scan whose OCR layer is invisible ([`ocr_layer`]).
    if markdown.as_deref().is_none_or(|m| ocr_layer::is_sparse(m, pages))
        && let Some(revealed) = ocr_layer::reveal(bytes)
        && let Ok((Some(fuller), _)) = convert(&revealed)
        && fuller.split_whitespace().count() > markdown.as_deref().map_or(0, |m| m.split_whitespace().count())
    {
        return Ok(tidy::tidy(&fuller));
    }
    match markdown {
        Some(markdown) if !markdown.trim().is_empty() => Ok(tidy::tidy(&markdown)),
        _ => Err(IoError::NotPdf("no text layer (a scanned PDF needs OCR)".into())),
    }
}

/// `pdf-inspector`'s Markdown (if any text) and page count.
fn convert(bytes: &[u8]) -> Result<(Option<String>, u32)> {
    let parsed = std::panic::catch_unwind(|| pdf_inspector::process_pdf_mem(bytes))
        .map_err(|_| IoError::NotPdf("the PDF could not be read (malformed)".into()))?;
    let result = parsed.map_err(|e| IoError::NotPdf(e.to_string()))?;
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
pub fn looks_like_pdf(bytes: &[u8]) -> bool {
    bytes.trim_ascii_start().starts_with(b"%PDF-")
}

/// Inspects `bytes`; an error if they are not a readable PDF.
pub fn inspect(bytes: &[u8]) -> Result<PdfInfo> {
    if !looks_like_pdf(bytes) {
        return Err(IoError::NotPdf("the file does not start with %PDF-".into()));
    }
    let options = pdf_inspector::PdfOptions::new().pages(PAGES_READ);
    let result =
        pdf_inspector::process_pdf_mem_with_options(bytes, options).map_err(|e| IoError::NotPdf(e.to_string()))?;
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
pub fn find_identifiers(text: &str) -> Vec<Identifier> {
    let mut found: Vec<Identifier> = Vec::new();
    // The extracted text is Markdown: identifiers also sit inside `[text](url)` and `<url>`.
    let words = text.split(|c: char| c.is_whitespace() || matches!(c, '[' | ']' | '(' | ')' | '<' | '>' | '"' | '|'));
    for word in words.filter(|w| !w.is_empty()) {
        let candidate = match (word.find("10."), word.to_ascii_lowercase().find("arxiv:")) {
            (_, Some(start)) => &word[start..],
            (Some(start), None) => &word[start..],
            (None, None) => continue,
        };
        let candidate = candidate.trim_end_matches(['.', ',', ';', ':', ')', ']', '}', '>', '"', '\'', '*', '_']);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A one-page PDF showing `text`, with a correct cross-reference table.
    pub(crate) fn tiny_pdf(text: &str) -> Vec<u8> {
        let stream = format!("BT /F1 12 Tf 72 720 Td ({text}) Tj ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
            format!("<< /Length {} >>\nstream\n{stream}\nendstream", stream.len()),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        ];
        let mut pdf = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for (i, body) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.push_str(&format!("{} 0 obj\n{body}\nendobj\n", i + 1));
        }
        let xref = pdf.len();
        pdf.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1));
        for offset in offsets {
            pdf.push_str(&format!("{offset:010} 00000 n \n"));
        }
        pdf.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1));
        pdf.into_bytes()
    }

    #[test]
    fn inspects_a_text_pdf() {
        let info = inspect(&tiny_pdf("Nature 500, 54 (2013) doi:10.1038/nature12373.")).unwrap();
        assert_eq!((info.kind, info.pages), (PdfKind::Text, 1));
        assert_eq!(info.identifiers.iter().map(|d| d.key()).collect::<Vec<_>>(), ["10.1038/NATURE12373"]);
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
        assert!(matches!(inspect(b"<!DOCTYPE html><title>Preparing to download ...</title>"), Err(IoError::NotPdf(_))));
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
}
