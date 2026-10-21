//! The user's configuration file (`config.toml`), handled like git's config
//! (`.knowledge/config.md`): `quire config get|set|unset|list`. Reading uses `toml` + serde;
//! writing uses `toml_edit` so the user's comments and layout are kept.
//!
//! - [`keys`]: which keys exist and how values are checked.
//! - [`edit`]: changing the file.

pub mod edit;
pub mod keys;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub use edit::{set, unset};
pub use keys::{KEYS, KeySpec, ValueKind};

use crate::error::{EngineError, Result};

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub user: User,
    #[serde(default)]
    pub library: LibrarySettings,
    /// Registered libraries by name (`[libraries.<name>]`).
    #[serde(default)]
    pub libraries: BTreeMap<String, LibraryEntry>,
    #[serde(default)]
    pub access: Access,
    #[serde(default)]
    pub reader: ReaderSettings,
    /// Export profiles by name (`[export.<name>]`); `default` applies when none is named.
    #[serde(default)]
    pub export: BTreeMap<String, ExportProfile>,
    /// Devices this one syncs with, by name (`[peers.<name>]`).
    #[serde(default)]
    pub peers: BTreeMap<String, PeerEntry>,
    /// The sync hub this device syncs through (`[hub]`).
    #[serde(default)]
    pub hub: HubEntry,
    /// How this device keeps PDF files (`[files]`): all, or as they are opened.
    #[serde(default)]
    pub files: FilesSettings,
    /// The file as read, for `get` and `list`.
    #[serde(skip)]
    raw: toml::Table,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct User {
    /// Sent to Crossref, NCBI and Unpaywall to identify the user (their usage policies ask for it).
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct LibrarySettings {
    /// Name of the library used when none is chosen.
    pub default: Option<String>,
    /// Before named libraries: the one library's folder. Moved by [`migrate`].
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LibraryEntry {
    pub path: PathBuf,
}

/// Another device of the user's (`quire sync peer add`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PeerEntry {
    /// Its device ID (`quire sync id` on it).
    pub id: String,
}

/// The sync hub (`.knowledge/sync-hub.md`): an always-on store of this person's devices' sealed
/// data, which cannot read it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct HubEntry {
    /// Its ID (`quire-hub id`, or printed when the hub starts).
    pub id: Option<String>,
}

/// How this device keeps PDF files (`.knowledge/sync-hub.md`, lazy PDFs).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct FilesSettings {
    /// `all` (default: download every PDF at sync) or `on-open` (only when opened, and those kept
    /// offline).
    pub download: Option<String>,
    /// Space for PDFs in megabytes; past it, files the hub holds are removed here, least recently
    /// opened first, never one kept offline (0 or unset: no limit).
    pub cache_mb: Option<u32>,
}

impl FilesSettings {
    /// Whether PDFs are downloaded only when opened.
    pub fn on_open(&self) -> bool {
        self.download.as_deref() == Some("on-open")
    }
}

/// Getting PDFs by hand in the user's own browser (`pdf get --browser`), where they log in with
/// the publisher themselves; quire never handles a login.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Access {
    /// Where the browser saves downloads; watched by `quire pdf get --browser`.
    pub downloads: Option<PathBuf>,
}

/// The TUI reader's layout (`reader.*`); unset values use the reader's defaults.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ReaderSettings {
    pub width: Option<u16>,
    pub justify: Option<bool>,
    /// `"1"` or `"2"`.
    pub spacing: Option<String>,
    pub theme: Option<String>,
    /// Tesseract's language codes for OCR of scans (`eng`, `deu`, `eng+fra`).
    pub ocr_language: Option<String>,
}

/// How to write an export (Better BibTeX's export options).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct ExportProfile {
    /// `biblatex` (default), `bibtex`, `csl-json`, `hayagriva`.
    pub format: Option<String>,
    /// Fields left out (BibLaTeX names for .bib and Hayagriva; CSL names for CSL-JSON).
    pub omit: Vec<String>,
    /// .bib: non-ASCII as LaTeX commands.
    pub ascii: bool,
    /// BibTeX months: `number` (default) or `macro` (`jul`).
    pub months: Option<String>,
    /// .bib preprints: `online` (default, for Biber) or `article` (for Typst).
    pub preprints: Option<String>,
}

impl Config {
    /// Reads `path`; a missing file is an empty configuration.
    pub fn load(path: &Path) -> Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(EngineError::file(path)(e)),
        };
        let invalid = |e: toml::de::Error| EngineError::Config(format!("{}: {e}", path.display()));
        let mut config: Self = toml::from_str(&text).map_err(invalid)?;
        config.raw = toml::from_str(&text).map_err(invalid)?;
        Ok(config)
    }

    /// The value of `key` as text (lists comma-separated), `None` if unset.
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        keys::spec(key)?;
        let mut value = None;
        let mut table = Some(&self.raw);
        for part in key.split('.') {
            value = table.and_then(|t| t.get(part));
            table = value.and_then(toml::Value::as_table);
        }
        Ok(value.map(display))
    }

    /// Every set value as `(key, value)`, in file order of tables, keys sorted within them.
    pub fn entries(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        flatten("", &self.raw, &mut out);
        out
    }
}

fn flatten(prefix: &str, table: &toml::Table, out: &mut Vec<(String, String)>) {
    for (name, value) in table {
        let key = format!("{prefix}{name}");
        match value {
            toml::Value::Table(inner) => flatten(&format!("{key}."), inner, out),
            other => out.push((key, display(other))),
        }
    }
}

fn display(value: &toml::Value) -> String {
    match value {
        toml::Value::String(text) => text.clone(),
        toml::Value::Array(items) => items.iter().map(display).collect::<Vec<_>>().join(","),
        other => other.to_string(),
    }
}

/// Name given to the library a pre-0.2 `library.path` pointed to.
pub(crate) const MIGRATED_NAME: &str = "default";

/// Moves a pre-0.2 `library.path` into `[libraries.default]` with `library.default = "default"`,
/// after copying the file to `config.toml.bak`. Returns a message for the user, `None` if
/// nothing needed moving.
pub fn migrate(path: &Path) -> Result<Option<String>> {
    let config = Config::load(path)?;
    let Some(old) = config.library.path.clone() else { return Ok(None) };
    let backup = path.with_extension("toml.bak");
    std::fs::copy(path, &backup).map_err(EngineError::file(&backup))?;
    let name = if config.libraries.contains_key(MIGRATED_NAME) { "migrated" } else { MIGRATED_NAME };
    edit::edit(path, |document| {
        let library = edit::table_at(document, &["library"])?;
        library.remove("path");
        if !library.contains_key("default") {
            library.insert("default", toml_edit::value(name));
        }
        edit::table_at(document, &["libraries", name])?.insert("path", toml_edit::value(old.display().to_string()));
        Ok(())
    })?;
    Ok(Some(format!(
        "config: `library.path` is now the library `{name}` ([libraries.{name}]); old file kept as {}",
        backup.display()
    )))
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
        set(&path, "libraries.lab.path", "/data/lab").unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("# my settings"), "{text}");
        assert!(text.contains("[libraries.lab]") && !text.contains("[libraries]\n"), "{text}");
        let config = Config::load(&path).unwrap();
        assert_eq!(config.get("user.email").unwrap().as_deref(), Some("jane@example.org"));
        assert_eq!(config.get("libraries.lab.path").unwrap().as_deref(), Some("/data/lab"));
        assert_eq!(config.libraries["lab"].path, PathBuf::from("/data/lab"));
        assert_eq!(config.get("library.default").unwrap(), None);
        assert_eq!(config.entries().len(), 3);

        assert!(unset(&path, "user.name").unwrap());
        assert!(!unset(&path, "user.name").unwrap());
    }

    #[test]
    fn rejects_unknown_keys_and_bad_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert!(set(&path, "user.mail", "x").is_err());
        assert!(set(&path, "user.email", "not an email").is_err());
        assert!(Config::default().get("nope").is_err());
    }

    #[test]
    fn migrates_library_path_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "# mine\n[user]\nemail = \"a@b.org\"\n\n[library]\npath = \"/old/lib\"\n").unwrap();
        let message = migrate(&path).unwrap().unwrap();
        assert!(message.contains("libraries.default"), "{message}");
        assert!(dir.path().join("config.toml.bak").exists());
        let config = Config::load(&path).unwrap();
        assert_eq!(config.library.path, None);
        assert_eq!(config.library.default.as_deref(), Some("default"));
        assert_eq!(config.libraries["default"].path, PathBuf::from("/old/lib"));
        assert!(std::fs::read_to_string(&path).unwrap().starts_with("# mine"));
        assert_eq!(migrate(&path).unwrap(), None);
    }
}
