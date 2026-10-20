//! Crossref's REST API (`https://api.crossref.org/works/<doi>`): the fallback when DOI content
//! negotiation fails for a Crossref DOI, and the notices that update a work (retractions,
//! expressions of concern, corrections; with Retraction Watch's data since 2023). The `message`
//! object uses Crossref's own type names and field shapes, which the CSL-JSON reader already
//! accepts (titles as arrays, `journal-article`).

use quire_core::RawRecord;
use quire_model::{Doi, ReferenceData, TextField, Timestamp};
use serde_json::Value;
use url::Url;

use crate::error::{FetchError, Result};
use crate::http::Http;
use crate::{Fetched, Fetcher};

/// Fetches the work `doi` from the API at `base` (normally `https://api.crossref.org/`).
pub async fn fetch(http: &Http, base: &Url, doi: &Doi) -> Result<Fetched> {
    let (url, body, value) = work(http, base, doi).await?;
    let message = message(&url, &value)?;
    let data = quire_io::from_csl_value(message).map_err(|e| FetchError::parse(&url, e))?;
    let raw = RawRecord {
        source: "api.crossref.org".into(),
        format: "crossref-json".into(),
        body,
        received_at: Timestamp::now(),
    };
    Ok(Fetched { data, raw, issues: Vec::new() })
}

/// Up to `limit` works matching `query` registered since `since` (`YYYY-MM-DD`), best first.
/// `author:` before words searches authors (`query.author`), the rest titles, journals and other
/// bibliographic fields (`query.bibliographic`). Crossref ranks works matching any word (sorted by
/// date, the newest rows would match one word only), so results are ranked by relevance within
/// the dates, and every word must appear in the title, journal or abstract here. Only papers, preprints, chapters, books, reports and theses (no peer-review
/// reports or components).
pub async fn search(http: &Http, base: &Url, query: &str, since: &str, limit: usize) -> Result<Vec<ReferenceData>> {
    let mut url = base.join("works").map_err(|_| FetchError::InvalidUrl(base.to_string()))?;
    let (authors, words): (Vec<&str>, Vec<&str>) = query.split_whitespace().partition(|w| w.starts_with("author:"));
    let authors: Vec<&str> = authors.iter().map(|w| w.trim_start_matches("author:")).collect();
    {
        let mut pairs = url.query_pairs_mut();
        if !words.is_empty() {
            pairs.append_pair("query.bibliographic", &words.join(" "));
        }
        if !authors.is_empty() {
            pairs.append_pair("query.author", &authors.join(" "));
        }
        let types = WORK_TYPES.iter().map(|t| format!(",type:{t}")).collect::<String>();
        pairs
            .append_pair("filter", &format!("from-created-date:{since}{types}"))
            .append_pair("rows", &limit.to_string());
    }
    let body = http.get_text(&url, "application/json").await?;
    let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse(&url, e))?;
    let items = message(&url, &value)?.get("items").and_then(Value::as_array).cloned().unwrap_or_default();
    // A record the CSL reader cannot read is left out, not an error for the whole search.
    let found = items.iter().filter_map(|item| quire_io::from_csl_value(item).ok());
    Ok(found.filter(|data| has_every_word(data, &words)).collect())
}

/// Crossref's types of works an alert is for.
const WORK_TYPES: &[&str] = &[
    "journal-article",
    "posted-content",
    "proceedings-article",
    "book-chapter",
    "book",
    "monograph",
    "report",
    "dissertation",
];

/// Whether each of `words` begins a word of the title, journal or abstract (ignoring case).
fn has_every_word(data: &ReferenceData, words: &[&str]) -> bool {
    let text = [TextField::Title, TextField::ContainerTitle, TextField::Abstract]
        .iter()
        .filter_map(|f| data.get(*f))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let present: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).collect();
    words.iter().all(|word| {
        let word = word.to_lowercase();
        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
        present.iter().any(|p| p.starts_with(word))
    })
}

/// A notice that updates a work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub kind: UpdateKind,
    /// The notice's DOI.
    pub notice: Option<String>,
    /// When it was issued (`YYYY-MM-DD`).
    pub date: Option<String>,
    /// Who reported it: `publisher` or `retraction-watch`.
    pub source: Option<String>,
}

/// Crossref's update types, grouped by what they mean for citing the work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateKind {
    /// Retraction, partial retraction, withdrawal or removal: do not cite as valid.
    Retracted(String),
    /// An expression of concern.
    Concern,
    /// A correction, erratum, addendum or another update (named as Crossref names it).
    Other(String),
}

impl Fetcher {
    /// The notices Crossref knows that update the work `doi` (its `updated-by` list).
    pub async fn updates(&self, doi: &Doi) -> Result<Vec<Update>> {
        let (url, _, value) = work(&self.http, &self.endpoints.crossref, doi).await?;
        Ok(updates_in(message(&url, &value)?))
    }
}

/// The `updated-by` entries of a work's `message`.
fn updates_in(message: &Value) -> Vec<Update> {
    let Some(list) = message.get("updated-by").and_then(Value::as_array) else { return Vec::new() };
    list.iter()
        .map(|entry| {
            let text = |key: &str| entry.get(key).and_then(Value::as_str).map(str::to_string);
            let kind = text("type").unwrap_or_default().to_ascii_lowercase().replace('-', "_");
            let kind = match kind.as_str() {
                "retraction" | "partial_retraction" | "withdrawal" | "removal" => UpdateKind::Retracted(kind),
                "expression_of_concern" => UpdateKind::Concern,
                _ => UpdateKind::Other(kind),
            };
            let date = entry
                .pointer("/updated/date-time")
                .and_then(Value::as_str)
                .and_then(|t| t.get(..10))
                .map(str::to_string);
            Update { kind, notice: text("DOI"), date, source: text("source") }
        })
        .collect()
}

/// GET `works/<doi>`: the URL, the body and its JSON.
async fn work(http: &Http, base: &Url, doi: &Doi) -> Result<(Url, String, Value)> {
    let url = base
        .join(&format!("works/{}", crate::doi::encode_path(doi.as_str())))
        .map_err(|_| FetchError::InvalidUrl(doi.to_string()))?;
    let body = http.get_text(&url, "application/json").await.map_err(|e| match e {
        FetchError::NotFound(_) => FetchError::NotFound(format!("DOI {doi} at Crossref")),
        other => other,
    })?;
    let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse(&url, e))?;
    Ok((url, body, value))
}

fn message<'v>(url: &Url, value: &'v Value) -> Result<&'v Value> {
    value.get("message").ok_or_else(|| FetchError::parse(url, "no `message` object"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retraction_and_correction_notices() {
        let body = &quire_testkit::fixture("crossref-works-wakefield-retracted.json");
        let value: Value = serde_json::from_str(body).unwrap();
        let updates = updates_in(&value["message"]);
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].kind, UpdateKind::Other("correction".into()));
        assert_eq!(updates[1].kind, UpdateKind::Retracted("retraction".into()));
        assert_eq!(updates[1].notice.as_deref(), Some("10.1016/s0140-6736(10)60175-4"));
        assert_eq!(updates[1].date.as_deref(), Some("2010-02-06"));
        assert_eq!(updates[1].source.as_deref(), Some("retraction-watch"));
        let plain: Value = serde_json::from_str(&quire_testkit::fixture("crossref-works-nature12373.json")).unwrap();
        assert!(updates_in(&plain["message"]).is_empty());
    }

    #[test]
    fn search_items_are_references() {
        let body = &quire_testkit::fixture("crossref-search-nanodiamond-thermometry.json");
        let value: Value = serde_json::from_str(body).unwrap();
        let items = value["message"]["items"].as_array().unwrap();
        let found: Vec<ReferenceData> = items.iter().filter_map(|i| quire_io::from_csl_value(i).ok()).collect();
        assert_eq!(found.len(), 3);
        assert_eq!(found[1].identifiers[0].to_string(), "doi:10.1103/nzhn-34tp");
        assert!(
            found[2].title().unwrap().contains("heating in 88 Sr + Coulomb"),
            "MathML dropped: {:?}",
            found[2].title()
        );
        let kept: Vec<_> = found.iter().filter(|d| has_every_word(d, &["Thermometry", "lattice"])).collect();
        assert_eq!(kept.len(), 1);
    }
}
