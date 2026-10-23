//! Sync through the hub (`quire_sync::hub`, `.knowledge/sync-hub.md`): the hub named in the
//! config (`[hub] id`), the library as a [`HubReplica`] (the same change log, merge and files as a
//! device session, plus the library key, the files here and a snapshot of every row).

use quire_sync::hub::{HubDevice, HubReplica, HubReport, Rotation, SyncOptions};
use quire_sync::keys::{BoxKey, LibraryKeys};
use quire_sync::transport::{self, Device};

use super::Engine;
use super::sync::LibraryReplica;
use crate::error::{EngineError, Result};

impl Engine {
    /// This device's name for the person's other devices: the setting `device.name`, else the
    /// host's name where there is one.
    pub fn device_name(&self) -> Option<String> {
        let set = self.config.device.name.as_deref().map(str::trim).filter(|n| !n.is_empty());
        set.map(str::to_string).or_else(|| std::env::var("HOSTNAME").ok().filter(|n| !n.trim().is_empty()))
    }

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
        let device = Device::bind(&key, self.relay()).await?;
        let options = SyncOptions { code: code.map(str::to_string), ..SyncOptions::default() };
        let report = device.sync_with_hub(&LibraryReplica(self), &hub, &options).await;
        device.close().await;
        self.mark_changed();
        self.after_hub_sync(report?).await
    }

    /// Records which files the hub holds (they may be evicted here now) and which it lacks (they may
    /// not, whatever an earlier session said), then keeps within the cache.
    async fn after_hub_sync(&self, report: HubReport) -> Result<HubReport> {
        let (on_hub, not_on_hub) = (report.on_hub.clone(), report.not_on_hub.clone());
        self.run(move |lib| {
            lib.clear_on_hub(&not_on_hub)?;
            lib.mark_on_hub(&on_hub)
        })
        .await?;
        self.evict_pdfs().await?;
        self.record_sync(&report).await?;
        Ok(report)
    }

    /// Downloads the PDF `hash` from the hub (sealed there; opened and checked here); `false` if
    /// the hub does not have it.
    pub async fn fetch_from_hub(&self, hash: &str) -> Result<bool> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key, self.relay()).await?;
        let found = device.fetch_from_hub(&LibraryReplica(self), &hub, hash).await;
        device.close().await;
        Ok(found?)
    }

    /// This person's devices known to the hub.
    pub async fn hub_devices(&self) -> Result<Vec<HubDevice>> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key, self.relay()).await?;
        let devices = device.hub_devices(&LibraryReplica(self), &hub).await;
        device.close().await;
        let devices = devices?;
        self.note_devices(&devices).await?;
        Ok(devices)
    }

    /// Unlinks `device` (a device ID) of this person: the hub refuses it from now on, and the
    /// library key rotates so it cannot read what is sealed after (`quire_sync::keys`).
    pub async fn hub_unlink(&self, device: &str) -> Result<Rotation> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let me = Device::bind(&key, self.relay()).await?;
        let unlinked = me.hub_unlink(&LibraryReplica(self), &hub, device).await;
        me.close().await;
        let rotation = unlinked?;
        self.record_activity(super::activity::kind::UNLINKED, rotation.name.as_deref(), &rotation.epoch.to_string())
            .await?;
        Ok(rotation)
    }

    /// Lets the hub accept `device`, a new device of this person being linked (once, for minutes).
    pub async fn hub_introduce(&self, device: &str) -> Result<()> {
        let hub = self.hub_id()?;
        let (key, _) = self.sync_key()?;
        let me = Device::bind(&key, self.relay()).await?;
        let introduced = me.hub_introduce(&LibraryReplica(self), &hub, device).await;
        me.close().await;
        Ok(introduced?)
    }

    /// [`Engine::hub_unlink`] over `reader`/`writer` (tests).
    pub async fn hub_unlink_session<Rd, Wr>(
        &self,
        hub: &str,
        device: &str,
        reader: &mut Rd,
        writer: &mut Wr,
    ) -> Result<Rotation>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        let rotation = quire_sync::hub::unlink(&LibraryReplica(self), hub, device, reader, writer).await?;
        self.record_activity(super::activity::kind::UNLINKED, rotation.name.as_deref(), &rotation.epoch.to_string())
            .await?;
        Ok(rotation)
    }

    /// [`Engine::hub_introduce`] over `reader`/`writer` (tests).
    pub async fn hub_introduce_session<Rd, Wr>(
        &self,
        hub: &str,
        device: &str,
        reader: &mut Rd,
        writer: &mut Wr,
    ) -> Result<()>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        Ok(quire_sync::hub::introduce(&LibraryReplica(self), hub, device, reader, writer).await?)
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

    /// This person's devices known to the hub, over `reader`/`writer` (tests; [`Engine::hub_devices`]
    /// in use).
    pub async fn hub_devices_session<Rd, Wr>(
        &self,
        hub: &str,
        reader: &mut Rd,
        writer: &mut Wr,
    ) -> Result<Vec<HubDevice>>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        Ok(quire_sync::hub::devices(&LibraryReplica(self), hub, reader, writer).await?)
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
    async fn library_keys(&self) -> quire_sync::Result<LibraryKeys> {
        replica(self.0.library_keys().await)
    }

    async fn save_library_keys(&self, keys: &LibraryKeys) -> quire_sync::Result<()> {
        let library = replica(self.0.run(|lib| lib.library_id()).await)?;
        replica(self.0.save_library_keys_of(&library, keys))
    }

    fn device_box(&self) -> Option<(BoxKey, Vec<u8>)> {
        self.0.device_box().ok()
    }

    fn device_name(&self) -> Option<String> {
        self.0.device_name()
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
