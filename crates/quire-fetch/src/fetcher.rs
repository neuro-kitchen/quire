//! [`Fetcher`]: one entry point that picks the sources for what the user typed and falls back
//! when one fails (`.knowledge/roadmap.md`, M4). Each fallback used is recorded on the result as
//! a `fallback` issue, so `quire ref check` shows which references came from a lesser source.
//!
//! - DOI → content negotiation at doi.org → Crossref REST → the DOI's landing page. A 404 from
//!   doi.org ends the chain: the DOI does not exist.
//! - arXiv ID → its DataCite DOI (`10.48550/arXiv.<id>`) → the arXiv API (also after a 404: new
//!   papers take a while to be registered).
//! - PMID / PMCID → NCBI's exporter; if the record has a DOI, the DOI chain is used instead
//!   (richer), keeping the PubMed IDs, with the PubMed record as the last fallback.
//! - URL → the page's tags and JSON-LD; if they name an identifier, that identifier's chain,
//!   with the page's own metadata as the last fallback.

use quire_core::RawRecord;
use quire_model::{ArxivId, Doi, Identifier, Issue, IssueCode, ReferenceData};
use url::Url;

use crate::error::{FetchError, Result};
use crate::http::Http;
use crate::pubmed::PubmedId;
use crate::web::PageResult;
use crate::{arxiv, crossref, doi, pubmed, web};

/// What to look up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup {
    Identifier(Identifier),
    Url(Url),
}

impl Lookup {
    /// An identifier in any pasted form (including doi.org, arxiv.org and PubMed URLs), else an
    /// `http(s)` URL.
    pub fn parse(input: &str) -> Result<Self> {
        if let Ok(identifier) = Identifier::parse(input) {
            return Ok(Self::Identifier(identifier));
        }
        match Url::parse(input.trim()) {
            Ok(url) if matches!(url.scheme(), "http" | "https") => Ok(Self::Url(url)),
            _ => Err(FetchError::Unrecognised(input.trim().into())),
        }
    }
}

/// A fetched reference, the record it was read from, and what went wrong on the way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub data: ReferenceData,
    pub raw: RawRecord,
    /// `fallback` issues: sources that failed before this one answered.
    pub issues: Vec<Issue>,
}

/// Service base URLs (replaced by a local mock server in tests).
#[derive(Debug, Clone)]
pub struct Endpoints {
    /// DOI resolver, `https://doi.org/`.
    pub doi: Url,
    /// NCBI Literature Citation Exporter, `https://pmc.ncbi.nlm.nih.gov/api/ctxp/v1/`.
    pub ncbi: Url,
    /// Crossref REST API, `https://api.crossref.org/`.
    pub crossref: Url,
    /// arXiv API, `https://export.arxiv.org/api/`.
    pub arxiv: Url,
    /// arXiv PDFs, `https://export.arxiv.org/pdf/`.
    pub arxiv_pdf: Url,
    /// arXiv's HTML papers, `https://arxiv.org/html/`.
    pub arxiv_html: Url,
    /// Unpaywall API, `https://api.unpaywall.org/v2/`.
    pub unpaywall: Url,
    /// Europe PMC REST API, `https://www.ebi.ac.uk/europepmc/webservices/rest/`.
    pub europepmc: Url,
    /// OpenAlex API, `https://api.openalex.org/`.
    pub openalex: Url,
    /// NCBI E-utilities, `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/`.
    pub eutils: Url,
}

impl Endpoints {
    /// Every service at `base` (tests).
    pub fn all_at(base: &Url) -> Self {
        let at = |path: &str| base.join(path).expect("relative path");
        Self {
            doi: base.clone(),
            ncbi: base.clone(),
            crossref: base.clone(),
            arxiv: base.clone(),
            arxiv_pdf: at("pdf/"),
            arxiv_html: at("html/"),
            unpaywall: at("unpaywall/"),
            europepmc: at("europepmc/"),
            openalex: at("openalex/"),
            eutils: at("eutils/"),
        }
    }
}

impl Default for Endpoints {
    fn default() -> Self {
        let url = |text| Url::parse(text).expect("valid URL");
        Self {
            doi: url("https://doi.org/"),
            ncbi: url("https://pmc.ncbi.nlm.nih.gov/api/ctxp/v1/"),
            crossref: url("https://api.crossref.org/"),
            arxiv: url("https://export.arxiv.org/api/"),
            arxiv_pdf: url("https://export.arxiv.org/pdf/"),
            arxiv_html: url("https://arxiv.org/html/"),
            unpaywall: url("https://api.unpaywall.org/v2/"),
            europepmc: url("https://www.ebi.ac.uk/europepmc/webservices/rest/"),
            openalex: url("https://api.openalex.org/"),
            eutils: url("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/"),
        }
    }
}

/// Failures of the sources tried so far in one chain.
#[derive(Default)]
struct Attempts(Vec<(String, FetchError)>);

impl Attempts {
    /// `result` if it succeeded (with a `fallback` issue naming the earlier failures), else
    /// records its error under `source`.
    fn take(&mut self, source: &str, result: Result<Fetched>) -> Option<Fetched> {
        match result {
            Ok(mut fetched) => {
                if !self.0.is_empty() {
                    let failed: Vec<String> = self.0.iter().map(|(s, e)| format!("{s}: {e}")).collect();
                    let detail = format!("{}; used {source}", failed.join("; "));
                    tracing::warn!(%detail, "metadata from a fallback source");
                    fetched.issues.push(Issue::recorded(IssueCode::Fallback, detail));
                }
                Some(fetched)
            }
            Err(error) => {
                tracing::debug!(source, %error, "source failed");
                self.0.push((source.into(), error));
                None
            }
        }
    }

    /// The error when every source failed: the only one, or all of them.
    fn error(mut self) -> FetchError {
        match self.0.len() {
            1 => self.0.remove(0).1,
            _ => FetchError::AllFailed(self.0),
        }
    }
}

pub struct Fetcher {
    pub(crate) http: Http,
    pub(crate) endpoints: Endpoints,
    /// The contact email, for Unpaywall's `email` parameter.
    pub(crate) email: Option<String>,
}

impl Fetcher {
    /// `email` identifies the user to Crossref and NCBI (sent in the User-Agent).
    pub fn new(email: Option<&str>) -> Result<Self> {
        Self::with_endpoints(email, Endpoints::default())
    }

    pub fn with_endpoints(email: Option<&str>, endpoints: Endpoints) -> Result<Self> {
        Ok(Self { http: Http::new(email)?, endpoints, email: email.map(str::to_string) })
    }

    pub async fn fetch(&self, lookup: &Lookup) -> Result<Fetched> {
        match lookup {
            Lookup::Identifier(identifier) => self.identifier(identifier).await,
            Lookup::Url(url) => self.page(url).await,
        }
    }

    async fn identifier(&self, identifier: &Identifier) -> Result<Fetched> {
        match identifier {
            Identifier::Doi(d) => self.doi(d).await,
            Identifier::Arxiv(a) => self.arxiv(a).await,
            Identifier::Pmid(p) => self.pubmed(PubmedId::Pmid(p.get())).await,
            Identifier::Pmcid(p) => self.pubmed(PubmedId::Pmcid(p.get())).await,
        }
    }

    async fn doi(&self, d: &Doi) -> Result<Fetched> {
        let mut attempts = Attempts::default();
        let first = doi::fetch(&self.http, &self.endpoints.doi, d).await;
        if let Err(error) = &first
            && error.is_final()
        {
            return first;
        }
        if let Some(found) = attempts.take("doi.org", first) {
            return Ok(found);
        }
        if let Some(found) =
            attempts.take("api.crossref.org", crossref::fetch(&self.http, &self.endpoints.crossref, d).await)
        {
            return Ok(found);
        }
        let landing = self
            .endpoints
            .doi
            .join(&doi::encode_path(d.as_str()))
            .map_err(|_| FetchError::InvalidUrl(d.to_string()))?;
        let page = web::fetch(&self.http, &landing).await.map(|result| match result {
            PageResult::Identifier { fallback, .. } | PageResult::Page(fallback) => {
                with_identifier(fallback, Identifier::Doi(d.clone()))
            }
        });
        attempts.take("landing page", page).ok_or_else(|| attempts.error())
    }

    async fn arxiv(&self, a: &ArxivId) -> Result<Fetched> {
        let mut attempts = Attempts::default();
        if let Some(found) =
            attempts.take("doi.org (DataCite)", doi::fetch(&self.http, &self.endpoints.doi, &a.doi()).await)
        {
            return Ok(found);
        }
        attempts
            .take("export.arxiv.org", arxiv::fetch(&self.http, &self.endpoints.arxiv, a).await)
            .ok_or_else(|| attempts.error())
    }

    async fn pubmed(&self, id: PubmedId) -> Result<Fetched> {
        let from_pubmed = pubmed::fetch(&self.http, &self.endpoints.ncbi, id).await?;
        let Some(Identifier::Doi(d)) =
            from_pubmed.data.identifiers.iter().find(|i| matches!(i, Identifier::Doi(_))).cloned()
        else {
            return Ok(from_pubmed);
        };
        match self.doi(&d).await {
            Ok(mut from_doi) => {
                add_missing_identifiers(&mut from_doi.data, &from_pubmed.data.identifiers);
                Ok(from_doi)
            }
            Err(error) => {
                let mut attempts = Attempts(vec![(format!("DOI {d}"), error)]);
                Ok(attempts.take("the PubMed record", Ok(from_pubmed)).expect("Ok is taken"))
            }
        }
    }

    async fn page(&self, url: &Url) -> Result<Fetched> {
        self.from_page(web::fetch(&self.http, url).await?).await
    }

    /// The reference a page describes: its identifier's record when it names one (the page's own
    /// metadata as fallback), else the page's metadata. For pages the user's browser already
    /// loaded (`quire serve`), read with [`web::read`].
    pub async fn from_page(&self, page: PageResult) -> Result<Fetched> {
        match page {
            PageResult::Page(fetched) => Ok(fetched),
            PageResult::Identifier { identifier, fallback } => match Box::pin(self.identifier(&identifier)).await {
                Ok(fetched) => Ok(fetched),
                Err(error) => {
                    let mut attempts = Attempts(vec![(identifier.to_string(), error)]);
                    Ok(attempts.take("the page's own tags", Ok(fallback)).expect("Ok is taken"))
                }
            },
        }
    }
}

/// `fetched` with `identifier` added if missing (a landing page may not repeat its DOI).
fn with_identifier(mut fetched: Fetched, identifier: Identifier) -> Fetched {
    add_missing_identifiers(&mut fetched.data, &[identifier]);
    fetched
}

fn add_missing_identifiers(data: &mut ReferenceData, others: &[Identifier]) {
    for id in others {
        if !data.identifiers.iter().any(|known| known.kind() == id.kind() && known.key() == id.key()) {
            data.identifiers.push(id.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_parsing() {
        assert!(matches!(Lookup::parse("10.1038/nature12373"), Ok(Lookup::Identifier(Identifier::Doi(_)))));
        assert!(matches!(
            Lookup::parse("https://arxiv.org/abs/1706.03762"),
            Ok(Lookup::Identifier(Identifier::Arxiv(_)))
        ));
        assert!(matches!(Lookup::parse("https://www.nature.com/articles/nature12373"), Ok(Lookup::Url(_))));
        assert!(Lookup::parse("ftp://example.org/x").is_err());
        assert!(Lookup::parse("not anything").is_err());
    }
}
