//! PDF pages through PDFium. Needs the PDFium library: set `QUIRE_PDFIUM_DIR` to its folder (the
//! app's Gradle build downloads it to `app/shared/build/pdfium/lib`); skipped otherwise.

use quire_ffi::{pdf_find, pdf_init, pdf_page_sizes, pdf_render_page};

/// A one-page US Letter PDF with `text` at the top left (as in the engine's tests).
fn tiny_pdf(text: &str) -> Vec<u8> {
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

#[tokio::test]
async fn pages_render_and_quotes_are_found() {
    let Ok(dir) = std::env::var("QUIRE_PDFIUM_DIR") else {
        eprintln!("skipped: QUIRE_PDFIUM_DIR is not set");
        return;
    };
    pdf_init(Some(dir)).unwrap();
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
    assert!(page.pixels.chunks(4).any(|p| p[0] < 128), "some dark (text) pixels");

    let found = pdf_find(path.clone(), vec!["not in it".into(), "fire in one".into()]).await.unwrap();
    assert_eq!(found.len(), 1);
    let (m, r) = (&found[0], found[0].rects[0]);
    assert_eq!((m.quote, m.page), (1, 0));
    // Text baseline at 720 pt from the bottom: about 72 pt from the top, from the left margin on.
    assert!(r.left > 72.0 && r.top > 60.0 && r.bottom < 80.0, "{r:?}");

    assert!(pdf_render_page(path, 3, 100).await.is_err(), "no page 3");
}
