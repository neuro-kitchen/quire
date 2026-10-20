//! The settings screen: the keys `config set` accepts (from the engine's registry), the values set
//! in the config file, changing them one by one (comments and layout kept), and the raw file for
//! the TOML tab, checked before it is written.

use std::path::{Path, PathBuf};

use quire_engine::config::{self, Config, keys::ValueKind};
use quire_engine::{paths, private};

use crate::{QuireError, Result};

/// A settable key: `*` segments are names the user picks (`export.*.format`).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ConfigKey {
    pub pattern: String,
    /// `email`, `text`, `path`, `library`, `device`, `bool`, `list`, `choice` or `number`.
    pub kind: String,
    /// The words a `choice` accepts.
    pub choices: Vec<String>,
    /// Bounds of a `number`.
    pub min: Option<u32>,
    pub max: Option<u32>,
    pub help: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ConfigEntry {
    pub key: String,
    /// As text; lists comma-separated.
    pub value: String,
}

/// The config file in use: `config_file` if given, else `QUIRE_CONFIG`, else the usual one.
#[uniffi::export]
pub fn config_path(config_file: Option<String>) -> Result<String> {
    Ok(path(config_file)?.display().to_string())
}

#[uniffi::export]
pub fn config_keys() -> Vec<ConfigKey> {
    config::keys::KEYS
        .iter()
        .map(|spec| {
            let (kind, choices, min, max) = match spec.kind {
                ValueKind::Email => ("email", vec![], None, None),
                ValueKind::Text => ("text", vec![], None, None),
                ValueKind::Path => ("path", vec![], None, None),
                ValueKind::LibraryName => ("library", vec![], None, None),
                ValueKind::DeviceId => ("device", vec![], None, None),
                ValueKind::Bool => ("bool", vec![], None, None),
                ValueKind::List => ("list", vec![], None, None),
                ValueKind::Choice(words) => ("choice", words.iter().map(|w| w.to_string()).collect(), None, None),
                ValueKind::Number(min, max) => ("number", vec![], Some(min), Some(max)),
            };
            ConfigKey { pattern: spec.pattern.into(), kind: kind.into(), choices, min, max, help: spec.help.into() }
        })
        .collect()
}

/// Every value set in the file.
#[uniffi::export]
pub fn config_entries(config_file: Option<String>) -> Result<Vec<ConfigEntry>> {
    let config = Config::load(&path(config_file)?)?;
    Ok(config.entries().into_iter().map(|(key, value)| ConfigEntry { key, value }).collect())
}

/// Sets `key` to `value`, checked against its kind.
#[uniffi::export]
pub fn config_set(config_file: Option<String>, key: String, value: String) -> Result<()> {
    Ok(config::edit::set(&path(config_file)?, &key, &value)?)
}

/// Removes `key`; `false` if it was not set.
#[uniffi::export]
pub fn config_unset(config_file: Option<String>, key: String) -> Result<bool> {
    Ok(config::edit::unset(&path(config_file)?, &key)?)
}

/// The file's text (empty if it does not exist yet).
#[uniffi::export]
pub fn config_read(config_file: Option<String>) -> Result<String> {
    let path = path(config_file)?;
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(QuireError::Invalid(format!("{}: {e}", path.display()))),
    }
}

/// What is wrong with `text` as a config file: a syntax error, or keys quire does not know.
/// Empty if it can be saved as is.
#[uniffi::export]
pub fn config_check(text: String) -> Vec<String> {
    let dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => return vec![e.to_string()],
    };
    let file = dir.path().join("config.toml");
    if let Err(e) = std::fs::write(&file, &text) {
        return vec![e.to_string()];
    }
    match Config::load(&file) {
        Err(e) => vec![e.to_string().replace(&file.display().to_string(), "config.toml")],
        Ok(config) => config
            .entries()
            .into_iter()
            .filter_map(|(key, _)| config::keys::spec(&key).err().map(|_| format!("unknown key `{key}`")))
            .collect(),
    }
}

/// Replaces the file with `text` if it reads as a config file (unknown keys are kept: the user
/// was shown them by [`config_check`]). Written whole or not at all, readable by the user only.
#[uniffi::export]
pub fn config_write(config_file: Option<String>, text: String) -> Result<()> {
    let path = path(config_file)?;
    let partial = path.with_extension("toml.partial");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
    }
    std::fs::write(&partial, &text).map_err(|e| io(&partial, e))?;
    if let Err(e) = Config::load(&partial) {
        let _ = std::fs::remove_file(&partial);
        return Err(QuireError::Invalid(e.to_string().replace(&partial.display().to_string(), "config.toml")));
    }
    private::file(&partial)?;
    std::fs::rename(&partial, &path).map_err(|e| io(&path, e))?;
    Ok(())
}

pub(crate) fn path(config_file: Option<String>) -> Result<PathBuf> {
    Ok(paths::config_file(config_file.as_deref().map(Path::new))?)
}

fn io(path: &Path, e: std::io::Error) -> QuireError {
    QuireError::Invalid(format!("{}: {e}", path.display()))
}
