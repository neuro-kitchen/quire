//! The HTTP client shared by all sources: an identifying User-Agent (Crossref and NCBI
//! etiquette), timeouts, a minimum interval between requests per host
//! (`.knowledge/metadata-sources.md`), one retry after HTTP 429 (or 503 with `Retry-After`, as
//! the arXiv API sends when busy), and recognising challenge pages.

use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::{ACCEPT, RETRY_AFTER};
use reqwest::{Client, Response, StatusCode};
use tokio::sync::Mutex;
use tokio::time::Instant;
use url::Url;

use crate::challenge;
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

/// Largest PDF downloaded.
const MAX_PDF_BYTES: u64 = 200_000_000;

/// Longest `Retry-After` honoured with one retry; longer means [`FetchError::RateLimited`].
const MAX_RETRY_WAIT: Duration = Duration::from_secs(20);

/// Wait before retrying a 429 that gave no `Retry-After` (arXiv sends none).
const DEFAULT_RETRY_WAIT: Duration = Duration::from_secs(5);

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

    /// GET `url` as a page: the body and the final URL after redirects (a proxy redirects to the
    /// publisher's proxied host, or to its login).
    pub async fn get_page(&self, url: &Url) -> Result<(String, Url)> {
        let response = self.get(url, "text/html,application/xhtml+xml").await?;
        let status = response.status();
        let final_url = response.url().clone();
        let body = response.text().await.map_err(network(url))?;
        match status {
            status if status.is_success() => Ok((body, final_url)),
            StatusCode::NOT_FOUND => Err(FetchError::NotFound(shown(url))),
            StatusCode::FORBIDDEN => Err(FetchError::Blocked { url: shown(url) }),
            status => Err(FetchError::Status { url: shown(url), status: status.as_u16() }),
        }
    }

    /// GET `url` with `Accept: accept`; the body as text. 404 is [`FetchError::NotFound`], a
    /// challenge page [`FetchError::Blocked`].
    pub async fn get_text(&self, url: &Url, accept: &str) -> Result<String> {
        let response = self.get(url, accept).await?;
        let cf_mitigated = response.headers().get("cf-mitigated").and_then(|v| v.to_str().ok()).map(str::to_string);
        let status = response.status();
        let body = response.text().await.map_err(network(url))?;
        let blocked = || FetchError::Blocked { url: shown(url) };
        match status {
            _ if challenge::is_challenge(cf_mitigated.as_deref(), &body, status.is_success())
                && status != StatusCode::NOT_FOUND =>
            {
                Err(blocked())
            }
            status if status.is_success() => Ok(body),
            StatusCode::NOT_FOUND if cf_mitigated.is_some() => Err(blocked()),
            StatusCode::NOT_FOUND => Err(FetchError::NotFound(shown(url))),
            status => Err(FetchError::Status { url: shown(url), status: status.as_u16() }),
        }
    }

    /// GET `url` expecting a PDF: the bytes and the final URL (after redirects). A challenge page
    /// is [`FetchError::Blocked`], any other non-PDF body [`FetchError::NotPdf`].
    pub async fn get_pdf(&self, url: &Url) -> Result<(Vec<u8>, Url)> {
        let response = self.get(url, "application/pdf,*/*;q=0.8").await?;
        let status = response.status();
        let final_url = response.url().clone();
        let cf_mitigated = response.headers().get("cf-mitigated").and_then(|v| v.to_str().ok()).map(str::to_string);
        if response.content_length().is_some_and(|n| n > MAX_PDF_BYTES) {
            return Err(FetchError::NotPdf(format!("{url} (larger than {} MB)", MAX_PDF_BYTES / 1_000_000)));
        }
        let bytes = response.bytes().await.map_err(network(url))?.to_vec();
        if bytes.trim_ascii_start().starts_with(b"%PDF-") && status.is_success() {
            return Ok((bytes, final_url));
        }
        let text = String::from_utf8_lossy(&bytes[..bytes.len().min(64 * 1024)]);
        match status {
            _ if challenge::is_challenge(cf_mitigated.as_deref(), &text, status.is_success()) => {
                Err(FetchError::Blocked { url: shown(url) })
            }
            StatusCode::NOT_FOUND => Err(FetchError::NotFound(shown(url))),
            // A PDF link answering "forbidden" is a site refusing scripts (MDPI, preprints.org);
            // the user's browser gets it.
            StatusCode::FORBIDDEN => Err(FetchError::Blocked { url: shown(url) }),
            status if status.is_success() => Err(FetchError::NotPdf(shown(url))),
            status => Err(FetchError::Status { url: shown(url), status: status.as_u16() }),
        }
    }

    /// Sends the request in this host's turn. After a 429, or a 503 with `Retry-After`, waits as
    /// asked (bounded) and retries once.
    async fn get(&self, url: &Url, accept: &str) -> Result<Response> {
        let mut retried = false;
        loop {
            self.wait_turn(url).await;
            tracing::debug!(url = %shown(url), accept, retried, "GET");
            let response = self.client.get(url.clone()).header(ACCEPT, accept).send().await.map_err(network(url))?;
            let status = response.status();
            let wait = match (status, retry_after(&response)) {
                (StatusCode::TOO_MANY_REQUESTS, wait) => wait.unwrap_or(DEFAULT_RETRY_WAIT),
                (StatusCode::SERVICE_UNAVAILABLE, Some(wait)) => wait.max(Duration::from_secs(1)),
                _ => return Ok(response),
            };
            if retried || wait > MAX_RETRY_WAIT {
                return match status {
                    StatusCode::TOO_MANY_REQUESTS => Err(FetchError::RateLimited { url: shown(url) }),
                    _ => Ok(response),
                };
            }
            tracing::debug!(url = %shown(url), ?wait, %status, "busy; retrying once");
            tokio::time::sleep(wait).await;
            retried = true;
        }
    }

    /// Reserves this host's next free slot (its interval after the previous one) and sleeps until
    /// then. The lock is released before sleeping, so other hosts are not held up.
    async fn wait_turn(&self, url: &Url) {
        let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
        let slot = {
            let mut last = self.last_request.lock().await;
            let now = Instant::now();
            let gap = interval(&host);
            let slot = last.get(&host).map_or(now, |previous| (*previous + gap).max(now));
            last.insert(host, slot);
            slot
        };
        tokio::time::sleep_until(slot).await;
    }
}

/// Query parameters whose values are personal (Unpaywall's `email`).
const PRIVATE_PARAMS: &[&str] = &["email", "mailto", "api_key", "token"];

/// `url` for messages, logs and stored issues: private parameter values replaced by `…`, so the
/// user's email never ends up in an error, a log or the library database.
pub fn shown(url: &Url) -> String {
    if !url.query_pairs().any(|(name, _)| PRIVATE_PARAMS.contains(&name.as_ref())) {
        return url.to_string();
    }
    let mut copy = url.clone();
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(name, value)| {
            let value = if PRIVATE_PARAMS.contains(&name.as_ref()) { "…".to_string() } else { value.into_owned() };
            (name.into_owned(), value)
        })
        .collect();
    copy.query_pairs_mut().clear().extend_pairs(pairs);
    copy.to_string()
}

/// A network error without its URL when the URL holds private parameters (reqwest prints it).
fn network(url: &Url) -> impl Fn(reqwest::Error) -> FetchError + '_ {
    move |error| {
        let private = url.query_pairs().any(|(name, _)| PRIVATE_PARAMS.contains(&name.as_ref()));
        FetchError::Http(if private { error.without_url() } else { error })
    }
}

/// `Retry-After` in seconds (the HTTP-date form is not used by the services we call).
fn retry_after(response: &Response) -> Option<Duration> {
    let seconds = response.headers().get(RETRY_AFTER)?.to_str().ok()?.trim().parse().ok()?;
    Some(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_parameters_are_hidden() {
        let url = Url::parse("https://api.unpaywall.org/v2/10.1/x?email=me%40example.org&x=1").unwrap();
        let text = shown(&url);
        assert!(!text.contains("example.org") && text.contains("x=1"), "{text}");
        let plain = Url::parse("https://doi.org/10.1/x").unwrap();
        assert_eq!(shown(&plain), plain.to_string());
    }

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
