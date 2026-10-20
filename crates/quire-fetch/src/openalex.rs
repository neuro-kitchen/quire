//! OpenAlex (`https://api.openalex.org/`): the citation graph. A work's references and related
//! works come with the work (one lookup, free); their titles and DOIs, and the works citing it,
//! are list calls (`filter=openalex:W…|W…`, up to 100 IDs; `filter=cites:W…`), $0.10 per 1,000
//! calls against a daily budget. No key is needed (10 requests/s); with a free key the budget is
//! 10× larger (help.openalex.org, "Authentication", 2026-10-15).

use quire_model::Identifier;
use serde_json::Value;
use url::Url;

use crate::Fetcher;
use crate::error::{FetchError, Result};

/// IDs per `openalex:` filter (OpenAlex's limit for OR-ed values).
const PER_FILTER: usize = 100;
/// The fields asked for in lists.
const LIST_FIELDS: &str = "id,doi,display_name,publication_year,authorships,cited_by_count";

/// A work as OpenAlex links it: its ID, and the IDs it cites and that resemble it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkLinks {
    /// `W2159974629`.
    pub id: String,
    pub references: Vec<String>,
    pub related: Vec<String>,
    pub cited_by: u64,
}

/// A work in a list: enough to show it and to add it by DOI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphWork {
    pub id: String,
    /// Bare DOI (`10.1021/nl3042917`).
    pub doi: Option<String>,
    pub title: Option<String>,
    pub year: Option<i64>,
    pub first_author: Option<String>,
    pub cited_by: u64,
}

impl Fetcher {
    /// The work with one of `identifiers` (DOI first, then PMID, PMCID, arXiv ID).
    pub async fn openalex_work(&self, identifiers: &[Identifier]) -> Result<WorkLinks> {
        let key = lookup_key(identifiers).ok_or_else(|| FetchError::NotFound("no DOI or PMID for OpenAlex".into()))?;
        let url = self
            .openalex_url(&format!("works/{key}"), &[("select", "id,referenced_works,related_works,cited_by_count")])?;
        let value = self.openalex_get(&url).await?;
        Ok(links(&value))
    }

    /// Titles and DOIs of the works with OpenAlex `ids`, in that order (unknown IDs left out).
    pub async fn openalex_works(&self, ids: &[String]) -> Result<Vec<GraphWork>> {
        let mut found = Vec::with_capacity(ids.len());
        for chunk in ids.chunks(PER_FILTER) {
            let filter = format!("openalex:{}", chunk.join("|"));
            let per_page = chunk.len().to_string();
            let url =
                self.openalex_url("works", &[("filter", &filter), ("select", LIST_FIELDS), ("per_page", &per_page)])?;
            found.extend(list(&self.openalex_get(&url).await?));
        }
        found.sort_by_key(|w| ids.iter().position(|id| *id == w.id));
        Ok(found)
    }

    /// Up to `limit` (at most 200) works citing the OpenAlex work `id`, most cited first.
    pub async fn openalex_citing(&self, id: &str, limit: usize) -> Result<Vec<GraphWork>> {
        let filter = format!("cites:{id}");
        let per_page = limit.clamp(1, 200).to_string();
        let url = self.openalex_url(
            "works",
            &[("filter", &filter), ("select", LIST_FIELDS), ("per_page", &per_page), ("sort", "cited_by_count:desc")],
        )?;
        Ok(list(&self.openalex_get(&url).await?))
    }

    fn openalex_url(&self, path: &str, query: &[(&str, &str)]) -> Result<Url> {
        let mut url = self.endpoints.openalex.join(path).map_err(|_| FetchError::InvalidUrl(path.into()))?;
        // No `mailto`: OpenAlex's terms no longer ask for it, and the contact email goes only to
        // services that do (`.knowledge/engineering.md`, security of the user's data).
        url.query_pairs_mut().extend_pairs(query);
        Ok(url)
    }

    async fn openalex_get(&self, url: &Url) -> Result<Value> {
        let body = self.http.get_text(url, "application/json").await?;
        serde_json::from_str(&body).map_err(|e| FetchError::parse(url, e))
    }
}

/// OpenAlex's path key for a work: `doi:…`, `pmid:…`, `pmcid:…` (arXiv IDs by their DOI).
fn lookup_key(identifiers: &[Identifier]) -> Option<String> {
    let doi = identifiers.iter().find_map(|id| match id {
        Identifier::Doi(doi) => Some(format!("doi:{}", doi.as_str())),
        Identifier::Arxiv(arxiv) => Some(format!("doi:10.48550/arXiv.{arxiv}")),
        _ => None,
    });
    doi.or_else(|| {
        identifiers.iter().find_map(|id| match id {
            Identifier::Pmid(pmid) => Some(format!("pmid:{pmid}")),
            Identifier::Pmcid(pmcid) => Some(format!("pmcid:{pmcid}")),
            _ => None,
        })
    })
}

/// `https://openalex.org/W123` → `W123`.
fn short_id(value: Option<&Value>) -> Option<String> {
    value?.as_str()?.rsplit('/').next().map(str::to_string)
}

fn ids(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |list| list.iter().filter_map(|v| short_id(Some(v))).collect())
}

fn links(value: &Value) -> WorkLinks {
    WorkLinks {
        id: short_id(value.get("id")).unwrap_or_default(),
        references: ids(value, "referenced_works"),
        related: ids(value, "related_works"),
        cited_by: value.get("cited_by_count").and_then(Value::as_u64).unwrap_or(0),
    }
}

fn list(value: &Value) -> Vec<GraphWork> {
    let results = value.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
    results
        .iter()
        .filter_map(|work| {
            let text = |key: &str| work.get(key).and_then(Value::as_str).map(str::to_string);
            Some(GraphWork {
                id: short_id(work.get("id"))?,
                doi: text("doi").map(|d| d.trim_start_matches("https://doi.org/").to_string()),
                title: text("display_name"),
                year: work.get("publication_year").and_then(Value::as_i64),
                first_author: work
                    .pointer("/authorships/0/author/display_name")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                cited_by: work.get("cited_by_count").and_then(Value::as_u64).unwrap_or(0),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Value {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn work_links_and_lists() {
        let work = links(&fixture("openalex-work-nature12373.json"));
        assert_eq!(work.id, "W2159974629");
        assert_eq!(work.references.len(), 35);
        assert_eq!(work.related.len(), 10);
        assert!(work.cited_by > 1000);

        let works = list(&fixture("openalex-works-filter.json"));
        assert_eq!(works.len(), 3);
        let nano = works.iter().find(|w| w.id == "W1559872368").unwrap();
        assert_eq!(nano.doi.as_deref(), Some("10.1021/nl3042917"));
        assert_eq!(nano.year, Some(2012));
        assert_eq!(nano.first_author.as_deref(), Some("Alex K. Shalek"));
        assert_eq!(list(&fixture("openalex-citing-nature12373.json")).len(), 3);
    }

    #[test]
    fn lookup_keys() {
        let doi = Identifier::parse("10.1038/nature12373").unwrap();
        let arxiv = Identifier::parse("arXiv:1706.03762").unwrap();
        let pmid = Identifier::parse("pmid:23903748").unwrap();
        assert_eq!(lookup_key(&[pmid.clone(), doi]).as_deref(), Some("doi:10.1038/nature12373"));
        assert_eq!(lookup_key(&[arxiv]).as_deref(), Some("doi:10.48550/arXiv.1706.03762"));
        assert_eq!(lookup_key(&[pmid]).as_deref(), Some("pmid:23903748"));
        assert_eq!(lookup_key(&[]), None);
    }
}
