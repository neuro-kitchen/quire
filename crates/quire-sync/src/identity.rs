//! Who is on the other end: a person's key (ed25519, the same on all their devices; the engine
//! keeps it in the keyring) certifies each of their devices' keys. A session's `Hello` carries
//! the device's [`Certificate`]; the other side checks it against the device the connection was
//! authenticated as, and so knows the person (`.knowledge/sync.md`).

use iroh::{PublicKey, SecretKey, Signature};
use serde::{Deserialize, Serialize};

use crate::error::{Result, SyncError};

/// What a certificate signs, before the device ID (so the signature means nothing else).
const CONTEXT: &[u8] = b"quire device certificate v1\n";

/// A new user key (32 bytes, to keep secret).
pub fn new_user_secret() -> [u8; 32] {
    SecretKey::generate().to_bytes()
}

/// The public ID of the person with `secret`: 64 hex digits (`quire user id`).
pub fn user_id(secret: &[u8; 32]) -> String {
    hex(SecretKey::from_bytes(secret).public().as_bytes())
}

/// "The device `device` is mine", signed by a person's key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate {
    /// The person's public key (hex).
    pub user: String,
    /// The device's ID (its iroh endpoint ID).
    pub device: String,
    /// Hex.
    pub signature: String,
}

impl Certificate {
    /// The certificate of `device`, signed with the user key `secret`.
    pub fn issue(secret: &[u8; 32], device: &str) -> Self {
        let key = SecretKey::from_bytes(secret);
        let signature = key.sign(&message(device));
        Self { user: hex(key.public().as_bytes()), device: device.to_string(), signature: hex(&signature.to_bytes()) }
    }

    /// The person this certificate names, if it is valid and for `device`.
    pub fn verify(&self, device: &str) -> Result<String> {
        let invalid = |why: &str| SyncError::Protocol(format!("the other device's certificate {why}"));
        if self.device != device {
            return Err(invalid("is for another device"));
        }
        let key: [u8; 32] = unhex(&self.user).and_then(|b| b.try_into().ok()).ok_or_else(|| invalid("names no key"))?;
        let key = PublicKey::from_bytes(&key).map_err(|_| invalid("names no key"))?;
        let signature: [u8; 64] =
            unhex(&self.signature).and_then(|b| b.try_into().ok()).ok_or_else(|| invalid("is unreadable"))?;
        key.verify(&message(device), &Signature::from_bytes(&signature))
            .map_err(|_| invalid("is not signed by its key"))?;
        Ok(self.user.clone())
    }
}

fn message(device: &str) -> Vec<u8> {
    [CONTEXT, device.as_bytes()].concat()
}

/// A 32-byte key as 64 hex digits (how keys are kept and shown).
pub fn key_to_hex(key: &[u8; 32]) -> String {
    hex(key)
}

/// A 32-byte key from 64 hex digits.
pub fn key_from_hex(text: &str) -> Option<[u8; 32]> {
    unhex(text.trim())?.try_into().ok()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certificates_name_their_person_and_device() {
        let (alice, mallory) = (new_user_secret(), new_user_secret());
        let device = crate::transport::device_id(&crate::transport::new_secret());
        let certificate = Certificate::issue(&alice, &device);
        assert_eq!(certificate.verify(&device).unwrap(), user_id(&alice));
        let other = crate::transport::device_id(&crate::transport::new_secret());
        assert!(certificate.verify(&other).is_err(), "not for another device");
        // Mallory cannot claim to be Alice: a signature by her key does not verify as Alice's.
        let forged = Certificate { user: user_id(&alice), ..Certificate::issue(&mallory, &device) };
        assert!(forged.verify(&device).is_err());
    }
}
