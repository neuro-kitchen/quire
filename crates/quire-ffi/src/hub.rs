//! Sync through the hub (`.knowledge/sync-hub.md`) and lazy PDFs. The hub's ID is a setting
//! (`hub.id`, through the config functions); after changing settings the app reopens the library.

use crate::{QuireError, QuireLibrary, Result, query};

/// What a hub sync did.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct HubReportRecord {
    /// This person registered with a code in this sync.
    pub registered: bool,
    /// This new, empty copy joined the library the hub holds.
    pub joined: bool,
    pub pushed: u32,
    pub applied: u32,
    pub kept: u32,
    pub refused: u32,
    pub files_uploaded: u32,
    pub files_downloaded: u32,
    /// PDFs not uploaded: the hub's storage is full.
    pub over_quota: u32,
    pub snapshot: bool,
    /// Bytes stored for this person on the hub, and the limit (0: none).
    pub used: u64,
    pub quota: u64,
}

/// One of this person's devices as the hub knows it.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct HubDeviceRecord {
    pub device: String,
    /// Milliseconds since 1970.
    pub first_seen: i64,
    pub last_seen: i64,
    pub revoked: bool,
}

/// A PDF ready to read; `source` `here`, `hub`, `peer` (with `detail` its name) or `found` (with
/// `detail` where it was found again).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct OpenedPdfRecord {
    pub path: String,
    pub hash: String,
    pub source: String,
    pub detail: Option<String>,
}

/// A `quire://hub?…` link read (`.tasks/T14-security.md`): the hub, our own relay, a one-time code.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct HubLinkRecord {
    pub id: String,
    pub relay: Option<String>,
    pub code: Option<String>,
}

/// Reads a hub link (pasted or scanned); fails with what is wrong with it.
#[uniffi::export]
pub fn parse_hub_link(text: String) -> Result<HubLinkRecord> {
    let link = quire_engine::HubLink::parse(&text).map_err(|e| QuireError::Invalid(e.to_string()))?;
    Ok(HubLinkRecord { id: link.id, relay: link.relay, code: link.code })
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct EvictionRecord {
    pub removed: u32,
    pub freed: u64,
    /// Bytes of PDFs on this device after it.
    pub kept: u64,
}

#[uniffi::export]
impl QuireLibrary {
    /// Syncs through the hub (`hub.id`); `code` registers this person the first time (the hub's
    /// setup code, an invitation, or a cloud account's link code).
    pub async fn sync_with_hub(&self, code: Option<String>) -> Result<HubReportRecord> {
        self.with(|engine| async move {
            let r = engine.sync_with_hub(code.as_deref()).await?;
            Ok(HubReportRecord {
                registered: r.registered,
                joined: r.joined,
                pushed: r.pushed as u32,
                applied: r.pulled.applied as u32,
                kept: r.pulled.kept as u32,
                refused: r.pulled.refused as u32,
                files_uploaded: r.files_uploaded as u32,
                files_downloaded: r.files_downloaded as u32,
                over_quota: r.over_quota as u32,
                snapshot: r.snapshot,
                used: r.used,
                quota: r.quota,
            })
        })
        .await
    }

    /// This person's devices the hub knows.
    pub async fn hub_devices(&self) -> Result<Vec<HubDeviceRecord>> {
        self.with(|engine| async move {
            Ok(engine
                .hub_devices()
                .await?
                .into_iter()
                .map(|d| HubDeviceRecord {
                    device: d.device,
                    first_seen: d.first_seen,
                    last_seen: d.last_seen,
                    revoked: d.revoked,
                })
                .collect())
        })
        .await
    }

    /// Makes the hub refuse `device` (a device ID) of this person from now on.
    pub async fn hub_revoke(&self, device: String) -> Result<()> {
        self.with(|engine| async move { Ok(engine.hub_revoke(&device).await?) }).await
    }

    /// What to keep safe before syncing through a hub: the hub cannot recover the library.
    pub async fn recovery_kit(&self) -> Result<String> {
        self.with(|engine| async move { Ok(engine.recovery_kit()?) }).await
    }

    /// The PDF of the reference `id`, fetched if it is not on this device (the hub, another of the
    /// user's devices, then the source).
    pub async fn open_pdf(&self, id: String) -> Result<OpenedPdfRecord> {
        self.with(|engine| async move {
            let opened = engine.open_pdf(&id).await?;
            let (source, detail) = match opened.source {
                quire_engine::PdfSource::Here => ("here", None),
                quire_engine::PdfSource::Hub => ("hub", None),
                quire_engine::PdfSource::Peer(name) => ("peer", Some(name)),
                quire_engine::PdfSource::Found(source) => ("found", Some(source)),
            };
            Ok(OpenedPdfRecord {
                path: opened.path.display().to_string(),
                hash: opened.hash,
                source: source.into(),
                detail,
            })
        })
        .await
    }

    /// Keeps (or stops keeping) the reference `id`'s PDFs downloaded on this device.
    pub async fn keep_reference_offline(&self, id: String, keep: bool) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.keep_reference_offline(&id, keep).await?) }).await
    }

    /// Keeps (or stops keeping) the PDFs of the project at `path` (and subprojects) downloaded.
    pub async fn keep_project_offline(&self, path: String, keep: bool) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.keep_project_offline(&query::slug_path(&path)?, keep).await?) }).await
    }

    /// Removes PDFs past `files.cache_mb` (only those the hub holds; never one kept offline).
    pub async fn evict_pdfs(&self) -> Result<EvictionRecord> {
        self.with(|engine| async move {
            let e = engine.evict_pdfs().await?;
            Ok(EvictionRecord { removed: e.removed as u32, freed: e.freed, kept: e.kept })
        })
        .await
    }
}
