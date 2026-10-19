//! The institutional proxy session (`quire access login|status|logout`): the cookie the user
//! copies from their own browser after logging in to the proxy, checked once and kept in the
//! system keyring. With it, `pdf get` downloads subscribed PDFs through the proxy.

use std::path::{Path, PathBuf};

use quire_fetch::ProxySession;
use url::Url;

use super::Engine;
use crate::config::Access;
use crate::error::{EngineError, Result};
use crate::secrets::{self, Store};

/// Name of the session in the keyring.
const SECRET: &str = "proxy-session";

/// What `access status` reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessStatus {
    pub prefix: Option<String>,
    /// Where the session is kept, if there is one.
    pub stored: Option<Store>,
    /// Whether the proxy accepted it just now (`None`: not checked).
    pub active: Option<bool>,
}

/// The fallback file for the session, next to the config file.
pub fn fallback_path(config_file: &Path) -> PathBuf {
    config_file.with_file_name(SECRET)
}

/// The stored session for the configured proxy, if any (loaded when the engine opens).
pub(crate) fn load(config_file: &Path, access: &Access) -> Option<ProxySession> {
    let prefix = access.proxy.as_deref()?;
    let (cookies, _) = secrets::get(SECRET, &fallback_path(config_file))?;
    Some(session(access, prefix, cookies))
}

/// A session for `prefix` with `cookies` and the user's extra certificates (`access.ca`).
fn session(access: &Access, prefix: &str, cookies: String) -> ProxySession {
    let extra_ca =
        access.ca.as_deref().map(crate::paths::expand_home).and_then(|path| std::fs::read_to_string(path).ok());
    ProxySession { prefix: prefix.to_string(), cookies, extra_ca }
}

impl Engine {
    fn proxy_prefix(&self) -> Result<String> {
        self.config.access.proxy.clone().ok_or_else(|| {
            EngineError::Config(
                "no proxy set: `quire config set access.proxy <prefix>`, e.g. https://login.lp.hscl.ufl.edu/login?url="
                    .into(),
            )
        })
    }

    /// The page to log in at: a well-known article through the proxy, so the user lands on it.
    pub fn proxy_login_page(&self) -> Result<Url> {
        let prefix = self.proxy_prefix()?;
        Url::parse(&format!("{prefix}https://doi.org/10.1038/nature12373"))
            .map_err(|_| EngineError::Config(format!("`{prefix}` is not a URL prefix")))
    }

    /// The domain the session cookie is for, e.g. `lp.hscl.ufl.edu`.
    pub fn proxy_domain(&self) -> Result<String> {
        Ok(ProxySession::domain(&self.proxy_prefix()?)?)
    }

    /// Checks the pasted cookie against the proxy and, if it is logged in, keeps it in the keyring.
    pub async fn proxy_login(&self, pasted: &str) -> Result<Store> {
        let prefix = self.proxy_prefix()?;
        let cookies = ProxySession::normalise_cookie(pasted).ok_or_else(|| {
            EngineError::Config("that is not a cookie value (copy the `ezproxy` cookie's value)".into())
        })?;
        let session = session(&self.config.access, &prefix, cookies);
        if !self.fetcher.check_proxy(&session).await? {
            return Err(EngineError::Config(
                "the proxy did not accept that cookie (not logged in yet, expired, or another cookie); log in again and copy `ezproxy`".into(),
            ));
        }
        secrets::set(SECRET, &session.cookies, &fallback_path(&self.config_file))
    }

    /// The proxy prefix, where the session is kept, and (`check`) whether it still works.
    pub async fn proxy_status(&self, check: bool) -> Result<AccessStatus> {
        let prefix = self.config.access.proxy.clone();
        let stored = secrets::get(SECRET, &fallback_path(&self.config_file));
        let active = match (&prefix, &stored, check) {
            (Some(prefix), Some((cookies, _)), true) => {
                Some(self.fetcher.check_proxy(&session(&self.config.access, prefix, cookies.clone())).await?)
            }
            _ => None,
        };
        Ok(AccessStatus { prefix, stored: stored.map(|(_, store)| store), active })
    }

    /// Forgets the session; `false` if there was none.
    pub fn proxy_logout(&self) -> Result<bool> {
        secrets::delete(SECRET, &fallback_path(&self.config_file))
    }
}
