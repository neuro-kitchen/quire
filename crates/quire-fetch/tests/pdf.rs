//! Finding PDFs against a mock server: arXiv, Unpaywall (a real record, its link pointed at the
//! mock), the landing page's `citation_pdf_url`, and PMC's proof-of-work page (blocked).

use quire_fetch::{Endpoints, FetchError, Fetcher};
use quire_model::{EntryKind, Identifier, ReferenceData};
use url::Url;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// The smallest valid PDF (header is all `get_pdf` checks).
const PDF: &[u8] = b"%PDF-1.4\n%quire test\n";

fn fetcher(server: &MockServer, email: Option<&str>) -> Fetcher {
    let base = Url::parse(&format!("{}/", server.uri())).unwrap();
    Fetcher::with_endpoints(email, Endpoints::all_at(&base)).unwrap()
}

fn data(ids: &[&str]) -> ReferenceData {
    let mut data = ReferenceData::new(EntryKind::ArticleJournal);
    data.identifiers = ids.iter().map(|id| Identifier::parse(id).unwrap()).collect();
    data
}

async fn serve(server: &MockServer, at: &str, response: ResponseTemplate) {
    Mock::given(method("GET")).and(path(at)).respond_with(response).mount(server).await;
}

#[tokio::test]
async fn arxiv_pdf() {
    let server = MockServer::start().await;
    serve(&server, "/pdf/1706.03762", ResponseTemplate::new(200).set_body_bytes(PDF)).await;
    let found = fetcher(&server, None).find_pdf(&data(&["arXiv:1706.03762"])).await.unwrap();
    assert_eq!((found.source.as_str(), found.bytes.as_slice()), ("arxiv", PDF));
}

#[tokio::test]
async fn unpaywall_link() {
    let server = MockServer::start().await;
    let record = fixture("unpaywall-10.1371-journal.pone.0173392.json")
        .replace("https://journals.plos.org/plosone/article/file", &format!("{}/plos/file", server.uri()));
    Mock::given(method("GET"))
        .and(path("/unpaywall/10.1371/journal.pone.0173392"))
        .and(query_param("email", "me@example.org"))
        .respond_with(ResponseTemplate::new(200).set_body_string(record))
        .mount(&server)
        .await;
    serve(&server, "/plos/file", ResponseTemplate::new(200).set_body_bytes(PDF)).await;
    let found =
        fetcher(&server, Some("me@example.org")).find_pdf(&data(&["10.1371/journal.pone.0173392"])).await.unwrap();
    assert_eq!(found.source, "unpaywall");
    assert!(found.issues.is_empty());
}

#[tokio::test]
async fn landing_page_link_after_unpaywall_needs_email() {
    let server = MockServer::start().await;
    let page =
        format!(r#"<html><head><meta name="citation_pdf_url" content="{}/article.pdf"></head></html>"#, server.uri());
    serve(&server, "/10.1000/oa", ResponseTemplate::new(200).set_body_string(page)).await;
    serve(&server, "/article.pdf", ResponseTemplate::new(200).set_body_bytes(PDF)).await;
    let found = fetcher(&server, None).find_pdf(&data(&["10.1000/oa"])).await.unwrap();
    assert_eq!(found.source, "landing page");
    assert!(found.issues[0].detail.as_deref().unwrap().contains("Unpaywall needs your email"), "{:?}", found.issues);
}

#[tokio::test]
async fn pmc_proof_of_work_page_is_blocked() {
    let server = MockServer::start().await;
    serve(&server, "/pmc/pdf", ResponseTemplate::new(200).set_body_string(fixture("challenge-pmc-pdf.html"))).await;
    let mut reference = data(&[]);
    reference.extra.insert("pdf-url".into(), format!("{}/pmc/pdf", server.uri()));
    let error = fetcher(&server, None).find_pdf(&reference).await.unwrap_err();
    assert!(error.is_blocked(), "{error}");
}

#[tokio::test]
async fn html_instead_of_pdf_is_not_a_pdf() {
    let server = MockServer::start().await;
    serve(&server, "/pdf/1706.03762", ResponseTemplate::new(200).set_body_string("<html>Sign in</html>")).await;
    let error = fetcher(&server, None).find_pdf(&data(&["arXiv:1706.03762"])).await.unwrap_err();
    assert!(matches!(&error, FetchError::NoPdf(_) | FetchError::AllFailed(_)), "{error}");
    assert!(error.to_string().contains("did not return a PDF"), "{error}");
}
