//! Sync of [`Engine`] with the user's other devices and with collaborators (`quire_sync`): this
//! device's key and the user's key (in the keyring, like the server token), its peers
//! (`[peers.<name>]` in the config), and the library as a [`Replica`] of the session: the store's
//! change log and merge (for a collaborator, filtered to what is shared with them and checked
//! against their role), and the file store.

use std::path::PathBuf;

use quire_core::library::sharing;
use quire_model::{ProjectMember, Role as MemberRole, SlugPath, UserId};
use quire_sync::identity::{self, Certificate};
use quire_sync::keys::{BoxKey, LibraryKeys};
use quire_sync::seal::LibraryKey;
use quire_sync::transport::{self, Device};
use quire_sync::{Peer, Replica, Report, Side, SyncError};

use super::Engine;
use crate::error::{EngineError, Result};
use crate::files::FileStore;
use crate::secrets::{self, Store};

/// Name of this device's sync key in the keyring.
const SECRET: &str = "sync-device-key";
/// Name of the user's key (the same on all their devices) in the keyring.
const USER_SECRET: &str = "sync-user-key";
/// Name of this device's box key (rotated library keys are sealed to it; `quire_sync::keys`).
const BOX_SECRET: &str = "sync-box-key";

impl Engine {
    /// Our own relay (`sync.relay`), if set: the only one sync then uses.
    pub(super) fn relay(&self) -> Option<&str> {
        self.config.sync.relay.as_deref().map(str::trim).filter(|r| !r.is_empty())
    }

    /// This device's sync key, created on first use, and where it is kept.
    pub(super) fn sync_key(&self) -> Result<([u8; 32], Store)> {
        self.key(SECRET)
    }

    /// The user's key, created on first use, and where it is kept.
    pub(super) fn user_key(&self) -> Result<([u8; 32], Store)> {
        self.key(USER_SECRET)
    }

    fn key(&self, name: &str) -> Result<([u8; 32], Store)> {
        let fallback = self.config_file().with_file_name(name);
        let (hex, store) = secrets::get_or_create(name, &fallback)?;
        let bytes =
            identity::key_from_hex(&hex).ok_or_else(|| EngineError::Config(format!("the key `{name}` is damaged")))?;
        Ok((bytes, store))
    }

    /// This person's user ID (what collaborators share projects with) and where its key is kept.
    pub fn user_id(&self) -> Result<(UserId, Store)> {
        let (key, store) = self.user_key()?;
        Ok((UserId::parse(&identity::user_id(&key))?, store))
    }

    /// The user key, to give another of the user's devices (`import_user_key` there): keep it
    /// secret, it lets a device act as this person.
    pub fn export_user_key(&self) -> Result<String> {
        let (key, _) = self.user_key()?;
        Ok(identity::key_to_hex(&key))
    }

    /// Makes `hex` (another device's exported key) this device's user key; returns the user ID.
    pub fn import_user_key(&self, hex: &str) -> Result<(UserId, Store)> {
        let key = identity::key_from_hex(hex)
            .ok_or_else(|| EngineError::Config("not a user key (64 hexadecimal digits)".into()))?;
        let fallback = self.config_file().with_file_name(USER_SECRET);
        let store = secrets::set(USER_SECRET, hex.trim(), &fallback)?;
        Ok((UserId::parse(&identity::user_id(&key))?, store))
    }

    /// The key this library's data is sealed with on its way to a hub: derived from the user key
    /// and the library ID, so all of this person's devices hold it without passing it around.
    pub async fn library_key(&self) -> Result<LibraryKey> {
        let (key, _) = self.user_key()?;
        let library = self.run(|lib| lib.library_id()).await?;
        Ok(LibraryKey::derive(&key, &library))
    }

    /// This device's box key and its signature by the device key (`quire_sync::keys`).
    pub fn device_box(&self) -> Result<(BoxKey, Vec<u8>)> {
        let (secret, _) = self.key(BOX_SECRET)?;
        let (device, _) = self.sync_key()?;
        let boxed = BoxKey::from_secret(secret);
        let signature = boxed.signature(&device);
        Ok((boxed, signature))
    }

    /// The keys of `library`: epoch 0 derived from the user key, later ones kept here.
    pub fn library_keys_of(&self, library: &str) -> Result<LibraryKeys> {
        let (user, _) = self.user_key()?;
        let mut keys = LibraryKeys::new(LibraryKey::derive(&user, library));
        let name = chain_secret(library);
        if let Some((text, _)) = secrets::get(&name, &self.config_file().with_file_name(&name)) {
            keys.merge(&LibraryKeys::parse_chain(&text));
        }
        Ok(keys)
    }

    /// The keys of this library.
    pub async fn library_keys(&self) -> Result<LibraryKeys> {
        let library = self.run(|lib| lib.library_id()).await?;
        self.library_keys_of(&library)
    }

    /// Keeps the later keys of `library` (after a rotation, or given by another device).
    pub fn save_library_keys_of(&self, library: &str, keys: &LibraryKeys) -> Result<()> {
        let name = chain_secret(library);
        secrets::set(&name, &keys.chain_text(), &self.config_file().with_file_name(&name))?;
        Ok(())
    }

    /// Takes the library keys of a recovery kit (`<library ID>=<epoch>:<key>,…`).
    pub fn import_library_keys(&self, text: &str) -> Result<()> {
        let (library, chain) = text
            .trim()
            .split_once('=')
            .ok_or_else(|| EngineError::Config("library keys look like `<library ID>=<epoch>:<key>,…`".into()))?;
        let chain = LibraryKeys::parse_chain(chain);
        if chain.is_empty() {
            return Err(EngineError::Config("no library key in that text".into()));
        }
        let mut keys = self.library_keys_of(library.trim())?;
        keys.merge(&chain);
        self.save_library_keys_of(library.trim(), &keys)
    }

    /// What to keep somewhere safe before syncing through a hub: the user key recovers every
    /// library from the hub on a new device (the hub cannot read them, so it cannot help otherwise),
    /// with this library's later keys once a device was unlinked (they are not derived from it).
    pub async fn recovery_kit(&self) -> Result<String> {
        let (user, _) = self.user_id()?;
        let key = self.export_user_key()?;
        let library = self.run(|lib| lib.library_id()).await?;
        let keys = self.library_keys_of(&library)?;
        let later = if keys.chain().is_empty() {
            String::new()
        } else {
            format!(
                "Library keys: {library}={}\n\
                 (a device was unlinked: newer data uses these; keep this kit again after each unlink;\n\
                 on a new device also `quire user library-keys import '<the text after Library keys:>'`)\n",
                keys.chain_text()
            )
        };
        Ok(format!(
            "quire recovery kit\n\
             \n\
             User ID:  {user}\n\
             User key: {key}\n\
             {later}\
             \n\
             Keep this offline (printed, or in a password manager). On a new device:\n\
             `quire user key import <user key>`, then add your sync hub and sync.\n\
             Whoever has this key can read your libraries and act as you; nobody, not the hub, can\n\
             recover your data without it.\n"
        ))
    }

    /// Shares the project at `path` with `user` (named `name`) as `role`.
    pub async fn share_project(
        &self,
        path: &SlugPath,
        user: &UserId,
        name: &str,
        role: MemberRole,
    ) -> Result<ProjectMember> {
        let (me, _) = self.user_id()?;
        let (path, user, name) = (path.clone(), user.clone(), name.to_string());
        let member = self.run(move |lib| sharing::share(lib, &me, &path, &user, &name, role)).await?;
        self.mark_changed();
        Ok(member)
    }

    pub async fn unshare_project(&self, path: &SlugPath, user: &UserId) -> Result<ProjectMember> {
        let (me, _) = self.user_id()?;
        let (path, user) = (path.clone(), user.clone());
        let member = self.run(move |lib| sharing::unshare(lib, &me, &path, &user)).await?;
        self.mark_changed();
        Ok(member)
    }

    /// The project at `path`, its owner and its members.
    pub async fn project_members(&self, path: &SlugPath) -> Result<(quire_model::Project, Vec<ProjectMember>)> {
        let path = path.clone();
        self.run(move |lib| sharing::members(lib, &path)).await
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
        let device = Device::bind(&key, self.relay()).await?;
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
        let device = Device::bind(&key, self.relay()).await?;
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
    /// Gets the file `hash` from the device `peer` with a sync that asks for that file only (the
    /// rows are exchanged as in any sync); `false` if the device does not have it.
    pub async fn fetch_from_peer(&self, peer: &str, hash: &str) -> Result<bool> {
        let id = self.peer_id(peer)?;
        let (key, _) = self.sync_key()?;
        let device = Device::bind(&key, self.relay()).await?;
        let report = device.sync_with(&Wanting { inner: LibraryReplica(self), hash: hash.to_string() }, &id).await;
        device.close().await;
        self.mark_changed();
        report?;
        Ok(self.files().path(hash, "pdf").is_file())
    }

    /// [`Engine::fetch_from_peer`] over `reader`/`writer` (tests).
    pub async fn fetch_session<Rd, Wr>(&self, hash: &str, side: Side, reader: &mut Rd, writer: &mut Wr) -> Result<bool>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        let wanting = Wanting { inner: LibraryReplica(self), hash: hash.to_string() };
        quire_sync::session::run(&wanting, side, None, reader, writer).await?;
        self.mark_changed();
        Ok(self.files().path(hash, "pdf").is_file())
    }

    /// One sync session over `reader`/`writer` (any stream: QUIC in use, a pipe in tests).
    pub async fn sync_session<Rd, Wr>(&self, side: Side, reader: &mut Rd, writer: &mut Wr) -> Result<Report>
    where
        Rd: tokio::io::AsyncRead + Unpin,
        Wr: tokio::io::AsyncWrite + Unpin,
    {
        let report = quire_sync::session::run(&LibraryReplica(self), side, None, reader, writer).await;
        self.mark_changed();
        Ok(report?)
    }
}

/// The open library as one side of a sync session.
pub(super) struct LibraryReplica<'a>(pub(super) &'a Engine);

fn replica<T>(result: Result<T>) -> quire_sync::Result<T> {
    result.map_err(|e| SyncError::Replica(e.to_string()))
}

impl LibraryReplica<'_> {
    pub(super) fn pdf(&self, hash: &str) -> PathBuf {
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

    fn certificate(&self) -> Certificate {
        // Both keys exist once the engine got this far (sync_with / serve created them); a damaged
        // key gives a certificate the other side refuses.
        let user = self.0.user_key().map(|(k, _)| k).unwrap_or([0; 32]);
        let device = self.0.sync_key().map(|(k, _)| transport::device_id(&k)).unwrap_or_default();
        Certificate::issue(&user, &device)
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

    async fn changes_for(&self, seq: i64, to: &Peer) -> quire_sync::Result<(Vec<quire_core::sync::Row>, i64)> {
        let (device, user) = (to.device.clone(), collaborator(to)?);
        replica(
            self.0
                .run(move |lib| {
                    let (rows, last) = lib.changes_since(seq, &device)?;
                    let rows = match &user {
                        Some(user) => lib.visible_changes(rows, user)?,
                        None => rows,
                    };
                    Ok((rows, last))
                })
                .await,
        )
    }

    async fn merge(
        &self,
        rows: Vec<quire_core::sync::Row>,
        from: &Peer,
    ) -> quire_sync::Result<quire_core::sync::MergeSummary> {
        let (device, user) = (from.device.clone(), collaborator(from)?);
        replica(
            self.0
                .run(move |lib| {
                    let (rows, refused) = match &user {
                        Some(user) => lib.authorize(rows, user)?,
                        None => (rows, 0),
                    };
                    let mut merged = lib.merge(rows, &device)?;
                    merged.refused += refused;
                    Ok(merged)
                })
                .await,
        )
    }

    async fn record_received(&self, device: &str, seq: i64) -> quire_sync::Result<()> {
        let device = device.to_string();
        replica(self.0.run(move |lib| lib.record_received(&device, seq)).await)
    }

    async fn missing_files(&self) -> quire_sync::Result<Vec<String>> {
        // All missing files, or with `files.download = "on-open"` only those kept offline.
        replica(self.0.wanted_files().await)
    }

    async fn read_file(&self, hash: &str, to: &Peer) -> quire_sync::Result<Option<Vec<u8>>> {
        // Only files of the file store, named by a well-formed hash (not a path).
        if !quire_model::is_content_hash(hash) {
            return Ok(None);
        }
        // A collaborator: only the PDFs of references shared with them.
        if let Some(user) = collaborator(to)? {
            let visible = replica(self.0.run(move |lib| lib.visible_hashes(&user)).await)?;
            if !visible.contains(hash) {
                return Ok(None);
            }
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

/// The collaborator's user ID, `None` for one of the user's own devices.
fn collaborator(peer: &Peer) -> quire_sync::Result<Option<UserId>> {
    if !peer.collaborator {
        return Ok(None);
    }
    UserId::parse(&peer.user).map(Some).map_err(|e| SyncError::Protocol(e.to_string()))
}

/// The library as a session side that asks the other device for one file only (lazy PDFs: the
/// rest stays on the other device until opened).
struct Wanting<'a> {
    inner: LibraryReplica<'a>,
    hash: String,
}

impl Replica for Wanting<'_> {
    async fn library_id(&self) -> quire_sync::Result<String> {
        self.inner.library_id().await
    }

    fn device(&self) -> String {
        self.inner.device()
    }

    fn certificate(&self) -> Certificate {
        self.inner.certificate()
    }

    async fn is_new(&self) -> quire_sync::Result<bool> {
        self.inner.is_new().await
    }

    async fn adopt(&self, library: &str) -> quire_sync::Result<()> {
        self.inner.adopt(library).await
    }

    async fn received_from(&self, device: &str) -> quire_sync::Result<i64> {
        self.inner.received_from(device).await
    }

    async fn changes_for(&self, seq: i64, to: &Peer) -> quire_sync::Result<(Vec<quire_core::sync::Row>, i64)> {
        self.inner.changes_for(seq, to).await
    }

    async fn merge(
        &self,
        rows: Vec<quire_core::sync::Row>,
        from: &Peer,
    ) -> quire_sync::Result<quire_core::sync::MergeSummary> {
        self.inner.merge(rows, from).await
    }

    async fn record_received(&self, device: &str, seq: i64) -> quire_sync::Result<()> {
        self.inner.record_received(device, seq).await
    }

    async fn missing_files(&self) -> quire_sync::Result<Vec<String>> {
        Ok(if self.inner.pdf(&self.hash).is_file() { Vec::new() } else { vec![self.hash.clone()] })
    }

    async fn read_file(&self, hash: &str, to: &Peer) -> quire_sync::Result<Option<Vec<u8>>> {
        self.inner.read_file(hash, to).await
    }

    async fn write_file(&self, hash: &str, bytes: Vec<u8>) -> quire_sync::Result<()> {
        self.inner.write_file(hash, bytes).await
    }
}

/// The secret holding a library's later keys: named by a hash of its ID (any text) so the name
/// suits a keyring entry and a file.
fn chain_secret(library: &str) -> String {
    format!("library-keys-{}", &blake3::hash(library.as_bytes()).to_hex()[..16])
}
