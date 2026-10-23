//! Links that set a device up in one step (`.tasks/T14-security.md`): shown as text or a QR code.
//!
//! `quire://hub?id=<hub ID>&relay=<relay URL>&code=<one-time code>` joins a sync hub: the hub prints
//! it on first start (with its setup code) and for invitations, wherever it runs (at home, on one's
//! own cloud). `relay` is left out when the hub uses n0's relays; `code` when the person is known.
//!
//! `quire://device?id=<one-time endpoint>&relay=<relay URL>&s=<one-time secret>` links a new device
//! to one of the person's devices (`crate::pairing`): shown as a QR code for two minutes.

use url::Url;

use crate::error::{Result, SyncError};
use crate::{identity, transport};

/// The scheme of every quire link.
pub const SCHEME: &str = "quire";

/// A sync hub to join: its ID, our own relay (if any) and a one-time code (if any).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubLink {
    pub id: String,
    pub relay: Option<String>,
    pub code: Option<String>,
}

impl HubLink {
    /// The link as text (`quire://hub?…`), values percent-encoded.
    pub fn to_uri(&self) -> String {
        let mut url = Url::parse("quire://hub").expect("a valid base");
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("id", &self.id);
            if let Some(relay) = &self.relay {
                query.append_pair("relay", relay);
            }
            if let Some(code) = &self.code {
                query.append_pair("code", code);
            }
        }
        url.into()
    }

    /// Reads a link (surrounding spaces ignored), checking the hub ID and the relay URL.
    pub fn parse(text: &str) -> Result<Self> {
        let bad = |why: &str| SyncError::Protocol(format!("not a quire hub link: {why}"));
        let url = Url::parse(text.trim()).map_err(|e| bad(&e.to_string()))?;
        if url.scheme() != SCHEME || url.host_str() != Some("hub") {
            return Err(bad("it should start with quire://hub?"));
        }
        let (mut id, mut relay, mut code) = (None, None, None);
        for (key, value) in url.query_pairs() {
            let value = value.trim().to_string();
            match key.as_ref() {
                "id" => id = Some(value),
                "relay" if !value.is_empty() => relay = Some(value),
                "code" if !value.is_empty() => code = Some(value),
                // Newer links may carry more; this version ignores what it does not know.
                _ => {}
            }
        }
        let id = id.ok_or_else(|| bad("no hub ID"))?;
        if !transport::is_device_id(&id) {
            return Err(bad("its hub ID is damaged"));
        }
        if let Some(relay) = &relay {
            transport::parse_relay(relay)?;
        }
        Ok(Self { id, relay, code })
    }
}

/// A device offering to link another (`crate::pairing`): where to reach it (a one-time endpoint's
/// ID, our own relay) and the one-time secret that proves the scanner saw its screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceLink {
    pub id: String,
    pub relay: Option<String>,
    pub secret: [u8; 32],
}

impl DeviceLink {
    /// The link as text (`quire://device?…`).
    pub fn to_uri(&self) -> String {
        let mut url = Url::parse("quire://device").expect("a valid base");
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("id", &self.id);
            if let Some(relay) = &self.relay {
                query.append_pair("relay", relay);
            }
            query.append_pair("s", &identity::key_to_hex(&self.secret));
        }
        url.into()
    }

    /// Reads a link (surrounding spaces ignored), checking every part.
    pub fn parse(text: &str) -> Result<Self> {
        let bad = |why: &str| SyncError::Protocol(format!("not a quire device link: {why}"));
        let url = Url::parse(text.trim()).map_err(|e| bad(&e.to_string()))?;
        if url.scheme() != SCHEME || url.host_str() != Some("device") {
            return Err(bad("it should start with quire://device?"));
        }
        let (mut id, mut relay, mut secret) = (None, None, None);
        for (key, value) in url.query_pairs() {
            let value = value.trim().to_string();
            match key.as_ref() {
                "id" => id = Some(value),
                "relay" if !value.is_empty() => relay = Some(value),
                "s" => secret = identity::key_from_hex(&value),
                _ => {}
            }
        }
        let id =
            id.filter(|id| transport::is_device_id(id)).ok_or_else(|| bad("its device ID is missing or damaged"))?;
        let secret = secret.ok_or_else(|| bad("its secret is missing or damaged"))?;
        if let Some(relay) = &relay {
            transport::parse_relay(relay)?;
        }
        Ok(Self { id, relay, secret })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> String {
        transport::device_id(&transport::new_secret())
    }

    #[test]
    fn a_link_reads_back_as_written() {
        let link =
            HubLink { id: id(), relay: Some("http://192.168.1.10:3340".into()), code: Some("82BU-YRXN-XYDQ".into()) };
        let text = link.to_uri();
        assert!(text.starts_with("quire://hub?id="), "{text}");
        assert_eq!(HubLink::parse(&format!("  {text}\n")).unwrap(), link);

        let bare = HubLink { id: id(), relay: None, code: None };
        assert_eq!(HubLink::parse(&bare.to_uri()).unwrap(), bare);
    }

    #[test]
    fn other_text_is_refused() {
        let hub = id();
        for text in [
            "https://example.com/?id=x".to_string(),
            format!("quire://device?id={hub}"),
            "quire://hub?code=ABC".to_string(),
            "quire://hub?id=not-an-id".to_string(),
            format!("quire://hub?id={hub}&relay=not%20a%20url"),
        ] {
            assert!(HubLink::parse(&text).is_err(), "{text}");
        }
        // Unknown fields from a newer version are ignored.
        assert!(HubLink::parse(&format!("quire://hub?id={hub}&v=2")).is_ok());
    }

    #[test]
    fn a_device_link_reads_back_and_needs_its_secret() {
        let link = DeviceLink { id: id(), relay: Some("https://relay.example.org".into()), secret: [7; 32] };
        assert_eq!(DeviceLink::parse(&link.to_uri()).unwrap(), link);
        let without = format!("quire://device?id={}", link.id);
        assert!(DeviceLink::parse(&without).is_err());
        assert!(DeviceLink::parse(&format!("{without}&s=abcd")).is_err());
        assert!(DeviceLink::parse(&link.to_uri().replace("quire://device", "quire://hub")).is_err());
    }
}
