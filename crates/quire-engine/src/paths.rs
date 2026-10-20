//! Where quire keeps things (`.knowledge/engineering.md`, Paths): XDG directories on Linux and
//! macOS (`etcetera`), Known Folders on Windows.
//!
//! - config file: `--config`, else `$QUIRE_CONFIG`, else `~/.config/quire/config.toml`;
//! - libraries: chosen by name or path ([`crate::libraries`]); the unregistered `default`
//!   library is `~/.local/share/quire/library/`, new ones go to `~/.local/share/quire/libraries/<name>/`.

use std::path::{Path, PathBuf};

use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};

use crate::error::{EngineError, Result};

pub const CONFIG_ENV: &str = "QUIRE_CONFIG";
/// A library folder (any path, registered or not).
pub const LIBRARY_ENV: &str = "QUIRE_LIBRARY";
/// A registered library's name.
pub const LIB_ENV: &str = "QUIRE_LIB";

/// The database file inside a library directory.
pub(crate) const DATABASE_FILE: &str = "quire.sqlite";

fn strategy() -> Result<impl AppStrategy> {
    let args = AppStrategyArgs { top_level_domain: "org".into(), author: "quire".into(), app_name: "quire".into() };
    choose_app_strategy(args).map_err(|e| EngineError::Config(format!("no home directory: {e}")))
}

/// The config file to use.
pub fn config_file(flag: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = flag {
        return Ok(path.into());
    }
    match std::env::var_os(CONFIG_ENV) {
        Some(path) => Ok(path.into()),
        None => Ok(strategy()?.config_dir().join("config.toml")),
    }
}

/// quire's own config folder (`~/.config/quire`), kept private.
pub(crate) fn config_dir() -> Result<PathBuf> {
    Ok(strategy()?.config_dir())
}

/// quire's own data folder (`~/.local/share/quire`), holding libraries; kept private.
pub(crate) fn data_dir() -> Result<PathBuf> {
    Ok(strategy()?.data_dir())
}

/// Folder of the `default` library when it is not registered (also where a single library lived
/// before named libraries).
pub(crate) fn default_library_dir() -> Result<PathBuf> {
    Ok(strategy()?.data_dir().join("library"))
}

/// Folder for a new library called `name` when no path is given.
pub(crate) fn new_library_dir(name: &str) -> Result<PathBuf> {
    Ok(strategy()?.data_dir().join("libraries").join(name))
}

/// `~/x` → `$HOME/x`; other paths unchanged.
pub fn expand_home(path: &Path) -> PathBuf {
    match (path.strip_prefix("~"), std::env::var_os("HOME")) {
        (Ok(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => path.into(),
    }
}
