//! [`Fetcher`]: one entry point that picks the source for what the user typed.
//!
//! - DOI → content negotiation at doi.org.
//! - arXiv ID → its DataCite DOI (`10.48550/arXiv.<id>`), same path.
//! - PMID / PMCID → NCBI's exporter; if the record has a DOI, the DOI record is used instead
//!   (richer), keeping the PubMed IDs.
//! - URL → the page's tags; if they name an identifier, that identifier's record.

use quire_core::RawRecord;
use quire_model::{Identifier, ReferenceData};
use url::Url;

use crate::error::{FetchError, Result};
use crate::http::Http;
use crate::pubmed::PubmedId;
use crate::web::PageResult;
use crate::{doi, pubmed, web};

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
            _ => Err(FetchError::InvalidUrl(input.into())),
        }
    }
}

/// A fetched reference and the record it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub data: ReferenceData,
    pub raw: RawRecord,
}

/// Service base URLs (replaced by a local mock server in tests).
#[derive(Debug, Clone)]
pub struct Endpoints {
    /// DOI resolver, `https://doi.org/`.
    pub doi: Url,
    /// NCBI Literature Citation Exporter, `https://pmc.ncbi.nlm.nih.gov/api/ctxp/v1/`.
    pub ncbi: Url,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            doi: Url::parse("https://doi.org/").expect("valid URL"),
            ncbi: Url::parse("https://pmc.ncbi.nlm.nih.gov/api/ctxp/v1/").expect("valid URL"),
        }
    }
}

pub struct Fetcher {
    http: Http,
    endpoints: Endpoints,
}

impl Fetcher {
    /// `email` identifies the user to Crossref and NCBI (sent in the User-Agent).
    pub fn new(email: Option<&str>) -> Result<Self> {
        Self::with_endpoints(email, Endpoints::default())
    }

    pub fn with_endpoints(email: Option<&str>, endpoints: Endpoints) -> Result<Self> {
        Ok(Self { http: Http::new(email)?, endpoints })
    }

    pub async fn fetch(&self, lookup: &Lookup) -> Result<Fetched> {
        match lookup {
            Lookup::Identifier(identifier) => self.identifier(identifier).await,
            Lookup::Url(url) => self.page(url).await,
        }
    }

    async fn identifier(&self, identifier: &Identifier) -> Result<Fetched> {
        match identifier {
            Identifier::Doi(d) => doi::fetch(&self.http, &self.endpoints.doi, d).await,
            Identifier::Arxiv(a) => doi::fetch(&self.http, &self.endpoints.doi, &a.doi()).await,
            Identifier::Pmid(p) => self.pubmed(PubmedId::Pmid(p.get())).await,
            Identifier::Pmcid(p) => self.pubmed(PubmedId::Pmcid(p.get())).await,
        }
    }

    async fn pubmed(&self, id: PubmedId) -> Result<Fetched> {
        let from_pubmed = pubmed::fetch(&self.http, &self.endpoints.ncbi, id).await?;
        let Some(Identifier::Doi(d)) = from_pubmed.data.identifiers.iter().find(|i| matches!(i, Identifier::Doi(_))) else {
            return Ok(from_pubmed);
        };
        match doi::fetch(&self.http, &self.endpoints.doi, d).await {
            Ok(mut from_doi) => {
                add_missing_identifiers(&mut from_doi.data, &from_pubmed.data.identifiers);
                Ok(from_doi)
            }
            Err(error) => {
                tracing::warn!(%error, "DOI record unavailable; using the PubMed record");
                Ok(from_pubmed)
            }
        }
    }

    async fn page(&self, url: &Url) -> Result<Fetched> {
        match web::fetch(&self.http, url).await? {
            PageResult::Page(fetched) => Ok(fetched),
            PageResult::Identifier { identifier, fallback } => match self.identifier(&identifier).await {
                Ok(fetched) => Ok(fetched),
                Err(error) => {
                    tracing::warn!(%error, %identifier, "identifier from the page unavailable; using the page's tags");
                    Ok(fallback)
                }
            },
        }
    }
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
        assert!(matches!(Lookup::parse("https://arxiv.org/abs/1706.03762"), Ok(Lookup::Identifier(Identifier::Arxiv(_)))));
        assert!(matches!(Lookup::parse("https://www.nature.com/articles/nature12373"), Ok(Lookup::Url(_))));
        assert!(Lookup::parse("ftp://example.org/x").is_err());
        assert!(Lookup::parse("not anything").is_err());
    }
}
