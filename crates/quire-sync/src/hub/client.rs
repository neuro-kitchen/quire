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
    self, Batch, BatchKind, Batches, ErrorCode, FileIds, GetFile, Hello, Pull, Push, PutFile, Register, Reply,
    ReplyBody, Request, RequestBody, Welcome,
};
use crate::error::{Result, SyncError};
use crate::seal::{Kind, LibraryKey};
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
    /// The key sealing this library's data for the hub.
    fn library_key(&self) -> impl Future<Output = Result<LibraryKey>> + Send;
    /// Hashes of the files here.
    fn local_files(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
    /// Every row of the library (a snapshot).
    fn snapshot(&self) -> impl Future<Output = Result<Vec<Row>>> + Send;
    /// Files to download in this session (missing here and wanted now: all of them, or only those
    /// kept offline when PDFs load as they are opened).
    fn wanted_files(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
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
}

/// The peer a hub is in the change log (its cursor, and the origin of rows pulled from it).
pub fn hub_peer(hub: &str, user: &str) -> Peer {
    Peer { device: format!("hub:{hub}"), user: user.to_string(), collaborator: false }
}

struct Session<'a, R, Rd, Wr> {
    replica: &'a R,
    reader: &'a mut Rd,
    writer: &'a mut Wr,
    key: LibraryKey,
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

    async fn hello(&mut self, code: Option<&str>, report: &mut HubReport) -> Result<Welcome> {
        let hello = Hello {
            protocol: protocol::PROTOCOL,
            certificate: Some((&self.replica.certificate()).into()),
            library: self.library.clone(),
            author: self.author.clone(),
        };
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
                let sealed = self.key.seal_batch(Kind::Changes, &self.library, &self.author, number, &plain)?;
                let batch = Batch {
                    author: self.author.clone(),
                    number,
                    kind: BatchKind::Changes as i32,
                    sealed,
                    covers: 0,
                    seq: 0,
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
                let plain =
                    self.key.open_batch(kind.into(), &self.library, &batch.author, batch.number, &batch.sealed)?;
                let mut part: Vec<Row> =
                    serde_json::from_slice(&plain).map_err(|e| SyncError::Protocol(format!("a batch's rows: {e}")))?;
                rows.append(&mut part);
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
            self.replica.local_files().await?.into_iter().map(|hash| (self.key.file_id(&hash), hash)).collect();
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
            let Some(bytes) = self.replica.read_file(hash, &own).await? else { continue };
            let sealed = self.key.seal_file(id, &bytes)?;
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
                Err(SyncError::Hub { code, .. }) if code == ErrorCode::Quota as i32 => report.over_quota += 1,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    async fn snapshot(&mut self, covers: u64) -> Result<()> {
        let rows = self.replica.snapshot().await?;
        let plain = serde_json::to_vec(&rows).map_err(|e| SyncError::Protocol(e.to_string()))?;
        let sealed = self.key.seal_batch(Kind::Snapshot, &self.library, &self.author, covers, &plain)?;
        let batch = Batch {
            author: self.author.clone(),
            number: covers,
            kind: BatchKind::Snapshot as i32,
            sealed,
            covers,
            seq: 0,
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
        key: replica.library_key().await?,
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
        session.key = replica.library_key().await?;
        welcome = session.hello(None, report).await?;
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
    let hello = Hello {
        protocol: protocol::PROTOCOL,
        certificate: Some((&session.replica.certificate()).into()),
        library: session.library.clone(),
        author: session.author.clone(),
    };
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
    let id = session.key.file_id(hash);
    let header = match session.ask(RequestBody::GetFile(GetFile { id: id.clone() })).await {
        Ok(ReplyBody::File(header)) => header,
        Ok(other) => return Err(unexpected("File", &other)),
        Err(SyncError::Hub { code, .. }) if code == ErrorCode::NotFound as i32 => return Ok(false),
        Err(e) => return Err(e),
    };
    let sealed = protocol::receive_bytes(session.reader, header.size).await?;
    let plain = session.key.open_file(&id, &sealed)?;
    // `write_file` refuses bytes whose BLAKE3 hash is not `hash`.
    session.replica.write_file(hash, plain).await?;
    Ok(true)
}

/// This person's devices known to the hub.
pub async fn devices<R, Rd, Wr>(
    replica: &R,
    hub: &str,
    reader: &mut Rd,
    writer: &mut Wr,
) -> Result<Vec<super::protocol::DeviceInfo>>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut report = HubReport::default();
    let (mut session, _) = open(replica, hub, None, reader, writer, &mut report).await?;
    let list = match session.ask(RequestBody::Devices(super::protocol::Empty {})).await {
        Ok(ReplyBody::Devices(list)) => list.devices,
        Ok(other) => return Err(session.end_with(unexpected("Devices", &other)).await),
        Err(error) => return Err(session.end_with(error).await),
    };
    session.bye().await?;
    Ok(list)
}

/// Makes the hub refuse `device` (a network ID) of this person from now on.
pub async fn revoke<R, Rd, Wr>(replica: &R, hub: &str, device: &str, reader: &mut Rd, writer: &mut Wr) -> Result<()>
where
    R: HubReplica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut report = HubReport::default();
    let (mut session, _) = open(replica, hub, None, reader, writer, &mut report).await?;
    match session.ask(RequestBody::Revoke(super::protocol::Revoke { device: device.trim().to_string() })).await {
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
        async fn library_key(&self) -> Result<LibraryKey> {
            Ok(LibraryKey::derive(&[7; 32], "lib_test"))
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
