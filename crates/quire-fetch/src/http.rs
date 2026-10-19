//! The HTTP client shared by all sources: an identifying User-Agent (Crossref and NCBI
//! etiquette), timeouts, and a minimum interval between requests per host
//! (`.knowledge/metadata-sources.md`).

use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::ACCEPT;
use reqwest::{Client, StatusCode};
use tokio::sync::Mutex;
use tokio::time::Instant;
use url::Url;

use crate::error::{FetchError, Result};

/// Minimum time between requests to one host. Crossref polite pool: 10/s; NCBI: 3/s; arXiv:
/// one every 3 s; anyone else (publisher pages): 1/s.
fn interval(host: &str) -> Duration {
    match host {
        "doi.org" | "api.crossref.org" | "data.crosscite.org" => Duration::from_millis(100),
        h if h.ends_with("ncbi.nlm.nih.gov") => Duration::from_millis(334),
        "export.arxiv.org" => Duration::from_secs(3),
        _ => Duration::from_secs(1),
    }
}

const TIMEOUT: Duration = Duration::from_secs(30);

/// A rate-limited HTTP client.
pub struct Http {
    client: Client,
    last_request: Mutex<HashMap<String, Instant>>,
}

impl Http {
    /// `email` goes into the User-Agent as `mailto:` (Crossref's polite pool) when given.
    pub fn new(email: Option<&str>) -> Result<Self> {
        let contact = email.map(|e| format!(" (mailto:{e})")).unwrap_or_default();
        let user_agent = format!("quire/{}{contact}", env!("CARGO_PKG_VERSION"));
        let client = Client::builder().user_agent(user_agent).timeout(TIMEOUT).build()?;
        Ok(Self { client, last_request: Mutex::new(HashMap::new()) })
    }

    /// GET `url` with `Accept: accept`; the body as text. 404 is [`FetchError::NotFound`].
    pub async fn get_text(&self, url: &Url, accept: &str) -> Result<String> {
        self.wait_turn(url).await;
        tracing::debug!(%url, accept, "GET");
        let response = self.client.get(url.clone()).header(ACCEPT, accept).send().await?;
        match response.status() {
            status if status.is_success() => Ok(response.text().await?),
            StatusCode::NOT_FOUND => Err(FetchError::NotFound(url.to_string())),
            status => Err(FetchError::Status { url: url.to_string(), status: status.as_u16() }),
        }
    }

    /// Reserves this host's next free slot (its interval after the previous one) and sleeps until
    /// then. The lock is released before sleeping, so other hosts are not held up.
    async fn wait_turn(&self, url: &Url) {
        let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
        let slot = {
            let mut last = self.last_request.lock().await;
            let now = Instant::now();
            let slot = last.get(&host).map_or(now, |previous| (*previous + interval(&host)).max(now));
            last.insert(host, slot);
            slot
        };
        tokio::time::sleep_until(slot).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervals_follow_service_limits() {
        assert_eq!(interval("doi.org"), Duration::from_millis(100));
        assert_eq!(interval("pmc.ncbi.nlm.nih.gov"), Duration::from_millis(334));
        assert_eq!(interval("export.arxiv.org"), Duration::from_secs(3));
        assert_eq!(interval("www.nature.com"), Duration::from_secs(1));
    }

    #[tokio::test(start_paused = true)]
    async fn second_request_to_a_host_waits() {
        let http = Http::new(None).unwrap();
        let url = Url::parse("https://www.example.org/a").unwrap();
        let start = Instant::now();
        http.wait_turn(&url).await;
        http.wait_turn(&url).await;
        assert!(start.elapsed() >= Duration::from_secs(1));
    }
}
