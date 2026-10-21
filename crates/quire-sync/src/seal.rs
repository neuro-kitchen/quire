//! Encryption on the device, before anything reaches a hub (`.knowledge/sync-hub.md`): the hub
//! stores sealed change batches and sealed files and can read neither.
//!
//! - The [`LibraryKey`] is derived from the user key and the library ID, so every device of the
//!   same person computes the same key and nothing has to be carried between them; the user key
//!   is the whole recovery kit.
//! - A batch is XChaCha20-Poly1305 under a subkey, with a random nonce; the associated data binds
//!   the library, the author device, its batch number and the kind of payload, so a hub cannot move
//!   a batch to another library or pass one kind off as another.
//! - A file is sealed in chunks (a STREAM construction: a random nonce prefix, the chunk number and
//!   a last-chunk flag in each nonce), so it is neither truncated nor reordered unnoticed. The hub
//!   names it by [`LibraryKey::file_id`], a keyed hash: the plain BLAKE3 hash that names files on
//!   devices would let the hub check whether someone holds a known paper.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

use crate::error::{Result, SyncError};

/// Context strings of `blake3::derive_key`: each subkey means one thing only.
const LIBRARY_CONTEXT: &str = "quire 2026-10-20 library key v1";
const BATCH_CONTEXT: &str = "quire 2026-10-20 batch key v1";
const FILE_CONTEXT: &str = "quire 2026-10-20 file key v1";
const FILE_ID_CONTEXT: &str = "quire 2026-10-20 file id v1";

const TAG: usize = 16;
const NONCE: usize = 24;
/// Plain bytes per file chunk.
const CHUNK: usize = 64 * 1024;
/// A sealed file starts with this, then the nonce prefix.
const FILE_MAGIC: &[u8; 4] = b"QSF1";
const PREFIX: usize = NONCE - 5;

/// What a sealed batch holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Changed rows.
    Changes,
    /// The whole library's rows (compaction: a new device starts from it).
    Snapshot,
}

impl Kind {
    fn tag(self) -> &'static [u8] {
        match self {
            Kind::Changes => b"changes",
            Kind::Snapshot => b"snapshot",
        }
    }
}

/// The key a library's data is sealed with on its way to a hub.
#[derive(Clone)]
pub struct LibraryKey([u8; 32]);

impl std::fmt::Debug for LibraryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LibraryKey(…)")
    }
}

impl Drop for LibraryKey {
    fn drop(&mut self) {
        // Best effort: the key does not outlive its use in this copy.
        self.0 = [0; 32];
    }
}

impl LibraryKey {
    /// The key of `library` for the person with the user key `user_secret`.
    pub fn derive(user_secret: &[u8; 32], library: &str) -> Self {
        let material = [user_secret.as_slice(), library.as_bytes()].concat();
        Self(blake3::derive_key(LIBRARY_CONTEXT, &material))
    }

    fn subkey(&self, context: &str) -> [u8; 32] {
        blake3::derive_key(context, &self.0)
    }

    /// How the hub names the file whose plain BLAKE3 hash is `hash` (hex): the same on all of this
    /// library's devices, unrelated to the hash itself without the key.
    pub fn file_id(&self, hash: &str) -> String {
        hex(blake3::keyed_hash(&self.subkey(FILE_ID_CONTEXT), hash.as_bytes()).as_bytes())
    }

    /// `plain` sealed as batch `number` of `device` in `library`: nonce, then ciphertext.
    pub fn seal_batch(&self, kind: Kind, library: &str, device: &str, number: u64, plain: &[u8]) -> Result<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new(&self.subkey(BATCH_CONTEXT).into());
        let nonce = random_nonce()?;
        let aad = batch_aad(kind, library, device, number);
        let sealed = cipher
            .encrypt(&XNonce::from(nonce), Payload { msg: plain, aad: &aad })
            .map_err(|_| SyncError::Protocol("could not seal a batch".into()))?;
        Ok([nonce.as_slice(), &sealed].concat())
    }

    /// The plain batch, if `sealed` is batch `number` of `device` in `library`, of `kind`, untouched.
    pub fn open_batch(&self, kind: Kind, library: &str, device: &str, number: u64, sealed: &[u8]) -> Result<Vec<u8>> {
        if sealed.len() < NONCE + TAG {
            return Err(refused("a batch"));
        }
        let (nonce, body) = sealed.split_at(NONCE);
        let cipher = XChaCha20Poly1305::new(&self.subkey(BATCH_CONTEXT).into());
        let aad = batch_aad(kind, library, device, number);
        let nonce: [u8; NONCE] = nonce.try_into().expect("split at NONCE");
        cipher.decrypt(&XNonce::from(nonce), Payload { msg: body, aad: &aad }).map_err(|_| refused("a batch"))
    }

    /// `plain` sealed as the file the hub names `file_id`.
    pub fn seal_file(&self, file_id: &str, plain: &[u8]) -> Result<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new(&self.subkey(FILE_CONTEXT).into());
        let prefix: [u8; PREFIX] = random()?;
        let chunks = plain.len().div_ceil(CHUNK).max(1);
        let mut out = Vec::with_capacity(FILE_MAGIC.len() + PREFIX + plain.len() + chunks * TAG);
        out.extend_from_slice(FILE_MAGIC);
        out.extend_from_slice(&prefix);
        for index in 0..chunks {
            let part = &plain[(index * CHUNK).min(plain.len())..((index + 1) * CHUNK).min(plain.len())];
            let nonce = chunk_nonce(&prefix, index, index + 1 == chunks)?;
            let sealed = cipher
                .encrypt(&XNonce::from(nonce), Payload { msg: part, aad: file_id.as_bytes() })
                .map_err(|_| SyncError::Protocol("could not seal a file".into()))?;
            out.extend_from_slice(&sealed);
        }
        Ok(out)
    }

    /// The plain file, if `sealed` is the whole, untouched file the hub names `file_id`.
    pub fn open_file(&self, file_id: &str, sealed: &[u8]) -> Result<Vec<u8>> {
        let header = FILE_MAGIC.len() + PREFIX;
        if sealed.len() < header + TAG || &sealed[..FILE_MAGIC.len()] != FILE_MAGIC {
            return Err(refused("a file"));
        }
        let prefix: [u8; PREFIX] = sealed[FILE_MAGIC.len()..header].try_into().expect("sliced to PREFIX");
        let body = &sealed[header..];
        let cipher = XChaCha20Poly1305::new(&self.subkey(FILE_CONTEXT).into());
        let chunks = body.len().div_ceil(CHUNK + TAG);
        let mut out = Vec::with_capacity(body.len());
        for index in 0..chunks {
            let part = &body[index * (CHUNK + TAG)..((index + 1) * (CHUNK + TAG)).min(body.len())];
            // The last chunk is sealed with the last flag: a file cut after a whole chunk fails here.
            let nonce = chunk_nonce(&prefix, index, index + 1 == chunks)?;
            let plain = cipher
                .decrypt(&XNonce::from(nonce), Payload { msg: part, aad: file_id.as_bytes() })
                .map_err(|_| refused("a file"))?;
            out.extend_from_slice(&plain);
        }
        Ok(out)
    }
}

fn batch_aad(kind: Kind, library: &str, device: &str, number: u64) -> Vec<u8> {
    let mut aad = b"quire batch v1\0".to_vec();
    for part in [kind.tag(), library.as_bytes(), device.as_bytes()] {
        aad.extend_from_slice(part);
        aad.push(0);
    }
    aad.extend_from_slice(&number.to_be_bytes());
    aad
}

fn chunk_nonce(prefix: &[u8; PREFIX], index: usize, last: bool) -> Result<[u8; NONCE]> {
    let index = u32::try_from(index).map_err(|_| SyncError::TooLarge)?;
    let mut nonce = [0; NONCE];
    nonce[..PREFIX].copy_from_slice(prefix);
    nonce[PREFIX..PREFIX + 4].copy_from_slice(&index.to_be_bytes());
    nonce[NONCE - 1] = u8::from(last);
    Ok(nonce)
}

fn random_nonce() -> Result<[u8; NONCE]> {
    random()
}

fn random<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|e| SyncError::Protocol(format!("no randomness: {e}")))?;
    Ok(bytes)
}

fn refused(what: &str) -> SyncError {
    SyncError::Protocol(format!("{what} from the hub failed its integrity check (damaged, or not this library's)"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(user: u8, library: &str) -> LibraryKey {
        LibraryKey::derive(&[user; 32], library)
    }

    #[test]
    fn the_same_person_derives_the_same_key_on_every_device() {
        let (a, b) = (key(1, "lib"), key(1, "lib"));
        assert_eq!(a.file_id("h"), b.file_id("h"));
        assert_ne!(a.file_id("h"), key(2, "lib").file_id("h"), "another person");
        assert_ne!(a.file_id("h"), key(1, "other").file_id("h"), "another library");
        assert_ne!(a.file_id("h"), "h");
        assert_eq!(a.file_id("h").len(), 64);
    }

    #[test]
    fn batches_open_only_where_they_were_sealed() {
        let k = key(1, "lib");
        let sealed = k.seal_batch(Kind::Changes, "lib", "dev", 7, b"rows").unwrap();
        assert_eq!(k.open_batch(Kind::Changes, "lib", "dev", 7, &sealed).unwrap(), b"rows");
        assert!(!sealed.windows(4).any(|w| w == b"rows"), "no plain text");
        assert_ne!(sealed, k.seal_batch(Kind::Changes, "lib", "dev", 7, b"rows").unwrap(), "fresh nonce");
        for (kind, library, device, number) in [
            (Kind::Snapshot, "lib", "dev", 7),
            (Kind::Changes, "other", "dev", 7),
            (Kind::Changes, "lib", "x", 7),
            (Kind::Changes, "lib", "dev", 8),
        ] {
            assert!(
                k.open_batch(kind, library, device, number, &sealed).is_err(),
                "{kind:?} {library} {device} {number}"
            );
        }
        assert!(key(2, "lib").open_batch(Kind::Changes, "lib", "dev", 7, &sealed).is_err(), "another person's key");
        let mut tampered = sealed.clone();
        *tampered.last_mut().unwrap() ^= 1;
        assert!(k.open_batch(Kind::Changes, "lib", "dev", 7, &tampered).is_err());
        assert!(k.open_batch(Kind::Changes, "lib", "dev", 7, &sealed[..10]).is_err());
    }

    #[test]
    fn files_round_trip_and_refuse_cuts_swaps_and_moves() {
        let k = key(1, "lib");
        for size in [0, 1, CHUNK - 1, CHUNK, CHUNK + 1, 3 * CHUNK + 17] {
            let plain: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
            let sealed = k.seal_file("f", &plain).unwrap();
            assert_eq!(k.open_file("f", &sealed).unwrap(), plain, "size {size}");
            assert!(k.open_file("g", &sealed).is_err(), "named as another file");
        }
        let plain = vec![7u8; 3 * CHUNK];
        let sealed = k.seal_file("f", &plain).unwrap();
        let header = FILE_MAGIC.len() + PREFIX;
        // Cut after two whole chunks: the second was not sealed as the last.
        assert!(k.open_file("f", &sealed[..header + 2 * (CHUNK + TAG)]).is_err(), "truncated");
        // Swap the first two chunks.
        let mut swapped = sealed[..header].to_vec();
        swapped.extend_from_slice(&sealed[header + CHUNK + TAG..header + 2 * (CHUNK + TAG)]);
        swapped.extend_from_slice(&sealed[header..header + CHUNK + TAG]);
        swapped.extend_from_slice(&sealed[header + 2 * (CHUNK + TAG)..]);
        assert!(k.open_file("f", &swapped).is_err(), "reordered");
        let mut flipped = sealed.clone();
        flipped[header + 5] ^= 1;
        assert!(k.open_file("f", &flipped).is_err(), "tampered");
        assert!(key(2, "lib").open_file("f", &sealed).is_err(), "another person's key");
    }
}
