//! The hub's messages (`.knowledge/sync-hub.md`): Protocol Buffers (`prost` derive), each with a
//! 4-byte big-endian length in front; a file's bytes follow its `PutFile` request or `FileHeader`
//! reply raw. The same messages can become a gRPC service later (`tonic`) for clients without
//! iroh. The schema is also written out in `proto/quire/hub/v1/hub.proto` (the tags must agree).
//!
//! A session is requests from the device, each answered by one reply. The hub sees only sealed
//! batches and sealed files ([`crate::seal`]), the library's ID and the devices' IDs.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Result, SyncError};

/// Protocol version; device and hub must speak the same one.
pub const PROTOCOL: u32 = 1;
/// Largest message (a reply with batches; files travel apart).
pub const MAX_MESSAGE: u32 = 64 * 1024 * 1024;
/// Largest sealed file.
pub const MAX_FILE: u64 = 2 * 1024 * 1024 * 1024 + 1024 * 1024;

#[derive(Clone, PartialEq, prost::Message)]
pub struct Certificate {
    #[prost(string, tag = "1")]
    pub user: String,
    #[prost(string, tag = "2")]
    pub device: String,
    #[prost(string, tag = "3")]
    pub signature: String,
}

impl From<&crate::identity::Certificate> for Certificate {
    fn from(c: &crate::identity::Certificate) -> Self {
        Self { user: c.user.clone(), device: c.device.clone(), signature: c.signature.clone() }
    }
}

impl From<&Certificate> for crate::identity::Certificate {
    fn from(c: &Certificate) -> Self {
        Self { user: c.user.clone(), device: c.device.clone(), signature: c.signature.clone() }
    }
}

/// What a sealed batch holds (as [`crate::seal::Kind`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum BatchKind {
    Changes = 0,
    Snapshot = 1,
}

impl From<BatchKind> for crate::seal::Kind {
    fn from(kind: BatchKind) -> Self {
        match kind {
            BatchKind::Changes => crate::seal::Kind::Changes,
            BatchKind::Snapshot => crate::seal::Kind::Snapshot,
        }
    }
}

/// A sealed batch of one library copy's changes (or its snapshot).
#[derive(Clone, PartialEq, prost::Message)]
pub struct Batch {
    /// The library copy that wrote it (its device ID in the library, not its network key).
    #[prost(string, tag = "1")]
    pub author: String,
    /// The author's change number at the end of the batch (increasing per author).
    #[prost(uint64, tag = "2")]
    pub number: u64,
    #[prost(enumeration = "BatchKind", tag = "3")]
    pub kind: i32,
    #[prost(bytes = "vec", tag = "4")]
    pub sealed: Vec<u8>,
    /// For a snapshot: the hub position its author had pulled up to (it contains all before).
    #[prost(uint64, tag = "5")]
    pub covers: u64,
    /// The hub's position of this batch (set by the hub in replies).
    #[prost(uint64, tag = "6")]
    pub seq: u64,
}

/// First request: who is asking, about which library.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Hello {
    #[prost(uint32, tag = "1")]
    pub protocol: u32,
    #[prost(message, optional, tag = "2")]
    pub certificate: Option<Certificate>,
    #[prost(string, tag = "3")]
    pub library: String,
    /// This library copy's device ID (the author of its batches).
    #[prost(string, tag = "4")]
    pub author: String,
}

/// Binds this person to the hub with a one-time code (the setup code, an invitation or a link code).
#[derive(Clone, PartialEq, prost::Message)]
pub struct Register {
    #[prost(string, tag = "1")]
    pub code: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Push {
    #[prost(message, repeated, tag = "1")]
    pub batches: Vec<Batch>,
}

/// Batches after hub position `after` (other authors' only), at most `limit`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Pull {
    #[prost(uint64, tag = "1")]
    pub after: u64,
    #[prost(uint32, tag = "2")]
    pub limit: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct FileIds {
    #[prost(string, repeated, tag = "1")]
    pub ids: Vec<String>,
}

/// A sealed file of `size` bytes follows.
#[derive(Clone, PartialEq, prost::Message)]
pub struct PutFile {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(uint64, tag = "2")]
    pub size: u64,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct GetFile {
    #[prost(string, tag = "1")]
    pub id: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Revoke {
    /// A device's network ID.
    #[prost(string, tag = "1")]
    pub device: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Empty {}

#[derive(Clone, PartialEq, prost::Oneof)]
pub enum RequestBody {
    #[prost(message, tag = "1")]
    Hello(Hello),
    #[prost(message, tag = "2")]
    Register(Register),
    #[prost(message, tag = "3")]
    Push(Push),
    #[prost(message, tag = "4")]
    Pull(Pull),
    #[prost(message, tag = "5")]
    HasFiles(FileIds),
    #[prost(message, tag = "6")]
    PutFile(PutFile),
    #[prost(message, tag = "7")]
    GetFile(GetFile),
    #[prost(message, tag = "8")]
    Devices(Empty),
    #[prost(message, tag = "9")]
    Revoke(Revoke),
    #[prost(message, tag = "10")]
    Bye(Empty),
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Request {
    #[prost(oneof = "RequestBody", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10")]
    pub body: Option<RequestBody>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum ErrorCode {
    Protocol = 0,
    /// This person has no account on the hub: register with a code.
    NotRegistered = 1,
    BadCode = 2,
    /// This device was revoked.
    Revoked = 3,
    /// Over the account's storage quota.
    Quota = 4,
    NotFound = 5,
    TooLarge = 6,
    /// Too many requests: try later.
    Busy = 7,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Error {
    #[prost(enumeration = "ErrorCode", tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub message: String,
}

/// The answer to `Hello` (and to `Register`).
#[derive(Clone, PartialEq, prost::Message)]
pub struct Welcome {
    /// This person has an account here.
    #[prost(bool, tag = "1")]
    pub registered: bool,
    /// The last batch number the hub has from this library copy (`Hello.author`).
    #[prost(uint64, tag = "2")]
    pub last_number: u64,
    /// The library's latest hub position.
    #[prost(uint64, tag = "3")]
    pub hub_seq: u64,
    /// Bytes stored for this account, and its limit (0: none).
    #[prost(uint64, tag = "4")]
    pub used: u64,
    #[prost(uint64, tag = "5")]
    pub quota: u64,
    /// The libraries this account holds here (a new, empty copy joins one of them).
    #[prost(string, repeated, tag = "6")]
    pub libraries: Vec<String>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Pushed {
    #[prost(uint64, tag = "1")]
    pub hub_seq: u64,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Batches {
    /// A snapshot (when the device is behind it) first, then batches in hub order.
    #[prost(message, repeated, tag = "1")]
    pub batches: Vec<Batch>,
    /// The hub position these cover (the next `Pull.after`), own batches skipped included.
    #[prost(uint64, tag = "2")]
    pub up_to: u64,
    #[prost(bool, tag = "3")]
    pub more: bool,
    /// Change batches stored since the latest snapshot: the device makes a new one when many.
    #[prost(uint64, tag = "4")]
    pub since_snapshot: u64,
}

/// A sealed file of `size` bytes follows.
#[derive(Clone, PartialEq, prost::Message)]
pub struct FileHeader {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(uint64, tag = "2")]
    pub size: u64,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct DeviceInfo {
    #[prost(string, tag = "1")]
    pub device: String,
    /// Milliseconds since 1970.
    #[prost(int64, tag = "2")]
    pub first_seen: i64,
    #[prost(int64, tag = "3")]
    pub last_seen: i64,
    #[prost(bool, tag = "4")]
    pub revoked: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct DeviceList {
    #[prost(message, repeated, tag = "1")]
    pub devices: Vec<DeviceInfo>,
}

#[derive(Clone, PartialEq, prost::Oneof)]
pub enum ReplyBody {
    #[prost(message, tag = "1")]
    Error(Error),
    #[prost(message, tag = "2")]
    Welcome(Welcome),
    #[prost(message, tag = "3")]
    Ok(Empty),
    #[prost(message, tag = "4")]
    Pushed(Pushed),
    #[prost(message, tag = "5")]
    Batches(Batches),
    #[prost(message, tag = "6")]
    Present(FileIds),
    #[prost(message, tag = "7")]
    File(FileHeader),
    #[prost(message, tag = "8")]
    Devices(DeviceList),
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Reply {
    #[prost(oneof = "ReplyBody", tags = "1, 2, 3, 4, 5, 6, 7, 8")]
    pub body: Option<ReplyBody>,
}

impl Reply {
    pub fn of(body: ReplyBody) -> Self {
        Self { body: Some(body) }
    }

    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::of(ReplyBody::Error(Error { code: code as i32, message: message.into() }))
    }
}

impl Request {
    pub fn of(body: RequestBody) -> Self {
        Self { body: Some(body) }
    }
}

/// Writes one message with its length in front.
pub async fn send<W: AsyncWrite + Unpin, M: prost::Message>(writer: &mut W, message: &M) -> Result<()> {
    let body = message.encode_to_vec();
    let length = u32::try_from(body.len()).ok().filter(|l| *l <= MAX_MESSAGE).ok_or(SyncError::TooLarge)?;
    writer.write_all(&length.to_be_bytes()).await?;
    writer.write_all(&body).await?;
    Ok(())
}

/// Reads one message; `None` when the other side closed cleanly before it.
pub async fn receive<R: AsyncRead + Unpin, M: prost::Message + Default>(reader: &mut R) -> Result<Option<M>> {
    let length = match reader.read_u32().await {
        Ok(length) => length,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if length > MAX_MESSAGE {
        return Err(SyncError::TooLarge);
    }
    let mut body = vec![0; length as usize];
    reader.read_exact(&mut body).await?;
    M::decode(body.as_slice()).map(Some).map_err(|e| SyncError::Protocol(format!("unreadable hub message: {e}")))
}

/// Reads `size` raw bytes (a file after its header).
pub async fn receive_bytes<R: AsyncRead + Unpin>(reader: &mut R, size: u64) -> Result<Vec<u8>> {
    if size > MAX_FILE {
        return Err(SyncError::TooLarge);
    }
    // Read as the bytes come, not allocated up front from what the header claims (on the hub,
    // any device's `PutFile` would otherwise cost the declared size in memory at once).
    let mut bytes = Vec::new();
    reader.take(size).read_to_end(&mut bytes).await?;
    if bytes.len() as u64 != size {
        return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// `(message, field, tag)` of every field and enum value in `hub.proto`.
    fn proto_fields(proto: &str) -> BTreeSet<(String, String, u32)> {
        let mut out = BTreeSet::new();
        let mut current = String::new();
        for line in proto.lines().map(|l| l.split("//").next().unwrap_or("").trim()) {
            let mut rest = line;
            for keyword in ["message ", "enum "] {
                if let Some(after) = line.strip_prefix(keyword) {
                    let (name, body) = after.split_once('{').unwrap_or((after, ""));
                    current = name.trim().to_string();
                    rest = body;
                }
            }
            for part in rest.split(';') {
                let Some((left, tag)) = part.split_once('=') else { continue };
                let name = left.split_whitespace().last().unwrap_or("").to_string();
                if let Ok(tag) = tag.trim().trim_end_matches('}').trim().parse() {
                    out.insert((current.clone(), name, tag));
                }
            }
        }
        out
    }

    /// The same, from this file's `prost` attributes (oneof variants and enum values included).
    fn derive_fields(source: &str) -> BTreeSet<(String, String, u32)> {
        let snake = |name: &str| {
            name.chars().enumerate().fold(String::new(), |mut s, (i, c)| {
                if c.is_uppercase() && i > 0 {
                    s.push('_');
                }
                s.push(c.to_ascii_lowercase());
                s
            })
        };
        let mut out = BTreeSet::new();
        let (mut current, mut tag): (String, Option<u32>) = (String::new(), None);
        let mut enumeration = false;
        for line in source.lines().map(str::trim).take_while(|l| *l != "#[cfg(test)]") {
            if line.contains("prost::Enumeration") {
                enumeration = true;
            } else if line.contains("prost::Message") || line.contains("prost::Oneof") {
                enumeration = false;
            }
            if let Some(name) = line.strip_prefix("pub struct ").or_else(|| line.strip_prefix("pub enum ")) {
                let name = name.trim_end_matches('{').trim();
                current = match name {
                    "RequestBody" => "Request".into(),
                    "ReplyBody" => "Reply".into(),
                    other => other.into(),
                };
            } else if let Some(at) = line.find("tag = \"") {
                tag = line[at + 7..].split('"').next().and_then(|t| t.parse().ok());
            } else if let (Some(n), Some(field)) = (tag, line.strip_prefix("pub ").and_then(|l| l.split(':').next())) {
                out.insert((current.clone(), field.to_string(), n));
                tag = None;
            } else if let (Some(n), Some((variant, _))) = (tag, line.split_once('(')) {
                out.insert((current.clone(), snake(variant), n));
                tag = None;
            } else if enumeration
                && let Some((variant, value)) = line.trim_end_matches(',').split_once(" = ")
                && let Ok(value) = value.parse()
            {
                out.insert((current.clone(), snake(variant).to_uppercase(), value));
            }
        }
        out
    }

    #[test]
    fn hub_proto_matches_the_derive() {
        let proto = proto_fields(include_str!("../../../../proto/quire/hub/v1/hub.proto"));
        let derive = derive_fields(include_str!("protocol.rs"));
        assert!(derive.len() > 50, "the parser found the derive's fields: {}", derive.len());
        let only_proto: Vec<_> = proto.difference(&derive).collect();
        let only_derive: Vec<_> = derive.difference(&proto).collect();
        assert!(
            only_proto.is_empty() && only_derive.is_empty(),
            "hub.proto only: {only_proto:?}\nderive only: {only_derive:?}"
        );
    }

    #[tokio::test]
    async fn messages_round_trip_with_raw_bytes_after() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        let push = Request::of(RequestBody::Push(Push {
            batches: vec![Batch {
                author: "d".into(),
                number: 3,
                kind: BatchKind::Snapshot as i32,
                sealed: vec![0, 1, 255],
                covers: 9,
                seq: 0,
            }],
        }));
        send(&mut a, &push).await.unwrap();
        send(&mut a, &Request::of(RequestBody::PutFile(PutFile { id: "f".into(), size: 3 }))).await.unwrap();
        a.write_all(b"abc").await.unwrap();
        drop(a);
        assert_eq!(receive::<_, Request>(&mut b).await.unwrap(), Some(push));
        let Some(Request { body: Some(RequestBody::PutFile(put)) }) = receive(&mut b).await.unwrap() else { panic!() };
        assert_eq!(receive_bytes(&mut b, put.size).await.unwrap(), b"abc");
        assert_eq!(receive::<_, Request>(&mut b).await.unwrap(), None, "a clean end");
    }
}
