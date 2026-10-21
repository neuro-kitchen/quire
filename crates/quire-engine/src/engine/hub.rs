//! Sync through the hub (`quire_sync::hub`, `.knowledge/sync-hub.md`): the hub named in the
//! config (`[hub] id`), the library as a [`HubReplica`] (the same change log, merge and files as a
//! device session, plus the library key, the files here and a snapshot of every row).

use quire_sync::hub::protocol::DeviceInfo;
use quire_sync::hub::{HubReplica, HubReport, SyncOptions};
use quire_sync::seal::LibraryKey;
use quire_sync::transport::{self, Device};

use super::Engine;
use super::sync::LibraryReplica;
use crate::error::{EngineError, Result};

impl Engine {
    /// The hub this device syncs through (its ID), from the config.
    pub fn hub_id(&self) -> Result<String> {
        match &self.config.hub.id {
            Some(id) if transport::is_device_id(id) => Ok(id.trim().to_string()),
            Some(id) => Err(EngineError::Config(format!("`{id}` is not a hub ID (`quire-hub id` prints it)"))),
            None => Err(EngineError::Config("no sync hub yet: `quire sync hub add <hub-id> <code>`".into())),
        }
    }

    /// Syncs this library through the hub; `code` registers this person there the first time
    /// (the hub's setup code, an invitation, or a cloud account's link code).
    pub async fn sync_with_hub(&self, code: Option<&str>) -> Result<HubReport> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key).await?;
        let options = SyncOptions { code: code.map(str::to_string), ..SyncOptions::default() };
        let report = device.sync_with_hub(&LibraryReplica(self), &hub, &options).await;
        device.close().await;
        self.mark_changed();
        self.after_hub_sync(report?).await
    }

    /// Records the files the hub holds (they may be evicted here now), then keeps within the cache.
    async fn after_hub_sync(&self, report: HubReport) -> Result<HubReport> {
        let on_hub = report.on_hub.clone();
        self.run(move |lib| lib.mark_on_hub(&on_hub)).await?;
        self.evict_pdfs().await?;
        Ok(report)
    }

    /// Downloads the PDF `hash` from the hub (sealed there; opened and checked here); `false` if
    /// the hub does not have it.
    pub async fn fetch_from_hub(&self, hash: &str) -> Result<bool> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key).await?;
        let found = device.fetch_from_hub(&LibraryReplica(self), &hub, hash).await;
        device.close().await;
        Ok(found?)
    }

    /// This person's devices known to the hub.
    pub async fn hub_devices(&self) -> Result<Vec<DeviceInfo>> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key).await?;
        let devices = device.hub_devices(&LibraryReplica(self), &hub).await;
        device.close().await;
        Ok(devices?)
    }

    /// Makes the hub refuse `device` (a device ID) of this person from now on (a lost phone).
    pub async fn hub_revoke(&self, device: &str) -> Result<()> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let me = Device::bind(&key).await?;
        let revoked = me.hub_revoke(&LibraryReplica(self), &hub, device).await;
        me.close().await;
        Ok(revoked?)
    }

    /// One hub session with `hub` (its ID) over `reader`/`writer` (any stream: QUIC in use, a
    /// pipe in tests).
    pub async fn hub_session<Rd, Wr>(
        &self,
        hub: &str,
        options: &SyncOptions,
        reader: &mut Rd,
        writer: &mut Wr,
    ) -> Result<HubReport>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        let report = quire_sync::hub::sync(&LibraryReplica(self), hub, options, reader, writer).await;
        self.mark_changed();
        self.after_hub_sync(report?).await
    }

    /// Downloads the file `hash` over `reader`/`writer` (tests; [`Engine::fetch_from_hub`] in use).
    pub async fn hub_fetch_session<Rd, Wr>(
        &self,
        hub: &str,
        hash: &str,
        reader: &mut Rd,
        writer: &mut Wr,
    ) -> Result<bool>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        let found = quire_sync::hub::fetch_file(&LibraryReplica(self), hub, hash, reader, writer).await?;
        if found {
            let on_hub = vec![hash.to_string()];
            self.run(move |lib| lib.mark_on_hub(&on_hub)).await?;
        }
        Ok(found)
    }
}

fn replica<T>(result: Result<T>) -> quire_sync::Result<T> {
    result.map_err(|e| quire_sync::SyncError::Replica(e.to_string()))
}

impl HubReplica for LibraryReplica<'_> {
    async fn library_key(&self) -> quire_sync::Result<LibraryKey> {
        replica(self.0.library_key().await)
    }

    async fn local_files(&self) -> quire_sync::Result<Vec<String>> {
        let hashes = replica(self.0.run(|lib| lib.attachment_hashes()).await)?;
        Ok(hashes.into_iter().filter(|hash| self.pdf(hash).is_file()).collect())
    }

    async fn wanted_files(&self) -> quire_sync::Result<Vec<String>> {
        replica(self.0.wanted_files().await)
    }

    async fn snapshot(&self) -> quire_sync::Result<Vec<quire_core::sync::Row>> {
        // The change log holds one entry per row (its latest state): all of it is the library.
        replica(
            self.0
                .run(|lib| {
                    let (mut all, mut seq) = (Vec::new(), 0);
                    loop {
                        let (rows, last) = lib.changes_since(seq, "")?;
                        all.extend(rows);
                        if last <= seq {
                            return Ok(all);
                        }
                        seq = last;
                    }
                })
                .await,
        )
    }
}
