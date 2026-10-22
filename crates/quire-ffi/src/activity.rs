//! This device's notifications (`quire_engine::activity`): what syncs brought from which of the
//! person's devices, and security events. Kept on this device only.

use crate::{QuireLibrary, Result};

/// One notification; `kind` is one of `quire_engine::activity::kind` (`sync`, `linked`,
/// `link_allowed`, `link_refused`, `wrong_code`, `unlinked`, `key_changed`, `new_device`).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ActivityRecord {
    pub id: i64,
    /// Milliseconds since 1970.
    pub at: i64,
    pub kind: String,
    /// The device it concerns, by name.
    pub source: Option<String>,
    /// `sync`: `kind=count,…` (reference, note, highlight, reading, project, tag, attachment, …);
    /// `wrong_code`: how many; `unlinked`, `key_changed`: the key's new epoch.
    pub detail: String,
    pub read: bool,
}

#[uniffi::export]
impl QuireLibrary {
    /// The newest notifications first, at most `limit`.
    pub async fn activity(&self, limit: u32) -> Result<Vec<ActivityRecord>> {
        self.with(|engine| async move {
            Ok(engine
                .activity(limit)
                .await?
                .into_iter()
                .map(|a| ActivityRecord {
                    id: a.id,
                    at: a.at,
                    kind: a.kind,
                    source: a.source,
                    detail: a.detail,
                    read: a.read,
                })
                .collect())
        })
        .await
    }

    pub async fn unread_activity(&self) -> Result<u32> {
        self.with(|engine| async move { Ok(engine.unread_activity().await?) }).await
    }

    pub async fn mark_activity_read(&self) -> Result<()> {
        self.with(|engine| async move { Ok(engine.mark_activity_read().await?) }).await
    }
}
