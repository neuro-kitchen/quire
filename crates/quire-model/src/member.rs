//! Sharing a project with other people: [`UserId`] (a person's public key; their devices share
//! it), [`Role`] (what a member may do), [`ProjectMember`] (one person's role on a project,
//! granted by an owner). A project shared with someone is visible to them with its subprojects,
//! their references and PDFs; tags, notes, highlights and reading status stay private
//! (`.knowledge/sync.md`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{ModelError, Result};
use crate::id::{DeviceId, ProjectId};
use crate::record::RecordMeta;

/// A person, by the public key their devices sign with: 64 lowercase hex digits (ed25519).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct UserId(String);

impl UserId {
    pub fn parse(text: &str) -> Result<Self> {
        let text = text.trim().to_ascii_lowercase();
        if text.len() == 64 && text.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Self(text))
        } else {
            Err(ModelError::InvalidUser(text))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first characters, enough to tell people apart on screen.
    pub fn short(&self) -> &str {
        &self.0[..12]
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for UserId {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<String> for UserId {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<UserId> for String {
    fn from(id: UserId) -> Self {
        id.0
    }
}

name_enum! {
    /// What a member may do with a shared project (and its subprojects).
    pub enum Role (unknown: ModelError::UnknownRole) {
        /// Sees the project, its references and PDFs.
        Reader => "reader",
        /// Also adds, edits and removes references, subprojects and PDFs.
        Editor => "editor",
        /// Also adds and removes members.
        Owner => "owner",
    }
}

impl Role {
    pub fn can_edit(self) -> bool {
        self >= Self::Editor
    }

    pub fn can_manage(self) -> bool {
        self == Self::Owner
    }
}

/// One person's role on one project. Removing a member keeps the row as a tombstone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMember {
    pub project: ProjectId,
    pub user: UserId,
    /// What the owner calls them (for lists; their own name is theirs to set).
    pub name: String,
    pub role: Role,
    /// The owner who wrote this version of the grant.
    pub granted_by: UserId,
    pub meta: RecordMeta,
}

impl ProjectMember {
    pub fn new(project: ProjectId, user: UserId, name: &str, role: Role, granted_by: UserId, device: DeviceId) -> Self {
        Self { project, user, name: name.trim().into(), role, granted_by, meta: RecordMeta::new(device) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn users_and_roles() {
        let id = "ab".repeat(32);
        assert_eq!(UserId::parse(&id.to_uppercase()).unwrap().as_str(), id);
        assert!(UserId::parse("abc").is_err());
        assert!(Role::Owner.can_manage() && Role::Owner.can_edit());
        assert!(Role::Editor.can_edit() && !Role::Editor.can_manage());
        assert!(!Role::Reader.can_edit());
        assert_eq!("editor".parse::<Role>().unwrap(), Role::Editor);
    }
}
