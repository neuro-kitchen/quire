//! Where quire keeps things (`.knowledge/engineering.md`, Paths): XDG directories on Linux and
//! macOS (`etcetera`), Known Folders on Windows.
//!
//! - config file: `$QUIRE_CONFIG`, else `~/.config/quire/config.toml`;
//! - library: `--library`, else `$QUIRE_LIBRARY`, else `library.path` in the config, else
//!   `~/.local/share/quire/library/` (holding `quire.sqlite`).

use std::path::{Path, PathBuf};

use etcetera::{choose_app_strategy, AppStrategy, AppStrategyArgs};

use crate::error::{EngineError, Result};

pub const CONFIG_ENV: &str = "QUIRE_CONFIG";
pub const LIBRARY_ENV: &str = "QUIRE_LIBRARY";

/// The database file inside a library directory.
pub const DATABASE_FILE: &str = "quire.sqlite";

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

/// The library directory to use: flag, environment, config, default.
pub fn library_dir(flag: Option<&Path>, configured: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = flag {
        return Ok(path.into());
    }
    if let Some(path) = std::env::var_os(LIBRARY_ENV) {
        return Ok(path.into());
    }
    match configured {
        Some(path) => Ok(path.into()),
        None => Ok(strategy()?.data_dir().join("library")),
    }
}
