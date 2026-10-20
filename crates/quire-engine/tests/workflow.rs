//! The first milestone end to end through the engine: create projects, add by DOI and arXiv
//! (mock doi.org), add again without fetching, import a `.bib`, export, trash.

mod common;
use common::{engine, fixture};

use quire_engine::{Added, AutoExportOutcome, Engine, ExportFormat, ExportRequest, OpenOptions, SlugPath};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn slug_path(text: &str) -> SlugPath {
    text.parse().unwrap()
}

#[tokio::test]
async fn projects_add_import_export() {
    let server = MockServer::start().await;
    let serve = |url: &'static str, file: &'static str| {
        Mock::given(method("GET"))
            .and(path(url))
            .respond_with(ResponseTemplate::new(200).set_body_string(fixture(file)))
    };
    serve("/10.1038/nature12373", "csl-crossref-nature12373.json").expect(1).mount(&server).await;
    serve("/10.48550/arXiv.1706.03762", "csl-arxiv-1706.03762.json").expect(1).mount(&server).await;

    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
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

    let thesis =
        |recursive| ExportRequest { project: Some(slug_path("thesis")), recursive, ..ExportRequest::default() };
    let (text, count) = engine.export(&thesis(false)).await.unwrap();
    assert_eq!(count, 2, "{text}");
    assert!(
        text.contains("@article{kucskoNanometreScaleThermometry2013,") && text.contains("@book{knuth1984,"),
        "{text}"
    );
    let (_, count) = engine.export(&thesis(true)).await.unwrap();
    assert_eq!(count, 3);
    let chapter = ExportRequest {
        project: Some(slug_path("thesis/chapter-1")),
        format: Some(ExportFormat::CslJson),
        ..ExportRequest::default()
    };
    let (json, _) = engine.export(&chapter).await.unwrap();
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
    let engine = engine(dir.path(), Some(&server)).await;
    let error = engine.add("10.1038/nature12373", Some(&slug_path("nope"))).await.unwrap_err();
    assert!(error.to_string().contains("no project `nope`"), "{error}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn library_persists_between_opens() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let device = {
        let engine = engine(dir.path(), Some(&server)).await;
        engine.create_project("kept", None).await.unwrap();
        engine.device()
    };
    let engine = engine(dir.path(), Some(&server)).await;
    assert_eq!(engine.device(), device);
    assert_eq!(engine.projects().await.unwrap().len(), 1);
}

#[tokio::test]
async fn copy_between_libraries_keeps_the_id() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let source = engine(dir.path(), Some(&server)).await;
    let bib = dir.path().join("refs.bib");
    std::fs::write(
        &bib,
        "@book{knuth1984, author = {Knuth, Donald E.}, title = {The TeXbook}, year = {1984}, doi = {10.1/texbook}}",
    )
    .unwrap();
    let original = source.import_bib(&bib, None).await.unwrap()[0].reference().clone();

    let options = OpenOptions {
        library: quire_engine::LibraryChoice::Path(dir.path().join("other")),
        config_file: Some(dir.path().join("config.toml")),
        endpoints: None,
    };
    let target = Engine::open(options).await.unwrap();
    target.create_project("Books", None).await.unwrap();
    let copied = source.copy_to("knuth1984", &target, Some(&slug_path("books"))).await.unwrap();
    assert!(matches!(&copied, Added::New(r) if r.id == original.id && r.cite_key.as_str() == "knuth1984"));
    assert!(matches!(source.copy_to("doi:10.1/TEXBOOK", &target, None).await.unwrap(), Added::Existing(_)));
    assert_eq!(target.list(Some(&slug_path("books")), false, false).await.unwrap().len(), 1);
    assert!(source.copy_to("knuth1984", &source, None).await.is_err(), "same library");
}

#[tokio::test]
async fn rename_and_remove_projects() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    engine.create_project("Thesis", None).await.unwrap();
    engine.create_project("Draft", Some(&slug_path("thesis"))).await.unwrap();
    let (path, _) = engine.rename_project(&slug_path("thesis/draft"), "Chapter 1").await.unwrap();
    assert_eq!(path.to_string(), "thesis/chapter-1");
    let summaries = engine.project_summaries().await.unwrap();
    assert_eq!(summaries.iter().map(|s| s.path.to_string()).collect::<Vec<_>>(), ["thesis", "thesis/chapter-1"]);
    assert!(engine.remove_project(&slug_path("thesis"), false).await.is_err());
    assert_eq!(engine.remove_project(&slug_path("thesis"), true).await.unwrap(), 2);
    assert!(engine.projects().await.unwrap().is_empty());
}

#[tokio::test]
async fn failed_lookups_are_recorded_until_added() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    engine.create_project("Thesis", None).await.unwrap();
    assert!(engine.add("10.1038/nature12373", Some(&slug_path("thesis"))).await.is_err());
    let failures = engine.failures().await.unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(
        (failures[0].0.input.as_str(), failures[0].1.as_ref().map(ToString::to_string)),
        ("10.1038/nature12373", Some("thesis".into()))
    );

    Mock::given(method("GET"))
        .and(path("/10.1038/nature12373"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("csl-crossref-nature12373.json")))
        .mount(&server)
        .await;
    engine.add("10.1038/nature12373", Some(&slug_path("thesis"))).await.unwrap();
    assert!(engine.failures().await.unwrap().is_empty());
}

#[tokio::test]
async fn check_reports_lints_and_recorded_fallbacks() {
    let server = MockServer::start().await;
    // doi.org fails, Crossref REST answers: the reference records a fallback.
    Mock::given(method("GET"))
        .and(path("/10.1038/nature12373"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/works/10.1038/nature12373"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("crossref-works-nature12373.json")))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    engine.add("10.1038/nature12373", None).await.unwrap();
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, "@misc{untitled, note = {nothing else}}").unwrap();
    engine.import_bib(&bib, None).await.unwrap();

    let checked = engine.check(None, false).await.unwrap();
    let codes = |key: &str| -> Vec<quire_engine::IssueCode> {
        checked.iter().find(|c| c.reference.cite_key.as_str() == key).unwrap().issues.iter().map(|i| i.code).collect()
    };
    assert_eq!(codes("kucskoNanometreScaleThermometry2013"), [quire_engine::IssueCode::Fallback]);
    assert!(codes("untitled").contains(&quire_engine::IssueCode::NoTitle));
}

#[tokio::test]
async fn refetch_keeps_user_edits() {
    let server = MockServer::start().await;
    let crossref = fixture("csl-crossref-nature12373.json");
    Mock::given(method("GET"))
        .and(path("/10.1038/nature12373"))
        .respond_with(ResponseTemplate::new(200).set_body_string(crossref.clone()))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let added = engine.add("10.1038/nature12373", None).await.unwrap().reference().clone();

    // The user edits the title and the cite key.
    let (id, text) = engine.edit_document("10.1038/nature12373").await.unwrap();
    let edited = text
        .replace("Nanometre-scale thermometry in a living cell", "Thermometry (edited)")
        .replace(added.cite_key.as_str(), "kucsko2013");
    let dry = engine.apply_edit(id, &edited, true).await.unwrap();
    assert!(dry.changed());
    assert!(engine.show("kucsko2013").await.is_err(), "dry run wrote nothing");
    let done = engine.apply_edit(id, &edited, false).await.unwrap();
    assert_eq!(
        (done.reference.cite_key.as_str(), done.reference.data.title()),
        ("kucsko2013", Some("Thermometry (edited)"))
    );

    // Nothing new at the source: refetch changes nothing and keeps the user's title.
    let refetched = engine.refetch("kucsko2013", false).await.unwrap();
    assert!(!refetched.changed(), "{:?}", refetched.changes);
    assert!(refetched.changes.iter().any(|c| c.outcome == quire_engine::Outcome::KeptUserValue));
    assert_eq!(engine.show("kucsko2013").await.unwrap().0.data.title(), Some("Thermometry (edited)"));
}

#[tokio::test]
async fn profiles_and_automatic_exports() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.toml"), "[export.typst]\nformat = \"hayagriva\"\nomit = [\"note\"]\n")
        .unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    engine.create_project("Paper", None).await.unwrap();
    let bib = dir.path().join("in.bib");
    std::fs::write(
        &bib,
        "@book{knuth1984, author = {Knuth, Donald E.}, title = {The TeXbook}, year = {1984}, note = {x}}",
    )
    .unwrap();
    engine.import_bib(&bib, Some(&slug_path("paper"))).await.unwrap();

    let request =
        ExportRequest { project: Some(slug_path("paper")), profile: Some("typst".into()), ..ExportRequest::default() };
    let (yaml, _) = engine.export(&request).await.unwrap();
    assert!(yaml.starts_with("knuth1984:") && !yaml.contains("note"), "{yaml}");
    assert!(engine.export(&ExportRequest { profile: Some("nope".into()), ..ExportRequest::default() }).await.is_err());

    let out = dir.path().join("doc/refs.yml");
    engine.keep_updated(&request, &out).await.unwrap();
    assert!(std::fs::read_to_string(&out).unwrap().contains("The TeXbook"));
    assert_eq!(engine.auto_exports().await.unwrap()[0].project.as_ref().map(ToString::to_string), Some("paper".into()));

    // Nothing changed since: nothing runs. A change rewrites the file.
    engine.run_auto_exports(false).await.unwrap();
    assert!(engine.run_auto_exports(false).await.unwrap().is_empty());
    engine.trash("knuth1984").await.unwrap();
    let results = engine.run_auto_exports(false).await.unwrap();
    assert_eq!(results[0].1, AutoExportOutcome::Written(0));
    assert!(!std::fs::read_to_string(&out).unwrap().contains("TeXbook"));
    assert!(engine.stop_auto_export(&out).await.unwrap());
}

#[tokio::test]
async fn keywords_become_tags_and_tags_export_as_keywords() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let bib = dir.path().join("in.bib");
    std::fs::write(
        &bib,
        "@article{a, title = {A}, author = {Doe, J.}, date = {2020}, journaltitle = {J}, keywords = {EMG; surface electrodes}}\n\
         @article{b, title = {B}, author = {Roe, R.}, date = {2021}, journaltitle = {J}}",
    )
    .unwrap();
    engine.import_bib(&bib, None).await.unwrap();
    engine.tag("b", &["to read".into(), "emg".into()]).await.unwrap();
    let tags: Vec<(String, usize)> = engine.tags().await.unwrap().into_iter().map(|(t, n)| (t.name, n)).collect();
    assert_eq!(tags, [("EMG".into(), 2), ("surface electrodes".into(), 1), ("to read".into(), 1)]);
    let tagged = engine.list_tagged(None, false, false, &["emg".into(), "TO READ".into()]).await.unwrap();
    assert_eq!(tagged.iter().map(|r| r.cite_key.as_str()).collect::<Vec<_>>(), ["b"]);

    let (bib_text, _) = engine.export(&ExportRequest::default()).await.unwrap();
    assert!(bib_text.contains("keywords = {EMG; surface electrodes}"), "imported keywords kept: {bib_text}");
    assert!(bib_text.contains("keywords = {EMG, to read}"), "tags as keywords: {bib_text}");
    let csl = ExportRequest { format: Some(ExportFormat::CslJson), ..ExportRequest::default() };
    assert!(engine.export(&csl).await.unwrap().0.contains("\"keyword\": \"EMG, to read\""));
}

#[tokio::test]
async fn subscribers_hear_about_changes() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let mut changes = engine.subscribe();
    let bib = dir.path().join("in.bib");
    std::fs::write(&bib, "@book{knuth1984, title = {The TeXbook}, author = {Knuth, D.}, date = {1984}}").unwrap();
    engine.import_bib(&bib, None).await.unwrap();
    assert!(changes.try_recv().is_ok(), "an import is a change");
    engine.list(None, false, false).await.unwrap();
    assert!(changes.try_recv().is_err(), "reading is not");
}
