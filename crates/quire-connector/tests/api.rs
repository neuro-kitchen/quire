//! The API over a real socket: token required, CORS preflight, adding from page HTML (the way a
//! browser extension sends it), attaching a PDF.

use std::sync::Arc;

use quire_engine::{Engine, LibraryChoice, OpenOptions};
use quire_testkit::tiny_pdf;
use serde_json::{Value, json};

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

async fn start(dir: &tempfile::TempDir) -> String {
    let options = OpenOptions {
        library: LibraryChoice::Path(dir.path().join("library")),
        config_file: Some(dir.path().join("config.toml")),
        endpoints: None,
    };
    let engine = Arc::new(Engine::open(options).await.unwrap());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(quire_connector::run(listener, engine, TOKEN.into()));
    format!("http://{address}")
}

#[tokio::test]
async fn token_cors_page_and_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let base = start(&dir).await;
    let client = reqwest::Client::new();

    let denied = client.get(format!("{base}/v1/status")).send().await.unwrap();
    assert_eq!(denied.status(), 401);
    let wrong = client.get(format!("{base}/v1/status")).bearer_auth("nope").send().await.unwrap();
    assert_eq!(wrong.status(), 401);
    let preflight = client
        .request(reqwest::Method::OPTIONS, format!("{base}/v1/page"))
        .header("origin", "moz-extension://abc")
        .send()
        .await
        .unwrap();
    assert_eq!(preflight.status(), 204);
    assert_eq!(preflight.headers()["access-control-allow-origin"], "moz-extension://abc");
    let status: Value =
        client.get(format!("{base}/v1/status")).bearer_auth(TOKEN).send().await.unwrap().json().await.unwrap();
    assert_eq!(status["app"], "quire");
    let projects: Value =
        client.get(format!("{base}/v1/projects")).bearer_auth(TOKEN).send().await.unwrap().json().await.unwrap();
    assert_eq!(projects, json!([]), "a new library has no projects");

    // A page the browser shows (no identifier, so nothing is fetched).
    let html = r#"<html><head><meta name="citation_title" content="A Blocked Paper"><meta name="citation_author" content="Doe, Jane">
        <meta name="citation_journal_title" content="J. Things"><meta name="citation_publication_date" content="2024/05/01"></head></html>"#;
    let body = json!({ "url": "https://publisher.example/article/1", "html": html });
    let added: Value = client
        .post(format!("{base}/v1/page"))
        .bearer_auth(TOKEN)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(added["status"], "added", "{added}");
    let key = added["reference"]["cite_key"].as_str().unwrap().to_string();
    assert_eq!(key, "doeBlockedPaper2024");

    let pdf = client
        .post(format!("{base}/v1/pdf?reference={key}&url=https://publisher.example/article/1.pdf"))
        .bearer_auth(TOKEN)
        .body(tiny_pdf("A Blocked Paper"))
        .send()
        .await
        .unwrap();
    assert_eq!(pdf.status(), 200);
    let attached: Value = pdf.json().await.unwrap();
    assert_eq!((attached["new"].as_bool(), attached["pages"].as_u64()), (Some(true), Some(1)));

    let not_pdf =
        client.post(format!("{base}/v1/pdf?reference={key}")).bearer_auth(TOKEN).body("<html>").send().await.unwrap();
    assert_eq!(not_pdf.status(), 422);
    let unknown = client
        .post(format!("{base}/v1/pdf?reference=nobody"))
        .bearer_auth(TOKEN)
        .body(tiny_pdf("x"))
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), 404);
}
