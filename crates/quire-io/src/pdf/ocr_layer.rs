//! Scanned papers with an OCR text layer: the words are drawn invisibly (text rendering mode 3)
//! behind the page image. `pdf-inspector` reads that layer only when a page has no visible text
//! at all, but an institution or publisher stamps every downloaded page with a visible line
//! ("Authorized licensed use limited to: University of Florida…", "Downloaded from
//! journals.physiology.org…"),
//! so the paper came out as that line alone (5 of 21 papers, 2026-10-19). Here the layer is made
//! visible (mode 3 → 0) in a copy of the PDF, which is then read like any other.

use lopdf::content::Content;
use lopdf::{Document, Object};

/// Fewer words per page than this: only stamps and page numbers were read.
const SPARSE_WORDS_PER_PAGE: usize = 40;

/// Whether `markdown` from a PDF of `pages` pages is too short to be the paper.
pub fn is_sparse(markdown: &str, pages: u32) -> bool {
    markdown.split_whitespace().count() < SPARSE_WORDS_PER_PAGE * pages.max(1) as usize
}

/// A copy of the PDF with invisible text made visible, or `None` when it has none (or cannot
/// be read and rewritten).
pub fn reveal(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut doc = Document::load_mem(bytes).ok()?;
    let mut changed = false;
    for page in doc.get_pages().into_values().collect::<Vec<_>>() {
        let content = doc.get_page_content(page);
        if let Some(new) = reveal_in(&content) {
            changed |= doc.change_page_content(page, new).is_ok();
        }
    }
    // Form XObjects (a page's text can sit in one).
    for object in doc.objects.values_mut() {
        let Object::Stream(stream) = object else { continue };
        let is_form = stream.dict.get(b"Subtype").and_then(Object::as_name).is_ok_and(|n| n == b"Form");
        if !is_form {
            continue;
        }
        let Ok(content) = stream.decompressed_content() else { continue };
        if let Some(new) = reveal_in(&content) {
            stream.set_plain_content(new);
            changed = true;
        }
    }
    if !changed {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len());
    doc.save_to(&mut out).ok()?;
    Some(out)
}

/// The content stream with every `3 Tr` (invisible) turned into `0 Tr` (fill), if it has any.
fn reveal_in(content: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Content::decode(content).ok()?;
    let mut found = false;
    for operation in &mut decoded.operations {
        if operation.operator == "Tr" && operation.operands.first().and_then(|o| o.as_i64().ok()) == Some(3) {
            operation.operands[0] = Object::Integer(0);
            found = true;
        }
    }
    if found { decoded.encode().ok() } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invisible_text_becomes_visible() {
        let stream = b"BT /F1 12 Tf 3 Tr 72 700 Td (Scanned words) Tj ET";
        let revealed = String::from_utf8(reveal_in(stream).unwrap()).unwrap();
        assert!(revealed.contains("0 Tr") && !revealed.contains("3 Tr"), "{revealed}");
        assert_eq!(reveal_in(b"BT /F1 12 Tf 72 700 Td (Plain) Tj ET"), None);
        assert!(is_sparse("Authorized licensed use limited to: University of Florida.", 8));
        assert!(!is_sparse(&"word ".repeat(400), 8));
    }
}
