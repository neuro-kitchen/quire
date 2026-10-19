//! PubMed and PubMed Central records from NCBI's Literature Citation Exporter
//! (`https://pmc.ncbi.nlm.nih.gov/api/ctxp/v1/{pubmed|pmc}/?format=csl&id=<number>`), which
//! answers CSL-JSON. Limit: 3 requests per second, no concurrent requests.

use quire_core::RawRecord;
use quire_model::{Identifier, Timestamp};
use url::Url;

use crate::error::{FetchError, Result};
use crate::http::Http;
use crate::Fetched;

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
    let mut items = quire_io::from_csl_json(&body).map_err(|source| FetchError::Parse { url: url.to_string(), source })?;
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
    Ok(Fetched { data, raw })
}
