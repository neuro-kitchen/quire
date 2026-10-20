//! PDFs through the engine: download and attach (arXiv, mock), the same file only once, a PDF
//! showing another DOI, importing a PDF as a new reference, and a blocked page recorded as an issue.

mod common;
use common::{engine, fixture, tiny_pdf};

use quire_engine::{IssueCode, PdfOutcome};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn serve(server: &MockServer, at: &str, response: ResponseTemplate) {
    Mock::given(method("GET")).and(path(at)).respond_with(response).mount(server).await;
}

#[tokio::test]
async fn download_attach_once_and_remove() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/10.48550/arXiv.1706.03762",
        ResponseTemplate::new(200).set_body_string(fixture("csl-arxiv-1706.03762.json")),
    )
    .await;
    serve(
        &server,
        "/pdf/1706.03762",
        ResponseTemplate::new(200).set_body_bytes(tiny_pdf("arXiv:1706.03762v7 [cs.CL]")),
    )
    .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    engine.add("arXiv:1706.03762", None).await.unwrap();

    let PdfOutcome::Downloaded { attached, source, .. } = engine.get_pdf("vaswaniAttentionAllYou2017").await.unwrap()
    else {
        panic!("downloaded")
    };
    assert_eq!(source, "arxiv");
    assert!(attached.new && attached.verdict.reason().is_none() && attached.path.is_file());
    assert_eq!(attached.attachment.filename, "vaswaniAttentionAllYou2017.pdf");
    assert!(matches!(engine.get_pdf("vaswaniAttentionAllYou2017").await.unwrap(), PdfOutcome::Present(..)));

    // The same file again (e.g. imported by hand): stored and attached once.
    let file = dir.path().join("copy.pdf");
    std::fs::write(&file, tiny_pdf("arXiv:1706.03762v7 [cs.CL]")).unwrap();
    assert!(!engine.attach_file("vaswaniAttentionAllYou2017", &file, false).await.unwrap().new);
    assert_eq!(engine.references_with_pdfs().await.unwrap().len(), 1);
    assert_eq!(engine.remove_pdfs("vaswaniAttentionAllYou2017").await.unwrap(), 1);
    assert!(engine.references_with_pdfs().await.unwrap().is_empty());
}

#[tokio::test]
async fn pdf_of_another_paper_is_flagged() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/10.1038/nature12373",
        ResponseTemplate::new(200).set_body_string(fixture("csl-crossref-nature12373.json")),
    )
    .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let reference = engine.add("10.1038/nature12373", None).await.unwrap().reference().clone();
    let file = dir.path().join("wrong.pdf");
    let other_paper =
        format!("Deep residual learning for image recognition doi:10.1000/other{}", " we present results".repeat(20));
    std::fs::write(&file, tiny_pdf(&other_paper)).unwrap();
    // Downloaded or picked up automatically: refused.
    let refused = engine.attach_file("10.1038/nature12373", &file, true).await.unwrap_err();
    assert!(refused.to_string().contains("not this paper"), "{refused}");
    // Chosen by the user: attached, flagged.
    let attached = engine.attach_file("10.1038/nature12373", &file, false).await.unwrap();
    assert!(matches!(attached.verdict, quire_engine::PdfMatch::Different(_)));
    let codes: Vec<IssueCode> = engine.issues(&reference).await.unwrap().iter().map(|i| i.code).collect();
    assert!(codes.contains(&IssueCode::PdfMismatch), "{codes:?}");
}

#[tokio::test]
async fn import_pdf_adds_its_reference() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/10.1038/nature12373",
        ResponseTemplate::new(200).set_body_string(fixture("csl-crossref-nature12373.json")),
    )
    .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let file = dir.path().join("paper.pdf");
    std::fs::write(&file, tiny_pdf("Nature 500, 54 doi:10.1038/nature12373")).unwrap();
    let (added, attached) = engine.import_pdf(&file, None).await.unwrap();
    assert_eq!(added.reference().cite_key.as_str(), "kucskoNanometreScaleThermometry2013");
    assert_eq!(attached.attachment.source, "file:paper.pdf");

    let blank = dir.path().join("blank.pdf");
    std::fs::write(&blank, tiny_pdf("no identifier here")).unwrap();
    assert!(engine.import_pdf(&blank, None).await.is_err());
    std::fs::write(&blank, "<html>not a pdf</html>").unwrap();
    assert!(engine.attach_file("kucskoNanometreScaleThermometry2013", &blank, false).await.is_err());
}

#[tokio::test]
async fn blocked_download_is_recorded() {
    let server = MockServer::start().await;
    let page = format!(
        r#"<html><head><meta name="citation_title" content="A paper"><meta name="citation_author" content="Doe, Jane">
        <meta name="citation_journal_title" content="J"><meta name="citation_pdf_url" content="{}/pmc/pdf"></head></html>"#,
        server.uri()
    );
    serve(&server, "/articles/1", ResponseTemplate::new(200).set_body_string(page)).await;
    serve(&server, "/pmc/pdf", ResponseTemplate::new(200).set_body_string(fixture("challenge-pmc-pdf.html"))).await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let reference = engine.add(&format!("{}/articles/1", server.uri()), None).await.unwrap().reference().clone();

    let error = engine.get_pdf(reference.cite_key.as_str()).await.unwrap_err();
    assert!(error.is_blocked(), "{error}");
    let codes: Vec<IssueCode> = engine.issues(&reference).await.unwrap().iter().map(|i| i.code).collect();
    assert!(codes.contains(&IssueCode::Blocked), "{codes:?}");
    assert_eq!(engine.browser_page(&reference, true).unwrap().as_str(), format!("{}/articles/1", server.uri()));

    // Fetched by hand in the browser and attached: the blocked issue goes away.
    let file = dir.path().join("downloaded.pdf");
    std::fs::write(&file, tiny_pdf("A paper")).unwrap();
    engine.attach_file(reference.cite_key.as_str(), &file, false).await.unwrap();
    let codes: Vec<IssueCode> = engine.issues(&reference).await.unwrap().iter().map(|i| i.code).collect();
    assert!(!codes.contains(&IssueCode::Blocked), "{codes:?}");
}

#[tokio::test]
async fn paper_text_is_converted_once_and_stored() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/10.1038/nature12373",
        ResponseTemplate::new(200).set_body_string(fixture("csl-crossref-nature12373.json")),
    )
    .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    engine.add("10.1038/nature12373", None).await.unwrap();
    let file = dir.path().join("paper.pdf");
    std::fs::write(&file, tiny_pdf("Nanometre scale thermometry doi:10.1038/nature12373")).unwrap();
    let attached = engine.attach_file("10.1038/nature12373", &file, false).await.unwrap();

    let (_, first, stored) = engine.paper_text("10.1038/nature12373").await.unwrap();
    assert!(!stored && first.contains("thermometry"));
    assert!(attached.path.with_extension("md").is_file(), "stored next to the PDF");
    let (_, again, stored) = engine.paper_text("10.1038/nature12373").await.unwrap();
    assert!(stored && again == first);

    // Text from an older converter is rebuilt.
    std::fs::write(attached.path.with_extension("md"), "<!-- quire reader text: old -->\nstale").unwrap();
    let (_, rebuilt, stored) = engine.paper_text("10.1038/nature12373").await.unwrap();
    assert!(!stored && rebuilt == first);
}

#[tokio::test]
async fn scans_without_ocr_tools_say_what_to_install() {
    if quire_engine::tools::installed("tesseract") || quire_engine::tools::installed("ocrmypdf") {
        return; // OCR would run here; this checks the message when it cannot.
    }
    let server = MockServer::start().await;
    serve(
        &server,
        "/10.1038/nature12373",
        ResponseTemplate::new(200).set_body_string(fixture("csl-crossref-nature12373.json")),
    )
    .await;
    let dir = tempfile::tempdir().unwrap();
    let engine = engine(dir.path(), Some(&server)).await;
    let added = engine.add("10.1038/nature12373", None).await.unwrap();
    let key = added.reference().cite_key.to_string();
    // A page with no text at all, as a scan without a text layer looks to the converter.
    let scan = dir.path().join("scan.pdf");
    std::fs::write(&scan, tiny_pdf("")).unwrap();
    engine.attach_file(&key, &scan, false).await.unwrap();
    let error = engine.ocr(&key).await.unwrap_err().to_string();
    assert!(error.contains("OCRmyPDF") && error.contains("Tesseract"), "{error}");
}
