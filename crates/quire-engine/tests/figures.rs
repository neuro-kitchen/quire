//! Figure images of full texts: downloaded once into the library folder, then read from there;
//! an HTML page in place of an image is refused.

mod common;
use common::engine;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn downloaded_once() {
    let server = MockServer::start().await;
    let png = b"\x89PNG\r\n\x1a\nnot really".to_vec();
    Mock::given(method("GET"))
        .and(path("/html/1706.03762v7/Figures/a.png"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "image/png").set_body_bytes(png.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/page.png"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/html").set_body_string("<html>"))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let url = format!("{}/html/1706.03762v7/Figures/a.png", server.uri());
    let first = engine.figure(&url).await.unwrap();
    assert_eq!(std::fs::read(&first).unwrap(), png);
    assert_eq!(first.extension().unwrap(), "png");
    assert_eq!(engine.figure(&url).await.unwrap(), first, "from the library folder the second time");
    assert!(engine.figure(&format!("{}/page.png", server.uri())).await.is_err());
}
