//! Full-text search on SQLite: metadata and abstracts, authors without accents, tags, notes and
//! highlights (kept current by triggers), the trash left out, and a paper's text once extracted.

mod common;
use common::{engine, tiny_pdf};

use quire_engine::Engine;

const BIB: &str = "@article{muller2020,
  title = {Spinal cord stimulation restores breathing},
  author = {M{\\\"u}ller, Anna and Smith, Bob},
  journal = {Journal of Neural Engineering},
  year = {2020},
  abstract = {Diaphragm pacing after injury.},
}
@article{other2021,
  title = {Wavelet denoising of electromyography},
  author = {Jones, Carl},
  journal = {Signal Processing},
  year = {2021},
}";

async fn keys(engine: &Engine, query: &str) -> Vec<String> {
    engine.search(query, 10).await.unwrap().into_iter().map(|(r, _)| r.cite_key.to_string()).collect()
}

#[tokio::test]
async fn metadata_tags_notes_trash_and_text() {
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), None).await;
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, BIB).unwrap();
    let added = engine.import_bib(&bib, None).await.unwrap();
    let muller = added[0].reference().cite_key.to_string();
    let other = added[1].reference().cite_key.to_string();

    assert_eq!(keys(&engine, "stimulation").await, [muller.as_str()], "title");
    assert_eq!(keys(&engine, "diaphr").await, [muller.as_str()], "abstract, word prefix");
    assert_eq!(keys(&engine, "author:muller").await, [muller.as_str()], "accents ignored (Müller)");
    assert_eq!(keys(&engine, "author:jones").await, [other.as_str()], "author field");
    assert!(keys(&engine, "title:jones").await.is_empty(), "not in the title");
    assert_eq!(keys(&engine, "\"neural engineering\"").await, [muller.as_str()], "phrase in the venue");

    engine.tag(&other, &["to-read".to_string()]).await.unwrap();
    assert_eq!(keys(&engine, "tag:to-read").await, [other.as_str()], "tags, by trigger");
    engine.add_note(Some(&other), "compare with the notch filter").await.unwrap();
    assert_eq!(keys(&engine, "notch").await, [other.as_str()], "notes, by trigger");
    engine.add_highlight(&muller, "restores breathing", "", Some("key result")).await.unwrap();
    assert_eq!(keys(&engine, "note:result").await, [muller.as_str()], "highlight comments");

    engine.trash(&other).await.unwrap();
    assert!(keys(&engine, "wavelet").await.is_empty(), "the trash is not searched");

    let (reference, _) = engine.show(&muller).await.unwrap();
    engine.attach_bytes(&reference, &tiny_pdf("phrenic motoneurons recover"), "test", false).await.unwrap();
    engine.reader_text(&muller).await.unwrap();
    let hits = engine.search("motoneurons", 10).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].1.in_text && hits[0].1.snippet.contains("«motoneurons»"), "{:?}", hits[0].1);
    engine.forget_text(&muller).await.unwrap();
    assert!(keys(&engine, "motoneurons").await.is_empty(), "forgotten with the stored text");
}
