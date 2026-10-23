//! PDFs of references (`.knowledge/roadmap.md`, M7): finding and downloading them, attaching
//! files the user has, importing a PDF as a new reference, and the page to open in a browser
//! when a publisher blocks automated downloads.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use quire_core::library::{attachments, references};
use quire_core::prelude::*;
use quire_core::{Added, check};
use quire_document::pdf::{self, PdfMatch};
use quire_model::{Attachment, Identifier, Issue, IssueCode, PdfAttempt, Reference, ReferenceId, SlugPath, TextField};
use url::Url;

use super::Engine;
use crate::error::{EngineError, Result};
use crate::files::FileStore;
use crate::tools;

const PDF_TYPE: &str = "application/pdf";

/// The result of attaching a PDF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attached {
    pub reference: Reference,
    pub attachment: Attachment,
    /// `false` if the reference already had this exact file.
    pub new: bool,
    /// Where the file is stored.
    pub path: PathBuf,
    /// Whether the PDF is the reference's paper (by identifier or title).
    pub verdict: PdfMatch,
}

/// A reference's PDF, as lists show it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfStatus {
    /// Attached; the largest page count among its PDFs.
    Available { pages: Option<u32> },
    /// The last `pdf get` found none, for this reason.
    Missing(PdfAttempt),
}

/// What `pdf get` did for one reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfOutcome {
    /// It already had a PDF; nothing was downloaded.
    Present(Box<Reference>, Vec<Attachment>),
    /// Downloaded from `source` and attached.
    Downloaded { attached: Box<Attached>, source: String, url: String },
}

impl Engine {
    /// The library's file store.
    pub fn files(&self) -> FileStore {
        FileStore::new(self.library_dir())
    }

    /// Finds, downloads and attaches a PDF of the reference `query` names, unless it has one. A
    /// publisher's "are you human" page is recorded as a `blocked` issue (see [`Engine::browser_page`]).
    pub async fn get_pdf(&self, query: &str) -> Result<PdfOutcome> {
        let (reference, _) = self.show(query).await?;
        let present = self.attachments(&reference).await?;
        if !present.is_empty() {
            return Ok(PdfOutcome::Present(Box::new(reference), present));
        }
        self.download_pdf(reference).await
    }

    /// Finds the PDF of `reference` (open access, Unpaywall, the publisher) and attaches it if it is
    /// this paper's; a failed attempt is recorded (a blocked site as an issue, for the browser).
    pub(super) async fn download_pdf(&self, reference: Reference) -> Result<PdfOutcome> {
        let found = match self.fetcher.find_pdf(&reference.data).await {
            Ok(found) => found,
            Err(error) => {
                let outcome = attempt_outcome(&reference, &error);
                let (id, detail) = (reference.id, error.to_string());
                let blocked = error.is_blocked();
                self.run(move |lib| {
                    if blocked {
                        check::record(lib, id, Issue::recorded(IssueCode::Blocked, format!("PDF: {detail}")))?;
                    }
                    lib.set_pdf_attempt(id, Some((outcome, &detail)))
                })
                .await?;
                self.mark_changed();
                return Err(error.into());
            }
        };
        let url = found.url.to_string();
        let attached = match self.attach_bytes(&reference, &found.bytes, &url, true).await {
            Ok(attached) => attached,
            Err(error @ EngineError::NotThisPaper(_)) => {
                let (id, detail) = (reference.id, format!("{url}: {error}"));
                self.run(move |lib| lib.set_pdf_attempt(id, Some((PdfAttempt::NotFound, &detail)))).await?;
                self.mark_changed();
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        Ok(PdfOutcome::Downloaded { attached: Box::new(attached), source: found.source, url })
    }

    /// Attaches the PDF at `path` to the reference `query` names; `strict` refuses another paper's
    /// PDF (see [`Engine::attach_bytes`]).
    pub async fn attach_file(&self, query: &str, path: &Path, strict: bool) -> Result<Attached> {
        let bytes = std::fs::read(path).map_err(EngineError::file(path))?;
        let (reference, _) = self.show(query).await?;
        let name = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
        self.attach_bytes(&reference, &bytes, &format!("file:{name}"), strict).await
    }

    /// Adds the work a PDF describes (by the DOI or arXiv ID on its first pages) and attaches the
    /// PDF to it. Fails if the PDF names no identifier.
    pub async fn import_pdf(&self, path: &Path, project: Option<&SlugPath>) -> Result<(Added, Attached)> {
        let bytes = std::fs::read(path).map_err(EngineError::file(path))?;
        let name = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
        self.import_pdf_bytes(&bytes, &format!("file:{name}"), project).await
    }

    /// [`Engine::import_pdf`] for a PDF in memory (e.g. sent by the browser extension), recorded
    /// as coming from `source`.
    pub async fn import_pdf_bytes(
        &self,
        bytes: &[u8],
        source: &str,
        project: Option<&SlugPath>,
    ) -> Result<(Added, Attached)> {
        let info = pdf::inspect(bytes)?;
        let identifier = info.identifiers.first().ok_or_else(|| {
            EngineError::Pdf(format!(
                "{source} prints no DOI or arXiv ID on its first pages; attach it to a reference instead"
            ))
        })?;
        let added = self.add(&identifier.to_string(), project).await?;
        let attached = self.attach_bytes(added.reference(), bytes, source, false).await?;
        Ok((added, attached))
    }

    /// Checks `bytes` are a PDF of `reference`'s paper (its identifier, else its title, on the first
    /// pages), stores them and attaches them; clears a `blocked` issue. With `strict` (automatic
    /// downloads, files picked up from the downloads folder) another paper's PDF is refused;
    /// otherwise (the user chose the file) it is attached with a `pdf-mismatch` issue.
    pub async fn attach_bytes(
        &self,
        reference: &Reference,
        bytes: &[u8],
        source: &str,
        strict: bool,
    ) -> Result<Attached> {
        let info = pdf::inspect(bytes)?;
        let verdict = pdf::verdict(&info, &reference.data.identifiers, reference.data.title());
        if strict && let Some(reason) = verdict.reason() {
            return Err(EngineError::NotThisPaper(reason));
        }
        let hash = self.files().put(bytes, "pdf")?;
        let path = self.files().path(&hash, "pdf");
        let mut attachment = Attachment::new(
            reference.id,
            hash,
            format!("{}.pdf", reference.cite_key),
            PDF_TYPE.into(),
            bytes.len() as u64,
            source.into(),
            self.device(),
        );
        attachment.pdf_kind = Some(info.kind);
        attachment.pages = Some(info.pages);
        let id = reference.id;
        let doubt = verdict.reason();
        let (attachment, new) = self
            .run(move |lib| {
                let attached = attachments::attach(lib, attachment)?;
                check::clear(lib, id, IssueCode::Blocked)?;
                lib.set_pdf_attempt(id, None)?;
                match doubt {
                    Some(reason) => {
                        check::record(lib, id, Issue::recorded(IssueCode::PdfMismatch, format!("the PDF: {reason}")))?
                    }
                    None => check::clear(lib, id, IssueCode::PdfMismatch)?,
                }
                Ok(attached)
            })
            .await?;
        self.mark_changed();
        Ok(Attached { reference: reference.clone(), attachment, new, path, verdict })
    }

    /// Live attachments of `reference`.
    pub async fn attachments(&self, reference: &Reference) -> Result<Vec<Attachment>> {
        let id = reference.id;
        self.run(move |lib| lib.attachments_of(id)).await
    }

    /// The reference `query` names, with its attachments and their stored paths.
    pub async fn pdfs_of(&self, query: &str) -> Result<(Reference, Vec<(Attachment, PathBuf)>)> {
        let (reference, _) = self.show(query).await?;
        let store = self.files();
        let attached = self.attachments(&reference).await?.into_iter().map(|a| {
            let path = store.path(&a.hash, "pdf");
            (a, path)
        });
        Ok((reference, attached.collect()))
    }

    /// Removes every attachment of the reference `query` names (the stored files stay); returns
    /// how many.
    pub async fn remove_pdfs(&self, query: &str) -> Result<usize> {
        let query = query.to_string();
        let removed = self
            .run(move |lib| {
                let reference = references::find(lib, &query)?;
                attachments::detach_all(lib, reference.id)
            })
            .await?;
        self.mark_changed();
        Ok(removed)
    }

    /// The PDF status of every reference that has a PDF or a recorded failed attempt; others
    /// were never tried.
    pub async fn pdf_statuses(&self) -> Result<HashMap<ReferenceId, PdfStatus>> {
        self.run(|lib| {
            let mut statuses: HashMap<ReferenceId, PdfStatus> =
                lib.pdf_attempts()?.into_iter().map(|(id, attempt)| (id, PdfStatus::Missing(attempt))).collect();
            for id in lib.references_with_attachments()? {
                let pages = lib.attachments_of(id)?.iter().filter_map(|a| a.pages).max();
                statuses.insert(id, PdfStatus::Available { pages });
            }
            Ok(statuses)
        })
        .await
    }

    /// The reference `query` names and its paper as Markdown (reader mode). Converted from its
    /// first PDF once and stored next to it in the file store (`<hash>.md`), then read from there;
    /// rebuilt when the converter changed. Also returns whether it came from the store.
    pub async fn paper_text(&self, query: &str) -> Result<(Reference, String, bool)> {
        let (reference, pdfs) = self.pdfs_of(query).await?;
        let (attachment, pdf_path) = pdfs.into_iter().next().ok_or_else(|| {
            EngineError::Pdf(format!(
                "`{}` has no PDF; `quire pdf get {}` looks for one",
                reference.cite_key, reference.cite_key
            ))
        })?;
        let cache = self.files().path(&attachment.hash, "md");
        let header = format!("<!-- quire reader text: {} -->\n", pdf::READER_CONVERTER);
        if let Ok(text) = std::fs::read_to_string(&cache)
            && let Some(body) = text.strip_prefix(&header)
        {
            return Ok((reference, body.to_string(), true));
        }
        let bytes = std::fs::read(&pdf_path).map_err(EngineError::file(&pdf_path))?;
        let pages = attachment.pages.unwrap_or(1);
        let language = self.ocr_language();
        // Conversion is CPU work (tens of ms per page): off the async threads.
        let markdown = tokio::task::spawn_blocking(move || {
            let converted = pdf::to_markdown(&bytes).map_err(EngineError::from);
            // Only stamps read (a scan whose OCR text pdf-inspector drops): poppler, if installed;
            // no text layer at all (a pure scan): OCR, if Tesseract is installed.
            let sparse = converted.as_deref().map_or(true, |m| pdf::is_sparse(m, pages));
            if !sparse {
                return converted;
            }
            if let Some(text) = tools::poppler_text(&pdf_path) {
                return Ok(pdf::plain::to_markdown(&text));
            }
            match tools::ocr_text(&pdf_path, &language) {
                Ok(Some(text)) => Ok(pdf::plain::to_markdown(&text)),
                Ok(None) => converted.map_err(|e| {
                    EngineError::Pdf(format!("{e}; install Tesseract and poppler (pdftoppm) for OCR of scans"))
                }),
                Err(error) => Err(error),
            }
        })
        .await
        .map_err(|e| EngineError::Pdf(format!("converting the PDF failed: {e}")))??;
        let temporary = cache.with_extension("md.partial");
        std::fs::write(&temporary, format!("{header}{markdown}")).map_err(EngineError::file(&temporary))?;
        std::fs::rename(&temporary, &cache).map_err(EngineError::file(&cache))?;
        Ok((reference, markdown, false))
    }

    /// Converts again every stored reader text made by an older converter (after an update), one
    /// paper at a time, so opening a paper does not wait for it; papers never opened (no stored
    /// text) are left alone. A paper that fails is skipped. Returns how many were rebuilt.
    pub async fn rebuild_stale_texts(&self) -> Result<usize> {
        let header = format!("<!-- quire reader text: {} -->\n", pdf::READER_CONVERTER);
        let mut rebuilt = 0;
        for id in self.references_with_pdfs().await? {
            let (_, pdfs) = self.pdfs_of(&id.to_string()).await?;
            let Some((attachment, _)) = pdfs.first() else { continue };
            let cache = self.files().path(&attachment.hash, "md");
            let Ok(stored) = std::fs::read_to_string(&cache) else { continue };
            if stored.starts_with(&header) {
                continue;
            }
            match self.paper_text(&id.to_string()).await {
                Ok(_) => rebuilt += 1,
                Err(error) => tracing::debug!(%error, "rebuilding a stale reader text"),
            }
        }
        Ok(rebuilt)
    }

    /// Re-checks every PDF of `references` against its reference (identifier, else title); changes
    /// nothing. For finding wrong attachments.
    pub async fn verify_pdfs(&self, references: Vec<Reference>) -> Result<Vec<(Reference, Attachment, PdfMatch)>> {
        let store = self.files();
        let mut out = Vec::new();
        for reference in references {
            for attachment in self.attachments(&reference).await? {
                let path = store.path(&attachment.hash, "pdf");
                let bytes = std::fs::read(&path).map_err(EngineError::file(&path))?;
                let verdict = tokio::task::spawn_blocking(move || pdf::inspect(&bytes))
                    .await
                    .map_err(|e| EngineError::Pdf(format!("checking the PDF failed: {e}")))?
                    .map(|info| pdf::verdict(&info, &reference.data.identifiers, reference.data.title()))?;
                out.push((reference.clone(), attachment, verdict));
            }
        }
        Ok(out)
    }

    /// IDs of references that have a PDF.
    pub async fn references_with_pdfs(&self) -> Result<HashSet<ReferenceId>> {
        Ok(self.run(|lib| lib.references_with_attachments()).await?.into_iter().collect())
    }

    /// The page to open in a browser to get the PDF by hand: the DOI's landing page, else the
    /// reference's URL, else its arXiv page. The user logs in there with the publisher.
    pub fn browser_page(&self, reference: &Reference) -> Option<Url> {
        let data = &reference.data;
        data.identifiers
            .iter()
            .find_map(|id| match id {
                Identifier::Doi(doi) => Url::parse(&format!("https://doi.org/{}", doi.as_str())).ok(),
                _ => None,
            })
            // Only web pages: a URL from a page's metadata could be `file:` or another scheme.
            .or_else(|| {
                data.get(TextField::Url)
                    .and_then(|u| Url::parse(u).ok())
                    .filter(|u| matches!(u.scheme(), "http" | "https"))
            })
            .or_else(|| {
                data.identifiers.iter().find_map(|id| match id {
                    Identifier::Arxiv(a) => Url::parse(&format!("https://arxiv.org/abs/{}", a.id())).ok(),
                    _ => None,
                })
            })
    }

    /// The folder the browser saves downloads to: `access.downloads`, else the XDG download
    /// folder, else `~/Downloads`.
    pub fn downloads_dir(&self) -> Option<PathBuf> {
        if let Some(dir) = &self.config.access.downloads {
            return Some(crate::paths::expand_home(dir));
        }
        let from_xdg = std::process::Command::new("xdg-user-dir")
            .arg("DOWNLOAD")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| PathBuf::from(s.trim()))
            .filter(|p| p.is_dir());
        from_xdg.or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Downloads")))
    }
}

/// Why `error` gave no PDF for `reference`.
fn attempt_outcome(reference: &Reference, error: &quire_fetch::FetchError) -> PdfAttempt {
    let searchable =
        reference.data.identifiers.iter().any(|id| matches!(id, Identifier::Doi(_) | Identifier::Arxiv(_)));
    if error.is_blocked() {
        PdfAttempt::Blocked
    } else if error.is_transient() {
        PdfAttempt::Error
    } else if !searchable && !reference.data.extra.contains_key("pdf-url") {
        PdfAttempt::NoIdentifier
    } else {
        PdfAttempt::NotFound
    }
}
