//! PDF pages and text layer through quire-ffi. Pure Rust (hayro): runs without any native binaries.

use quire_ffi::{
    pdf_find, pdf_nearest_glyph, pdf_page_sizes, pdf_render_page, pdf_search, pdf_text_range, pdf_word_at,
};
use quire_testkit::tiny_pdf;

#[tokio::test]
async fn pages_render_and_quotes_are_found() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("paper.pdf");
    std::fs::write(&path, tiny_pdf("Place cells fire in one place")).unwrap();
    let path = path.display().to_string();

    let sizes = pdf_page_sizes(path.clone()).await.unwrap();
    assert_eq!(sizes.len(), 1);
    assert_eq!((sizes[0].width, sizes[0].height), (612.0, 792.0));

    let page = pdf_render_page(path.clone(), 0, 306).await.unwrap();
    assert_eq!((page.width, page.height), (306, 396), "half size, same aspect");
    assert_eq!(page.pixels.len(), 306 * 396 * 4);
    assert!(page.pixels.chunks(4).any(|p| p[0] < 128), "some dark text pixels");

    let found = pdf_find(path.clone(), vec!["not in it".into(), "fire in one".into()]).await.unwrap();
    assert_eq!(found.len(), 1);
    let (m, r) = (&found[0], found[0].rects[0]);
    assert_eq!((m.quote, m.page), (1, 0));
    assert!(r.left >= 72.0 && r.top >= 60.0 && r.bottom <= 80.0, "{r:?}");

    // Hit-testing words
    let word = pdf_word_at(path.clone(), 0, 75.0, 70.0).await.unwrap();
    assert!(word.is_some());
    assert_eq!(word.unwrap().text, "Place");

    // Nearest glyph
    let nearest = pdf_nearest_glyph(path.clone(), 0, 72.0, 70.0).await.unwrap();
    assert_eq!(nearest, Some(0));

    // Text range selection
    let range = pdf_text_range(path.clone(), 0, 0, 5).await.unwrap();
    assert_eq!(range.text, "Place");
    assert_eq!(range.rects.len(), 1);

    // Find: every place, the last word as typed so far.
    let hits = pdf_search(path.clone(), "place".into(), 10).await.unwrap();
    assert_eq!(hits.len(), 2, "Place … place");
    assert!(hits[0].rects[0].left < hits[1].rects[0].left);
    assert_eq!(pdf_search(path.clone(), "in on".into(), 10).await.unwrap().len(), 1);
    assert_eq!(pdf_search(path.clone(), "place".into(), 1).await.unwrap().len(), 1, "at most the limit");

    assert!(pdf_render_page(path, 3, 100).await.is_err(), "no page 3");
}
