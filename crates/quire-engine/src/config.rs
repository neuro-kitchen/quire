//! The user's configuration file (`config.toml`), handled like git's config
//! (`.knowledge/config.md`): `quire config get|set <key>`. Reading uses `toml` + serde; writing
//! uses `toml_edit` so the user's comments and layout are kept.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use toml_edit::{value, DocumentMut};

use crate::error::{EngineError, Result};

/// Keys `config get|set` accepts.
pub const KEYS: &[&str] = &["user.email", "user.name", "library.path"];

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub user: User,
    #[serde(default)]
    pub library: LibrarySettings,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct User {
    /// Sent to Crossref and NCBI to identify the user (their usage policies ask for it).
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct LibrarySettings {
    pub path: Option<PathBuf>,
}

impl Config {
    /// Reads `path`; a missing file is an empty configuration.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| EngineError::Config(format!("{}: {e}", path.display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(EngineError::file(path)(e)),
        }
    }

    /// The value of `key`, `None` if unset.
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(match check_key(key)? {
            "user.email" => self.user.email.clone(),
            "user.name" => self.user.name.clone(),
            _ => self.library.path.as_ref().map(|p| p.display().to_string()),
        })
    }
}

/// Sets `key` to `new` in the file at `path` (created with its directory if missing), keeping
/// everything else in the file as it was.
pub fn set(path: &Path, key: &str, new: &str) -> Result<()> {
    let key = check_key(key)?;
    if key == "user.email" && !looks_like_email(new) {
        return Err(EngineError::Config(format!("`{new}` is not an email address")));
    }
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(EngineError::file(path)(e)),
    };
    let mut document: DocumentMut = text.parse().map_err(|e| EngineError::Config(format!("{}: {e}", path.display())))?;
    let (table, field) = key.split_once('.').expect("keys are `table.field`");
    document[table][field] = value(new);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(EngineError::file(dir))?;
    }
    std::fs::write(path, document.to_string()).map_err(EngineError::file(path))
}

fn check_key(key: &str) -> Result<&str> {
    KEYS.iter().find(|k| **k == key).copied().ok_or_else(|| EngineError::Config(format!("unknown key `{key}` (known: {})", KEYS.join(", "))))
}

fn looks_like_email(text: &str) -> bool {
    matches!(text.split_once('@'), Some((user, domain)) if !user.is_empty() && domain.contains('.') && !text.contains(char::is_whitespace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_keeps_comments_and_get_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("quire/config.toml");
        assert_eq!(Config::load(&path).unwrap(), Config::default());

        set(&path, "user.email", "jane@example.org").unwrap();
        let with_comment = format!("# my settings\n{}", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, with_comment).unwrap();
        set(&path, "user.name", "Jane Doe").unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("# my settings"), "{text}");
        let config = Config::load(&path).unwrap();
        assert_eq!(config.get("user.email").unwrap().as_deref(), Some("jane@example.org"));
        assert_eq!(config.get("user.name").unwrap().as_deref(), Some("Jane Doe"));
        assert_eq!(config.get("library.path").unwrap(), None);
    }

    #[test]
    fn rejects_unknown_keys_and_bad_emails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert!(set(&path, "user.mail", "x").is_err());
        assert!(set(&path, "user.email", "not an email").is_err());
        assert!(Config::default().get("nope").is_err());
    }
}
