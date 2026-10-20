//! The text reader mode shows: a structured full text when arXiv (HTML) or Europe PMC (JATS)
//! has one, with real equations, captions and tables; else the PDF's text. Full texts are kept in
//! the library folder (`text/<reference>.md`, private) so a paper is fetched once; when neither
//! source has a paper, a marker says so and they are asked again after [`RETRY_AFTER`].

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use quire_fetch::{FullText, FullTextSource};
use quire_model::{Identifier, Reference};

use super::Engine;
use crate::error::{EngineError, Result};
use crate::private;

/// Where a reader text came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSource {
    ArxivHtml,
    EuropePmc,
    Pdf,
}

impl TextSource {
    /// Shown in the reader's tab bar.
    pub fn label(self) -> &'static str {
        match self {
            Self::ArxivHtml => "arXiv HTML",
            Self::EuropePmc => "Europe PMC",
            Self::Pdf => "PDF",
        }
    }

    /// Whether the text has LaTeX math (`$…$`); PDF text has none.
    pub fn has_math(self) -> bool {
        self != Self::Pdf
    }
}

/// A paper's text for the reader, as Markdown (`quire_fetch::fulltext` describes it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaperText {
    pub markdown: String,
    pub source: TextSource,
}

/// Version of the stored full texts: raised when the converters change, so texts are fetched
/// and converted again. 3: figure images (arXiv).
const FORMAT: u32 = 3;
/// How long "no full text online" is believed before asking again (papers get HTML versions
/// and open-access copies later).
const RETRY_AFTER: Duration = Duration::from_secs(30 * 24 * 3600);
const NONE: &str = "none";

impl Engine {
    /// The text of the paper `query` names for reader mode: a stored full text, else one from
    /// arXiv or Europe PMC (then stored), else its PDF's text. An error when there is none.
    pub async fn reader_text(&self, query: &str) -> Result<(Reference, PaperText)> {
        let (reference, text) = self.find_reader_text(query).await?;
        // Searchable from now on (texts stored before the index existed get indexed here too).
        self.index_paper_text(reference.id, Some(text.markdown.clone())).await?;
        Ok((reference, text))
    }

    async fn find_reader_text(&self, query: &str) -> Result<(Reference, PaperText)> {
        let (reference, _) = self.show(query).await?;
        let path = self.text_path(&reference);
        let mut ask_online = online_candidate(&reference);
        match read_stored(&path) {
            Some(Stored::Text(text)) => return Ok((reference, text)),
            Some(Stored::NoneFresh) => ask_online = false,
            None => {}
        }
        if ask_online {
            match self.fetcher.full_text(&reference.data.identifiers).await {
                Ok(Some(full)) => {
                    let text = store(&path, Some(&full))?;
                    return Ok((reference, text.expect("stored a text")));
                }
                Ok(None) => {
                    store(&path, None)?;
                }
                // Offline or a service down: the PDF for now, online again next time.
                Err(error) => tracing::debug!(%error, "full text"),
            }
        }
        if self.attachments(&reference).await?.is_empty() {
            return Err(EngineError::Pdf(format!(
                "`{}` has no PDF and no full text online; `quire pdf get {}` looks for a PDF",
                reference.cite_key, reference.cite_key
            )));
        }
        let (reference, markdown, _) = self.paper_text(&reference.id.to_string()).await?;
        Ok((reference, PaperText { markdown, source: TextSource::Pdf }))
    }

    /// Removes the stored texts of the reference `query` names (its full text or "none" marker,
    /// and each PDF's converted Markdown), so [`Engine::reader_text`] fetches or converts again.
    /// Returns how many files were removed.
    pub async fn forget_text(&self, query: &str) -> Result<usize> {
        let (reference, _) = self.show(query).await?;
        let mut paths = vec![self.text_path(&reference)];
        let store = self.files();
        for attachment in self.attachments(&reference).await? {
            paths.push(store.path(&attachment.hash, "md"));
        }
        self.index_paper_text(reference.id, None).await?;
        let mut removed = 0;
        for path in paths {
            match std::fs::remove_file(&path) {
                Ok(()) => removed += 1,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(EngineError::File { path, source: error }),
            }
        }
        Ok(removed)
    }

    /// Indexes the texts already stored (full texts and converted PDFs) for search, without
    /// fetching or converting anything; returns how many were indexed. For texts stored before
    /// the index existed.
    pub async fn index_stored_texts(&self) -> Result<usize> {
        let mut indexed = 0;
        for reference in self.list(None, false, false).await? {
            let stored = match read_stored(&self.text_path(&reference)) {
                Some(Stored::Text(text)) => Some(text.markdown),
                _ => self.stored_pdf_text(&reference).await?,
            };
            if let Some(text) = stored {
                self.index_paper_text(reference.id, Some(text)).await?;
                indexed += 1;
            }
        }
        Ok(indexed)
    }

    /// The converted text of the reference's first PDF, if stored (any converter version).
    async fn stored_pdf_text(&self, reference: &Reference) -> Result<Option<String>> {
        let store = self.files();
        for attachment in self.attachments(reference).await? {
            if let Ok(text) = std::fs::read_to_string(store.path(&attachment.hash, "md"))
                && let Some((header, body)) = text.split_once('\n')
                && header.starts_with("<!-- quire reader text:")
            {
                return Ok(Some(body.to_string()));
            }
        }
        Ok(None)
    }

    fn text_path(&self, reference: &Reference) -> PathBuf {
        self.library_dir().join("text").join(format!("{}.md", reference.id))
    }

    /// The file of a figure image of a full text (`![Figure](url)` in its Markdown), downloaded
    /// once into the library folder (`text/figures/<hash of the URL>.<ext>`).
    pub async fn figure(&self, url: &str) -> Result<PathBuf> {
        let extension = url
            .rsplit('/')
            .next()
            .and_then(|name| name.rsplit_once('.'))
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .filter(|ext| matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp"))
            .unwrap_or_else(|| "img".into());
        let dir = self.library_dir().join("text").join("figures");
        let path = dir.join(format!("{}.{extension}", crate::files::FileStore::hash(url.as_bytes())));
        if path.is_file() {
            return Ok(path);
        }
        let bytes = self.fetcher.figure(url).await?;
        std::fs::create_dir_all(&dir).map_err(EngineError::file(&dir))?;
        private::dir(&dir)?;
        let temporary = path.with_extension("partial");
        std::fs::write(&temporary, bytes).map_err(EngineError::file(&temporary))?;
        std::fs::rename(&temporary, &path).map_err(EngineError::file(&path))?;
        Ok(path)
    }
}

/// Whether arXiv or Europe PMC could have the paper (an arXiv ID, DOI, PMID or PMCID).
pub fn online_candidate(reference: &Reference) -> bool {
    reference
        .data
        .identifiers
        .iter()
        .any(|id| matches!(id, Identifier::Arxiv(_) | Identifier::Doi(_) | Identifier::Pmid(_) | Identifier::Pmcid(_)))
}

enum Stored {
    Text(PaperText),
    /// Asked recently; neither source had it.
    NoneFresh,
}

fn header(source: &str) -> String {
    format!("<!-- quire reader text: {source} {FORMAT} -->\n")
}

fn read_stored(path: &std::path::Path) -> Option<Stored> {
    let text = std::fs::read_to_string(path).ok()?;
    for source in [FullTextSource::ArxivHtml, FullTextSource::EuropePmc] {
        if let Some(body) = text.strip_prefix(&header(source.as_str())) {
            let source = match source {
                FullTextSource::ArxivHtml => TextSource::ArxivHtml,
                FullTextSource::EuropePmc => TextSource::EuropePmc,
            };
            return Some(Stored::Text(PaperText { markdown: body.to_string(), source }));
        }
    }
    let age = std::fs::metadata(path).ok()?.modified().ok().and_then(|m| SystemTime::now().duration_since(m).ok());
    (text.starts_with(&header(NONE)) && age.is_some_and(|a| a < RETRY_AFTER)).then_some(Stored::NoneFresh)
}

/// Stores `full` (or the "none" marker) at `path`, private; returns the stored text.
fn store(path: &std::path::Path, full: Option<&FullText>) -> Result<Option<PaperText>> {
    let folder = path.parent().expect("text/<file>");
    private::dir(folder)?;
    let (content, text) = match full {
        Some(full) => {
            let source = match full.source {
                FullTextSource::ArxivHtml => TextSource::ArxivHtml,
                FullTextSource::EuropePmc => TextSource::EuropePmc,
            };
            let text = PaperText { markdown: full.markdown.clone(), source };
            (format!("{}{}", header(full.source.as_str()), full.markdown), Some(text))
        }
        None => (header(NONE), None),
    };
    let temporary = path.with_extension("md.partial");
    std::fs::write(&temporary, content).map_err(EngineError::file(&temporary))?;
    private::file(&temporary)?;
    std::fs::rename(&temporary, path).map_err(EngineError::file(path))?;
    Ok(text)
}
