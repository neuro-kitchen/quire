//! DOI metadata by content negotiation: `GET https://doi.org/<doi>` with
//! `Accept: application/vnd.citationstyles.csl+json`. doi.org redirects to the registration
//! agency (Crossref, DataCite — which covers arXiv —, mEDRA), which answers CSL-JSON.

use quire_core::RawRecord;
use quire_model::{Doi, Timestamp};
use url::Url;

use crate::Fetched;
use crate::error::{FetchError, Result};
use crate::http::Http;

const CSL_JSON: &str = "application/vnd.citationstyles.csl+json";

/// Fetches the CSL-JSON record of `doi` from the resolver at `base` (normally `https://doi.org/`).
pub async fn fetch(http: &Http, base: &Url, doi: &Doi) -> Result<Fetched> {
    let url = base.join(&encode_path(doi.as_str())).map_err(|_| FetchError::InvalidUrl(doi.to_string()))?;
    let body = http.get_text(&url, CSL_JSON).await.map_err(|e| match e {
        FetchError::NotFound(_) => FetchError::NotFound(format!("DOI {doi}")),
        other => other,
    })?;
    let mut items = quire_io::from_csl_json(&body).map_err(|e| FetchError::parse(&url, e))?;
    let data = items.pop().ok_or_else(|| FetchError::NoMetadata(url.to_string()))?;
    let raw = RawRecord { source: "doi.org".into(), format: "csl-json".into(), body, received_at: Timestamp::now() };
    Ok(Fetched { data, raw, issues: Vec::new() })
}

/// Percent-encodes what may not appear raw in a URL path (DOIs allow almost any character,
/// e.g. `<`, `#`, `?`); `/` stays, as doi.org expects `prefix/suffix`.
pub(crate) fn encode_path(doi: &str) -> String {
    let mut out = String::with_capacity(doi.len());
    for byte in doi.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' | b';' | b'(' | b')' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_unsafe_characters() {
        assert_eq!(encode_path("10.1038/nature12373"), "10.1038/nature12373");
        assert_eq!(encode_path("10.1002/(SICI)1097-4636"), "10.1002/(SICI)1097-4636");
        assert_eq!(encode_path("10.1000/a<b>#c?d e"), "10.1000/a%3Cb%3E%23c%3Fd%20e");
    }
}
