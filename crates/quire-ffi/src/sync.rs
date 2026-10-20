//! Sync between the user's devices and sharing projects with collaborators. Peers are config
//! entries (`peers.<name>.id`, set through the config functions). Serving sync to other devices
//! is a long-running service, not a call: the app runs `quire sync serve` (or a later API) for it.

use std::str::FromStr;

use quire_engine::{MemberRole, ProjectMember, UserId};

use crate::{QuireError, QuireLibrary, Result, query};

/// What a sync session did.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SyncReportRecord {
    pub peer: String,
    /// The other person, when the peer is a collaborator's device.
    pub collaborator: Option<String>,
    /// This copy joined the peer's library (it was new).
    pub joined: bool,
    pub sent: u32,
    /// Rows received and written; kept (older than what was here); refused (stamped in the future).
    pub applied: u32,
    pub kept: u32,
    pub refused: u32,
    pub files_sent: u32,
    pub files_received: u32,
}

/// A person a project is shared with: `role` `reader`, `editor` or `owner`.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct MemberRecord {
    pub user: String,
    pub name: String,
    pub role: String,
}

#[uniffi::export]
impl QuireLibrary {
    /// This device's ID: what the other devices add as a peer.
    pub async fn device_id(&self) -> Result<String> {
        self.with(|engine| async move { Ok(engine.sync_id()?.0) }).await
    }

    /// This person's user ID: what collaborators share projects with.
    pub async fn user_id(&self) -> Result<String> {
        self.with(|engine| async move { Ok(engine.user_id()?.0.to_string()) }).await
    }

    /// The user key, to give another of the user's devices (keep it secret).
    pub async fn export_user_key(&self) -> Result<String> {
        self.with(|engine| async move { Ok(engine.export_user_key()?) }).await
    }

    /// Makes another device's exported key this device's user key; returns the user ID.
    pub async fn import_user_key(&self, key: String) -> Result<String> {
        self.with(|engine| async move { Ok(engine.import_user_key(&key)?.0.to_string()) }).await
    }

    /// Syncs with the device `peer` (a peer's name from the config, or a device ID).
    pub async fn sync_with(&self, peer: String) -> Result<SyncReportRecord> {
        self.with(|engine| async move {
            let report = engine.sync_with(&peer).await?;
            Ok(SyncReportRecord {
                peer: report.peer,
                collaborator: report.collaborator,
                joined: report.joined,
                sent: report.sent as u32,
                applied: report.received.applied as u32,
                kept: report.received.kept as u32,
                refused: report.received.refused as u32,
                files_sent: report.files_sent as u32,
                files_received: report.files_received as u32,
            })
        })
        .await
    }

    /// Shares the project at `path` with `user` (their user ID, named `name`) as `role`.
    pub async fn share_project(&self, path: String, user: String, name: String, role: String) -> Result<MemberRecord> {
        self.with(|engine| async move {
            let role = MemberRole::from_str(&role)
                .map_err(|_| QuireError::Invalid(format!("`{role}`: reader, editor or owner")))?;
            Ok(member(engine.share_project(&query::slug_path(&path)?, &user_id(&user)?, &name, role).await?))
        })
        .await
    }

    pub async fn unshare_project(&self, path: String, user: String) -> Result<()> {
        self.with(|engine| async move {
            engine.unshare_project(&query::slug_path(&path)?, &user_id(&user)?).await?;
            Ok(())
        })
        .await
    }

    /// The people the project at `path` is shared with.
    pub async fn project_members(&self, path: String) -> Result<Vec<MemberRecord>> {
        self.with(|engine| async move {
            Ok(engine.project_members(&query::slug_path(&path)?).await?.1.into_iter().map(member).collect())
        })
        .await
    }
}

fn member(member: ProjectMember) -> MemberRecord {
    MemberRecord { user: member.user.to_string(), name: member.name, role: member.role.as_str().to_string() }
}

fn user_id(text: &str) -> Result<UserId> {
    UserId::parse(text).map_err(|_| QuireError::Invalid(format!("`{text}` is not a user ID")))
}
