//! Sync of [`Engine`] with the user's other devices (`quire_sync`): this device's key (in the
//! keyring, like the server token), its peers (`[peers.<name>]` in the config), and the library
//! as a [`Replica`] of the session: the store's change log and merge, and the file store.

use std::path::PathBuf;

use quire_sync::transport::{self, Device};
use quire_sync::{Replica, Report, Role, SyncError};

use super::Engine;
use crate::error::{EngineError, Result};
use crate::files::FileStore;
use crate::secrets::{self, Store};

/// Name of this device's sync key in the keyring.
const SECRET: &str = "sync-device-key";

impl Engine {
    /// This device's sync key, created on first use, and where it is kept.
    fn sync_key(&self) -> Result<([u8; 32], Store)> {
        let fallback = self.config_file().with_file_name(SECRET);
        let (hex, store) = secrets::get_or_create(SECRET, &fallback)?;
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(hex.get(i..i + 2).unwrap_or("zz"), 16))
            .collect::<std::result::Result<Vec<u8>, _>>()
            .ok()
            .and_then(|b| <[u8; 32]>::try_from(b).ok())
            .ok_or_else(|| EngineError::Config(format!("the sync key `{SECRET}` is damaged")))?;
        Ok((bytes, store))
    }

    /// This device's ID (what the other devices add as a peer) and where its key is kept.
    pub fn sync_id(&self) -> Result<(String, Store)> {
        let (key, store) = self.sync_key()?;
        Ok((transport::device_id(&key), store))
    }

    /// The device ID `peer` names: a peer's name from the config, or an ID itself.
    pub fn peer_id(&self, peer: &str) -> Result<String> {
        if let Some(found) = self.config.peers.get(peer) {
            return Ok(found.id.clone());
        }
        if transport::is_device_id(peer) {
            return Ok(peer.trim().to_string());
        }
        Err(EngineError::Config(format!("no peer `{peer}`: `quire sync peer add {peer} <device-id>`")))
    }

    /// Syncs this library with the device `peer` (a peer's name or a device ID).
    pub async fn sync_with(&self, peer: &str) -> Result<Report> {
        let id = self.peer_id(peer)?;
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key).await?;
        let report = device.sync_with(&LibraryReplica(self), &id).await;
        device.close().await;
        self.mark_changed();
        Ok(report?)
    }

    /// Serves sync sessions to this device's peers until the process ends; `done` hears about
    /// each one. Returns this device's ID once listening (via `listening`).
    pub async fn sync_serve(&self, listening: impl FnOnce(&str), mut done: impl FnMut(Result<Report>)) -> Result<()> {
        let peers: Vec<String> = self.config.peers.values().map(|p| p.id.clone()).collect();
        if peers.is_empty() {
            return Err(EngineError::Config("no peers yet: `quire sync peer add <name> <device-id>`".into()));
        }
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key).await?;
        listening(&device.id());
        device
            .serve(&LibraryReplica(self), &peers, |outcome| {
                self.mark_changed();
                done(outcome.map_err(EngineError::from));
            })
            .await?;
        Ok(())
    }
}

impl Engine {
    /// One sync session over `reader`/`writer` (any stream: QUIC in use, a pipe in tests).
    pub async fn sync_session<Rd, Wr>(&self, role: Role, reader: &mut Rd, writer: &mut Wr) -> Result<Report>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        let report = quire_sync::session::run(&LibraryReplica(self), role, reader, writer).await;
        self.mark_changed();
        Ok(report?)
    }
}

/// The open library as one side of a sync session.
struct LibraryReplica<'a>(&'a Engine);

fn replica<T>(result: Result<T>) -> quire_sync::Result<T> {
    result.map_err(|e| SyncError::Replica(e.to_string()))
}

impl LibraryReplica<'_> {
    fn pdf(&self, hash: &str) -> PathBuf {
        self.0.files().path(hash, "pdf")
    }
}

impl Replica for LibraryReplica<'_> {
    async fn library_id(&self) -> quire_sync::Result<String> {
        replica(self.0.run(|lib| lib.library_id()).await)
    }

    fn device(&self) -> String {
        self.0.device.to_string()
    }

    async fn is_new(&self) -> quire_sync::Result<bool> {
        replica(self.0.run(|lib| lib.has_content()).await).map(|has| !has)
    }

    async fn adopt(&self, library: &str) -> quire_sync::Result<()> {
        let library = library.to_string();
        replica(self.0.run(move |lib| lib.adopt_library_id(&library)).await)
    }

    async fn received_from(&self, device: &str) -> quire_sync::Result<i64> {
        let device = device.to_string();
        replica(self.0.run(move |lib| lib.received_from(&device)).await)
    }

    async fn changes_since(&self, seq: i64, to: &str) -> quire_sync::Result<(Vec<quire_core::sync::Row>, i64)> {
        let to = to.to_string();
        replica(self.0.run(move |lib| lib.changes_since(seq, &to)).await)
    }

    async fn merge(
        &self,
        rows: Vec<quire_core::sync::Row>,
        from: &str,
    ) -> quire_sync::Result<quire_core::sync::Merged> {
        let from = from.to_string();
        replica(self.0.run(move |lib| lib.merge(rows, &from)).await)
    }

    async fn record_received(&self, device: &str, seq: i64) -> quire_sync::Result<()> {
        let device = device.to_string();
        replica(self.0.run(move |lib| lib.record_received(&device, seq)).await)
    }

    async fn missing_files(&self) -> quire_sync::Result<Vec<String>> {
        let hashes = replica(self.0.run(|lib| lib.attachment_hashes()).await)?;
        Ok(hashes.into_iter().filter(|hash| !self.pdf(hash).is_file()).collect())
    }

    async fn read_file(&self, hash: &str) -> quire_sync::Result<Option<Vec<u8>>> {
        // Only files of the file store, named by a well-formed hash (not a path).
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) || hash.len() != 64 {
            return Ok(None);
        }
        Ok(std::fs::read(self.pdf(hash)).ok())
    }

    async fn write_file(&self, hash: &str, bytes: Vec<u8>) -> quire_sync::Result<()> {
        if FileStore::hash(&bytes) != hash {
            return Err(SyncError::Protocol(format!("file {hash} arrived damaged (its content has another hash)")));
        }
        replica(self.0.files().put(&bytes, "pdf").map(|_| ()))
    }
}
