//! A library's keys over time, and how a new one reaches the person's devices
//! (`.tasks/T14-security.md`, rotation on unlink).
//!
//! The library key of epoch 0 is derived from the user key (`LibraryKey::derive`): every device
//! the person ever had can compute it. When a device is unlinked, another device makes a **random**
//! key for the next epoch, not derivable from the user key, and seals the whole chain (every
//! epoch's key, so old data stays readable) to each remaining device's **box key**: an X25519 key
//! pair each device makes for itself, whose public half it signs with its device key (ed25519,
//! its iroh identity), so a hub cannot swap in a key of its own. The hub stores these sealed
//! envelopes and cannot open them; the unlinked device is refused by the hub and has no box key
//! anyone seals to.
//!
//! Envelope: an ephemeral X25519 key, a nonce, then the chain sealed (XChaCha20-Poly1305) under a
//! key derived (BLAKE3) from the shared secret and both public keys; the associated data binds
//! the library, the device and the epoch.

use std::collections::BTreeMap;
use std::str::FromStr;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use curve25519_dalek::MontgomeryPoint;
use iroh::{PublicKey, SecretKey};

use crate::error::{Result, SyncError};
use crate::seal::{self, LibraryKey};

const ENVELOPE_CONTEXT: &str = "quire 2026-10-22 key envelope v1";
const BOX_SIGNATURE_CONTEXT: &[u8] = b"quire box key v1\0";
const NONCE: usize = 24;

/// A device's box key pair: the secret stays on the device (with its other secrets).
#[derive(Clone)]
pub struct BoxKey {
    secret: [u8; 32],
}

impl std::fmt::Debug for BoxKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BoxKey(…)")
    }
}

impl Drop for BoxKey {
    fn drop(&mut self) {
        self.secret = [0; 32];
    }
}

impl BoxKey {
    pub fn from_secret(secret: [u8; 32]) -> Self {
        Self { secret }
    }

    pub fn public(&self) -> [u8; 32] {
        MontgomeryPoint::mul_base_clamped(self.secret).to_bytes()
    }

    /// The public half signed with the device's key (`device_secret`, its iroh identity).
    pub fn signature(&self, device_secret: &[u8; 32]) -> Vec<u8> {
        let message = [BOX_SIGNATURE_CONTEXT, self.public().as_slice()].concat();
        SecretKey::from_bytes(device_secret).sign(&message).to_bytes().to_vec()
    }
}

/// Whether `signature` is `device`'s (its ID) over the box key `public`.
pub fn box_key_is_devices(device: &str, public: &[u8], signature: &[u8]) -> bool {
    let (Ok(key), Ok(public), Ok(signature)) =
        (PublicKey::from_str(device.trim()), <[u8; 32]>::try_from(public), <[u8; 64]>::try_from(signature))
    else {
        return false;
    };
    let message = [BOX_SIGNATURE_CONTEXT, public.as_slice()].concat();
    key.verify(&message, &iroh::Signature::from_bytes(&signature)).is_ok()
}

/// A library's keys by epoch; the newest seals what is sent now, any opens what was sealed then.
#[derive(Clone, Debug)]
pub struct LibraryKeys {
    keys: BTreeMap<u32, LibraryKey>,
}

impl LibraryKeys {
    /// Epoch 0 only: the key derived from the user key.
    pub fn new(base: LibraryKey) -> Self {
        Self { keys: BTreeMap::from([(0, base)]) }
    }

    /// The newest epoch and its key.
    pub fn current(&self) -> (u32, &LibraryKey) {
        let (epoch, key) = self.keys.last_key_value().expect("epoch 0 is always there");
        (*epoch, key)
    }

    pub fn get(&self, epoch: u32) -> Result<&LibraryKey> {
        self.keys.get(&epoch).ok_or_else(|| {
            SyncError::Link(format!(
                "this device lacks the library key of epoch {epoch} (a device was unlinked since): \
                 link it again from one of your devices"
            ))
        })
    }

    /// Epoch 0's key: file IDs stay under it (the same on every device, whatever the epoch).
    pub fn base(&self) -> &LibraryKey {
        &self.keys[&0]
    }

    /// The next epoch with a new random key (a rotation); the chain then includes it.
    pub fn rotate(&mut self) -> Result<u32> {
        let epoch = self.current().0 + 1;
        self.keys.insert(epoch, LibraryKey::from_bytes(seal::random()?));
        Ok(epoch)
    }

    /// Adds keys of another copy of the chain (epoch 0 is never replaced: it is derived here).
    pub fn merge(&mut self, chain: &[(u32, [u8; 32])]) {
        for (epoch, key) in chain {
            if *epoch > 0 {
                self.keys.entry(*epoch).or_insert_with(|| LibraryKey::from_bytes(*key));
            }
        }
    }

    /// The random keys (epochs above 0), to keep or to give.
    pub fn chain(&self) -> Vec<(u32, [u8; 32])> {
        self.keys.iter().filter(|(e, _)| **e > 0).map(|(e, k)| (*e, k.to_bytes())).collect()
    }

    /// The chain as text, to keep as a secret: `epoch:hex,epoch:hex`.
    pub fn chain_text(&self) -> String {
        self.chain()
            .iter()
            .map(|(e, k)| format!("{e}:{}", crate::identity::key_to_hex(k)))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Reads [`LibraryKeys::chain_text`]; parts that do not read are skipped.
    pub fn parse_chain(text: &str) -> Vec<(u32, [u8; 32])> {
        text.split(',')
            .filter_map(|part| {
                let (epoch, key) = part.trim().split_once(':')?;
                Some((epoch.parse().ok()?, crate::identity::key_from_hex(key)?))
            })
            .collect()
    }
}

/// The chain sealed for the device `device` (its box key `public`), as epoch `epoch` of `library`.
pub fn seal_chain(
    chain: &[(u32, [u8; 32])],
    public: &[u8; 32],
    library: &str,
    device: &str,
    epoch: u32,
) -> Result<Vec<u8>> {
    let ephemeral = BoxKey::from_secret(seal::random()?);
    let key = envelope_key(&ephemeral.secret, public, &ephemeral.public(), public)?;
    let plain: Vec<u8> = chain.iter().flat_map(|(e, k)| [e.to_be_bytes().as_slice(), k.as_slice()].concat()).collect();
    let nonce: [u8; NONCE] = seal::random()?;
    let sealed = XChaCha20Poly1305::new(&key.into())
        .encrypt(&XNonce::from(nonce), Payload { msg: &plain, aad: &envelope_aad(library, device, epoch) })
        .map_err(|_| SyncError::Protocol("could not seal the library keys".into()))?;
    Ok([ephemeral.public().as_slice(), nonce.as_slice(), &sealed].concat())
}

/// The chain in `sealed`, if it is epoch `epoch` of `library` for this device (`device`, `mine`).
pub fn open_chain(
    mine: &BoxKey,
    sealed: &[u8],
    library: &str,
    device: &str,
    epoch: u32,
) -> Result<Vec<(u32, [u8; 32])>> {
    let refused =
        || SyncError::Protocol("the library keys from the hub do not open (damaged, or not for this device)".into());
    if sealed.len() < 32 + NONCE + 16 {
        return Err(refused());
    }
    let (ephemeral, rest) = sealed.split_at(32);
    let (nonce, body) = rest.split_at(NONCE);
    let ephemeral: [u8; 32] = ephemeral.try_into().expect("split at 32");
    let key = envelope_key(&mine.secret, &ephemeral, &ephemeral, &mine.public())?;
    let nonce: [u8; NONCE] = nonce.try_into().expect("split at NONCE");
    let plain = XChaCha20Poly1305::new(&key.into())
        .decrypt(&XNonce::from(nonce), Payload { msg: body, aad: &envelope_aad(library, device, epoch) })
        .map_err(|_| refused())?;
    if plain.len() % 36 != 0 {
        return Err(refused());
    }
    Ok(plain
        .chunks(36)
        .map(|c| (u32::from_be_bytes(c[..4].try_into().expect("4 bytes")), c[4..].try_into().expect("32 bytes")))
        .collect())
}

/// The envelope key from our secret and their public key; `ephemeral` and `recipient` (the two
/// public keys) go in, so the key belongs to this exchange only.
fn envelope_key(secret: &[u8; 32], theirs: &[u8; 32], ephemeral: &[u8; 32], recipient: &[u8; 32]) -> Result<[u8; 32]> {
    let shared = MontgomeryPoint(*theirs).mul_clamped(*secret).to_bytes();
    // A low-order public key gives an all-zero secret: refused.
    if shared == [0; 32] {
        return Err(SyncError::Protocol("a box key that cannot be used".into()));
    }
    Ok(blake3::derive_key(ENVELOPE_CONTEXT, &[shared.as_slice(), ephemeral, recipient].concat()))
}

fn envelope_aad(library: &str, device: &str, epoch: u32) -> Vec<u8> {
    [b"quire key envelope v1\0".as_slice(), library.as_bytes(), b"\0", device.as_bytes(), b"\0", &epoch.to_be_bytes()]
        .concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(seed: u8) -> ([u8; 32], String) {
        let secret = [seed; 32];
        (secret, crate::transport::device_id(&secret))
    }

    #[test]
    fn a_box_key_is_signed_by_its_device_only() {
        let (secret, id) = device(1);
        let (_, other) = device(2);
        let boxed = BoxKey::from_secret([9; 32]);
        let signature = boxed.signature(&secret);
        assert!(box_key_is_devices(&id, &boxed.public(), &signature));
        assert!(!box_key_is_devices(&other, &boxed.public(), &signature), "another device's ID");
        let swapped = BoxKey::from_secret([8; 32]).public();
        assert!(!box_key_is_devices(&id, &swapped, &signature), "a key the hub swapped in");
    }

    #[test]
    fn the_chain_opens_for_its_device_epoch_and_library_only() {
        let mut keys = LibraryKeys::new(LibraryKey::derive(&[1; 32], "lib"));
        let first = keys.rotate().unwrap();
        let second = keys.rotate().unwrap();
        assert_eq!((first, second, keys.current().0), (1, 2, 2));
        let (mine, other) = (BoxKey::from_secret([5; 32]), BoxKey::from_secret([6; 32]));
        let sealed = seal_chain(&keys.chain(), &mine.public(), "lib", "dev", 2).unwrap();
        assert_eq!(open_chain(&mine, &sealed, "lib", "dev", 2).unwrap(), keys.chain());
        assert!(open_chain(&other, &sealed, "lib", "dev", 2).is_err(), "another device's box key");
        assert!(open_chain(&mine, &sealed, "lib", "dev", 3).is_err(), "another epoch");
        assert!(open_chain(&mine, &sealed, "other", "dev", 2).is_err(), "another library");
        let mut damaged = sealed.clone();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(open_chain(&mine, &damaged, "lib", "dev", 2).is_err());
    }

    #[test]
    fn a_rotated_key_is_not_derivable_and_the_chain_survives_text() {
        let base = LibraryKey::derive(&[1; 32], "lib");
        let mut keys = LibraryKeys::new(base.clone());
        keys.rotate().unwrap();
        assert_ne!(keys.current().1.to_bytes(), base.to_bytes());
        let mut copy = LibraryKeys::new(base);
        copy.merge(&LibraryKeys::parse_chain(&keys.chain_text()));
        assert_eq!(copy.chain(), keys.chain());
        assert!(copy.get(2).is_err(), "a missing epoch says so");
    }

    #[test]
    fn a_low_order_key_is_refused() {
        assert!(seal_chain(&[], &[0; 32], "lib", "dev", 1).is_err());
    }
}
