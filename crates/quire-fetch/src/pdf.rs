//! Finding a PDF of a reference (`.knowledge/roadmap.md`, M7), in order:
//!
//! 1. the PDF link the publisher's page gave when the reference was added (`citation_pdf_url`);
//! 2. arXiv, for preprints (`export.arxiv.org/pdf/<id>`, arXiv's host for automated access);
//! 3. Unpaywall's open-access copies (needs the contact email);
//! 4. the publisher's landing page (`doi.org/<doi>`): its `citation_pdf_url`, or the PDF path of
//!    its platform (Atypon, Wiley) when it tags none (works for open-access journals, and on a
//!    campus network for subscribed ones);
//! 5. the same through the institutional proxy, with the user's session (`proxy`).
//!
//! Each link must give a real PDF; a challenge page counts as blocked (the user can then fetch it
//! in their browser). PMC's PDF links answer a proof-of-work page since 2026, so PMC copies are
//! reached through the browser.

use quire_model::{Doi, Identifier, Issue, IssueCode, ReferenceData};
use serde_json::Value;
use url::Url;

use crate::error::{FetchError, Result};
use crate::fetcher::Fetcher;
use crate::web;

/// A PDF that was found and downloaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundPdf {
    pub bytes: Vec<u8>,
    /// Where it was downloaded from (after redirects).
    pub url: Url,
    /// Which source gave the link: `page`, `arxiv`, `unpaywall`, `landing page`.
    pub source: String,
    /// Sources that failed before.
    pub issues: Vec<Issue>,
}

impl Fetcher {
    /// Finds and downloads a PDF of the work `data` describes.
    pub async fn find_pdf(&self, data: &ReferenceData) -> Result<FoundPdf> {
        let mut failures: Vec<(String, FetchError)> = Vec::new();
        let mut tried: Vec<Url> = Vec::new();
        for (source, url) in self.direct_links(data) {
            if let Some(found) = self.try_link(&source, url, &mut tried, &mut failures).await {
                return Ok(with_failures(found, &failures));
            }
        }
        let Some(doi) = data.identifiers.iter().find_map(|id| match id {
            Identifier::Doi(doi) => Some(doi.clone()),
            _ => None,
        }) else {
            return Err(give_up(failures, "no DOI to search open-access copies by"));
        };
        match self.unpaywall_links(&doi).await {
            Ok(links) => {
                for url in links {
                    if let Some(found) = self.try_link("unpaywall", url, &mut tried, &mut failures).await {
                        return Ok(with_failures(found, &failures));
                    }
                }
            }
            Err(error) => failures.push(("unpaywall".into(), error)),
        }
        match self.landing_page_links(&doi).await {
            Ok(links) => {
                for url in links {
                    if let Some(found) = self.try_link("landing page", url, &mut tried, &mut failures).await {
                        return Ok(with_failures(found, &failures));
                    }
                }
            }
            Err(error) => failures.push(("landing page".into(), error)),
        }
        match self.proxied_pdf(&doi).await {
            Some(Ok(found)) => return Ok(with_failures(found, &failures)),
            Some(Err(error)) => failures.push(("proxy".into(), error)),
            None => {}
        }
        Err(give_up(failures, "no open-access copy and no PDF link on the publisher's page"))
    }

    /// Links known without asking anyone: the page's PDF link, arXiv.
    fn direct_links(&self, data: &ReferenceData) -> Vec<(String, Url)> {
        let mut links = Vec::new();
        if let Some(url) = data.extra.get("pdf-url").and_then(|u| Url::parse(u).ok()) {
            links.push(("page".to_string(), url));
        }
        for id in &data.identifiers {
            if let Identifier::Arxiv(arxiv) = id
                && let Ok(url) = self.endpoints.arxiv_pdf.join(arxiv.id())
            {
                links.push(("arxiv".to_string(), url));
            }
        }
        links
    }

    /// Downloads `url` unless already tried; records a failure.
    async fn try_link(
        &self,
        source: &str,
        url: Url,
        tried: &mut Vec<Url>,
        failures: &mut Vec<(String, FetchError)>,
    ) -> Option<FoundPdf> {
        if tried.contains(&url) {
            return None;
        }
        tried.push(url.clone());
        match self.http.get_pdf(&url).await {
            Ok((bytes, url)) => Some(FoundPdf { bytes, url, source: source.into(), issues: Vec::new() }),
            Err(error) => {
                tracing::debug!(source, %url, %error, "PDF link failed");
                failures.push((format!("{source} ({url})"), error));
                None
            }
        }
    }

    /// PDF links Unpaywall knows for `doi`, best first. Needs the contact email.
    async fn unpaywall_links(&self, doi: &Doi) -> Result<Vec<Url>> {
        let Some(email) = &self.email else {
            return Err(FetchError::NoPdf(
                "Unpaywall needs your email: `quire config set user.email <address>`".into(),
            ));
        };
        let mut url = self
            .endpoints
            .unpaywall
            .join(&crate::doi::encode_path(doi.as_str()))
            .map_err(|_| FetchError::InvalidUrl(doi.to_string()))?;
        url.query_pairs_mut().append_pair("email", email);
        let body = self.http.get_text(&url, "application/json").await?;
        let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse("api.unpaywall.org", e))?;
        Ok(unpaywall_pdf_links(&value))
    }

    /// The `citation_pdf_url` of the DOI's landing page, if it has one.
    async fn landing_page_links(&self, doi: &Doi) -> Result<Vec<Url>> {
        let landing = self
            .endpoints
            .doi
            .join(&crate::doi::encode_path(doi.as_str()))
            .map_err(|_| FetchError::InvalidUrl(doi.to_string()))?;
        let (mut html, mut landed) = self.http.get_page(&landing).await?;
        if let Some(next) = web::html_redirect(&html, &landed) {
            (html, landed) = self.http.get_page(&next).await?;
        }
        Ok(pdf_links(&html, &landed, doi))
    }
}

/// Where a publisher's page (`html`, at `landed`) keeps the PDF of `doi`: its `citation_pdf_url`,
/// then the paths of platforms that do not tag it. Atypon (APS physiology journals, Taylor &
/// Francis, SAGE, ACS, …) serves `/doi/pdf/<doi>`; Wiley `/doi/pdfdirect/<doi>`.
pub fn pdf_links(html: &str, landed: &Url, doi: &Doi) -> Vec<Url> {
    let mut links: Vec<Url> =
        web::read_tags(html).first(&["citation_pdf_url"]).and_then(|u| landed.join(u).ok()).into_iter().collect();
    if landed.path().contains("/doi/") {
        for path in ["/doi/pdf/", "/doi/pdfdirect/"] {
            if let Ok(url) = landed.join(&format!("{path}{}", doi.as_str()))
                && !links.contains(&url)
            {
                links.push(url);
            }
        }
    }
    links
}

/// `url_for_pdf` of the best location, then of the others, without repeats.
pub fn unpaywall_pdf_links(record: &Value) -> Vec<Url> {
    let best = record.get("best_oa_location").into_iter();
    let others = record.get("oa_locations").and_then(Value::as_array).into_iter().flatten();
    let mut links: Vec<Url> = Vec::new();
    for location in best.chain(others) {
        if let Some(url) = location.get("url_for_pdf").and_then(Value::as_str).and_then(|u| Url::parse(u).ok())
            && !links.contains(&url)
        {
            links.push(url);
        }
    }
    links
}

fn with_failures(mut found: FoundPdf, failures: &[(String, FetchError)]) -> FoundPdf {
    if !failures.is_empty() {
        let failed: Vec<String> = failures.iter().map(|(s, e)| format!("{s}: {e}")).collect();
        found
            .issues
            .push(Issue::recorded(IssueCode::Fallback, format!("PDF: {}; used {}", failed.join("; "), found.source)));
    }
    found
}

/// The error when nothing gave a PDF: blocked if any source was, else every failure.
fn give_up(failures: Vec<(String, FetchError)>, why: &str) -> FetchError {
    match failures.len() {
        0 => FetchError::NoPdf(why.into()),
        _ => FetchError::AllFailed(failures),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_pdf_paths() {
        let doi = Doi::parse("10.1152/jappl.1996.81.1.1").unwrap();
        let landed =
            Url::parse("https://journals-physiology-org.lp.hscl.ufl.edu/doi/10.1152/jappl.1996.81.1.1").unwrap();
        let links: Vec<String> = pdf_links("<html></html>", &landed, &doi).iter().map(Url::to_string).collect();
        assert_eq!(
            links,
            [
                "https://journals-physiology-org.lp.hscl.ufl.edu/doi/pdf/10.1152/jappl.1996.81.1.1",
                "https://journals-physiology-org.lp.hscl.ufl.edu/doi/pdfdirect/10.1152/jappl.1996.81.1.1",
            ]
        );
        let tagged = r#"<meta name="citation_pdf_url" content="/content/x.pdf">"#;
        let nature = Url::parse("https://www.nature.com/articles/x").unwrap();
        assert_eq!(pdf_links(tagged, &nature, &doi).len(), 1, "no platform guess off /doi/ paths");
    }
}
