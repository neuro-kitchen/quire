//! A device's side of a hub session (`.knowledge/sync-hub.md`), over any byte stream (a QUIC
//! stream in use, a pipe in tests):
//!
//! 1. `Hello` (certificate, library, this copy's author ID); with a code, `Register` first time.
//! 2. Push: this copy's changes after the last batch the hub has from it, sealed in batches.
//! 3. Pull: the other copies' batches after this copy's hub position (a snapshot first if it is
//!    behind one), opened and merged with the hub as their origin (so they are not pushed back).
//! 4. Files: the PDFs here the hub lacks, sealed. Nothing is downloaded: PDFs come when opened
//!    ([`fetch_file`]).
//! 5. A snapshot when the hub reports many batches since the last one (the hub then drops them).

use std::collections::HashMap;
use std::future::Future;

use quire_core::sync::{MergeSummary, Row};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use super::protocol::{
    self, Batch, BatchKind, Batches, ErrorCode, FileIds, GetFile, Hello, Introduce, KeyEnvelope, Keys, Pull, Push,
    PutFile, PutKeys, Register, Reply, ReplyBody, Request, RequestBody, Welcome,
};
use crate::error::{Result, SyncError};
use crate::keys::{self, BoxKey, LibraryKeys};
use crate::seal::{self, Kind};
use crate::session::{Peer, Replica};

/// Rows per pushed batch (a batch is one sealed message).
const ROWS_PER_BATCH: usize = 500;
/// Batches asked for per pull.
const PULL_LIMIT: u32 = 200;
/// Change batches since the last snapshot after which this device uploads a new one.
pub const SNAPSHOT_AFTER: u64 = 200;

/// A library copy as a hub session sees it: a [`Replica`] (change log, merge, files) plus the key
/// and what a hub needs that a device session does not.
pub trait HubReplica: Replica {
    /// The library's keys: epoch 0 (derived from the user key) and any later ones kept here.
    fn library_keys(&self) -> impl Future<Output = Result<LibraryKeys>> + Send;
    /// Keeps the library's keys after this device got or made a later epoch.
    fn save_library_keys(&self, keys: &LibraryKeys) -> impl Future<Output = Result<()>> + Send;
    /// This device's box key (rotated keys are sealed to it) and its signature by the device key.
    fn device_box(&self) -> Option<(BoxKey, Vec<u8>)> {
        None
    }
    /// Hashes of the files here.
    fn local_files(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
    /// Every row of the library (a snapshot).
    fn snapshot(&self) -> impl Future<Output = Result<Vec<Row>>> + Send;
    /// Files to download in this session (missing here and wanted now: all of them, or only those
    /// kept offline when PDFs load as they are opened).
    fn wanted_files(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
    /// This device's name for the person's other devices (sealed before it reaches the hub).
    fn device_name(&self) -> Option<String> {
        None
    }
}

/// What unlinking a device did to the library's key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rotation {
    /// The new key epoch.
    pub epoch: u32,
    /// Devices it was sealed for (this one included).
    pub sealed_for: usize,
    /// Remaining devices that could not get it (an older version: no box key, or a bad signature):
    /// they cannot read what is sealed from now on until linked again.
    pub left_out: Vec<String>,
    /// The unlinked device's name, if it had sent one.
    pub name: Option<String>,
}

/// One of this person's devices as the hub knows it, its name opened here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubDevice {
    pub device: String,
    /// `None`: it never sent one (an older version), or it does not open with this library's key.
    pub name: Option<String>,
    /// Milliseconds since 1970.
    pub first_seen: i64,
    pub last_seen: i64,
    pub revoked: bool,
    /// Its last session that ended (0: none recorded), and what it moved then.
    pub last_session: i64,
    pub batches_in: u32,
    pub batches_out: u32,
    pub files_in: u32,
    pub files_out: u32,
    /// Its box key and the signature by its device key (empty: an older version).
    pub box_key: Vec<u8>,
    pub box_signature: Vec<u8>,
    /// The library copies (author IDs) it syncs: whose changes are its own.
    pub authors: Vec<String>,
}

/// How a hub session runs.
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Registers this person on a hub that does not know them yet (setup, invitation or link code).
    pub code: Option<String>,
    /// Upload a snapshot once the hub holds this many change batches since the last one.
    pub snapshot_after: u64,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self { code: None, snapshot_after: SNAPSHOT_AFTER }
    }
}

/// What a hub session did, for the user.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HubReport {
    /// This person registered with a code in this session.
    pub registered: bool,
    /// Rows sent, in how many batches.
    pub pushed: usize,
    pub batches_pushed: usize,
    pub pulled: MergeSummary,
    pub files_uploaded: usize,
    /// A snapshot was uploaded.
    pub snapshot: bool,
    /// Bytes stored for this account, and its limit (0: none).
    pub used: u64,
    pub quota: u64,
    /// Files not uploaded because the quota was reached.
    pub over_quota: usize,
    /// This new, empty copy joined the library the hub holds for this person.
    pub joined: bool,
    pub files_downloaded: usize,
    /// Files (by hash) the hub is known to hold after this session: here, they may be evicted.
    pub on_hub: Vec<String>,
    /// Files here the hub said it lacks and did not take (over quota): no longer safe to evict,
    /// whatever an earlier session said.
    pub not_on_hub: Vec<String>,
    /// What came in, by the library copy that last changed it (an author ID) and kind: counts.
    pub arrived: Vec<(String, String, usize)>,
    /// This person's devices after the sync (their names opened here): who sent what, and new ones.
    pub devices: Vec<HubDevice>,
    /// The library key moved to this epoch in this session (another device unlinked one).
    pub key_received: Option<u32>,
}

/// The peer a hub is in the change log (its cursor, and the origin of rows pulled from it).
pub fn hub_peer(hub: &str, user: &str) -> Peer {
    Peer { device: format!("hub:{hub}"), user: user.to_string(), collaborator: false }
}

struct Session<'a, R, Rd, Wr> {
    replica: &'a R,
    reader: &'a mut Rd,
    writer: &'a mut Wr,
    keys: LibraryKeys,
    library: String,
    author: String,
    peer: Peer,
}

impl<R, Rd, Wr> Session<'_, R, Rd, Wr>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    async fn ask(&mut self, body: RequestBody) -> Result<ReplyBody> {
        protocol::send(self.writer, &Request::of(body)).await?;
        self.writer.flush().await?;
        self.answer().await
    }

    async fn answer(&mut self) -> Result<ReplyBody> {
        let reply: Reply = protocol::receive(self.reader)
            .await?
            .ok_or_else(|| SyncError::Network("the hub closed the connection".into()))?;
        match reply.body {
            Some(ReplyBody::Error(error)) => Err(hub_error(error.code, error.message)),
            Some(body) => Ok(body),
            None => Err(SyncError::Protocol("an empty reply from the hub".into())),
        }
    }

    /// Who is talking: the certificate, the library and author, this device's name (sealed with
    /// the newest key) and its box key (signed by the device key).
    fn hello_message(&self) -> Hello {
        let certificate = self.replica.certificate();
        let (epoch, key) = self.keys.current();
        let sealed_name = self
            .replica
            .device_name()
            .and_then(|name| key.seal_name(&self.library, &certificate.device, epoch, &name).ok())
            .unwrap_or_default();
        let (box_key, box_signature) =
            self.replica.device_box().map(|(key, signature)| (key.public().to_vec(), signature)).unwrap_or_default();
        Hello {
            protocol: protocol::PROTOCOL,
            certificate: Some((&certificate).into()),
            library: self.library.clone(),
            author: self.author.clone(),
            sealed_name,
            box_key,
            box_signature,
        }
    }

    /// Gets this device's envelope of the library's newest epoch and keeps the keys in it; returns
    /// that epoch.
    async fn fetch_keys(&mut self) -> Result<u32> {
        let Keys { epoch, sealed } = match self.ask(RequestBody::GetKeys(protocol::Empty {})).await? {
            ReplyBody::Keys(keys) => keys,
            other => return Err(unexpected("Keys", &other)),
        };
        let Some((mine, _)) = self.replica.device_box().filter(|_| !sealed.is_empty()) else {
            return Err(self.keys.get(epoch).map(|_| ()).expect_err("an epoch beyond this device's"));
        };
        let device = self.replica.certificate().device;
        let chain = keys::open_chain(&mine, &sealed, &self.library, &device, epoch)?;
        self.keys.merge(&chain);
        self.keys.get(epoch)?;
        self.replica.save_library_keys(&self.keys).await?;
        Ok(epoch)
    }

    /// This person's devices, their names opened with this library's keys.
    async fn devices(&mut self) -> Result<Vec<HubDevice>> {
        let list = match self.ask(RequestBody::Devices(super::protocol::Empty {})).await? {
            ReplyBody::Devices(list) => list.devices,
            other => return Err(unexpected("Devices", &other)),
        };
        Ok(list
            .into_iter()
            .map(|d| HubDevice {
                name: seal::name_epoch(&d.sealed_name)
                    .and_then(|epoch| self.keys.get(epoch).ok())
                    .and_then(|key| key.open_name(&self.library, &d.device, &d.sealed_name)),
                device: d.device,
                first_seen: d.first_seen,
                last_seen: d.last_seen,
                revoked: d.revoked,
                last_session: d.last_session,
                batches_in: d.batches_in,
                batches_out: d.batches_out,
                files_in: d.files_in,
                files_out: d.files_out,
                box_key: d.box_key,
                box_signature: d.box_signature,
                authors: d.authors,
            })
            .collect())
    }

    async fn hello(&mut self, code: Option<&str>, report: &mut HubReport) -> Result<Welcome> {
        let hello = self.hello_message();
        let welcome = match self.ask(RequestBody::Hello(hello)).await? {
            ReplyBody::Welcome(welcome) => welcome,
            other => return Err(unexpected("Welcome", &other)),
        };
        if welcome.registered {
            return Ok(welcome);
        }
        let Some(code) = code else { return Err(SyncError::NotRegistered) };
        match self.ask(RequestBody::Register(Register { code: code.trim().to_string() })).await? {
            ReplyBody::Welcome(welcome) => {
                report.registered = true;
                Ok(welcome)
            }
            other => Err(unexpected("Welcome", &other)),
        }
    }

    async fn push(&mut self, since: u64, report: &mut HubReport) -> Result<()> {
        let mut seq = i64::try_from(since).unwrap_or(i64::MAX);
        loop {
            let (rows, last) = self.replica.changes_for(seq, &self.peer).await?;
            if last <= seq {
                return Ok(());
            }
            // Rows pulled from the hub are not sent back (their origin is the hub): a batch can be
            // empty; nothing to send then, the next one goes on from `last`.
            for part in rows.chunks(ROWS_PER_BATCH) {
                let plain = serde_json::to_vec(part).map_err(|e| SyncError::Protocol(e.to_string()))?;
                let number = u64::try_from(last).unwrap_or(0);
                let (epoch, key) = self.keys.current();
                let sealed = key.seal_batch(Kind::Changes, &self.library, &self.author, number, epoch, &plain)?;
                let batch = Batch {
                    author: self.author.clone(),
                    number,
                    kind: BatchKind::Changes as i32,
                    sealed,
                    covers: 0,
                    seq: 0,
                    epoch,
                };
                match self.ask(RequestBody::Push(Push { batches: vec![batch] })).await? {
                    ReplyBody::Pushed(_) => {}
                    other => return Err(unexpected("Pushed", &other)),
                }
                report.pushed += part.len();
                report.batches_pushed += 1;
            }
            seq = last;
        }
    }

    /// Merges the other copies' batches; returns the hub position reached and how many change
    /// batches the hub holds since its last snapshot.
    async fn pull(&mut self, report: &mut HubReport) -> Result<(u64, u64)> {
        let mut after = u64::try_from(self.replica.received_from(&self.peer.device).await?).unwrap_or(0);
        loop {
            let Batches { batches, up_to, more, since_snapshot } =
                match self.ask(RequestBody::Pull(Pull { after, limit: PULL_LIMIT })).await? {
                    ReplyBody::Batches(batches) => batches,
                    other => return Err(unexpected("Batches", &other)),
                };
            let mut rows = Vec::new();
            for batch in &batches {
                let kind =
                    BatchKind::try_from(batch.kind).map_err(|_| SyncError::Protocol("unknown batch kind".into()))?;
                let key = self.keys.get(batch.epoch)?;
                let plain = key.open_batch(
                    kind.into(),
                    &self.library,
                    &batch.author,
                    batch.number,
                    batch.epoch,
                    &batch.sealed,
                )?;
                let mut part: Vec<Row> =
                    serde_json::from_slice(&plain).map_err(|e| SyncError::Protocol(format!("a batch's rows: {e}")))?;
                rows.append(&mut part);
            }
            // What came in, by the copy that last changed it and kind (for the person's notifications).
            for row in &rows {
                let (by, kind) = (row.version().1.to_string(), row.kind());
                match report.arrived.iter_mut().find(|(b, k, _)| *b == by && k == kind) {
                    Some((_, _, count)) => *count += 1,
                    None => report.arrived.push((by, kind.to_string(), 1)),
                }
            }
            // Rows of one delivery merge together, in hub order, so they find what they point to.
            if !rows.is_empty() {
                report.pulled += self.replica.merge(rows, &self.peer).await?;
            }
            // "More" from the same position again would be asked for forever (a broken hub).
            if more && up_to <= after {
                return Err(SyncError::Protocol("the hub says more batches follow but gives no new ones".into()));
            }
            if up_to > after {
                self.replica.record_received(&self.peer.device, i64::try_from(up_to).unwrap_or(i64::MAX)).await?;
                after = up_to;
            }
            if !more {
                return Ok((after, since_snapshot));
            }
        }
    }

    async fn upload_files(&mut self, report: &mut HubReport) -> Result<()> {
        let ids: HashMap<String, String> =
            self.replica.local_files().await?.into_iter().map(|hash| (self.keys.base().file_id(&hash), hash)).collect();
        if ids.is_empty() {
            return Ok(());
        }
        let present = match self.ask(RequestBody::HasFiles(FileIds { ids: ids.keys().cloned().collect() })).await? {
            ReplyBody::Present(present) => present.ids,
            other => return Err(unexpected("Present", &other)),
        };
        report.on_hub.extend(present.iter().filter_map(|id| ids.get(id).cloned()));
        let own = Peer { collaborator: false, ..self.peer.clone() };
        for (id, hash) in ids.iter().filter(|(id, _)| !present.contains(id)) {
            let Some(bytes) = self.replica.read_file(hash, &own).await? else {
                report.not_on_hub.push(hash.clone());
                continue;
            };
            let (epoch, key) = self.keys.current();
            let sealed = key.seal_file(id, epoch, &bytes)?;
            protocol::send(
                self.writer,
                &Request::of(RequestBody::PutFile(PutFile { id: id.clone(), size: sealed.len() as u64 })),
            )
            .await?;
            self.writer.write_all(&sealed).await?;
            self.writer.flush().await?;
            match self.answer().await {
                Ok(_) => {
                    report.files_uploaded += 1;
                    report.on_hub.push(hash.clone());
                }
                Err(SyncError::Hub { code, .. }) if code == ErrorCode::Quota as i32 => {
                    report.over_quota += 1;
                    report.not_on_hub.push(hash.clone());
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    async fn snapshot(&mut self, covers: u64) -> Result<()> {
        let rows = self.replica.snapshot().await?;
        let plain = serde_json::to_vec(&rows).map_err(|e| SyncError::Protocol(e.to_string()))?;
        let (epoch, key) = self.keys.current();
        let sealed = key.seal_batch(Kind::Snapshot, &self.library, &self.author, covers, epoch, &plain)?;
        let batch = Batch {
            author: self.author.clone(),
            number: covers,
            kind: BatchKind::Snapshot as i32,
            sealed,
            covers,
            seq: 0,
            epoch,
        };
        match self.ask(RequestBody::Push(Push { batches: vec![batch] })).await? {
            ReplyBody::Pushed(_) => Ok(()),
            other => Err(unexpected("Pushed", &other)),
        }
    }

    /// Says `Bye` after a failure (best effort), so the hub does not wait for more; returns `error`.
    async fn end_with(&mut self, error: SyncError) -> SyncError {
        let _ = self.bye().await;
        error
    }

    async fn bye(&mut self) -> Result<()> {
        protocol::send(self.writer, &Request::of(RequestBody::Bye(protocol::Empty {}))).await?;
        self.writer.flush().await?;
        Ok(())
    }
}

/// Opens a session: `Hello` (and `Register` with `code`); a new, empty copy joins the library the
/// hub holds for this person (the key is derived after, from the library joined).
async fn open<'a, R: HubReplica, Rd, Wr>(
    replica: &'a R,
    hub: &str,
    code: Option<&str>,
    reader: &'a mut Rd,
    writer: &'a mut Wr,
    report: &mut HubReport,
) -> Result<(Session<'a, R, Rd, Wr>, Welcome)>
where
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let certificate = replica.certificate();
    let library = replica.library_id().await?;
    let mut session = Session {
        replica,
        reader,
        writer,
        keys: replica.library_keys().await?,
        library,
        author: replica.device(),
        peer: hub_peer(hub, &certificate.user),
    };
    let mut welcome = match session.hello(code, report).await {
        Ok(welcome) => welcome,
        Err(error) => return Err(session.end_with(error).await),
    };
    if !welcome.libraries.contains(&session.library) && !welcome.libraries.is_empty() && replica.is_new().await? {
        let [only] = welcome.libraries.as_slice() else {
            let error = SyncError::Protocol(format!(
                "the hub holds {} of your libraries; sync from the device that has the one to use first",
                welcome.libraries.len()
            ));
            return Err(session.end_with(error).await);
        };
        replica.adopt(only).await?;
        report.joined = true;
        session.library = replica.library_id().await?;
        session.keys = replica.library_keys().await?;
        welcome = session.hello(None, report).await?;
    }
    // A device was unlinked since this one last synced: the library key moved on.
    if welcome.key_epoch > session.keys.current().0 {
        match session.fetch_keys().await {
            Ok(epoch) => report.key_received = Some(epoch),
            Err(error) => return Err(session.end_with(error).await),
        }
    }
    Ok((session, welcome))
}

/// One sync with the hub `hub` (its network ID: the cursor's name).
pub async fn sync<R, Rd, Wr>(
    replica: &R,
    hub: &str,
    options: &SyncOptions,
    reader: &mut Rd,
    writer: &mut Wr,
) -> Result<HubReport>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut report = HubReport::default();
    let (mut session, welcome) = open(replica, hub, options.code.as_deref(), reader, writer, &mut report).await?;
    match exchange(&mut session, &welcome, options, &mut report).await {
        Ok(()) => {
            session.bye().await?;
            Ok(report)
        }
        Err(error) => Err(session.end_with(error).await),
    }
}

/// Push, pull, files and maybe a snapshot; then the account's use and quota.
async fn exchange<R, Rd, Wr>(
    session: &mut Session<'_, R, Rd, Wr>,
    welcome: &Welcome,
    options: &SyncOptions,
    report: &mut HubReport,
) -> Result<()>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    session.push(welcome.last_number, report).await?;
    let (position, since_snapshot) = session.pull(report).await?;
    session.upload_files(report).await?;
    for hash in session.replica.wanted_files().await? {
        if download(session, &hash).await? {
            report.files_downloaded += 1;
            report.on_hub.push(hash);
        }
    }
    if since_snapshot >= options.snapshot_after {
        session.snapshot(position).await?;
        report.snapshot = true;
    }
    report.devices = session.devices().await?;
    let hello = session.hello_message();
    if let ReplyBody::Welcome(after) = session.ask(RequestBody::Hello(hello)).await? {
        (report.used, report.quota) = (after.used, after.quota);
    }
    Ok(())
}

/// Downloads the file `hash` from the hub, opens and stores it (checked against its hash);
/// `false` if the hub does not have it.
pub async fn fetch_file<R, Rd, Wr>(replica: &R, hub: &str, hash: &str, reader: &mut Rd, writer: &mut Wr) -> Result<bool>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut report = HubReport::default();
    let (mut session, _) = open(replica, hub, None, reader, writer, &mut report).await?;
    match download(&mut session, hash).await {
        Ok(found) => {
            session.bye().await?;
            Ok(found)
        }
        Err(error) => Err(session.end_with(error).await),
    }
}

async fn download<R, Rd, Wr>(session: &mut Session<'_, R, Rd, Wr>, hash: &str) -> Result<bool>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let id = session.keys.base().file_id(hash);
    let header = match session.ask(RequestBody::GetFile(GetFile { id: id.clone() })).await {
        Ok(ReplyBody::File(header)) => header,
        Ok(other) => return Err(unexpected("File", &other)),
        Err(SyncError::Hub { code, .. }) if code == ErrorCode::NotFound as i32 => return Ok(false),
        Err(e) => return Err(e),
    };
    let sealed = protocol::receive_bytes(session.reader, header.size).await?;
    let epoch =
        seal::file_epoch(&sealed).ok_or_else(|| SyncError::Protocol("a file from the hub is not sealed".into()))?;
    let plain = session.keys.get(epoch)?.open_file(&id, &sealed)?;
    // `write_file` refuses bytes whose BLAKE3 hash is not `hash`.
    session.replica.write_file(hash, plain).await?;
    Ok(true)
}

/// This person's devices known to the hub.
pub async fn devices<R, Rd, Wr>(replica: &R, hub: &str, reader: &mut Rd, writer: &mut Wr) -> Result<Vec<HubDevice>>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut report = HubReport::default();
    let (mut session, _) = open(replica, hub, None, reader, writer, &mut report).await?;
    match session.devices().await {
        Ok(devices) => {
            session.bye().await?;
            Ok(devices)
        }
        Err(error) => Err(session.end_with(error).await),
    }
}

/// Unlinks `device` (a network ID) of this person: the hub refuses it from now on, and the
/// library key rotates (`crate::keys`): a new random key, sealed for each remaining device whose
/// box key its device key signed. Returns what the rotation did.
pub async fn unlink<R, Rd, Wr>(
    replica: &R,
    hub: &str,
    device: &str,
    reader: &mut Rd,
    writer: &mut Wr,
) -> Result<Rotation>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut report = HubReport::default();
    let (mut session, _) = open(replica, hub, None, reader, writer, &mut report).await?;
    let revoke = RequestBody::Revoke(super::protocol::Revoke { device: device.trim().to_string() });
    let done = async {
        match session.ask(revoke).await? {
            ReplyBody::Ok(_) => {}
            other => return Err(unexpected("Ok", &other)),
        }
        // Another device may rotate at the same moment: then take its keys and rotate after it.
        match rotate(&mut session, device).await {
            Err(SyncError::Hub { code, .. }) if code == ErrorCode::StaleEpoch as i32 => {
                session.fetch_keys().await?;
                rotate(&mut session, device).await
            }
            other => other,
        }
    }
    .await;
    match done {
        Ok(rotation) => {
            session.bye().await?;
            Ok(rotation)
        }
        Err(error) => Err(session.end_with(error).await),
    }
}

/// The next epoch: a random key, sealed with the whole chain for each device not revoked
/// (`unlinked`: the device just unlinked, named in the result).
async fn rotate<R, Rd, Wr>(session: &mut Session<'_, R, Rd, Wr>, unlinked: &str) -> Result<Rotation>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let devices = session.devices().await?;
    let mut keys = session.keys.clone();
    let epoch = keys.rotate()?;
    let chain = keys.chain();
    let name = devices.iter().find(|d| d.device == unlinked.trim()).and_then(|d| d.name.clone());
    let mut rotation = Rotation { epoch, name, ..Rotation::default() };
    let mut envelopes = Vec::new();
    for d in devices.iter().filter(|d| !d.revoked) {
        let public: Option<[u8; 32]> = d.box_key.as_slice().try_into().ok();
        match public.filter(|p| keys::box_key_is_devices(&d.device, p, &d.box_signature)) {
            Some(public) => envelopes.push(KeyEnvelope {
                device: d.device.clone(),
                sealed: keys::seal_chain(&chain, &public, &session.library, &d.device, epoch)?,
            }),
            None => rotation.left_out.push(d.device.clone()),
        }
    }
    rotation.sealed_for = envelopes.len();
    match session.ask(RequestBody::PutKeys(PutKeys { epoch, envelopes })).await? {
        ReplyBody::Ok(_) => {}
        other => return Err(unexpected("Ok", &other)),
    }
    // Kept only once the hub holds the envelopes: what is sealed from now on is readable elsewhere.
    session.keys = keys;
    session.replica.save_library_keys(&session.keys).await?;
    Ok(rotation)
}

/// Lets the hub accept `device` (a network ID), a new device of this person being linked.
pub async fn introduce<R, Rd, Wr>(replica: &R, hub: &str, device: &str, reader: &mut Rd, writer: &mut Wr) -> Result<()>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut report = HubReport::default();
    let (mut session, _) = open(replica, hub, None, reader, writer, &mut report).await?;
    match session.ask(RequestBody::Introduce(Introduce { device: device.trim().to_string() })).await {
        Ok(ReplyBody::Ok(_)) => session.bye().await,
        Ok(other) => Err(session.end_with(unexpected("Ok", &other)).await),
        Err(error) => Err(session.end_with(error).await),
    }
}

fn hub_error(code: i32, message: String) -> SyncError {
    if code == ErrorCode::NotRegistered as i32 {
        return SyncError::NotRegistered;
    }
    SyncError::Hub { code, message }
}

fn unexpected(wanted: &str, got: &ReplyBody) -> SyncError {
    let name = match got {
        ReplyBody::Error(_) => "Error",
        ReplyBody::Welcome(_) => "Welcome",
        ReplyBody::Ok(_) => "Ok",
        ReplyBody::Pushed(_) => "Pushed",
        ReplyBody::Batches(_) => "Batches",
        ReplyBody::Present(_) => "Present",
        ReplyBody::File(_) => "File",
        ReplyBody::Devices(_) => "Devices",
        ReplyBody::Keys(_) => "Keys",
    };
    SyncError::Protocol(format!("the hub answered {name}, expected {wanted}"))
}

#[cfg(test)]
mod stalling_hub {
    use std::time::Duration;

    use super::*;
    use crate::Certificate;
    use crate::hub::protocol::{Reply, ReplyBody, Welcome};

    /// A library copy with nothing to send.
    struct Empty {
        certificate: Certificate,
    }

    impl Replica for Empty {
        async fn library_id(&self) -> Result<String> {
            Ok("lib_test".into())
        }
        fn device(&self) -> String {
            self.certificate.device.clone()
        }
        fn certificate(&self) -> Certificate {
            self.certificate.clone()
        }
        async fn is_new(&self) -> Result<bool> {
            Ok(false)
        }
        async fn adopt(&self, _: &str) -> Result<()> {
            Ok(())
        }
        async fn received_from(&self, _: &str) -> Result<i64> {
            Ok(0)
        }
        async fn changes_for(&self, seq: i64, _: &Peer) -> Result<(Vec<Row>, i64)> {
            Ok((Vec::new(), seq))
        }
        async fn merge(&self, _: Vec<Row>, _: &Peer) -> Result<MergeSummary> {
            Ok(MergeSummary::default())
        }
        async fn record_received(&self, _: &str, _: i64) -> Result<()> {
            Ok(())
        }
        async fn missing_files(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
        async fn read_file(&self, _: &str, _: &Peer) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
        async fn write_file(&self, _: &str, _: Vec<u8>) -> Result<()> {
            Ok(())
        }
    }

    impl HubReplica for Empty {
        async fn library_keys(&self) -> Result<LibraryKeys> {
            Ok(LibraryKeys::new(crate::seal::LibraryKey::derive(&[7; 32], "lib_test")))
        }

        async fn save_library_keys(&self, _: &LibraryKeys) -> Result<()> {
            Ok(())
        }
        async fn local_files(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
        async fn snapshot(&self) -> Result<Vec<Row>> {
            Ok(Vec::new())
        }
        async fn wanted_files(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
    }

    /// A hub that answers every pull with "more" at the same position kept the device pulling
    /// forever; a pull that does not move on is now an error.
    #[tokio::test]
    async fn a_hub_that_never_moves_on_is_left() {
        let device = crate::transport::device_id(&[1; 32]);
        let replica = Empty { certificate: Certificate::issue(&[2; 32], &device) };
        let (mut ours, mut theirs) = tokio::io::duplex(1 << 16);
        tokio::spawn(async move {
            let (mut reader, mut writer) = tokio::io::split(&mut theirs);
            while let Ok(Some(request)) = protocol::receive::<_, Request>(&mut reader).await {
                let body = match request.body {
                    Some(RequestBody::Hello(_)) => ReplyBody::Welcome(Welcome {
                        registered: true,
                        libraries: vec!["lib_test".into()],
                        ..Welcome::default()
                    }),
                    Some(RequestBody::Pull(_)) => {
                        ReplyBody::Batches(Batches { batches: Vec::new(), up_to: 0, more: true, since_snapshot: 0 })
                    }
                    _ => return,
                };
                if protocol::send(&mut writer, &Reply::of(body)).await.is_err() {
                    return;
                }
                let _ = writer.flush().await;
            }
        });
        let (mut reader, mut writer) = tokio::io::split(&mut ours);
        let outcome = tokio::time::timeout(
            Duration::from_secs(5),
            sync(&replica, "hub", &SyncOptions::default(), &mut reader, &mut writer),
        )
        .await
        .expect("the session ends instead of pulling forever");
        assert!(matches!(outcome, Err(SyncError::Protocol(_))), "{outcome:?}");
    }
}
