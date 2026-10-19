//! Crossref's REST API (`https://api.crossref.org/works/<doi>`): the fallback when DOI content
//! negotiation fails for a Crossref DOI. The `message` object uses Crossref's own type names and
//! field shapes, which the CSL-JSON reader already accepts (titles as arrays, `journal-article`).

use quire_core::RawRecord;
use quire_model::{Doi, Timestamp};
use serde_json::Value;
use url::Url;

use crate::Fetched;
use crate::error::{FetchError, Result};
use crate::http::Http;

/// Fetches the work `doi` from the API at `base` (normally `https://api.crossref.org/`).
pub async fn fetch(http: &Http, base: &Url, doi: &Doi) -> Result<Fetched> {
    let url = base
        .join(&format!("works/{}", crate::doi::encode_path(doi.as_str())))
        .map_err(|_| FetchError::InvalidUrl(doi.to_string()))?;
    let body = http.get_text(&url, "application/json").await.map_err(|e| match e {
        FetchError::NotFound(_) => FetchError::NotFound(format!("DOI {doi} at Crossref")),
        other => other,
    })?;
    let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse(&url, e))?;
    let message = value.get("message").ok_or_else(|| FetchError::parse(&url, "no `message` object"))?;
    let data = quire_io::from_csl_value(message).map_err(|e| FetchError::parse(&url, e))?;
    let raw = RawRecord {
        source: "api.crossref.org".into(),
        format: "crossref-json".into(),
        body,
        received_at: Timestamp::now(),
    };
    Ok(Fetched { data, raw, issues: Vec::new() })
}
