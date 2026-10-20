//! Duplicates through the engine on SQLite: the arXiv preprint (mock doi.org) and the journal
//! version imported from a `.bib` are found, then merged (identifier, note, tag and PDF move; the
//! other goes to the trash), or recorded as distinct.

mod common;
use common::{engine, fixture};
use quire_testkit::tiny_pdf;

use quire_engine::{Engine, IssueCode};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const JOURNAL_BIB: &str = "@inproceedings{vaswani2017neurips,
  title = {Attention is All you Need},
  author = {Vaswani, Ashish and Shazeer, Noam},
  booktitle = {Advances in Neural Information Processing Systems},
  year = {2017},
  doi = {10.5555/3295222.3295349},
}";

async fn preprint_and_journal(dir: &tempfile::TempDir) -> (Engine, String, String) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/10.48550/arXiv.1706.03762"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("csl-arxiv-1706.03762.json")))
        .mount(&server)
        .await;
    let engine = engine(dir.path(), Some(&server)).await;
    let preprint = engine.add("arXiv:1706.03762", None).await.unwrap().reference().cite_key.to_string();
    let bib = dir.path().join("journal.bib");
    std::fs::write(&bib, JOURNAL_BIB).unwrap();
    let journal = engine.import_bib(&bib, None).await.unwrap()[0].reference().cite_key.to_string();
    (engine, preprint, journal)
}

#[tokio::test]
async fn found_and_merged() {
    let dir = tempfile::tempdir().unwrap();
    let (engine, preprint, journal) = preprint_and_journal(&dir).await;

    let groups = engine.duplicates().await.unwrap();
    assert_eq!(groups.len(), 1);
    let checked = engine.list_with_issues(None, false, false, &[]).await.unwrap();
    assert!(checked.iter().all(|c| c.issues.iter().any(|i| i.code == IssueCode::PossibleDuplicate)));

    engine.add_note(Some(&journal), "the NeurIPS version").await.unwrap();
    engine.tag(&journal, &["transformers".to_string()]).await.unwrap();
    let (journal_ref, _) = engine.show(&journal).await.unwrap();
    engine.attach_bytes(&journal_ref, &tiny_pdf("Attention is All you Need"), "test", false).await.unwrap();
    let merged = engine.merge(&preprint, &journal).await.unwrap();
    assert_eq!((merged.notes, merged.tags, merged.attachments), (1, 1, 1));
    let pdfs_of_kept = engine.pdfs_of(&preprint).await.unwrap().1;
    assert_eq!(pdfs_of_kept.len(), 1, "the PDF moved to the kept reference");

    // The journal DOI now finds the preprint's record; the other is in the trash.
    let (kept, _) = engine.show("doi:10.5555/3295222.3295349").await.unwrap();
    assert_eq!(kept.cite_key.as_str(), preprint);
    assert!(kept.data.identifiers.len() == 2);
    let (removed, _) = engine.show(&journal).await.unwrap();
    assert!(removed.meta.is_deleted());
    assert!(engine.duplicates().await.unwrap().is_empty());
}

#[tokio::test]
async fn distinct_pairs_are_not_reported() {
    let dir = tempfile::tempdir().unwrap();
    let (engine, preprint, journal) = preprint_and_journal(&dir).await;
    assert!(engine.dismiss_duplicate(&preprint, &journal).await.unwrap());
    assert!(engine.duplicates().await.unwrap().is_empty());
    assert!(!engine.dismiss_duplicate(&journal, &preprint).await.unwrap(), "already recorded");
}
