//! The reader's page API on a one-page PDF: sizes, rendering, text layer, quotes, hit-testing.

use quire_document::pdf::{locate, nearest_glyph, page_sizes, page_text, render_page_at, text_range, word_at};
use quire_testkit::tiny_pdf;

#[test]
fn pages_text_and_hit_testing() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test.pdf");
    std::fs::write(&path, tiny_pdf("Place cells fire in one place")).unwrap();

    // Page sizes
    let sizes = page_sizes(&path).unwrap();
    assert_eq!(sizes.len(), 1);
    assert_eq!((sizes[0].width, sizes[0].height), (612.0, 792.0));

    // Render page
    let page = render_page_at(&path, 0, 306).unwrap();
    assert_eq!((page.width, page.height), (306, 396));
    assert_eq!(page.pixels.len(), 306 * 396 * 4);

    // Page text
    let text = page_text(&path, 0).unwrap();
    assert_eq!(text.text.trim(), "Place cells fire in one place");
    assert_eq!(text.words.len(), 6);
    assert_eq!(text.words[0].text, "Place");

    // Locate quotes
    let matches = locate(&path, &["not in it".into(), "fire in one".into()]).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].quote, 1);
    assert_eq!(matches[0].page, 0);
    assert!(!matches[0].rects.is_empty());

    // Hit testing
    let word = word_at(&path, 0, 75.0, 72.0).unwrap();
    assert!(word.is_some());
    assert_eq!(word.unwrap().text, "Place");

    // Nearest glyph
    let nearest = nearest_glyph(&path, 0, 72.0, 72.0).unwrap();
    assert_eq!(nearest, Some(0));

    // Text range
    let (selected, rects) = text_range(&path, 0, 0, 5).unwrap();
    eprintln!("Selected: {:?}, rects: {:?}", selected, rects);
    assert_eq!(selected, "Place");
    assert_eq!(rects.len(), 1);
}

/// A figure is the area above its caption; an empty area there (a caption whose figure is on
/// another page, as manuscripts place them) is not one. Seen in real preprints: a crop of blank
/// page with a line number.
#[test]
fn figures_are_found_above_their_captions_and_empty_areas_left_out() {
    let body = "BT /F1 10 Tf 72 720 Td (Place cells fire in one place and grid cells tile the space around) Tj ET";
    let chart = "100 450 m 500 450 l 500 650 l 100 650 l s 100 450 m 300 600 l 500 500 l S";
    let caption = "BT /F1 10 Tf 72 420 Td (Figure 1. Firing rate over the arena.) Tj ET";
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("figures.pdf");
    std::fs::write(
        &path,
        quire_testkit::pdf_pages(&[&format!("{body} {chart} {caption}"), &format!("{body} {caption}")]),
    )
    .unwrap();
    let found = quire_document::pdf::figures(&path).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    let figure = &found[0];
    assert_eq!((figure.page, figure.label.as_str()), (0, "Figure 1"));
    assert!(figure.caption.starts_with("Figure 1. Firing rate"));
    // The chart spans 142-342 pt from the top: inside the figure.
    assert!(figure.rect.top <= 142.0 && figure.rect.bottom >= 342.0, "{:?}", figure.rect);
}
