//! Every lookup path against a local mock server serving real responses saved on 2026-10-18 and
//! 2026-10-19 (`quire-testkit`'s `fixtures/`): doi.org CSL-JSON (Crossref, DataCite/arXiv), Crossref REST, the
//! arXiv API, NCBI's exporter, publisher pages, JSON-LD pages and challenge pages.

use quire_fetch::{Endpoints, FetchError, Fetcher, Lookup};
use quire_model::{EntryKind, IdentifierKind, IssueCode, TextField};
use quire_testkit::fixture;
use url::Url;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CSL: &str = "application/vnd.citationstyles.csl+json";

async fn serve(server: &MockServer, url_path: &str, body: String) {
    Mock::given(method("GET"))
        .and(path(url_path))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(server)
        .await;
}

fn fetcher(server: &MockServer) -> Fetcher {
    let base = Url::parse(&format!("{}/", server.uri())).unwrap();
    let endpoints = Endpoints { ncbi: base.join("ctxp/").unwrap(), ..Endpoints::all_at(&base) };
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
    assert_eq!(fetched.issues[0].code, IssueCode::Fallback);
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
    assert!(fetched.data.identifiers.iter().any(|i| i.kind() == IdentifierKind::Doi), "the page's DOI is kept");
    assert_eq!(fetched.issues[0].code, IssueCode::Fallback);
}

#[tokio::test]
async fn unknown_doi_is_not_found() {
    let server = MockServer::start().await;
    let error = fetcher(&server).fetch(&Lookup::parse("10.1000/missing").unwrap()).await.unwrap_err();
    assert!(matches!(error, FetchError::NotFound(ref what) if what == "DOI 10.1000/missing"), "{error}");
}

#[tokio::test]
async fn doi_falls_back_to_crossref_rest() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/10.1038/nature12373"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    serve(&server, "/works/10.1038/nature12373", fixture("crossref-works-nature12373.json")).await;
    let fetched = fetcher(&server).fetch(&Lookup::parse("10.1038/nature12373").unwrap()).await.unwrap();
    assert_eq!(fetched.raw.source, "api.crossref.org");
    assert_eq!(fetched.data.kind, EntryKind::ArticleJournal);
    assert_eq!(fetched.data.title(), Some("Nanometre-scale thermometry in a living cell"));
    assert_eq!(fetched.data.get(TextField::ContainerTitle), Some("Nature"));
    let issue = &fetched.issues[0];
    assert_eq!(issue.code, IssueCode::Fallback);
    assert!(issue.detail.as_deref().unwrap().contains("HTTP 503"), "{issue:?}");
}

#[tokio::test]
async fn arxiv_falls_back_to_the_arxiv_api() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/query"))
        .and(query_param("id_list", "1706.03762"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("arxiv-api-1706.03762.xml")))
        .mount(&server)
        .await;
    let fetched = fetcher(&server).fetch(&Lookup::parse("arXiv:1706.03762").unwrap()).await.unwrap();
    assert_eq!(fetched.raw.source, "export.arxiv.org");
    let data = &fetched.data;
    assert_eq!((data.kind, data.title()), (EntryKind::Article, Some("Attention Is All You Need")));
    assert_eq!(data.creators.len(), 8);
    assert_eq!(data.creators[5].display_name(), "Gomez, Aidan N.");
    assert_eq!(data.issued.unwrap().to_string(), "2017-06-12");
    assert_eq!(data.get(TextField::Publisher), Some("arXiv"));
    assert_eq!(data.identifiers[0].kind(), IdentifierKind::Arxiv);
    assert_eq!(fetched.issues[0].code, IssueCode::Fallback);
}

#[tokio::test]
async fn unknown_arxiv_id_fails_with_both_sources() {
    let server = MockServer::start().await;
    serve(&server, "/query", fixture("arxiv-api-not-found.xml")).await;
    let error = fetcher(&server).fetch(&Lookup::parse("arXiv:2601.99999").unwrap()).await.unwrap_err();
    let FetchError::AllFailed(failures) = &error else { panic!("{error}") };
    assert_eq!(failures.len(), 2);
    assert!(error.to_string().contains("export.arxiv.org: not found"), "{error}");
}

#[tokio::test]
async fn challenge_pages_are_blocked() {
    let server = MockServer::start().await;
    let challenge = |file: &str, cf: bool| {
        let mut response = ResponseTemplate::new(403).set_body_string(fixture(file));
        if cf {
            response = response.insert_header("cf-mitigated", "challenge");
        }
        response
    };
    Mock::given(method("GET"))
        .and(path("/oup"))
        .respond_with(challenge("challenge-cloudflare-oup.html", true))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rg"))
        .respond_with(challenge("challenge-researchgate.html", false))
        .mount(&server)
        .await;
    for page in ["oup", "rg"] {
        let lookup = Lookup::parse(&format!("{}/{page}", server.uri())).unwrap();
        let error = fetcher(&server).fetch(&lookup).await.unwrap_err();
        assert!(matches!(error, FetchError::Blocked { .. }) && error.is_blocked(), "{page}: {error}");
    }
}

#[tokio::test]
async fn json_ld_fills_what_tags_lack() {
    let server = MockServer::start().await;
    serve(&server, "/blog/post", fixture("jsonld-github-blog.html")).await;
    let lookup = Lookup::parse(&format!("{}/blog/post", server.uri())).unwrap();
    let fetched = fetcher(&server).fetch(&lookup).await.unwrap();
    let data = &fetched.data;
    assert_eq!(data.title(), Some("GitHub Availability Report: September 2025"));
    assert_eq!(data.kind, EntryKind::PostWeblog, "type from JSON-LD");
    assert_eq!(data.creators[0].display_name(), "Oleksy, Jakub", "author from JSON-LD");
    assert_eq!(data.issued.unwrap().to_string(), "2025-10-09");
    assert_eq!(data.get(TextField::ContainerTitle), Some("The GitHub Blog"));
}

/// Real time: paused time would also fire reqwest's timeouts at once.
#[tokio::test]
async fn rate_limit_is_retried_once() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/10.1038/nature12373"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "1"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    serve(&server, "/10.1038/nature12373", fixture("csl-crossref-nature12373.json")).await;
    let fetched = fetcher(&server).fetch(&Lookup::parse("10.1038/nature12373").unwrap()).await.unwrap();
    assert_eq!(fetched.raw.source, "doi.org");
    assert!(fetched.issues.is_empty());
}

#[tokio::test]
async fn europe_pmc_abstract_by_doi() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/europepmc/search"))
        .and(query_param("query", "DOI:\"10.1038/nature12373\""))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("europepmc-nature12373.json")))
        .mount(&server)
        .await;
    let ids = [quire_model::Identifier::parse("10.1038/nature12373").unwrap()];
    let text = fetcher(&server).abstract_for(&ids).await.unwrap().unwrap();
    assert!(text.starts_with("Sensitive probing of temperature variations"), "{text}");
    assert_eq!(fetcher(&server).abstract_for(&[]).await.unwrap(), None);
}

#[tokio::test]
async fn full_text_from_arxiv_html_else_europe_pmc() {
    use quire_fetch::FullTextSource;
    use quire_model::Identifier;
    let server = MockServer::start().await;
    serve(&server, "/html/1706.03762", fixture("arxiv-html-1706.03762v7.html")).await;
    let arxiv: Identifier = "arXiv:1706.03762".parse().unwrap();
    let full = fetcher(&server).full_text(&[arxiv]).await.unwrap().unwrap();
    assert_eq!(full.source, FullTextSource::ArxivHtml);
    assert!(full.markdown.contains("$$\\mathrm{Attention}(Q,K,V)"));
    // A DOI that is not in Europe PMC's open-access subset: no full text, no error.
    Mock::given(method("GET"))
        .and(path("/europepmc/search"))
        .and(query_param("resultType", "lite"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"resultList":{"result":[]}}"#))
        .mount(&server)
        .await;
    let doi: Identifier = "doi:10.1038/nature12373".parse().unwrap();
    assert_eq!(fetcher(&server).full_text(&[doi]).await.unwrap(), None);
}
