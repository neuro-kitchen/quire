//! Downloading through the user's institutional proxy (EZproxy style, e.g. UF's
//! `https://login.lp.hscl.ufl.edu/login?url=`), with the session cookie the user copied from
//! their own browser after logging in (`quire access login`). The cookie is sent to the proxy's
//! domain only, requests are paced (5 s), and quire never sees the user's password.
//!
//! The proxy answers a wrapped URL with the publisher's page on a rewritten host
//! (`www-nature-com.lp.hscl.ufl.edu`) when the session is valid, and with its login (or the
//! university's single sign-on) when it is not.

use quire_model::{Doi, Issue, IssueCode};
use url::Url;

use crate::error::{FetchError, Result};
use crate::fetcher::Fetcher;
use crate::http::Http;
use crate::pdf::FoundPdf;

/// A proxy prefix and the session cookies for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxySession {
    /// What goes before a URL, e.g. `https://login.lp.hscl.ufl.edu/login?url=`.
    pub prefix: String,
    /// `name=value; …`, as normalised by [`ProxySession::normalise_cookie`].
    pub cookies: String,
    /// The user's own extra intermediate certificates (PEM), from `access.ca`.
    pub extra_ca: Option<String>,
}

/// Intermediates proxies are known to leave out (`certs/README.md`).
const BUNDLED_CA: &[&str] = &[include_str!("../certs/incommon-rsa-ov-ssl-ca-3.pem")];

/// A page used to check a session: any well-known article (a DOI the proxy can follow).
const CHECK_DOI: &str = "10.1038/nature12373";

impl ProxySession {
    /// The proxy's login host, e.g. `login.lp.hscl.ufl.edu`.
    pub fn login_host(prefix: &str) -> Result<String> {
        let url = Url::parse(prefix).map_err(|_| FetchError::InvalidUrl(prefix.into()))?;
        url.host_str().map(str::to_ascii_lowercase).ok_or_else(|| FetchError::InvalidUrl(prefix.into()))
    }

    /// The domain the proxy serves publishers under, which gets the cookie: the login host
    /// without a leading `login.` (`lp.hscl.ufl.edu`).
    pub fn domain(prefix: &str) -> Result<String> {
        let host = Self::login_host(prefix)?;
        Ok(host.strip_prefix("login.").map(str::to_string).unwrap_or(host))
    }

    /// `text` as pasted (`abc123`, `ezproxy=abc123`, `Cookie: ezproxy=abc; ezproxyl=def`) as
    /// `name=value` pairs; a bare value is EZproxy's usual `ezproxy` cookie. `None` if it holds
    /// characters a cookie cannot.
    pub fn normalise_cookie(text: &str) -> Option<String> {
        let text = text.trim();
        let text = text.strip_prefix("Cookie:").or_else(|| text.strip_prefix("cookie:")).unwrap_or(text).trim();
        if text.is_empty() || text.chars().any(|c| c.is_control() || c == ',') {
            return None;
        }
        if !text.contains('=') {
            return Some(format!("ezproxy={text}"));
        }
        let pairs: Vec<&str> = text.split(';').map(str::trim).filter(|p| !p.is_empty()).collect();
        pairs
            .iter()
            .all(|p| p.split_once('=').is_some_and(|(n, v)| !n.trim().is_empty() && !v.is_empty()))
            .then(|| pairs.join("; "))
    }

    /// `url` through the proxy.
    pub fn wrap(&self, url: &Url) -> Result<Url> {
        Url::parse(&format!("{}{url}", self.prefix)).map_err(|_| FetchError::InvalidUrl(self.prefix.clone()))
    }

    /// Whether `url` is a page the proxy served for a publisher (not its login, not single sign-on).
    fn is_proxied(&self, url: &Url) -> bool {
        let (Ok(login), Ok(domain)) = (Self::login_host(&self.prefix), Self::domain(&self.prefix)) else {
            return false;
        };
        url.host_str().is_some_and(|h| h != login && h.ends_with(domain.as_str()))
    }
}

/// The proxy part of a [`Fetcher`].
pub(crate) struct Proxy {
    session: ProxySession,
    http: Http,
}

impl Proxy {
    pub(crate) fn new(email: Option<&str>, session: ProxySession) -> Result<Self> {
        let domain = ProxySession::domain(&session.prefix)?;
        let mut extra: Vec<String> = BUNDLED_CA.iter().map(|pem| pem.to_string()).collect();
        extra.extend(session.extra_ca.clone());
        let http = Http::with_proxy_cookies(email, &domain, &session.cookies, &extra)?;
        Ok(Self { session, http })
    }
}

impl Fetcher {
    /// This fetcher, also downloading through `session`.
    pub fn with_proxy(mut self, session: ProxySession) -> Result<Self> {
        self.proxy = Some(Proxy::new(self.email.as_deref(), session)?);
        Ok(self)
    }

    /// Whether `session` is logged in: one request for a well-known article through the proxy.
    pub async fn check_proxy(&self, session: &ProxySession) -> Result<bool> {
        let proxy = Proxy::new(self.email.as_deref(), session.clone())?;
        let article = Url::parse(&format!("https://doi.org/{CHECK_DOI}")).expect("valid URL");
        let (_, landed) = proxy.http.get_page(&proxy.session.wrap(&article)?).await?;
        Ok(proxy.session.is_proxied(&landed))
    }

    /// The PDF of `doi` through the proxy: its landing page there, then the page's PDF link.
    pub(crate) async fn proxied_pdf(&self, doi: &Doi) -> Option<Result<FoundPdf>> {
        let proxy = self.proxy.as_ref()?;
        Some(proxied(proxy, doi).await)
    }
}

async fn proxied(proxy: &Proxy, doi: &Doi) -> Result<FoundPdf> {
    let session = &proxy.session;
    let landing = Url::parse(&format!("https://doi.org/{}", crate::doi::encode_path(doi.as_str())))
        .map_err(|_| FetchError::InvalidUrl(doi.to_string()))?;
    let (mut html, mut landed) = proxy.http.get_page(&session.wrap(&landing)?).await?;
    if !session.is_proxied(&landed) {
        return Err(not_proxied(session, &landed, &html));
    }
    // A redirect page (Elsevier's linkinghub) leads to the article: follow it once.
    if let Some(next) = crate::web::html_redirect(&html, &landed) {
        let next = if session.is_proxied(&next) { next } else { session.wrap(&next)? };
        (html, landed) = proxy.http.get_page(&next).await?;
        if !session.is_proxied(&landed) {
            return Err(not_proxied(session, &landed, &html));
        }
    }
    let links = crate::pdf::pdf_links(&html, &landed, doi);
    let mut last = FetchError::NoPdf(format!(
        "no PDF link on the publisher's page through the proxy ({})",
        landed.host_str().unwrap_or_default()
    ));
    for link in links {
        // EZproxy rewrites links on the page; one it left on the publisher's own host is wrapped.
        let link = if session.is_proxied(&link) { link } else { session.wrap(&link)? };
        match proxy.http.get_pdf(&link).await {
            Ok((bytes, url)) => {
                return Ok(FoundPdf { bytes, url, source: "proxy".into(), issues: Vec::<Issue>::new() });
            }
            Err(error) => last = error,
        }
    }
    Err(last)
}

/// Why the proxy did not show the publisher's page: a login (the university's single sign-on, or
/// a login form on the proxy), or a publisher the proxy is not set up for (EZproxy answers on its
/// own host without a form; not a reason to stop downloading the rest).
fn not_proxied(session: &ProxySession, landed: &Url, html: &str) -> FetchError {
    let on_proxy =
        ProxySession::login_host(&session.prefix).is_ok_and(|login| landed.host_str() == Some(login.as_str()));
    let asks_login =
        html.contains("type=\"password\"") || html.contains("type='password'") || html.contains("type=password");
    // Single sign-on (Shibboleth, CAS, …) lives at addresses like these.
    let path = landed.path().to_ascii_lowercase();
    let sign_on = ["/idp/", "saml", "/sso", "/cas/", "/login", "/auth"].iter().any(|p| path.contains(p))
        || landed.host_str().is_some_and(|h| h.starts_with("login.") || h.starts_with("sso.") || h.starts_with("idp."));
    if asks_login || (!on_proxy && sign_on) {
        FetchError::ProxyLogin
    } else if on_proxy {
        FetchError::NoPdf(format!("the proxy cannot open this publisher ({})", crate::http::shown(landed)))
    } else {
        // EZproxy sends sites it does not cover straight to the publisher (often open access):
        // not a login problem; the open landing page was already tried.
        FetchError::NoPdf(format!("the proxy does not cover {}", landed.host_str().unwrap_or_default()))
    }
}

/// The issue recorded when the proxy needs a new login.
pub fn login_issue() -> Issue {
    Issue::recorded(IssueCode::Blocked, "the proxy session has expired: run `quire access login`")
}

#[cfg(test)]
mod tests {
    use super::*;

    const UF: &str = "https://login.lp.hscl.ufl.edu/login?url=";

    #[test]
    fn domains() {
        assert_eq!(ProxySession::login_host(UF).unwrap(), "login.lp.hscl.ufl.edu");
        assert_eq!(ProxySession::domain(UF).unwrap(), "lp.hscl.ufl.edu");
    }

    #[test]
    fn cookie_forms() {
        let n = ProxySession::normalise_cookie;
        assert_eq!(n("abc123").as_deref(), Some("ezproxy=abc123"));
        assert_eq!(n(" ezproxy=abc123 ").as_deref(), Some("ezproxy=abc123"));
        assert_eq!(n("Cookie: ezproxy=abc; ezproxyl=def").as_deref(), Some("ezproxy=abc; ezproxyl=def"));
        assert_eq!(n("a\\nb"), Some("ezproxy=a\\nb".into()));
        assert_eq!(n("bad\nvalue"), None);
        assert_eq!(n("=x"), None);
        assert_eq!(n(""), None);
    }

    #[test]
    fn login_versus_unconfigured_publisher() {
        let session = ProxySession { prefix: UF.into(), cookies: "ezproxy=x".into(), extra_ca: None };
        let url = |u: &str| Url::parse(u).unwrap();
        let sso = not_proxied(&session, &url("https://login.ufl.edu/idp/profile/SAML2"), "<form>");
        assert!(sso.is_proxy_login());
        let form = not_proxied(&session, &url("https://login.lp.hscl.ufl.edu/login"), r#"<input type="password">"#);
        assert!(form.is_proxy_login());
        let unconfigured =
            not_proxied(&session, &url("https://login.lp.hscl.ufl.edu/login?url=x"), "<p>not configured</p>");
        assert!(!unconfigured.is_proxy_login());
    }

    #[test]
    fn proxied_hosts() {
        let session = ProxySession { prefix: UF.into(), cookies: "ezproxy=x".into(), extra_ca: None };
        let url = |u: &str| Url::parse(u).unwrap();
        assert!(session.is_proxied(&url("https://www-nature-com.lp.hscl.ufl.edu/articles/x")));
        assert!(!session.is_proxied(&url("https://login.lp.hscl.ufl.edu/login?url=x")));
        assert!(!session.is_proxied(&url("https://login.ufl.edu/idp/profile/SAML2")));
        let wrapped = session.wrap(&url("https://doi.org/10.1/x")).unwrap();
        assert_eq!(wrapped.as_str(), "https://login.lp.hscl.ufl.edu/login?url=https://doi.org/10.1/x");
    }
}
