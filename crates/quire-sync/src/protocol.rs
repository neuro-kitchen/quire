//! The messages of a sync session and how they travel: each is JSON with a 4-byte big-endian
//! length in front; a file follows its [`Message::File`] header as raw bytes.

use quire_core::sync::Row;

use crate::identity::Certificate;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Result, SyncError};

/// Largest message (a batch of rows; files are sent apart).
const MAX_MESSAGE: u32 = 64 * 1024 * 1024;
/// Largest file.
pub(crate) const MAX_FILE: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Who is talking, about which library; `new`: this copy has nothing yet and may join;
    /// `certificate`: the person this device belongs to.
    Hello { protocol: u32, library: String, device: String, new: bool, certificate: Certificate },
    /// The highest change number of the other side's already merged here.
    Start { received: i64 },
    /// Changed rows, up to the sender's change number `last`; `more`: another batch follows.
    Changes { rows: Vec<Row>, last: i64, more: bool },
    /// Files (by BLAKE3 hash) the sender's attachments name but it does not have.
    Want { hashes: Vec<String> },
    /// A file of `size` bytes follows.
    File { hash: String, size: u64 },
    /// The files asked for that the sender has were all sent.
    End,
}

pub async fn send<W: AsyncWrite + Unpin>(writer: &mut W, message: &Message) -> Result<()> {
    let body = serde_json::to_vec(message).map_err(|e| SyncError::Protocol(e.to_string()))?;
    let length = u32::try_from(body.len()).ok().filter(|l| *l <= MAX_MESSAGE).ok_or(SyncError::TooLarge)?;
    writer.write_all(&length.to_be_bytes()).await?;
    writer.write_all(&body).await?;
    Ok(())
}

pub async fn receive<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Message> {
    let length = reader.read_u32().await?;
    if length > MAX_MESSAGE {
        return Err(SyncError::TooLarge);
    }
    let mut body = vec![0; length as usize];
    reader.read_exact(&mut body).await?;
    serde_json::from_slice(&body).map_err(|e| SyncError::Protocol(format!("unreadable message: {e}")))
}

pub(crate) async fn send_file<W: AsyncWrite + Unpin>(writer: &mut W, hash: &str, bytes: &[u8]) -> Result<()> {
    send(writer, &Message::File { hash: hash.to_string(), size: bytes.len() as u64 }).await?;
    writer.write_all(bytes).await?;
    Ok(())
}

pub(crate) async fn receive_bytes<R: AsyncRead + Unpin>(reader: &mut R, size: u64) -> Result<Vec<u8>> {
    if size > MAX_FILE {
        return Err(SyncError::TooLarge);
    }
    // Read as the bytes come, not allocated up front from what the header claims.
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

    #[tokio::test]
    async fn frames_round_trip() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        let certificate = Certificate { user: "u".into(), device: "d".into(), signature: "s".into() };
        let hello = Message::Hello { protocol: 1, library: "lib".into(), device: "dev".into(), new: true, certificate };
        send(&mut a, &hello).await.unwrap();
        send_file(&mut a, "abc", b"PDF bytes").await.unwrap();
        assert_eq!(receive(&mut b).await.unwrap(), hello);
        let Message::File { size, .. } = receive(&mut b).await.unwrap() else { panic!() };
        assert_eq!(receive_bytes(&mut b, size).await.unwrap(), b"PDF bytes");
    }
}
