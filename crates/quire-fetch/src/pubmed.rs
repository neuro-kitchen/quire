//! PubMed and PubMed Central records from NCBI's Literature Citation Exporter
//! (`https://pmc.ncbi.nlm.nih.gov/api/ctxp/v1/{pubmed|pmc}/?format=csl&id=<number>`), which
//! answers CSL-JSON; and searches for alerts with the E-utilities (`esearch` for the PMIDs added
//! since a day, `esummary` for their titles, authors and DOIs). Limit: 3 requests per second, no
//! concurrent requests.

use quire_core::RawRecord;
use quire_model::{Creator, CreatorRole, EntryKind, Identifier, ReferenceData, TextField, Timestamp};
use serde_json::Value;
use url::Url;

use crate::Fetched;
use crate::error::{FetchError, Result};
use crate::http::Http;

/// Which database and number to ask for.
#[derive(Debug, Clone, Copy)]
pub enum PubmedId {
    Pmid(u64),
    /// The PMCID's number, without `PMC`.
    Pmcid(u64),
}

/// Fetches the CSL-JSON record from the exporter at `base`; the queried ID is added to the
/// record's identifiers if the record lacks it.
pub async fn fetch(http: &Http, base: &Url, id: PubmedId) -> Result<Fetched> {
    let (database, number, label) = match id {
        PubmedId::Pmid(n) => ("pubmed", n, format!("PMID {n}")),
        PubmedId::Pmcid(n) => ("pmc", n, format!("PMCID PMC{n}")),
    };
    let mut url = base.join(&format!("{database}/")).map_err(|_| FetchError::InvalidUrl(base.to_string()))?;
    url.query_pairs_mut().append_pair("format", "csl").append_pair("id", &number.to_string());
    // The exporter answers 406 to `Accept: application/json` or the CSL media type; only `*/*`
    // works (tested 2026-10-18). The body is still JSON.
    let body = http.get_text(&url, "*/*").await.map_err(|e| match e {
        FetchError::NotFound(_) | FetchError::Status { status: 400, .. } => FetchError::NotFound(label.clone()),
        other => other,
    })?;
    let mut items = quire_io::from_csl_json(&body).map_err(|e| FetchError::parse(&url, e))?;
    let mut data = items.pop().ok_or_else(|| FetchError::NotFound(label))?;
    let queried = match id {
        PubmedId::Pmid(n) => Identifier::parse(&format!("pmid:{n}")),
        PubmedId::Pmcid(n) => Identifier::parse(&format!("PMC{n}")),
    }
    .expect("a number is a valid PMID/PMCID");
    if !data.identifiers.iter().any(|known| known.kind() == queried.kind()) {
        data.identifiers.push(queried);
    }
    let raw = RawRecord { source: "ncbi-ctxp".into(), format: "csl-json".into(), body, received_at: Timestamp::now() };
    Ok(Fetched { data, raw, issues: Vec::new() })
}
/// Up to `limit` PubMed records matching `query` (PubMed's own syntax: `smith j[au]`,
/// `thermometry[ti]`, plain words) added since `since` (`YYYY-MM-DD`), newest first. `email`
/// identifies the user to NCBI, as their E-utilities guidelines ask.
pub async fn search(
    http: &Http,
    base: &Url,
    email: Option<&str>,
    query: &str,
    since: &str,
    limit: usize,
) -> Result<Vec<ReferenceData>> {
    let mut url = base.join("esearch.fcgi").map_err(|_| FetchError::InvalidUrl(base.to_string()))?;
    url.query_pairs_mut()
        .append_pair("db", "pubmed")
        .append_pair("term", query)
        .append_pair("datetype", "edat")
        .append_pair("mindate", &since.replace('-', "/"))
        .append_pair("maxdate", "3000")
        .append_pair("retmode", "json")
        .append_pair("retmax", &limit.to_string())
        .append_pair("sort", "pub_date");
    identify(&mut url, email);
    let body = http.get_text(&url, "application/json").await?;
    let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse(&url, e))?;
    let ids: Vec<String> = value
        .pointer("/esearchresult/idlist")
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut url = base.join("esummary.fcgi").map_err(|_| FetchError::InvalidUrl(base.to_string()))?;
    url.query_pairs_mut().append_pair("db", "pubmed").append_pair("id", &ids.join(",")).append_pair("retmode", "json");
    identify(&mut url, email);
    let body = http.get_text(&url, "application/json").await?;
    let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse(&url, e))?;
    Ok(summaries(&value))
}

fn identify(url: &mut Url, email: Option<&str>) {
    url.query_pairs_mut().append_pair("tool", "quire");
    if let Some(email) = email {
        url.query_pairs_mut().append_pair("email", email);
    }
}

/// The records of an `esummary` answer, in its order.
fn summaries(value: &Value) -> Vec<ReferenceData> {
    let Some(result) = value.get("result") else { return Vec::new() };
    let uids = result.get("uids").and_then(Value::as_array).cloned().unwrap_or_default();
    uids.iter()
        .filter_map(|uid| {
            let record = result.get(uid.as_str()?)?;
            let text = |key: &str| record.get(key).and_then(Value::as_str).filter(|t| !t.is_empty());
            let mut data = ReferenceData::new(EntryKind::ArticleJournal);
            data.set(TextField::Title, text("title")?.trim_end_matches('.'));
            if let Some(journal) = text("fulljournalname") {
                data.set(TextField::ContainerTitle, journal);
            }
            for (key, field) in [("volume", TextField::Volume), ("issue", TextField::Issue), ("pages", TextField::Page)]
            {
                if let Some(value) = text(key) {
                    data.set(field, value);
                }
            }
            data.issued = text("sortpubdate").and_then(crate::web::date);
            data.creators = record
                .get("authors")
                .and_then(Value::as_array)
                .map(|authors| authors.iter().filter_map(|a| a.get("name")?.as_str()).map(author).collect())
                .unwrap_or_default();
            data.identifiers.extend(Identifier::parse(&format!("pmid:{}", uid.as_str()?)).ok());
            for id in record.get("articleids").and_then(Value::as_array).into_iter().flatten() {
                let (kind, value) = (id.get("idtype").and_then(Value::as_str), id.get("value").and_then(Value::as_str));
                if let (Some("doi" | "pmc"), Some(value)) = (kind, value)
                    && let Ok(identifier) = Identifier::parse(value)
                {
                    data.identifiers.push(identifier);
                }
            }
            Some(data)
        })
        .collect()
}

/// PubMed's `Family AB` (initials last) as a person.
fn author(name: &str) -> Creator {
    match name.rsplit_once(' ') {
        Some((family, initials)) if initials.chars().all(|c| c.is_uppercase()) => {
            Creator::person(CreatorRole::Author, family, Some(initials.to_string()))
        }
        _ => Creator::literal(CreatorRole::Author, name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_are_references() {
        let body = include_str!("../tests/fixtures/pubmed-esummary-nanodiamond-thermometry.json");
        let found = summaries(&serde_json::from_str(body).unwrap());
        assert_eq!(found.len(), 3);
        let first = &found[0];
        assert!(first.title().unwrap().starts_with("All-optical fluorescence lifetime nanodiamond thermometry"));
        assert_eq!(first.get(TextField::ContainerTitle), Some("Nanoscale"));
        let ids: Vec<String> = first.identifiers.iter().map(ToString::to_string).collect();
        assert_eq!(ids, ["pmid:42496000", "doi:10.1039/d6nr00839a"]);
        assert_eq!(first.issued.map(|d| d.get_year()), Some(2026));
        assert!(matches!(&first.creators[0].name, quire_model::Name::Person(p) if p.family == "Camarneiro"));
        assert_eq!(found[1].identifiers.len(), 3, "PMID, PMCID and DOI");
    }
}
