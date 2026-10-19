//! Every lookup path against a local mock server serving real responses saved on 2026-10-18
//! (`tests/fixtures`): doi.org CSL-JSON (Crossref, DataCite/arXiv), NCBI's exporter, and
//! publisher pages.

use quire_fetch::{Endpoints, FetchError, Fetcher, Lookup};
use quire_model::{EntryKind, IdentifierKind, TextField};
use url::Url;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CSL: &str = "application/vnd.citationstyles.csl+json";

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

async fn serve(server: &MockServer, url_path: &str, body: String) {
    Mock::given(method("GET")).and(path(url_path)).respond_with(ResponseTemplate::new(200).set_body_string(body)).mount(server).await;
}

fn fetcher(server: &MockServer) -> Fetcher {
    let base = Url::parse(&format!("{}/", server.uri())).unwrap();
    let endpoints = Endpoints { doi: base.clone(), ncbi: base.join("ctxp/").unwrap() };
    Fetcher::with_endpoints(Some("test@example.org"), endpoints).unwrap()
}

#[tokio::test]
async fn doi_by_content_negotiation() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/10.1038/nature12373"))
        .and(header("accept", CSL))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("csl-crossref-nature12373.json")))
        .mount(&server)
        .await;
    let fetched = fetcher(&server).fetch(&Lookup::parse("doi:10.1038/nature12373").unwrap()).await.unwrap();
    assert_eq!(fetched.data.kind, EntryKind::ArticleJournal);
    assert_eq!(fetched.data.title(), Some("Nanometre-scale thermometry in a living cell"));
    assert_eq!((fetched.raw.source.as_str(), fetched.raw.format.as_str()), ("doi.org", "csl-json"));
}

#[tokio::test]
async fn arxiv_through_its_doi() {
    let server = MockServer::start().await;
    serve(&server, "/10.48550/arXiv.1706.03762", fixture("csl-arxiv-1706.03762.json")).await;
    let fetched = fetcher(&server).fetch(&Lookup::parse("arXiv:1706.03762v5").unwrap()).await.unwrap();
    assert_eq!(fetched.data.kind, EntryKind::Article);
    assert_eq!(fetched.data.identifiers[0].kind(), IdentifierKind::Arxiv);
}

#[tokio::test]
async fn pubmed_falls_back_to_its_own_record_when_the_doi_fails() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ctxp/pubmed/"))
        .and(header("accept", "*/*"))
        .and(query_param("format", "csl"))
        .and(query_param("id", "31452104"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("ctxp-pubmed-31452104.json")))
        .mount(&server)
        .await;
    // The record's DOI is not served: the mock answers 404.
    let fetched = fetcher(&server).fetch(&Lookup::parse("pmid:31452104").unwrap()).await.unwrap();
    assert_eq!(fetched.data.title(), Some("Molegro Virtual Docker for Docking"));
    let kinds: Vec<_> = fetched.data.identifiers.iter().map(|i| i.kind()).collect();
    assert!(kinds.contains(&IdentifierKind::Pmid) && kinds.contains(&IdentifierKind::Doi), "{kinds:?}");
    assert_eq!(fetched.raw.source, "ncbi-ctxp");
}

#[tokio::test]
async fn pubmed_prefers_the_doi_record_and_keeps_the_pmid() {
    let server = MockServer::start().await;
    serve(&server, "/ctxp/pubmed/", fixture("ctxp-pubmed-31452104.json")).await;
    serve(&server, "/10.1007/978-1-4939-9752-7_10", fixture("csl-crossref-nature12373.json")).await;
    let fetched = fetcher(&server).fetch(&Lookup::parse("31452104").unwrap()).await.unwrap();
    assert_eq!(fetched.raw.source, "doi.org");
    assert!(fetched.data.identifiers.iter().any(|i| i.kind() == IdentifierKind::Pmid));
}

#[tokio::test]
async fn publisher_page_with_a_doi_uses_the_doi_record() {
    let server = MockServer::start().await;
    serve(&server, "/articles/nature12373", fixture("nature-nature12373.html")).await;
    serve(&server, "/10.1038/nature12373", fixture("csl-crossref-nature12373.json")).await;
    let page = Lookup::parse(&format!("{}/articles/nature12373", server.uri())).unwrap();
    let fetched = fetcher(&server).fetch(&page).await.unwrap();
    assert_eq!(fetched.raw.source, "doi.org");
    assert_eq!(fetched.data.get(TextField::Volume), Some("500"));
}

#[tokio::test]
async fn arxiv_abstract_page_uses_the_arxiv_doi() {
    let server = MockServer::start().await;
    serve(&server, "/abs/1706.03762", fixture("arxiv-abs-1706.03762.html")).await;
    serve(&server, "/10.48550/arXiv.1706.03762", fixture("csl-arxiv-1706.03762.json")).await;
    let page = Lookup::parse(&format!("{}/abs/1706.03762", server.uri())).unwrap();
    let fetched = fetcher(&server).fetch(&page).await.unwrap();
    assert_eq!(fetched.data.identifiers[0].kind(), IdentifierKind::Arxiv);
}

#[tokio::test]
async fn page_tags_are_used_when_the_identifier_fails() {
    let server = MockServer::start().await;
    serve(&server, "/plosone/article", fixture("plos-journal.pone.0173392.html")).await;
    let page = Lookup::parse(&format!("{}/plosone/article", server.uri())).unwrap();
    let fetched = fetcher(&server).fetch(&page).await.unwrap();
    assert_eq!(fetched.raw.format, "html-meta");
    assert_eq!(fetched.data.kind, EntryKind::ArticleJournal);
    assert!(fetched.data.title().is_some() && fetched.data.creators.len() == 3);
}

#[tokio::test]
async fn unknown_doi_is_not_found() {
    let server = MockServer::start().await;
    let error = fetcher(&server).fetch(&Lookup::parse("10.1000/missing").unwrap()).await.unwrap_err();
    assert!(matches!(error, FetchError::NotFound(ref what) if what == "DOI 10.1000/missing"), "{error}");
}
