//! A reference's PDFs: download, attach a file, import a file as a new reference, remove, status,
//! verify, OCR, and the browser flow for what publishers block. Paths are local files.

use std::path::Path;

use quire_engine::{AbstractOutcome, Attached, Engine, OcrOutcome, PdfMatch, PdfOutcome, PdfStatus};

use crate::records::ReferenceRecord;
use crate::references::AddResult;
use crate::{QuireLibrary, Result, query};

/// A PDF attached to a reference.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AttachedRecord {
    pub reference: ReferenceRecord,
    /// `false` if the reference already had this exact file.
    pub new: bool,
    /// Where the file is stored.
    pub path: String,
    /// Whether it is the reference's paper: `identifier`, `title`, `unknown`, `different`, `unlikely`.
    pub verdict: String,
}

/// What getting a PDF did: `downloaded` from `source`, or the files it already had.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PdfFetchRecord {
    pub downloaded: bool,
    pub source: Option<String>,
    pub url: Option<String>,
    /// The new file when downloaded, else the files already attached.
    pub files: Vec<String>,
    pub verdict: Option<String>,
}

/// A reference's PDF state: `available` (with its page count), or why the last download found none
/// (`blocked`, `not-found`, `no-identifier`, `error`).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PdfStatusRecord {
    pub id: String,
    pub available: bool,
    pub pages: Option<u32>,
    pub missing_reason: Option<String>,
}

/// A stored PDF checked again against its reference.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct VerifiedPdf {
    pub id: String,
    pub path: String,
    pub verdict: String,
}

/// A PDF file added as a new reference (by the identifier it prints) and attached to it.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ImportedPdf {
    pub added: AddResult,
    pub attached: AttachedRecord,
}

#[uniffi::export]
impl QuireLibrary {
    /// Finds, downloads and attaches a PDF, unless the reference has one. A publisher's "are you
    /// human" page is recorded as a `blocked` issue: then [`QuireLibrary::browser_page`].
    pub async fn get_pdf(&self, id: String) -> Result<PdfFetchRecord> {
        self.with(|engine| async move {
            Ok(match engine.get_pdf(&id).await? {
                PdfOutcome::Present(reference, _) => {
                    let (_, files) = engine.pdfs_of(&reference.id.to_string()).await?;
                    PdfFetchRecord {
                        downloaded: false,
                        source: None,
                        url: None,
                        files: files.into_iter().map(|(_, path)| path.display().to_string()).collect(),
                        verdict: None,
                    }
                }
                PdfOutcome::Downloaded { attached, source, url } => PdfFetchRecord {
                    downloaded: true,
                    source: Some(source),
                    url: Some(url),
                    files: vec![attached.path.display().to_string()],
                    verdict: Some(verdict(&attached.verdict).into()),
                },
            })
        })
        .await
    }

    /// Attaches the PDF at `path`; `strict` refuses a PDF of another paper.
    pub async fn attach_pdf(&self, id: String, path: String, strict: bool) -> Result<AttachedRecord> {
        self.with(
            |engine| async move { attached(&engine, engine.attach_file(&id, Path::new(&path), strict).await?).await },
        )
        .await
    }

    /// Adds the work the PDF at `path` describes (by the identifier on its first pages) and attaches it.
    pub async fn import_pdf(&self, path: String, project: Option<String>) -> Result<ImportedPdf> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let (added, file) = engine.import_pdf(Path::new(&path), project.as_ref()).await?;
            let new = matches!(added, quire_engine::Added::New(_));
            let reference = query::reference(&engine, &added.reference().id.to_string()).await?;
            Ok(ImportedPdf { added: AddResult { new, reference }, attached: attached(&engine, file).await? })
        })
        .await
    }

    /// Removes every PDF of a reference (the stored files stay); returns how many.
    pub async fn remove_pdfs(&self, id: String) -> Result<u32> {
        self.with(|engine| async move { Ok(engine.remove_pdfs(&id).await? as u32) }).await
    }

    /// The PDF state of every reference that has a PDF or a failed attempt.
    pub async fn pdf_statuses(&self) -> Result<Vec<PdfStatusRecord>> {
        self.with(|engine| async move {
            let mut statuses: Vec<PdfStatusRecord> = engine
                .pdf_statuses()
                .await?
                .into_iter()
                .map(|(id, status)| match status {
                    PdfStatus::Available { pages } => {
                        PdfStatusRecord { id: id.to_string(), available: true, pages, missing_reason: None }
                    }
                    PdfStatus::Missing(attempt) => PdfStatusRecord {
                        id: id.to_string(),
                        available: false,
                        pages: None,
                        missing_reason: Some(attempt.as_str().to_string()),
                    },
                })
                .collect();
            statuses.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(statuses)
        })
        .await
    }

    /// Checks the PDFs of `ids` against their references again.
    pub async fn verify_pdfs(&self, ids: Vec<String>) -> Result<Vec<VerifiedPdf>> {
        self.with(|engine| async move {
            let mut references = Vec::new();
            for id in &ids {
                references.push(engine.show(id).await?.0);
            }
            let checked = engine.verify_pdfs(references).await?;
            let files = engine.files();
            Ok(checked
                .into_iter()
                .map(|(reference, attachment, verdict_)| VerifiedPdf {
                    id: reference.id.to_string(),
                    path: files.path(&attachment.hash, "pdf").display().to_string(),
                    verdict: verdict(&verdict_).into(),
                })
                .collect())
        })
        .await
    }

    /// OCR of a scanned paper's first PDF: `true` if the PDF got a text layer (OCRmyPDF), `false`
    /// if Tesseract read its text (then the paper's text, for the reader and search).
    pub async fn ocr(&self, id: String) -> Result<bool> {
        self.with(|engine| async move { Ok(matches!(engine.ocr(&id).await?.1, OcrOutcome::TextLayer)) }).await
    }

    /// The page to open in the browser to get a blocked PDF by hand (the DOI's landing page).
    pub async fn browser_page(&self, id: String) -> Result<Option<String>> {
        self.with(|engine| async move {
            let (reference, _) = engine.show(&id).await?;
            Ok(engine.browser_page(&reference).map(|url| url.to_string()))
        })
        .await
    }

    /// Where the browser saves downloads (the configured folder, else the system's).
    pub fn downloads_dir(&self) -> Option<String> {
        self.engine.downloads_dir().map(|path| path.display().to_string())
    }

    /// Fills a missing abstract (Europe PMC, then with `from_pdf` the PDF's text); returns it if found.
    pub async fn fill_abstract(&self, id: String, from_pdf: bool) -> Result<Option<String>> {
        self.with(|engine| async move {
            Ok(match engine.fill_abstract(&id, from_pdf, false).await?.1 {
                AbstractOutcome::Found { text, .. } => Some(text),
                AbstractOutcome::Present | AbstractOutcome::NotFound => None,
            })
        })
        .await
    }

    /// Forgets a paper's stored text, so the reader converts it again; returns how many texts.
    pub async fn forget_text(&self, id: String) -> Result<u32> {
        self.with(|engine| async move { Ok(engine.forget_text(&id).await? as u32) }).await
    }
}

async fn attached(engine: &Engine, attached: Attached) -> Result<AttachedRecord> {
    Ok(AttachedRecord {
        reference: query::reference(engine, &attached.reference.id.to_string()).await?,
        new: attached.new,
        path: attached.path.display().to_string(),
        verdict: verdict(&attached.verdict).into(),
    })
}

fn verdict(verdict: &PdfMatch) -> &'static str {
    match verdict {
        PdfMatch::Identifier => "identifier",
        PdfMatch::Title => "title",
        PdfMatch::Unknown => "unknown",
        PdfMatch::Different(_) => "different",
        PdfMatch::Unlikely => "unlikely",
    }
}
