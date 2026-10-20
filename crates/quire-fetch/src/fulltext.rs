//! A paper's full text with its structure, for reader mode: arXiv's HTML version for arXiv papers,
//! else Europe PMC's JATS XML for open-access papers in PubMed Central. Converted to reader
//! Markdown by [`quire_document::html`].

use quire_document::html::{arxiv, jats};
use quire_model::{ArxivId, Identifier};
use serde_json::Value;
use url::Url;

use crate::error::{FetchError, Result};
use crate::fetcher::Fetcher;

/// Where a full text came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullTextSource {
    ArxivHtml,
    EuropePmc,
}

impl FullTextSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArxivHtml => "arxiv-html",
            Self::EuropePmc => "europepmc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullText {
    pub source: FullTextSource,
    pub markdown: String,
}

/// Shortest Markdown taken as a full text (an HTML page with only an error is shorter).
const MIN_LENGTH: usize = 2000;

impl Fetcher {
    /// The structured full text of the work with `identifiers`: arXiv's HTML for an arXiv ID,
    /// else Europe PMC's for an open-access paper (by PMCID, or found by DOI or PMID). `None`
    /// when neither has it.
    pub async fn full_text(&self, identifiers: &[Identifier]) -> Result<Option<FullText>> {
        let arxiv = identifiers.iter().find_map(|id| match id {
            Identifier::Arxiv(id) => Some(id),
            _ => None,
        });
        if let Some(id) = arxiv
            && let Some(markdown) = self.arxiv_html(id).await?
        {
            return Ok(Some(FullText { source: FullTextSource::ArxivHtml, markdown }));
        }
        let Some(pmcid) = self.open_access_pmcid(identifiers).await? else { return Ok(None) };
        let url = self.endpoints.europepmc.join(&format!("{pmcid}/fullTextXML")).map_err(|_| invalid())?;
        let xml = match self.http.get_text(&url, "application/xml").await {
            Ok(xml) => xml,
            Err(FetchError::NotFound(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        let markdown = jats::to_markdown(&xml).map_err(|e| FetchError::parse("europepmc", e))?;
        Ok((markdown.len() >= MIN_LENGTH).then_some(FullText { source: FullTextSource::EuropePmc, markdown }))
    }

    /// arXiv's HTML version of `id` as reader Markdown; `None` when arXiv has none (papers whose
    /// LaTeX it could not convert).
    async fn arxiv_html(&self, id: &ArxivId) -> Result<Option<String>> {
        let url = self.endpoints.arxiv_html.join(&id.to_string()).map_err(|_| invalid())?;
        let html = match self.http.get_text(&url, "text/html").await {
            Ok(html) => html,
            Err(FetchError::NotFound(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        let markdown = arxiv::to_markdown(&html, &url);
        Ok((markdown.len() >= MIN_LENGTH).then_some(markdown))
    }

    /// A figure image of a full text (its URL as the converter wrote it).
    pub async fn figure(&self, url: &str) -> Result<Vec<u8>> {
        let url = Url::parse(url).map_err(|_| FetchError::InvalidUrl(url.into()))?;
        self.http.get_image(&url).await
    }

    /// The PMCID of the work, if it is in Europe PMC's open-access subset (whose full text it serves).
    async fn open_access_pmcid(&self, identifiers: &[Identifier]) -> Result<Option<String>> {
        let query = identifiers.iter().find_map(|id| match id {
            Identifier::Pmcid(pmcid) => Some(format!("PMCID:PMC{}", pmcid.get())),
            _ => None,
        });
        let query = query.or_else(|| {
            identifiers.iter().find_map(|id| match id {
                Identifier::Doi(doi) => Some(format!("DOI:\"{}\"", doi.as_str())),
                Identifier::Pmid(pmid) => Some(format!("EXT_ID:{} AND SRC:MED", pmid.get())),
                _ => None,
            })
        });
        let Some(query) = query else { return Ok(None) };
        let mut url: Url = self.endpoints.europepmc.join("search").map_err(|_| invalid())?;
        url.query_pairs_mut()
            .append_pair("query", &format!("{query} AND OPEN_ACCESS:y"))
            .append_pair("resultType", "lite")
            .append_pair("format", "json")
            .append_pair("pageSize", "1");
        let body = self.http.get_text(&url, "application/json").await?;
        let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse("europepmc", e))?;
        Ok(value.pointer("/resultList/result/0/pmcid").and_then(Value::as_str).map(str::to_string))
    }
}

fn invalid() -> FetchError {
    FetchError::InvalidUrl("full text".into())
}
