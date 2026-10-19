//! Full-text search on SQLite: metadata and abstracts, authors without accents, tags, notes and
//! highlights (kept current by triggers), the trash left out, and a paper's text once extracted.

use quire_engine::{Engine, LibraryChoice, OpenOptions};

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

/// A one-page PDF showing `text`.
fn tiny_pdf(text: &str) -> Vec<u8> {
    let stream = format!("BT /F1 12 Tf 72 720 Td ({text}) Tj ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
        format!("<< /Length {} >>\nstream\n{stream}\nendstream", stream.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{body}\nendobj\n", i + 1));
    }
    let xref = pdf.len();
    pdf.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1));
    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1));
    pdf.into_bytes()
}

async fn engine(dir: &tempfile::TempDir) -> Engine {
    let options = OpenOptions {
        library: LibraryChoice::Path(dir.path().join("library")),
        config_file: Some(dir.path().join("config.toml")),
        endpoints: None,
    };
    Engine::open(options).await.unwrap()
}

async fn keys(engine: &Engine, query: &str) -> Vec<String> {
    engine.search(query, 10).await.unwrap().into_iter().map(|(r, _)| r.cite_key.to_string()).collect()
}

#[tokio::test]
async fn metadata_tags_notes_trash_and_text() {
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(&dir).await;
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
