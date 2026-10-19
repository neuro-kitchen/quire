//! The access token the browser extension sends: 32 random bytes as hex, kept in the system
//! keyring (`quire_engine::secrets`), or in a private `serve-token` file next to the config when
//! no keyring is available.

use std::path::{Path, PathBuf};

pub use quire_engine::secrets::Store;

/// Name of the secret in the keyring.
pub const SECRET: &str = "serve-token";

/// The fallback file for the config file `config`.
pub fn fallback_path(config: &Path) -> PathBuf {
    config.with_file_name(SECRET)
}

/// The token (created if missing) and where it is kept.
pub fn load_or_create(config: &Path) -> quire_engine::Result<(String, Store)> {
    quire_engine::secrets::get_or_create(SECRET, &fallback_path(config))
}

/// Deletes the token (a new one is made on the next start); `false` if there was none.
pub fn reset(config: &Path) -> quire_engine::Result<bool> {
    quire_engine::secrets::delete(SECRET, &fallback_path(config))
}
