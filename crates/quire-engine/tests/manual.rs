//! Adding a reference typed in by hand, and looking one up without adding it.

mod common;
use common::{engine, fixture};

use quire_engine::{Added, Reference};
use quire_model::{Creator, CreatorRole, Date, EntryKind, ReferenceData, TextField};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn typed() -> ReferenceData {
    let mut data = ReferenceData::new(EntryKind::ArticleJournal);
    data.set(TextField::Title, "Place cells in the hippocampus");
    data.creators.push(Creator::person(CreatorRole::Author, "O'Keefe", Some("John".into())));
    data.issued = Some(Date::year(1971));
    data
}

fn reference(added: Added) -> Reference {
    match added {
        Added::New(reference) | Added::Existing(reference) => reference,
    }
}

#[tokio::test]
async fn a_typed_reference_with_a_doi_is_added_once() {
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), None).await;
    let mut data = typed();
    data.identifiers.push(quire_model::Identifier::parse("10.1016/0006-8993(71)90358-1").unwrap());
    let first = engine.add_manual(data.clone(), None).await.unwrap();
    assert!(matches!(first, Added::New(_)), "{first:?}");
    let reference = reference(first);
    assert_eq!(reference.data.title(), Some("Place cells in the hippocampus"));
    assert!(reference.cite_key.to_string().starts_with("okeefe"), "{}", reference.cite_key);
    let again = engine.add_manual(data, None).await.unwrap();
    assert!(matches!(again, Added::Existing(_)), "{again:?}");
}

/// Without an identifier a second copy is added, and the duplicates check finds the pair.
#[tokio::test]
async fn typed_look_alikes_are_left_to_the_duplicates_check() {
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), None).await;
    engine.add_manual(typed(), None).await.unwrap();
    engine.add_manual(typed(), None).await.unwrap();
    assert_eq!(engine.list(None, false, false).await.unwrap().len(), 2);
    assert_eq!(engine.duplicates().await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_lookup_stores_nothing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/10.1038/nature12373"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("csl-crossref-nature12373.json")))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let data = engine.lookup("10.1038/nature12373").await.unwrap();
    assert!(data.title().is_some_and(|t| t.contains("thermometry")), "{:?}", data.title());
    assert!(engine.list(None, false, false).await.unwrap().is_empty());
}
