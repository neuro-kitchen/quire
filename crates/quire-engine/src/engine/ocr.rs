//! OCR of scanned PDFs (no text layer). With OCRmyPDF installed, the PDF itself gets a text layer
//! (it replaces the scan as the reference's PDF: same pages, now searchable and selectable in
//! any viewer); otherwise Tesseract reads the pages and the text is stored as the paper's text.

use quire_core::library::attachments;
use quire_model::Reference;

use super::Engine;
use crate::error::{EngineError, Result};
use crate::tools;

/// Tesseract's language when none is configured.
const DEFAULT_LANGUAGE: &str = "eng";

/// What [`Engine::ocr`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrOutcome {
    /// OCRmyPDF added a text layer; the new PDF replaced the scan.
    TextLayer,
    /// Tesseract read the pages; the text is the paper's text (reader, search).
    Text { words: usize },
}

impl Engine {
    /// The configured OCR language (`reader.ocr_language`), else English.
    pub(crate) fn ocr_language(&self) -> String {
        self.config().reader.ocr_language.clone().unwrap_or_else(|| DEFAULT_LANGUAGE.into())
    }

    /// OCR of the first PDF of the paper `query` names.
    pub async fn ocr(&self, query: &str) -> Result<(Reference, OcrOutcome)> {
        let (reference, pdfs) = self.pdfs_of(query).await?;
        let (attachment, path) = pdfs.into_iter().next().ok_or_else(|| {
            EngineError::Pdf(format!(
                "`{}` has no PDF; `quire pdf get {}` looks for one",
                reference.cite_key, reference.cite_key
            ))
        })?;
        let language = self.ocr_language();
        let dir = tempfile::tempdir().map_err(EngineError::file(&path))?;
        let output = dir.path().join("ocr.pdf");
        let (input, out, lang) = (path.clone(), output.clone(), language.clone());
        let layered = tokio::task::spawn_blocking(move || tools::ocrmypdf(&input, &out, &lang))
            .await
            .map_err(|e| EngineError::Pdf(format!("ocrmypdf: {e}")))??;
        if layered {
            let bytes = std::fs::read(&output).map_err(EngineError::file(&output))?;
            self.attach_bytes(&reference, &bytes, "ocr:ocrmypdf", false).await?;
            let old = attachment.id;
            self.run(move |lib| attachments::detach(lib, old)).await?;
            self.forget_text(&reference.id.to_string()).await?;
            self.mark_changed();
            return Ok((reference, OcrOutcome::TextLayer));
        }
        // No OCRmyPDF: Tesseract reads the pages when the paper's text is converted again.
        if !tools::installed("tesseract") || !tools::installed("pdftoppm") {
            return Err(EngineError::Pdf(
                "OCR needs OCRmyPDF, or Tesseract with poppler's pdftoppm; none is installed".into(),
            ));
        }
        self.forget_text(&reference.id.to_string()).await?;
        let (reference, markdown, _) = self.paper_text(&reference.id.to_string()).await?;
        self.index_paper_text(reference.id, Some(markdown.clone())).await?;
        Ok((reference, OcrOutcome::Text { words: markdown.split_whitespace().count() }))
    }
}
