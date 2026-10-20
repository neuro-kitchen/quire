//! Importing a Zotero data folder (`tests/fixtures/zotero.sql`): references, collections as
//! projects, tags, notes, related items, the PDF and its highlight; importing again adds nothing.

mod common;
use common::{engine, fixture, tiny_pdf};

use quire_engine::SlugPath;

fn zotero_folder(dir: &std::path::Path) -> std::path::PathBuf {
    let folder = dir.join("Zotero");
    std::fs::create_dir_all(folder.join("storage/PDF44444")).unwrap();
    let conn = rusqlite::Connection::open(folder.join("zotero.sqlite")).unwrap();
    conn.execute_batch(&fixture("zotero.sql")).unwrap();
    std::fs::write(folder.join("storage/PDF44444/kucsko.pdf"), tiny_pdf("doi 10.1038/nature12373")).unwrap();
    folder
}

#[tokio::test]
async fn library_collections_notes_pdf_and_highlights() {
    let dir = tempfile::tempdir().unwrap();
    let folder = zotero_folder(dir.path());
    let engine = engine(dir.path(), None).await;
    engine.create_project("From Zotero", None).await.unwrap();
    let under: SlugPath = "from-zotero".parse().unwrap();

    let report = engine.import_zotero(&folder, Some(&under), None).await.unwrap();
    let keys: Vec<String> = report.added.iter().map(|r| r.cite_key.to_string()).collect();
    assert_eq!(keys.len(), 2, "{report:?}");
    assert!(keys.contains(&"plainPaper2020".to_string()), "Better BibTeX key kept: {keys:?}");
    assert_eq!((report.projects, report.notes, report.pdfs, report.highlights), (2, 2, 1, 1), "{report:?}");

    let tree: Vec<String> = engine.projects().await.unwrap().into_iter().map(|(path, _)| path.to_string()).collect();
    assert!(tree.contains(&"from-zotero/thesis/chapter-2".to_string()), "{tree:?}");
    let in_chapter = engine.list(Some(&"from-zotero/thesis/chapter-2".parse().unwrap()), false, false).await.unwrap();
    let kucsko = &in_chapter[0];
    assert_eq!(kucsko.data.identifiers[0].to_string(), "doi:10.1038/nature12373");

    let key = kucsko.cite_key.to_string();
    let tags = engine.tags_of(vec![kucsko.id]).await.unwrap().remove(&kucsko.id).unwrap();
    assert_eq!(tags.len(), 2, "{tags:?}");
    let notes = engine.notes(Some(&key)).await.unwrap();
    assert_eq!(notes[0].body, "Key idea: NV centres");
    assert_eq!(engine.related(&key).await.unwrap().len(), 1);
    let (_, highlights) = engine.highlights(&key).await.unwrap();
    assert_eq!(highlights[0].quote, "quantum spin defects");
    assert_eq!(highlights[0].comment.as_deref(), Some("useful"));

    // Again: everything is known.
    let again = engine.import_zotero(&folder, Some(&under), None).await.unwrap();
    assert!(again.added.is_empty(), "{again:?}");
    assert_eq!((again.existing, again.projects, again.notes, again.pdfs, again.highlights), (2, 0, 0, 0, 0));
}
