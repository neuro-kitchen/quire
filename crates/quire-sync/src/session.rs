//! One sync session between two devices, over any byte stream (a QUIC stream in use, an
//! in-memory pipe in tests). The side that connected (the client) speaks first at each step:
//!
//! 1. `Hello` both ways: protocol, library, device, whether the copy is new. Different libraries
//!    stop the session, unless one copy is new: it joins the other's library.
//! 2. `Start` both ways: how far each has merged the other's changes.
//! 3. The client sends its changes since then (in batches), the server merges them; then the
//!    server sends its own, the client merges them. Each side merges a whole delivery at once,
//!    so rows always find what they point to.
//! 4. Files: each side asks for the PDFs it lacks (`Want`), the other sends those it has.

use std::future::Future;

use quire_core::sync::{Merged, Row};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::error::{Result, SyncError};
use crate::protocol::{self, Message};

/// Protocol version; both sides must speak the same one.
pub const PROTOCOL: u32 = 1;

/// A library copy as a session sees it (the engine implements it on its database and files).
pub trait Replica: Sync {
    fn library_id(&self) -> impl Future<Output = Result<String>> + Send;
    /// This device's ID.
    fn device(&self) -> String;
    /// Whether this copy has nothing yet (it may join another library).
    fn is_new(&self) -> impl Future<Output = Result<bool>> + Send;
    fn adopt(&self, library: &str) -> impl Future<Output = Result<()>> + Send;
    /// The highest change number of `device`'s merged here.
    fn received_from(&self, device: &str) -> impl Future<Output = Result<i64>> + Send;
    /// A batch of rows changed after `seq` to send to device `to` (not those merged from it),
    /// and the last change number in it.
    fn changes_since(&self, seq: i64, to: &str) -> impl Future<Output = Result<(Vec<Row>, i64)>> + Send;
    /// Merges rows that came from device `from`.
    fn merge(&self, rows: Vec<Row>, from: &str) -> impl Future<Output = Result<Merged>> + Send;
    fn record_received(&self, device: &str, seq: i64) -> impl Future<Output = Result<()>> + Send;
    /// Hashes of files the attachments here name but are not here.
    fn missing_files(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
    fn read_file(&self, hash: &str) -> impl Future<Output = Result<Option<Vec<u8>>>> + Send;
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
    /// This copy joined the other's library (it was new).
    pub joined: bool,
    pub sent: usize,
    pub received: Merged,
    pub files_sent: usize,
    pub files_received: usize,
}

pub async fn run<R, Rd, Wr>(replica: &R, role: Role, reader: &mut Rd, writer: &mut Wr) -> Result<Report>
where
    R: Replica,
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mine = Message::Hello {
        protocol: PROTOCOL,
        library: replica.library_id().await?,
        device: replica.device(),
        new: replica.is_new().await?,
    };
    let theirs = exchange(role, reader, writer, &mine).await?;
    let (Message::Hello { library: my_library, new: i_am_new, .. }, Message::Hello { protocol, library, device, new }) =
        (&mine, &theirs)
    else {
        return Err(SyncError::Protocol("expected Hello".into()));
    };
    if *protocol != PROTOCOL {
        return Err(SyncError::Protocol(format!(
            "the other device speaks sync protocol {protocol}, this one {PROTOCOL}"
        )));
    }
    let mut report = Report { peer: device.clone(), ..Report::default() };
    if library != my_library {
        if *i_am_new {
            replica.adopt(library).await?;
            report.joined = true;
        } else if !*new {
            return Err(SyncError::DifferentLibrary);
        }
    }
    let received = replica.received_from(device).await?;
    let Message::Start { received: they_have } = exchange(role, reader, writer, &Message::Start { received }).await?
    else {
        return Err(SyncError::Protocol("expected Start".into()));
    };

    // Changes: the client's first, then the server's.
    if role == Role::Client {
        report.sent = send_changes(replica, writer, they_have, device).await?;
        report.received = receive_changes(replica, reader, device).await?;
    } else {
        report.received = receive_changes(replica, reader, device).await?;
        report.sent = send_changes(replica, writer, they_have, device).await?;
    }

    // Files: the client asks first, then serves the server's request.
    if role == Role::Client {
        report.files_received = fetch_files(replica, reader, writer).await?;
        report.files_sent = serve_files(replica, reader, writer).await?;
    } else {
        report.files_sent = serve_files(replica, reader, writer).await?;
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
    to: &str,
) -> Result<usize> {
    let mut seq = since;
    let mut sent = 0;
    loop {
        let (rows, last) = replica.changes_since(seq, to).await?;
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
    peer: &str,
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
    replica.record_received(peer, up_to).await?;
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
) -> Result<usize> {
    let Message::Want { hashes } = protocol::receive(reader).await? else {
        return Err(SyncError::Protocol("expected Want".into()));
    };
    let mut sent = 0;
    for hash in hashes {
        if let Some(bytes) = replica.read_file(&hash).await? {
            protocol::send_file(writer, &hash, &bytes).await?;
            sent += 1;
        }
    }
    protocol::send(writer, &Message::End).await?;
    writer.flush().await?;
    Ok(sent)
}
