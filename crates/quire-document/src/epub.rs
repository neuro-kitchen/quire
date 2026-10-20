//! EPUB, a reflowable format: a zip holding `META-INF/container.xml`, which names the package
//! (OPF) file, whose spine lists the XHTML chapters in reading order. Each chapter becomes reader
//! Markdown ([`crate::html::xhtml`]); the book reads as one document.
//!
//! Highlights are anchored by quote and prefix, as in every format. Positions inside the book (EPUB
//! CFI) come with a reader that renders EPUBs.

use std::io::{Cursor, Read};

use roxmltree::Document;

use crate::error::{DocumentError, Result};
use crate::html::{Writer, xhtml};

/// Most bytes read from one entry, and from the whole book (a zip bomb stops here).
const MAX_ENTRY: u64 = 64 * 1024 * 1024;
const MAX_TOTAL: u64 = 256 * 1024 * 1024;

/// Whether `bytes` are an EPUB: a zip whose `mimetype` entry says `application/epub+zip`.
pub fn looks_like_epub(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
        && Book::open(bytes)
            .is_ok_and(|mut book| book.text("mimetype").is_ok_and(|m| m.trim() == "application/epub+zip"))
}

/// The whole book as reader Markdown, chapter after chapter in spine order.
pub fn to_markdown(bytes: &[u8]) -> Result<String> {
    let mut book = Book::open(bytes)?;
    let container = book.text("META-INF/container.xml")?;
    let package_path = attribute(&container, "rootfile", "full-path")?;
    let package = book.text(&package_path)?;
    let base = package_path.rsplit_once('/').map_or("", |(dir, _)| dir);
    let mut out = Writer::default();
    for href in spine(&package)? {
        let path = if base.is_empty() { href } else { format!("{base}/{href}") };
        xhtml::body(&book.text(&path)?, &mut out);
    }
    let markdown = out.finish();
    if markdown.trim().is_empty() {
        return Err(not_epub("no text in its chapters"));
    }
    Ok(markdown)
}

struct Book<'a> {
    zip: zip::ZipArchive<Cursor<&'a [u8]>>,
    read: u64,
}

impl<'a> Book<'a> {
    fn open(bytes: &'a [u8]) -> Result<Self> {
        let zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| not_epub(&e.to_string()))?;
        Ok(Self { zip, read: 0 })
    }

    /// Entry `name` as text, within [`MAX_ENTRY`] and [`MAX_TOTAL`].
    fn text(&mut self, name: &str) -> Result<String> {
        let entry = self.zip.by_name(name).map_err(|e| not_epub(&format!("{name}: {e}")))?;
        let limit = MAX_ENTRY.min(MAX_TOTAL.saturating_sub(self.read));
        let mut bytes = Vec::new();
        entry.take(limit + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > limit {
            return Err(not_epub(&format!("{name} is too large")));
        }
        self.read += bytes.len() as u64;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// The chapters' paths (relative to the package file) in reading order: the spine's `itemref`s
/// looked up in the manifest, XHTML only.
fn spine(package: &str) -> Result<Vec<String>> {
    let document = Document::parse(package).map_err(|e| not_epub(&format!("package: {e}")))?;
    let elements = || document.descendants().filter(|n| n.is_element());
    let manifest: Vec<(&str, &str, &str)> = elements()
        .filter(|n| n.tag_name().name() == "item")
        .filter_map(|n| Some((n.attribute("id")?, n.attribute("href")?, n.attribute("media-type").unwrap_or(""))))
        .collect();
    let chapters: Vec<String> = elements()
        .filter(|n| n.tag_name().name() == "itemref")
        .filter_map(|n| n.attribute("idref"))
        .filter_map(|id| manifest.iter().find(|(item, ..)| *item == id))
        .filter(|(.., kind)| kind.contains("html"))
        .map(|(_, href, _)| percent_decoded(href.split('#').next().unwrap_or(href)))
        .collect();
    if chapters.is_empty() {
        return Err(not_epub("its spine lists no chapters"));
    }
    Ok(chapters)
}

/// The value of `name` on the first `element` of the XML document `xml`.
fn attribute(xml: &str, element: &str, name: &str) -> Result<String> {
    let document = Document::parse(xml).map_err(|e| not_epub(&e.to_string()))?;
    document
        .descendants()
        .find(|n| n.tag_name().name() == element)
        .and_then(|n| n.attribute(name))
        .map(str::to_string)
        .ok_or_else(|| not_epub(&format!("no {element} {name}")))
}

/// `%20` and other escapes in a manifest `href` (zip entry names are not escaped).
fn percent_decoded(href: &str) -> String {
    let bytes = href.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn not_epub(detail: &str) -> DocumentError {
    DocumentError::NotEpub(detail.to_string())
}

#[cfg(test)]
mod tests {
    use quire_testkit::tiny_epub;

    use super::*;

    #[test]
    fn chapters_in_spine_order() {
        let book = tiny_epub(&["<h1>Introduction</h1><p>Place cells fire.</p>", "<h1>Methods</h1><p>Rats ran.</p>"]);
        assert!(looks_like_epub(&book));
        assert_eq!(to_markdown(&book).unwrap(), "## Introduction\n\nPlace cells fire.\n\n## Methods\n\nRats ran.\n");
    }

    #[test]
    fn not_an_epub() {
        assert!(!looks_like_epub(b"%PDF-1.4"));
        assert!(matches!(to_markdown(b"PK\x03\x04 broken"), Err(DocumentError::NotEpub(_))));
        assert!(!looks_like_epub(&quire_testkit::tiny_pdf("x")));
    }

    #[test]
    fn escaped_hrefs() {
        assert_eq!(percent_decoded("text/chapter%201.xhtml"), "text/chapter 1.xhtml");
        assert_eq!(percent_decoded("100%"), "100%");
    }
}
