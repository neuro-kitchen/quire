//! Test fixtures shared by quire's crates; used only as a dev-dependency.
//!
//! - [`tiny_pdf`]: a one-page PDF showing a line of text; [`pdf_page`]: one drawing any content;
//!   [`pdf_pages`]: several.
//! - [`tiny_epub`]: an EPUB 3 of a few XHTML chapters.
//! - [`fixture`]: a metadata service's recorded response (`fixtures/`, saved 2026-10-19: doi.org,
//!   Crossref, arXiv, PubMed, Europe PMC, OpenAlex, Unpaywall, publisher pages, challenge pages).

/// A one-page US Letter PDF showing `text` in Helvetica 12 pt, its baseline 72 pt from the top-left.
pub fn tiny_pdf(text: &str) -> Vec<u8> {
    pdf_page(&format!("BT /F1 12 Tf 72 720 Td ({text}) Tj ET"), "")
}

/// A one-page US Letter PDF drawing `content` (a content stream), with `page_attributes` added to
/// its page dictionary (e.g. `/Rotate 90`). Fonts: `/F1` Helvetica; `/F2` Helvetica whose code 1
/// is the `fi` ligature glyph.
pub fn pdf_page(content: &str, page_attributes: &str) -> Vec<u8> {
    build(&[(content, page_attributes)])
}

/// A US Letter PDF of one page per content stream in `contents`, with the fonts of [`pdf_page`].
pub fn pdf_pages(contents: &[&str]) -> Vec<u8> {
    build(&contents.iter().map(|c| (*c, "")).collect::<Vec<_>>())
}

/// Objects: 1 catalog, 2 page tree, 3 and 4 fonts, then each page and its content stream.
fn build(pages: &[(&str, &str)]) -> Vec<u8> {
    let kids: Vec<String> = (0..pages.len()).map(|i| format!("{} 0 R", 5 + 2 * i)).collect();
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding << /Type /Encoding /BaseEncoding /WinAnsiEncoding /Differences [1 /fi] >> >>".to_string(),
    ];
    for (i, (content, page_attributes)) in pages.iter().enumerate() {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] {page_attributes} /Contents {} 0 R /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> >>",
            6 + 2 * i
        ));
        objects.push(format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()));
    }
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

/// An EPUB 3 whose spine is `chapters` (XHTML `<body>` contents) in order. The chapters are listed in
/// the manifest in reverse, so a reader that ignores the spine gets them in the wrong order.
pub fn tiny_epub(chapters: &[&str]) -> Vec<u8> {
    use std::io::Write;
    use zip::CompressionMethod::{Deflated, Stored};
    use zip::write::SimpleFileOptions;

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(Stored);
    let deflated = SimpleFileOptions::default().compression_method(Deflated);
    let mut file = |name: &str, options, text: &str| {
        zip.start_file(name, options).unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    };
    file("mimetype", stored, "application/epub+zip");
    file(
        "META-INF/container.xml",
        deflated,
        r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#,
    );
    let manifest: String = (0..chapters.len())
        .rev()
        .map(|i| format!(r#"<item id="c{i}" href="text/chapter%20{i}.xhtml" media-type="application/xhtml+xml"/>"#))
        .collect();
    let spine: String = (0..chapters.len()).map(|i| format!(r#"<itemref idref="c{i}"/>"#)).collect();
    file(
        "OEBPS/content.opf",
        deflated,
        &format!(
            r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test</dc:title></metadata><manifest>{manifest}</manifest><spine>{spine}</spine></package>"#
        ),
    );
    for (i, body) in chapters.iter().enumerate() {
        file(
            &format!("OEBPS/text/chapter {i}.xhtml"),
            deflated,
            &format!(
                r#"<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>c</title></head><body>{body}</body></html>"#
            ),
        );
    }
    zip.finish().unwrap().into_inner()
}

/// The recorded response `name` from `fixtures/`.
pub fn fixture(name: &str) -> String {
    let path = format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}
