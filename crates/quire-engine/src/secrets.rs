//! Secrets (the `quire serve` token now; sync and API credentials later) live in the system
//! keyring, as gh keeps its login: Secret Service (GNOME Keyring, KWallet) on Linux, Keychain on
//! macOS, Credential Manager on Windows. Where none is available (a server over SSH, a container,
//! or `QUIRE_NO_KEYRING=1`), a file only the user can read is used instead, and callers say so.
//!
//! Not secrets, so kept in `config.toml` (private, `600`): the contact email (sent to every
//! service as their terms ask) and the proxy address (public; the login stays in the browser).

use std::path::{Path, PathBuf};

use crate::error::{EngineError, Result};
use crate::private;

/// Keyring service name.
const SERVICE: &str = "quire";

/// Set to `1` to never use the keyring (tests, headless machines).
pub const NO_KEYRING_ENV: &str = "QUIRE_NO_KEYRING";

/// Where a secret is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Store {
    Keyring,
    /// The fallback file, readable by the user only.
    File(PathBuf),
}

/// The secret `name`, created (32 random bytes, hex) if it does not exist yet. `fallback` is the
/// file used without a keyring; a secret found there is moved into the keyring when one appears.
pub fn get_or_create(name: &str, fallback: &Path) -> Result<(String, Store)> {
    let from_file = read_file(fallback);
    match keyring_entry(name) {
        Some(entry) => match entry.get_password() {
            Ok(secret) => Ok((secret, Store::Keyring)),
            Err(keyring::Error::NoEntry) => {
                let secret = from_file.clone().map_or_else(random_hex, Ok)?;
                match entry.set_password(&secret) {
                    Ok(()) => {
                        if from_file.is_some() {
                            std::fs::remove_file(fallback).map_err(EngineError::file(fallback))?;
                        }
                        Ok((secret, Store::Keyring))
                    }
                    Err(error) => file_fallback(name, fallback, Some(secret), &error.to_string()),
                }
            }
            Err(error) => file_fallback(name, fallback, from_file, &error.to_string()),
        },
        None => file_fallback(name, fallback, from_file, "keyring disabled or unavailable"),
    }
}

/// The secret `name` if it exists (keyring first, then the fallback file).
pub fn get(name: &str, fallback: &Path) -> Option<(String, Store)> {
    if let Some(entry) = keyring_entry(name)
        && let Ok(secret) = entry.get_password()
    {
        return Some((secret, Store::Keyring));
    }
    read_any(fallback).map(|secret| (secret, Store::File(fallback.to_path_buf())))
}

/// Stores `secret` as `name`: in the keyring, else in the private fallback file.
pub fn set(name: &str, secret: &str, fallback: &Path) -> Result<Store> {
    if let Some(entry) = keyring_entry(name)
        && entry.set_password(secret).is_ok()
    {
        if fallback.is_file() {
            std::fs::remove_file(fallback).map_err(EngineError::file(fallback))?;
        }
        return Ok(Store::Keyring);
    }
    if let Some(dir) = fallback.parent() {
        private::dir(dir)?;
    }
    std::fs::write(fallback, secret).map_err(EngineError::file(fallback))?;
    private::file(fallback)?;
    Ok(Store::File(fallback.to_path_buf()))
}

/// Deletes the secret `name` from the keyring and the fallback file; `false` if neither had it.
pub fn delete(name: &str, fallback: &Path) -> Result<bool> {
    let in_keyring = keyring_entry(name).is_some_and(|entry| entry.delete_credential().is_ok());
    let in_file = fallback.is_file();
    if in_file {
        std::fs::remove_file(fallback).map_err(EngineError::file(fallback))?;
    }
    Ok(in_keyring || in_file)
}

fn keyring_entry(name: &str) -> Option<keyring::Entry> {
    if std::env::var_os(NO_KEYRING_ENV).is_some_and(|v| v == "1") {
        return None;
    }
    keyring::Entry::new(SERVICE, name).ok()
}

/// Uses (or creates) the private fallback file.
fn file_fallback(name: &str, file: &Path, existing: Option<String>, why: &str) -> Result<(String, Store)> {
    tracing::debug!(name, why, "secret kept in a private file instead of the keyring");
    let secret = match existing {
        Some(secret) => secret,
        None => {
            let secret = random_hex()?;
            if let Some(dir) = file.parent() {
                private::dir(dir)?;
            }
            std::fs::write(file, &secret).map_err(EngineError::file(file))?;
            secret
        }
    };
    private::file(file)?;
    Ok((secret, Store::File(file.to_path_buf())))
}

/// Any non-empty secret in `file`.
fn read_any(file: &Path) -> Option<String> {
    let text = std::fs::read_to_string(file).ok()?;
    Some(text.trim().to_string()).filter(|t| !t.is_empty())
}

fn read_file(file: &Path) -> Option<String> {
    let text = std::fs::read_to_string(file).ok()?;
    Some(text.trim().to_string()).filter(|t| t.len() >= 32)
}

/// 32 random bytes as hex.
fn random_hex() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| EngineError::Config(format!("no randomness for a secret: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_fallback_creates_once_and_is_private() {
        // SAFETY: tests in this module run with the keyring off; no other thread reads it.
        unsafe { std::env::set_var(NO_KEYRING_ENV, "1") };
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("serve-token");
        let (first, store) = get_or_create("test-secret", &file).unwrap();
        assert_eq!((first.len(), store), (64, Store::File(file.clone())));
        assert_eq!(get_or_create("test-secret", &file).unwrap().0, first);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o600);
        }
        assert!(delete("test-secret", &file).unwrap());
        assert!(!file.exists());
        assert_eq!(set("test-secret", "secret-value", &file).unwrap(), Store::File(file.clone()));
        assert_eq!(get("test-secret", &file).unwrap().0, "secret-value");
    }
}
