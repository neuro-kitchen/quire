//! This device's notifications (`.tasks/T14-security.md`): what syncs brought from which of the
//! person's devices, and security events. Kept in the library on this device only (never synced).

use quire_store::Activity;
use quire_sync::hub::{HubDevice, HubReport};

use super::Engine;
use crate::error::Result;

/// What happened, as the pane shows it.
pub mod kind {
    /// A sync brought changes from a device (`detail`: `kind=count,…`).
    pub const SYNC: &str = "sync";
    /// This device was linked to the person's devices.
    pub const LINKED: &str = "linked";
    /// This device let a new one in (`source`: its name).
    pub const LINK_ALLOWED: &str = "link_allowed";
    pub const LINK_REFUSED: &str = "link_refused";
    /// Wrong link codes were tried while this device offered one (`detail`: how many).
    pub const WRONG_CODE: &str = "wrong_code";
    /// This device unlinked one (`detail`: the new key epoch).
    pub const UNLINKED: &str = "unlinked";
    /// The library key changed because another device unlinked one (`detail`: the epoch).
    pub const KEY_CHANGED: &str = "key_changed";
    /// A device this one had not seen before is on the hub.
    pub const NEW_DEVICE: &str = "new_device";
}

/// What a sync brought from one device: counts by kind.
type Counts = Vec<(String, usize)>;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

impl Engine {
    /// Adds an entry to this device's notifications.
    pub async fn record_activity(&self, kind: &str, source: Option<&str>, detail: &str) -> Result<()> {
        let (kind, source, detail) = (kind.to_string(), source.map(str::to_string), detail.to_string());
        self.run(move |lib| lib.add_activity(now(), &kind, source.as_deref(), &detail)).await
    }

    /// The newest notifications first, at most `limit`.
    pub async fn activity(&self, limit: u32) -> Result<Vec<Activity>> {
        self.run(move |lib| lib.activity(limit)).await
    }

    pub async fn unread_activity(&self) -> Result<u32> {
        self.run(|lib| lib.unread_activity()).await
    }

    pub async fn mark_activity_read(&self) -> Result<()> {
        self.run(|lib| lib.mark_activity_read()).await
    }

    /// Notes what a hub sync brought (per device that made the changes), a key that changed, and
    /// devices new to this one.
    pub(super) async fn record_sync(&self, report: &HubReport) -> Result<()> {
        let me = self.device.to_string();
        let mut by_device: Vec<(Option<String>, Counts)> = Vec::new();
        for (author, kind, count) in report.arrived.iter().filter(|(author, _, _)| *author != me) {
            let name = name_of_author(&report.devices, author);
            match by_device.iter_mut().find(|(n, _)| *n == name) {
                Some((_, kinds)) => kinds.push((kind.clone(), *count)),
                None => by_device.push((name, vec![(kind.clone(), *count)])),
            }
        }
        for (name, kinds) in by_device {
            let detail = kinds.iter().map(|(k, n)| format!("{k}={n}")).collect::<Vec<_>>().join(",");
            self.record_activity(kind::SYNC, name.as_deref(), &detail).await?;
        }
        if let Some(epoch) = report.key_received {
            self.record_activity(kind::KEY_CHANGED, None, &epoch.to_string()).await?;
        }
        self.note_devices(&report.devices).await
    }

    /// Notes devices this one had not seen yet (the first list only teaches them).
    pub(super) async fn note_devices(&self, devices: &[HubDevice]) -> Result<()> {
        if devices.is_empty() {
            return Ok(());
        }
        let (mine, _) = self.sync_id()?;
        let list: Vec<(String, Option<String>, bool)> =
            devices.iter().map(|d| (d.device.clone(), d.name.clone(), d.revoked)).collect();
        let at = now();
        let new: Vec<Option<String>> = self
            .run(move |lib| {
                let first = !lib.knows_devices()?;
                let mut new = Vec::new();
                for (device, name, revoked) in list {
                    if lib.see_known_device(&device, at)? && !first && device != mine && !revoked {
                        new.push(name);
                    }
                }
                Ok(new)
            })
            .await?;
        for name in new {
            self.record_activity(kind::NEW_DEVICE, name.as_deref(), "").await?;
        }
        Ok(())
    }
}

/// The name of the device that syncs the library copy `author`.
fn name_of_author(devices: &[HubDevice], author: &str) -> Option<String> {
    devices.iter().find(|d| d.authors.iter().any(|a| a == author)).and_then(|d| d.name.clone())
}
