//! The text layer on pages that are not plain horizontal text: rotated pages and text, ligatures,
//! superscripts, invisible OCR text.

use quire_document::pdf::{page_sizes, page_text, word_at};
use quire_testkit::pdf_page;

fn words(content: &str, page_attributes: &str) -> (tempfile::TempDir, std::path::PathBuf, Vec<String>) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("paper.pdf");
    std::fs::write(&path, pdf_page(content, page_attributes)).unwrap();
    let words = page_text(&path, 0).unwrap().words.iter().map(|w| w.text.clone()).collect();
    (tmp, path, words)
}

/// A landscape table: the page turned with `/Rotate 90`. Positions are on the page as shown.
#[test]
fn a_rotated_page() {
    let (_tmp, path, words) = words("BT /F1 12 Tf 72 720 Td (Place cells fire) Tj ET", "/Rotate 90");
    assert_eq!(words, ["Place", "cells", "fire"]);
    let size = page_sizes(&path).unwrap()[0];
    assert_eq!((size.width, size.height), (792.0, 612.0));
    let text = page_text(&path, 0).unwrap();
    let rect = text.words[0].rect;
    assert!(rect.right <= size.width && rect.bottom <= size.height, "{rect:?} on {size:?}");
    let (x, y) = ((rect.left + rect.right) / 2.0, (rect.top + rect.bottom) / 2.0);
    assert_eq!(word_at(&path, 0, x, y).unwrap().unwrap().text, "Place");
}

/// Text running upwards (a rotated axis label or table header), on an upright page.
#[test]
fn rotated_text_reads_in_its_own_direction() {
    let (_tmp, _path, words) = words("BT /F1 12 Tf 0 1 -1 0 300 200 Tm (Firing rate) Tj ET", "");
    assert_eq!(words, ["Firing", "rate"]);
}

/// The `fi` ligature glyph reads as the two letters, so copy and search find "find".
#[test]
fn a_ligature_reads_as_its_letters() {
    let (_tmp, _path, words) = words(r"BT /F2 12 Tf 72 720 Td (\001nd) Tj ET", "");
    assert_eq!(words, ["find"]);
}

/// A superscript (smaller, raised) stays on its line and in its word.
#[test]
fn a_superscript_stays_with_its_word() {
    let (_tmp, path, words) =
        words("BT /F1 12 Tf 72 720 Td (x) Tj /F1 7 Tf 4 Ts (2) Tj 0 Ts /F1 12 Tf ( plus y) Tj ET", "");
    assert_eq!(words, ["x2", "plus", "y"]);
    assert_eq!(page_text(&path, 0).unwrap().lines.len(), 1);
}

/// A scan's OCR layer: text drawn invisibly (rendering mode 3) is still the page's text.
#[test]
fn invisible_ocr_text_is_read() {
    let (_tmp, _path, words) = words("BT 3 Tr /F1 12 Tf 72 720 Td (Scanned words) Tj ET", "");
    assert_eq!(words, ["Scanned", "words"]);
}

/// A page's horizontal text comes before a rotated label on it.
#[test]
fn horizontal_text_comes_before_a_rotated_label() {
    let content = "BT /F1 12 Tf 72 720 Td (Results here) Tj ET BT /F1 9 Tf 0 1 -1 0 60 400 Tm (Rate axis) Tj ET";
    let (_tmp, _path, words) = words(content, "");
    assert_eq!(words, ["Results", "here", "Rate", "axis"]);
}

/// Upside-down text (a page drawn turned half round) reads in its own order.
#[test]
fn upside_down_text_reads_in_its_own_order() {
    let (_tmp, _path, words) = words("BT /F1 12 Tf -1 0 0 -1 400 300 Tm (Place cells) Tj ET", "");
    assert_eq!(words, ["Place", "cells"]);
}

/// A diagonal watermark across two lines of text does not merge them: it is read apart, last.
#[test]
fn a_diagonal_watermark_stays_apart() {
    let content = "BT /F1 12 Tf 72 720 Td (First line here) Tj 0 -14 Td (Second line there) Tj ET \
                   BT /F1 60 Tf 0.707 0.707 -0.707 0.707 60 640 Tm (DRAFT) Tj ET";
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("paper.pdf");
    std::fs::write(&path, pdf_page(content, "")).unwrap();
    let text = page_text(&path, 0).unwrap();
    let lines: Vec<&str> = text.lines.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(&lines[..2], ["First line here", "Second line there"]);
}
