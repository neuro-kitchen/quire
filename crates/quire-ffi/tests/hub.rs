//! The hub and lazy-PDF calls that need no network: the recovery kit, keeping a paper offline,
//! eviction (nothing without a hub copy), opening a PDF that is here.

mod common;
use common::fixture;
use quire_testkit::tiny_pdf;

#[tokio::test]
async fn recovery_kit_keep_offline_eviction_and_opening() {
    let f = fixture().await;
    let lib = &f.library;
    assert!(lib.recovery_kit().await.unwrap().contains(&lib.export_user_key().await.unwrap()));

    let pdf = f.dir.path().join("paper.pdf");
    std::fs::write(&pdf, tiny_pdf("The hippocampus as a spatial map")).unwrap();
    lib.attach_pdf("okeefe1971".into(), pdf.display().to_string(), false).await.unwrap();

    assert!(lib.keep_reference_offline("okeefe1971".into(), true).await.unwrap());
    assert!(!lib.keep_reference_offline("okeefe1971".into(), true).await.unwrap(), "already kept");
    assert!(lib.keep_project_offline("nowhere".into(), true).await.is_err(), "no such project");

    let opened = lib.open_pdf("okeefe1971".into()).await.unwrap();
    assert_eq!((opened.source.as_str(), opened.detail), ("here", None));
    assert!(std::path::Path::new(&opened.path).is_file());

    let eviction = lib.evict_pdfs().await.unwrap();
    assert_eq!(eviction.removed, 0, "no limit set, and no hub copy");
    assert!(eviction.kept > 0);

    let no_hub = lib.sync_with_hub(None).await.unwrap_err();
    assert!(no_hub.to_string().contains("no sync hub"), "{no_hub}");
}
