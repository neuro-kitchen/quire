//! Search, saved searches, alerts, duplicates and checks through the bridge (no network).

mod common;
use common::fixture;

use quire_ffi::{CreatorInput, ReferenceFields};

fn typed(title: &str) -> ReferenceFields {
    ReferenceFields {
        kind: "article-journal".into(),
        title: title.into(),
        creators: vec![CreatorInput {
            role: "author".into(),
            family: "Moser".into(),
            given: Some("Edvard".into()),
            literal: false,
        }],
        issued: Some("2008".into()),
        container_title: None,
        publisher: None,
        volume: None,
        issue: None,
        pages: None,
        url: None,
        abstract_text: None,
        identifiers: vec![],
    }
}

#[tokio::test]
async fn search_and_saved_searches() {
    let f = fixture().await;
    let hits = f.library.search("hippocampus".into(), 10).await.unwrap();
    assert_eq!(hits.iter().map(|h| h.reference.cite_key.as_str()).collect::<Vec<_>>(), ["okeefe1971"]);
    f.library.save_search("maps".into(), "hippocampus".into()).await.unwrap();
    assert_eq!(f.library.run_saved("maps".into(), 10).await.unwrap().len(), 1);
    f.library.rename_saved("maps".into(), "spatial".into()).await.unwrap();
    assert_eq!(f.library.saved_searches().await.unwrap().into_iter().map(|s| s.name).collect::<Vec<_>>(), ["spatial"]);
    f.library.remove_saved("spatial".into()).await.unwrap();
    assert!(f.library.saved_searches().await.unwrap().is_empty());
}

#[tokio::test]
async fn alerts_are_kept() {
    let f = fixture().await;
    let alert = f.library.add_alert("grid".into(), "arxiv".into(), "grid cells".into()).await.unwrap();
    assert_eq!((alert.source.as_str(), alert.checked), ("arxiv", None));
    assert_eq!(f.library.alerts().await.unwrap().len(), 1);
    assert!(f.library.add_alert("x".into(), "scholar".into(), "y".into()).await.is_err(), "unknown source");
    f.library.remove_alert("grid".into()).await.unwrap();
    assert!(f.library.alerts().await.unwrap().is_empty());
}

#[tokio::test]
async fn duplicates_merge_dismiss_and_check() {
    let f = fixture().await;
    let a = f.library.add_manual(typed("Place cells, grid cells, and the brain's spatial system"), None).await.unwrap();
    let b = f.library.add_manual(typed("Place cells, grid cells, and the brain's spatial system"), None).await.unwrap();
    let groups = f.library.duplicates().await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].duplicates[0].reason, "title");
    f.library.tag(b.reference.id.clone(), vec!["grid".into()]).await.unwrap();
    let merged = f.library.merge(a.reference.id.clone(), b.reference.id.clone()).await.unwrap();
    assert_eq!((merged.removed_id.as_str(), merged.tags), (b.reference.id.as_str(), 1));
    assert_eq!(merged.kept.tags, ["grid"]);
    assert!(f.library.duplicates().await.unwrap().is_empty());

    let c = f.library.add_manual(typed("Place cells, grid cells, and the brain's spatial system"), None).await.unwrap();
    assert!(f.library.dismiss_duplicate(a.reference.id.clone(), c.reference.id.clone()).await.unwrap());
    assert!(f.library.duplicates().await.unwrap().is_empty(), "dismissed");

    // Only references with issues come back, each with at least one.
    let checked = f.library.check(None, true).await.unwrap();
    assert!(!checked.is_empty());
    assert!(checked.iter().all(|c| !c.issues.is_empty() && c.issues.iter().all(|i| !i.code.is_empty())));
}
