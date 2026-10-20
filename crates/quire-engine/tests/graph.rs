//! The citation graph through the engine (OpenAlex mocked with saved responses): works cited,
//! marked when already in the library, and works citing.

mod common;
use common::{engine, fixture};

use quire_engine::GraphKind;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn serve(server: &MockServer, at: &str, name: &str) {
    let body = ResponseTemplate::new(200).set_body_string(fixture(name));
    Mock::given(method("GET")).and(path(at)).respond_with(body).mount(server).await;
}

#[tokio::test]
async fn cited_and_citing_works() {
    let server = MockServer::start().await;
    serve(&server, "/10.1038/nature12373", "csl-crossref-nature12373.json").await;
    serve(&server, "/openalex/works/doi:10.1038/nature12373", "openalex-work-nature12373.json").await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let nature = engine.add("10.1038/nature12373", None).await.unwrap();
    let key = nature.reference().cite_key.to_string();

    // The saved filter response holds 3 of the 35 cited works.
    let list = ResponseTemplate::new(200).set_body_string(fixture("openalex-works-filter.json"));
    Mock::given(method("GET"))
        .and(path("/openalex/works"))
        .and(query_param("per_page", "35"))
        .respond_with(list)
        .mount(&server)
        .await;
    let (_, cited) = engine.graph(&key, GraphKind::References, 50).await.unwrap();
    assert_eq!(cited.len(), 3);
    assert!(cited.iter().all(|e| e.in_library.is_none()));
    assert_eq!(cited[0].work.id, "W1557405337", "in the order the paper lists them");

    let citing = ResponseTemplate::new(200).set_body_string(fixture("openalex-citing-nature12373.json"));
    Mock::given(method("GET"))
        .and(path("/openalex/works"))
        .and(query_param("filter", "cites:W2159974629"))
        .respond_with(citing)
        .mount(&server)
        .await;
    let (_, citing) = engine.graph(&key, GraphKind::Citing, 3).await.unwrap();
    assert_eq!(citing[0].work.doi.as_deref(), Some("10.1103/revmodphys.89.035002"));
}
