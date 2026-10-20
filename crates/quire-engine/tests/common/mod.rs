//! What the engine's integration tests share: opening an engine in a temporary folder, fixture
//! files, and a minimal PDF.

#![allow(dead_code)] // each test file uses part of it

use std::path::Path;

use quire_engine::{Endpoints, Engine, LibraryChoice, OpenOptions};
use url::Url;
use wiremock::MockServer;

/// An engine on a new library in `dir`, its services at `server` (none: the real defaults, for
/// tests that make no request).
pub async fn engine(dir: &Path, server: Option<&MockServer>) -> Engine {
    // Keys (sync, user, server token) in files next to this test's config, not the user's keyring.
    // SAFETY: set before the engine starts threads that read it; every test sets the same value.
    unsafe { std::env::set_var(quire_engine::secrets::NO_KEYRING_ENV, "1") };
    let endpoints = server.map(|server| Endpoints::all_at(&Url::parse(&format!("{}/", server.uri())).unwrap()));
    let options = OpenOptions {
        library: LibraryChoice::Path(dir.join("library")),
        config_file: Some(dir.join("config.toml")),
        endpoints,
    };
    Engine::open(options).await.unwrap()
}

/// A file from `tests/fixtures`, else from `quire-fetch`'s (the services' saved responses).
pub fn fixture(name: &str) -> String {
    let here = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let fetch = format!("{}/../quire-fetch/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&here).or_else(|_| std::fs::read_to_string(&fetch)).unwrap()
}

/// A one-page PDF showing `text`.
pub fn tiny_pdf(text: &str) -> Vec<u8> {
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
