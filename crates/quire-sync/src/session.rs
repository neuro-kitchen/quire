//! One sync session between two devices, over any byte stream (a QUIC stream in use, an
//! in-memory pipe in tests). The side that connected (the client) speaks first at each step:
//!
//! 1. `Hello` both ways: protocol, library, device, whether the copy is new, and the device's
//!    certificate (whose person it is; checked against the device the connection authenticated).
//!    The same library (or a new copy joining it) is one of the user's own devices: everything is
//!    exchanged. Another library is a collaborator's: only what is shared with them is sent, and
//!    only what their role allows is taken from them (`Replica::changes_for`, `Replica::merge`).
//! 2. `Start` both ways: how far each has merged the other's changes.
//! 3. The client sends its changes since then (in batches), the server merges them; then the
//!    server sends its own, the client merges them. Each side merges a whole delivery at once,
//!    so rows always find what they point to.
//! 4. Files: each side asks for the PDFs it lacks (`Want`), the other sends those it has.

use std::future::Future;

use quire_core::sync::{Merged, Row};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::error::{Result, SyncError};
use crate::identity::Certificate;
use crate::protocol::{self, Message};

/// Protocol version; both sides must speak the same one. 2: certificates, collaborators.
pub const PROTOCOL: u32 = 2;

/// The other side of a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    /// Its library copy's device ID.
    pub device: String,
    /// Its person (from its certificate).
    pub user: String,
    /// Another person's library: what is shared with them only.
    pub collaborator: bool,
}

/// A library copy as a session sees it (the engine implements it on its database and files).
pub trait Replica: Sync {
    fn library_id(&self) -> impl Future<Output = Result<String>> + Send;
    /// This device's ID.
    fn device(&self) -> String;
    /// This device's certificate (its person's key signing its network ID).
    fn certificate(&self) -> Certificate;
    /// Whether this copy has nothing yet (it may join another library).
    fn is_new(&self) -> impl Future<Output = Result<bool>> + Send;
    fn adopt(&self, library: &str) -> impl Future<Output = Result<()>> + Send;
    /// The highest change number of `device`'s merged here.
    fn received_from(&self, device: &str) -> impl Future<Output = Result<i64>> + Send;
    /// A batch of rows changed after `seq` to send to `to` (not those merged from it; for a
    /// collaborator, only what is shared with them), and the last change number in it.
    fn changes_for(&self, seq: i64, to: &Peer) -> impl Future<Output = Result<(Vec<Row>, i64)>> + Send;
    /// Merges rows that came from `from` (a collaborator's: only what their role allows).
    fn merge(&self, rows: Vec<Row>, from: &Peer) -> impl Future<Output = Result<Merged>> + Send;
    fn record_received(&self, device: &str, seq: i64) -> impl Future<Output = Result<()>> + Send;
    /// Hashes of files the attachments here name but are not here.
    fn missing_files(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
    /// The file `hash`, if it is here and `to` may have it (a collaborator: PDFs of shared
    /// references only).
    fn read_file(&self, hash: &str, to: &Peer) -> impl Future<Output = Result<Option<Vec<u8>>>> + Send;
    /// Stores a received file (refused if its content does not have that hash).
    fn write_file(&self, hash: &str, bytes: Vec<u8>) -> impl Future<Output = Result<()>> + Send;
}

/// Which side of the connection this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Connected to the other device (speaks first).
    Client,
    /// Accepted the connection.
    Server,
}

/// What a session did, for the user.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// The other device.
    pub peer: String,
    /// Its person; set when it is a collaborator's (another library).
    pub collaborator: Option<String>,
    /// This copy joined the other's library (it was new).
    pub joined: bool,
    pub sent: usize,
    pub received: Merged,
    pub files_sent: usize,
    pub files_received: usize,
}

/// Runs a session with the device the connection authenticated as `authenticated` (its network
/// ID; `None` in tests over a pipe: its certificate is then only checked for its own device).
pub async fn run<R, Rd, Wr>(
    replica: &R,
    role: Role,
    authenticated: Option<&str>,
    reader: &mut Rd,
    writer: &mut Wr,
) -> Result<Report>
where
    R: Replica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let my_certificate = replica.certificate();
    let mine = Message::Hello {
        protocol: PROTOCOL,
        library: replica.library_id().await?,
        device: replica.device(),
        new: replica.is_new().await?,
        certificate: my_certificate.clone(),
    };
    let theirs = exchange(role, reader, writer, &mine).await?;
    let (
        Message::Hello { library: my_library, new: i_am_new, .. },
        Message::Hello { protocol, library, device, new, certificate },
    ) = (&mine, &theirs)
    else {
        return Err(SyncError::Protocol("expected Hello".into()));
    };
    if *protocol != PROTOCOL {
        return Err(SyncError::Protocol(format!(
            "the other device speaks sync protocol {protocol}, this one {PROTOCOL}"
        )));
    }
    let user = certificate.verify(authenticated.unwrap_or(&certificate.device))?;
    let mut report = Report { peer: device.clone(), ..Report::default() };
    // Everything is exchanged only with the same person's devices (the same user key): a library
    // ID is no secret (collaborators see it), so it proves nothing on its own.
    let collaborator = user != my_certificate.user;
    if collaborator {
        if library == my_library {
            return Err(SyncError::SameLibraryOtherUser);
        }
        report.collaborator = Some(user.clone());
    } else if library != my_library {
        if *i_am_new {
            replica.adopt(library).await?;
            report.joined = true;
        } else if !*new {
            return Err(SyncError::DifferentLibrary);
        }
    }
    let peer = Peer { device: device.clone(), user, collaborator };
    let received = replica.received_from(&peer.device).await?;
    let Message::Start { received: they_have } = exchange(role, reader, writer, &Message::Start { received }).await?
    else {
        return Err(SyncError::Protocol("expected Start".into()));
    };

    // Changes: the client's first, then the server's.
    if role == Role::Client {
        report.sent = send_changes(replica, writer, they_have, &peer).await?;
        report.received = receive_changes(replica, reader, &peer).await?;
    } else {
        report.received = receive_changes(replica, reader, &peer).await?;
        report.sent = send_changes(replica, writer, they_have, &peer).await?;
    }

    // Files: the client asks first, then serves the server's request.
    if role == Role::Client {
        report.files_received = fetch_files(replica, reader, writer).await?;
        report.files_sent = serve_files(replica, reader, writer, &peer).await?;
    } else {
        report.files_sent = serve_files(replica, reader, writer, &peer).await?;
        report.files_received = fetch_files(replica, reader, writer).await?;
    }
    writer.flush().await?;
    Ok(report)
}

/// Sends `mine` and receives theirs, in the order of `role`.
async fn exchange<Rd: AsyncRead + Unpin, Wr: AsyncWrite + Unpin>(
    role: Role,
    reader: &mut Rd,
    writer: &mut Wr,
    mine: &Message,
) -> Result<Message> {
    if role == Role::Client {
        protocol::send(writer, mine).await?;
        writer.flush().await?;
        protocol::receive(reader).await
    } else {
        let theirs = protocol::receive(reader).await?;
        protocol::send(writer, mine).await?;
        writer.flush().await?;
        Ok(theirs)
    }
}

async fn send_changes<R: Replica, Wr: AsyncWrite + Unpin>(
    replica: &R,
    writer: &mut Wr,
    since: i64,
    to: &Peer,
) -> Result<usize> {
    let mut seq = since;
    let mut sent = 0;
    loop {
        let (rows, last) = replica.changes_for(seq, to).await?;
        // A batch can be all echoes (empty) yet not the end: go on while the log advances.
        let more = last > seq;
        sent += rows.len();
        protocol::send(writer, &Message::Changes { rows, last, more }).await?;
        if !more {
            break;
        }
        seq = last;
    }
    writer.flush().await?;
    Ok(sent)
}

async fn receive_changes<R: Replica, Rd: AsyncRead + Unpin>(
    replica: &R,
    reader: &mut Rd,
    peer: &Peer,
) -> Result<Merged> {
    let mut all = Vec::new();
    let up_to = loop {
        let Message::Changes { rows, last, more } = protocol::receive(reader).await? else {
            return Err(SyncError::Protocol("expected Changes".into()));
        };
        all.extend(rows);
        if !more {
            break last;
        }
    };
    let merged = if all.is_empty() { Merged::default() } else { replica.merge(all, peer).await? };
    replica.record_received(&peer.device, up_to).await?;
    Ok(merged)
}

async fn fetch_files<R: Replica, Rd: AsyncRead + Unpin, Wr: AsyncWrite + Unpin>(
    replica: &R,
    reader: &mut Rd,
    writer: &mut Wr,
) -> Result<usize> {
    let hashes = replica.missing_files().await?;
    protocol::send(writer, &Message::Want { hashes }).await?;
    writer.flush().await?;
    let mut got = 0;
    loop {
        match protocol::receive(reader).await? {
            Message::File { hash, size } => {
                let bytes = protocol::receive_bytes(reader, size).await?;
                replica.write_file(&hash, bytes).await?;
                got += 1;
            }
            Message::End => return Ok(got),
            _ => return Err(SyncError::Protocol("expected File or End".into())),
        }
    }
}

async fn serve_files<R: Replica, Rd: AsyncRead + Unpin, Wr: AsyncWrite + Unpin>(
    replica: &R,
    reader: &mut Rd,
    writer: &mut Wr,
    to: &Peer,
) -> Result<usize> {
    let Message::Want { hashes } = protocol::receive(reader).await? else {
        return Err(SyncError::Protocol("expected Want".into()));
    };
    let mut sent = 0;
    for hash in hashes {
        if let Some(bytes) = replica.read_file(&hash, to).await? {
            protocol::send_file(writer, &hash, &bytes).await?;
            sent += 1;
        }
    }
    protocol::send(writer, &Message::End).await?;
    writer.flush().await?;
    Ok(sent)
}
