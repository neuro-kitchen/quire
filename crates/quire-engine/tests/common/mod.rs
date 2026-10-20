//! What the engine's integration tests share: opening an engine in a temporary folder and fixture
//! files (a minimal PDF: `quire_testkit::tiny_pdf`).

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

/// A file from `tests/fixtures`, else a service's recorded response (`quire-testkit`).
pub fn fixture(name: &str) -> String {
    let here = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&here).unwrap_or_else(|_| quire_testkit::fixture(name))
}
