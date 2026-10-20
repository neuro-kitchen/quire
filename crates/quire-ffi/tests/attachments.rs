//! A reference's PDFs through the bridge: attach, status, verify, the browser page, remove.

mod common;
use common::fixture;

use quire_testkit::tiny_pdf;

#[tokio::test]
async fn a_pdf_is_attached_checked_and_removed() {
    let f = fixture().await;
    let file = f.dir.path().join("paper.pdf");
    std::fs::write(&file, tiny_pdf("Brain Research doi:10.1016/0006-8993(71)90358-1")).unwrap();
    let attached = f.library.attach_pdf("okeefe1971".into(), file.display().to_string(), true).await.unwrap();
    assert!(attached.new);
    assert_eq!(attached.verdict, "identifier");
    assert!(attached.reference.has_pdf);
    assert!(std::path::Path::new(&attached.path).exists());

    let id = attached.reference.id.clone();
    let statuses = f.library.pdf_statuses().await.unwrap();
    let status = statuses.iter().find(|s| s.id == id).unwrap();
    assert!(status.available && status.pages == Some(1), "{status:?}");

    let verified = f.library.verify_pdfs(vec![id.clone()]).await.unwrap();
    assert_eq!((verified.len(), verified[0].verdict.as_str()), (1, "identifier"));
    assert_eq!(f.library.pdf_files(id.clone()).await.unwrap(), std::slice::from_ref(&attached.path));

    let page = f.library.browser_page("okeefe1971".into()).await.unwrap();
    assert!(page.is_some_and(|p| p.contains("10.1016")), "the DOI's landing page");

    assert_eq!(f.library.remove_pdfs(id.clone()).await.unwrap(), 1);
    assert!(f.library.pdf_files(id).await.unwrap().is_empty());
}

#[tokio::test]
async fn another_papers_pdf_is_refused_when_strict() {
    let f = fixture().await;
    // Enough text to compare (a page with too little reads as a scan: nothing to refuse).
    let lines = [
        "Nanometre-scale thermometry in a living cell",
        "Nature 500, 54-58 (2013) doi:10.1038/nature12373",
        "Sensitive probing of temperature variations on nanometre scales is an outstanding challenge",
        "in many areas of modern science and technology. In particular, a thermometer capable of",
        "subdegree temperature resolution over a large range of temperatures as well as integration",
    ];
    let content: String =
        lines.iter().enumerate().map(|(i, l)| format!("BT /F1 10 Tf 72 {} Td ({l}) Tj ET ", 740 - i * 14)).collect();
    let file = f.dir.path().join("other.pdf");
    std::fs::write(&file, quire_testkit::pdf_page(&content, "")).unwrap();
    let error = f.library.attach_pdf("okeefe1971".into(), file.display().to_string(), true).await.unwrap_err();
    assert!(!error.to_string().is_empty());
    assert!(f.library.pdf_files("okeefe1971".into()).await.unwrap().is_empty());
}
