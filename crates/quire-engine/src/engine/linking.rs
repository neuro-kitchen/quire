//! Linking a new device to this person's devices by QR code (`quire_sync::pairing`,
//! `.tasks/T14-security.md`): this device offers (and gives its user key, hub and relay to the one
//! the person allows), or joins through another's code (and takes what it gives).

use quire_model::UserId;
use quire_sync::link::DeviceLink;
use quire_sync::pairing::{Grant, Join, Offer};

use super::Engine;
use crate::config;
use crate::error::{EngineError, Result};
use crate::secrets::Store;

impl Engine {
    /// Opens an offer to link a device, and what an allowed device will get from this one. Only
    /// once this device syncs through a hub: the new device needs it to get the library, and the
    /// link travels through the hub's relay (not n0's, when the person runs their own).
    pub async fn link_offer(&self) -> Result<(Offer, Grant)> {
        self.hub_id().map_err(|_| {
            EngineError::Config("this device has no sync hub yet: join your hub first, then link your devices".into())
        })?;
        let library = self.run(|lib| lib.library_id()).await?;
        let grant = Grant {
            user_key: self.export_user_key()?,
            hub: self.config.hub.id.clone(),
            relay: self.relay().map(str::to_string),
            library_keys: self.library_keys_of(&library)?.chain_text(),
            library: Some(library),
        };
        Ok((Offer::open(self.relay()).await?, grant))
    }

    /// Dials the device whose code is `link` and asks as `name` (shown there).
    pub async fn link_join(&self, link: &str, name: &str) -> Result<Join> {
        let (device, _) = self.sync_id()?;
        Ok(Join::connect(&DeviceLink::parse(link)?, name, &device).await?)
    }

    /// Takes what an allowed link gave: the person's user key, and their hub and relay (the
    /// settings; reopen the library to use them). Returns the user ID now in use.
    pub async fn apply_grant(&self, grant: &Grant) -> Result<(UserId, Store)> {
        let taken = self.import_user_key(&grant.user_key)?;
        // The library's later keys, for when this copy joins that library through the hub.
        if let Some(library) = grant.library.as_deref().filter(|_| !grant.library_keys.is_empty()) {
            let mut keys = self.library_keys_of(library)?;
            keys.merge(&quire_sync::keys::LibraryKeys::parse_chain(&grant.library_keys));
            self.save_library_keys_of(library, &keys)?;
        }
        let file = self.config_file();
        match &grant.hub {
            Some(hub) => config::set(file, "hub.id", hub)?,
            None => _ = config::unset(file, "hub.id")?,
        }
        match &grant.relay {
            Some(relay) => config::set(file, "sync.relay", relay)?,
            None => _ = config::unset(file, "sync.relay")?,
        }
        self.record_activity(super::activity::kind::LINKED, None, "").await?;
        Ok(taken)
    }
}
