//! Linking a new device to this person's devices by QR code (`quire_sync::pairing`,
//! `.tasks/T14-security.md`): this device offers (and gives its user key, hub and relay to the one
//! the person allows), or joins through another's code (and takes what it gives).

use quire_model::UserId;
use quire_sync::link::DeviceLink;
use quire_sync::pairing::{Grant, Join, Offer};

use super::Engine;
use crate::config;
use crate::error::Result;
use crate::secrets::Store;

impl Engine {
    /// Opens an offer to link a device, and what an allowed device will get from this one.
    pub async fn link_offer(&self) -> Result<(Offer, Grant)> {
        let grant = Grant {
            user_key: self.export_user_key()?,
            hub: self.config.hub.id.clone(),
            relay: self.relay().map(str::to_string),
        };
        Ok((Offer::open(self.relay()).await?, grant))
    }

    /// Dials the device whose code is `link` and asks as `name` (shown there).
    pub async fn link_join(&self, link: &str, name: &str) -> Result<Join> {
        Ok(Join::connect(&DeviceLink::parse(link)?, name).await?)
    }

    /// Takes what an allowed link gave: the person's user key, and their hub and relay (the
    /// settings; reopen the library to use them). Returns the user ID now in use.
    pub fn apply_grant(&self, grant: &Grant) -> Result<(UserId, Store)> {
        let taken = self.import_user_key(&grant.user_key)?;
        let file = self.config_file();
        match &grant.hub {
            Some(hub) => config::set(file, "hub.id", hub)?,
            None => _ = config::unset(file, "hub.id")?,
        }
        match &grant.relay {
            Some(relay) => config::set(file, "sync.relay", relay)?,
            None => _ = config::unset(file, "sync.relay")?,
        }
        Ok(taken)
    }
}
