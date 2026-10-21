//! Lazy PDFs (`.knowledge/sync-hub.md`): with `files.download = "on-open"` a device fetches a PDF
//! when it is opened; papers and projects kept offline are fetched at sync; past `files.cache_mb`
//! the least recently opened files go, but only those the hub confirmed it holds (never what may
//! be the last copy), never one kept offline. Without a hub nothing is ever removed.
//!
//! Opening a paper looks, in order: here → the hub → another of the user's devices → the source
//! again (open access, Unpaywall, the publisher; the same check that the PDF is this paper's).

use std::collections::HashSet;
use std::path::PathBuf;

use quire_model::SlugPath;
use quire_store::KeepTarget;

use super::Engine;
use super::pdfs::PdfOutcome;
use crate::error::{EngineError, Result};

/// Where an opened PDF came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfSource {
    /// Already on this device.
    Here,
    /// Downloaded from the hub (sealed there, opened and checked here).
    Hub,
    /// Fetched from another of the user's devices (its peer name).
    Peer(String),
    /// Found again at the source (which), and attached.
    Found(String),
}

/// A PDF ready to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedPdf {
    pub path: PathBuf,
    pub hash: String,
    pub source: PdfSource,
}

/// What removing files to stay within `files.cache_mb` did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Eviction {
    pub removed: usize,
    pub freed: u64,
    /// Bytes of PDFs here after it.
    pub kept: u64,
}

impl Engine {
    /// Keeps (or stops keeping) the PDFs of the reference `query` downloaded; `false` if no change.
    pub async fn keep_reference_offline(&self, query: &str, keep: bool) -> Result<bool> {
        let (reference, _) = self.show(query).await?;
        let id = reference.id;
        self.run(move |lib| lib.set_keep_offline(KeepTarget::Reference(id), keep)).await
    }

    /// Keeps (or stops keeping) the PDFs of the project at `path` and its subprojects downloaded.
    pub async fn keep_project_offline(&self, path: &SlugPath, keep: bool) -> Result<bool> {
        let id = self.project_id(Some(path)).await?.ok_or_else(|| EngineError::Config(format!("no project {path}")))?;
        self.run(move |lib| lib.set_keep_offline(KeepTarget::Project(id), keep)).await
    }

    /// Files the next hub sync should download: every missing one, or with `on-open` only those
    /// kept offline.
    pub(super) async fn wanted_files(&self) -> Result<Vec<String>> {
        let missing = self.missing_files().await?;
        if !self.config.files.on_open() {
            return Ok(missing);
        }
        let kept: HashSet<String> = self.run(|lib| lib.kept_hashes()).await?;
        Ok(missing.into_iter().filter(|hash| kept.contains(hash)).collect())
    }

    /// Hashes the live attachments name whose files are not here.
    pub(super) async fn missing_files(&self) -> Result<Vec<String>> {
        let hashes = self.run(|lib| lib.attachment_hashes()).await?;
        let files = self.files();
        Ok(hashes.into_iter().filter(|hash| !files.path(hash, "pdf").is_file()).collect())
    }

    /// The PDF of the reference `query`, fetched if it is not here (hub, own device, the source).
    pub async fn open_pdf(&self, query: &str) -> Result<OpenedPdf> {
        let (reference, _) = self.show(query).await?;
        let attachments = self.attachments(&reference).await?;
        let files = self.files();
        let hashes: Vec<String> = attachments.iter().map(|a| a.hash.clone()).collect();
        let opened = |hash: &str, source| OpenedPdf { path: files.path(hash, "pdf"), hash: hash.to_string(), source };

        let mut found = hashes.iter().find(|h| files.path(h, "pdf").is_file()).map(|h| opened(h, PdfSource::Here));
        if found.is_none() && self.config.hub.id.is_some() {
            for hash in &hashes {
                match self.fetch_from_hub(hash).await {
                    Ok(true) => {
                        let on_hub = vec![hash.clone()];
                        self.run(move |lib| lib.mark_on_hub(&on_hub)).await?;
                        found = Some(opened(hash, PdfSource::Hub));
                        break;
                    }
                    Ok(false) => {}
                    Err(error) => tracing::debug!(%error, "hub not reachable for a PDF"),
                }
            }
        }
        if found.is_none() {
            'peers: for (name, _) in self.config.peers.clone() {
                for hash in &hashes {
                    match self.fetch_from_peer(&name, hash).await {
                        Ok(true) => {
                            found = Some(opened(hash, PdfSource::Peer(name.clone())));
                            break 'peers;
                        }
                        Ok(false) => {}
                        Err(error) => tracing::debug!(%error, peer = %name, "device not reachable for a PDF"),
                    }
                }
            }
        }
        if found.is_none() {
            // Found again at the source: often other bytes than the original (publishers stamp
            // downloads), so another attachment of this paper; highlights follow by their text.
            match self.download_pdf(reference).await {
                Ok(PdfOutcome::Downloaded { attached, source, .. }) => {
                    let hash = attached.attachment.hash.clone();
                    found = Some(opened(&hash, PdfSource::Found(source)));
                }
                Ok(PdfOutcome::Present(..)) => {}
                Err(error) if hashes.is_empty() => return Err(error),
                Err(error) => {
                    return Err(EngineError::Config(format!(
                        "this paper's PDF is not on this device, and neither the hub, your other devices nor the \
                         source had it ({error})"
                    )));
                }
            }
        }
        let found = found.ok_or_else(|| EngineError::Config("this paper has no PDF".into()))?;
        let hash = found.hash.clone();
        self.run(move |lib| lib.touch_file(&hash)).await?;
        Ok(found)
    }

    /// Removes PDFs to stay within `files.cache_mb`: least recently opened first, only those the
    /// hub holds, never one kept offline.
    pub async fn evict_pdfs(&self) -> Result<Eviction> {
        let limit = u64::from(self.config.files.cache_mb.unwrap_or(0)) * 1024 * 1024;
        let files = self.files();
        let mut here: Vec<_> = self
            .run(|lib| lib.file_states())
            .await?
            .into_iter()
            .filter(|f| files.path(&f.hash, "pdf").is_file())
            .collect();
        let mut eviction = Eviction { kept: here.iter().map(|f| f.size).sum(), ..Eviction::default() };
        if limit == 0 || eviction.kept <= limit {
            return Ok(eviction);
        }
        let kept: HashSet<String> = self.run(|lib| lib.kept_hashes()).await?;
        here.retain(|f| f.on_hub && !kept.contains(&f.hash));
        here.sort_by_key(|f| f.last_opened.unwrap_or(i64::MIN));
        for file in here {
            if eviction.kept <= limit {
                break;
            }
            let path = files.path(&file.hash, "pdf");
            std::fs::remove_file(&path).map_err(EngineError::file(&path))?;
            eviction.removed += 1;
            eviction.freed += file.size;
            eviction.kept -= file.size;
        }
        Ok(eviction)
    }
}
