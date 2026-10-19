//! Abstracts from Europe PMC's REST search (`https://www.ebi.ac.uk/europepmc/webservices/rest/`,
//! free, no key): by DOI or PMID it returns the PubMed record of a paper, including the abstract
//! that many publishers do not send to Crossref (2026-10-19: Nature and PLOS papers had one there,
//! none at doi.org).

use quire_model::Identifier;
use serde_json::Value;
use url::Url;

use crate::error::{FetchError, Result};
use crate::fetcher::Fetcher;

impl Fetcher {
    /// The abstract Europe PMC has for the work with `identifiers` (DOI first, then PMID), if any.
    pub async fn abstract_for(&self, identifiers: &[Identifier]) -> Result<Option<String>> {
        let query = identifiers.iter().find_map(|id| match id {
            Identifier::Doi(doi) => Some(format!("DOI:\"{}\"", doi.as_str())),
            _ => None,
        });
        let query = query.or_else(|| {
            identifiers.iter().find_map(|id| match id {
                Identifier::Pmid(pmid) => Some(format!("EXT_ID:{} AND SRC:MED", pmid.get())),
                Identifier::Pmcid(pmcid) => Some(format!("PMCID:PMC{}", pmcid.get())),
                _ => None,
            })
        });
        let Some(query) = query else { return Ok(None) };
        let mut url: Url =
            self.endpoints.europepmc.join("search").map_err(|_| FetchError::InvalidUrl("europepmc".into()))?;
        url.query_pairs_mut()
            .append_pair("query", &query)
            .append_pair("resultType", "core")
            .append_pair("format", "json")
            .append_pair("pageSize", "1");
        let body = self.http.get_text(&url, "application/json").await?;
        let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse("europepmc", e))?;
        Ok(abstract_in(&value))
    }
}

/// The first result's `abstractText`, without markup, if it has one.
pub fn abstract_in(response: &Value) -> Option<String> {
    let text = response.pointer("/resultList/result/0/abstractText")?.as_str()?;
    let plain = strip_tags(text);
    Some(plain).filter(|t| t.chars().count() >= 40)
}

/// `<h4>Background</h4>Text<sup>2</sup>` → `Background Text2`: Europe PMC keeps PubMed's markup
/// (section headings, sub/superscripts).
fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markup() {
        assert_eq!(strip_tags("<h4>Background</h4>Temperature<sup>2</sup>  rises."), "Background Temperature 2 rises.");
        assert_eq!(abstract_in(&serde_json::json!({"resultList": {"result": []}})), None);
    }
}
