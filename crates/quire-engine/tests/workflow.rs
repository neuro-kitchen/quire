//! The first milestone end to end through the engine: create projects, add by DOI and arXiv
//! (mock doi.org), add again without fetching, import a `.bib`, export, trash.

use quire_engine::{Added, Endpoints, Engine, ExportFormat, OpenOptions, SlugPath};
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn slug_path(text: &str) -> SlugPath {
    text.parse().unwrap()
}

async fn engine(dir: &tempfile::TempDir, server: &MockServer) -> Engine {
    let base = Url::parse(&format!("{}/", server.uri())).unwrap();
    let options = OpenOptions {
        library: Some(dir.path().join("library")),
        config_file: Some(dir.path().join("config.toml")),
        endpoints: Some(Endpoints { doi: base.clone(), ncbi: base }),
    };
    Engine::open(options).await.unwrap()
}

#[tokio::test]
async fn projects_add_import_export() {
    let server = MockServer::start().await;
    let serve = |url: &'static str, file: &'static str| {
        Mock::given(method("GET")).and(path(url)).respond_with(ResponseTemplate::new(200).set_body_string(fixture(file)))
    };
    serve("/10.1038/nature12373", "csl-crossref-nature12373.json").expect(1).mount(&server).await;
    serve("/10.48550/arXiv.1706.03762", "csl-arxiv-1706.03762.json").expect(1).mount(&server).await;

    let dir = tempfile::tempdir().unwrap();
    let engine = engine(&dir, &server).await;
    engine.create_project("Thesis", None).await.unwrap();
    engine.create_project("Chapter 1", Some(&slug_path("thesis"))).await.unwrap();

    let nature = engine.add("https://doi.org/10.1038/nature12373", Some(&slug_path("thesis"))).await.unwrap();
    assert!(matches!(nature, Added::New(_)));
    let arxiv = engine.add("arXiv:1706.03762", Some(&slug_path("thesis/chapter-1"))).await.unwrap();
    assert_eq!(arxiv.reference().cite_key.as_str(), "vaswaniAttentionAllYou2017");

    // Same work again, other spelling, other project: no request (mock expects 1), only a link.
    let again = engine.add("doi:10.1038/NATURE12373", Some(&slug_path("thesis/chapter-1"))).await.unwrap();
    assert!(matches!(again, Added::Existing(_)));

    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, "@book{knuth1984, author = {Knuth, Donald E.}, title = {The TeXbook}, year = {1984}, publisher = {Addison-Wesley}}").unwrap();
    let imported = engine.import_bib(&bib, Some(&slug_path("thesis"))).await.unwrap();
    assert_eq!(imported[0].reference().cite_key.as_str(), "knuth1984");

    let (text, count) = engine.export(Some(&slug_path("thesis")), false, ExportFormat::Biblatex).await.unwrap();
    assert_eq!(count, 2, "{text}");
    assert!(text.contains("@article{kucskoNanometreScaleThermometry2013,") && text.contains("@book{knuth1984,"), "{text}");
    let (_, count) = engine.export(Some(&slug_path("thesis")), true, ExportFormat::Biblatex).await.unwrap();
    assert_eq!(count, 3);
    let (json, _) = engine.export(Some(&slug_path("thesis/chapter-1")), false, ExportFormat::CslJson).await.unwrap();
    assert!(json.contains("\"id\": \"vaswaniAttentionAllYou2017\""), "{json}");

    let (shown, projects) = engine.show("kucskoNanometreScaleThermometry2013").await.unwrap();
    assert_eq!(shown.id, nature.reference().id);
    let mut paths: Vec<String> = projects.iter().map(ToString::to_string).collect();
    paths.sort();
    assert_eq!(paths, ["thesis", "thesis/chapter-1"]);

    engine.trash("knuth1984").await.unwrap();
    assert_eq!(engine.list(None, false, true).await.unwrap().len(), 1);
    assert_eq!(engine.list(None, false, false).await.unwrap().len(), 2);
    server.verify().await;
}

#[tokio::test]
async fn unknown_project_is_checked_before_fetching() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(&dir, &server).await;
    let error = engine.add("10.1038/nature12373", Some(&slug_path("nope"))).await.unwrap_err();
    assert!(error.to_string().contains("no project `nope`"), "{error}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn library_persists_between_opens() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let device = {
        let engine = engine(&dir, &server).await;
        engine.create_project("kept", None).await.unwrap();
        engine.device()
    };
    let engine = engine(&dir, &server).await;
    assert_eq!(engine.device(), device);
    assert_eq!(engine.projects().await.unwrap().len(), 1);
}
