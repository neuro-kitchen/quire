//! Backup and restore: the archive holds the database (a consistent copy, the library open),
//! PDFs and texts; the restored library has the same data and a new device ID.

mod common;
use common::{engine, tiny_pdf};

use quire_engine::{Engine, LibraryChoice, OpenOptions};

#[tokio::test]
async fn backup_and_restore() {
    let dir = tempfile::tempdir().unwrap();
    let original = engine(dir.path(), None).await;
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, "@article{rivera2025, title = {Diaphragm pacing}, author = {Rivera, Ana}, year = {2025}}")
        .unwrap();
    let key = original.import_bib(&bib, None).await.unwrap()[0].reference().cite_key.to_string();
    original.add_note(Some(&key), "keep this").await.unwrap();
    let pdf = dir.path().join("rivera.pdf");
    std::fs::write(&pdf, tiny_pdf("Diaphragm pacing")).unwrap();
    original.attach_file(&key, &pdf, false).await.unwrap();

    let archive = dir.path().join("library.tar.gz");
    let report = original.backup(&archive).await.unwrap();
    assert!(report.files >= 2 && report.bytes > 0, "{report:?}");

    let target = dir.path().join("restored");
    let manifest = quire_engine::engine::backup::restore(&archive, &target).unwrap();
    assert_eq!(manifest.format, 1);
    let options = OpenOptions {
        library: LibraryChoice::Path(target.clone()),
        config_file: Some(dir.path().join("config.toml")),
        endpoints: None,
    };
    let restored = Engine::open(options).await.unwrap();
    assert_ne!(restored.device(), original.device(), "a new device ID");
    assert_eq!(restored.notes(Some(&key)).await.unwrap()[0].body, "keep this");
    let (_, pdfs) = restored.pdfs_of(&key).await.unwrap();
    assert!(pdfs[0].1.is_file(), "the PDF file came along");

    assert!(quire_engine::engine::backup::restore(&archive, &target).is_err(), "not into a folder in use");
}
