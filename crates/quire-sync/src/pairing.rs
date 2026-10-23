//! Linking a new device to one of the person's devices, as Signal and WhatsApp do it
//! (`.tasks/T14-security.md`): no key is ever shown or copied.
//!
//! The offering device (it has the person's keys) opens a one-time endpoint and shows a
//! [`DeviceLink`] as a QR code: that endpoint's ID, our own relay and a 256-bit one-time secret.
//! The new device scans it, dials that ID (iroh's TLS proves it reached the device on the screen)
//! and proves with the secret that it saw the screen. Both then show the same three check words
//! (from the secret and both endpoints: someone who connected first with a photo of the code would
//! get other words); the person compares them and allows the new device on the offering one, which
//! sends the user key, the hub and the relay inside the encrypted channel.
//!
//! One request with the right secret is answered, within [`WAIT`]; wrong ones are refused (and the
//! offer closes after [`MAX_WRONG`]). No PAKE: the secret is long and random, and the QR code is the
//! trusted channel. The messages are JSON with a 4-byte length in front, as in sync sessions.

use std::str::FromStr;
use std::time::{Duration, Instant};

use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::{Endpoint, EndpointId, SecretKey};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Result, SyncError};
use crate::link::DeviceLink;
use crate::{identity, seal, transport};

/// The application protocol on the QUIC connection.
pub const ALPN: &[u8] = b"quire/link/1";
/// How long an offer stays open (the QR code is valid).
pub const WAIT: Duration = Duration::from_secs(120);
/// Wrong secrets an offer takes before it closes.
pub const MAX_WRONG: usize = 8;
/// How long each side waits for the other's next message (the person's decision excepted).
const ANSWER_WAIT: Duration = Duration::from_secs(15);
/// Largest message (a grant is a few hundred bytes).
const MAX_MESSAGE: u32 = 16 * 1024;
/// Longest device name kept (it is only shown).
const MAX_NAME: usize = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Message {
    /// The new device: the secret from the QR code, its name to show, and its device ID (the
    /// offering device introduces it to the hub, which lets no unknown device in otherwise).
    Ask {
        secret: String,
        name: String,
        #[serde(default)]
        device: String,
    },
    /// The secret is right: the person decides on the offering device.
    Wait,
    /// Allowed: what the new device needs.
    Give {
        grant: Grant,
    },
    Refuse {
        reason: String,
    },
}

/// What the offering device gives the new one: the person's user key, their hub and relay, and
/// the library's later keys (after an unlink they are not derived from the user key).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub user_key: String,
    pub hub: Option<String>,
    pub relay: Option<String>,
    /// The library those keys are of, and the chain (`LibraryKeys::chain_text`; empty: none).
    #[serde(default)]
    pub library: Option<String>,
    #[serde(default)]
    pub library_keys: String,
}

impl std::fmt::Debug for Grant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never the keys in logs.
        f.debug_struct("Grant").field("hub", &self.hub).field("relay", &self.relay).finish_non_exhaustive()
    }
}

/// Short, distinct words for the check (64: 6 bits each, 18 bits for three).
const WORDS: [&str; 64] = [
    "apple", "arrow", "badge", "berry", "blade", "bloom", "brick", "cabin", "candle", "cedar", "chalk", "cliff",
    "cloud", "comet", "coral", "crane", "delta", "dune", "eagle", "ember", "fern", "flint", "frost", "garden", "glass",
    "grape", "harbor", "hazel", "honey", "island", "ivory", "jungle", "kettle", "lemon", "lilac", "maple", "marble",
    "meadow", "mint", "nectar", "ocean", "olive", "orbit", "otter", "pebble", "pepper", "pine", "plum", "prairie",
    "quartz", "raven", "reef", "river", "saddle", "shadow", "spruce", "stone", "thunder", "tulip", "velvet", "violet",
    "walnut", "willow", "zephyr",
];

/// The three words both devices show: from the secret and both endpoints' IDs.
pub fn check_words(secret: &[u8; 32], offering: &EndpointId, joining: &EndpointId) -> Vec<String> {
    let mut hasher = blake3::Hasher::new_derive_key("quire 2026-10-22 device link check words");
    hasher.update(secret);
    hasher.update(offering.as_bytes());
    hasher.update(joining.as_bytes());
    // Three different words (easier to compare): the digest's bytes in turn, repeats skipped.
    let mut words: Vec<String> = Vec::with_capacity(3);
    for byte in hasher.finalize().as_bytes() {
        let word = WORDS[usize::from(*byte) % WORDS.len()];
        if !words.iter().any(|w| w == word) {
            words.push(word.to_string());
            if words.len() == 3 {
                break;
            }
        }
    }
    words
}

/// Whether `given` (hex) is the offer's secret; compared as BLAKE3 hashes (equality in constant time).
fn secret_matches(given: &str, secret: &[u8; 32]) -> bool {
    identity::key_from_hex(given).is_some_and(|given| blake3::hash(&given) == blake3::hash(secret))
}

/// A name fit to show: no control characters, at most [`MAX_NAME`] characters.
fn clean_name(name: &str) -> String {
    let name: String = name.chars().filter(|c| !c.is_control()).take(MAX_NAME).collect();
    if name.trim().is_empty() { "a device".to_string() } else { name.trim().to_string() }
}

async fn send<W: AsyncWrite + Unpin>(writer: &mut W, message: &Message) -> Result<()> {
    let body = serde_json::to_vec(message).map_err(|e| SyncError::Protocol(e.to_string()))?;
    let length = u32::try_from(body.len()).ok().filter(|l| *l <= MAX_MESSAGE).ok_or(SyncError::TooLarge)?;
    writer.write_all(&length.to_be_bytes()).await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

async fn receive<R: AsyncRead + Unpin>(reader: &mut R, wait: Duration) -> Result<Message> {
    let read = async {
        let length = reader.read_u32().await?;
        if length > MAX_MESSAGE {
            return Err(SyncError::TooLarge);
        }
        let mut body = vec![0; length as usize];
        reader.read_exact(&mut body).await?;
        serde_json::from_slice(&body).map_err(|e| SyncError::Protocol(format!("unreadable link message: {e}")))
    };
    tokio::time::timeout(wait, read).await.map_err(|_| SyncError::Link("the other device did not answer".into()))?
}

/// The offering side of one request: the asker's name and device ID if it gave the right secret
/// (it is told to wait), `None` if not (it is refused).
async fn answer<R, W>(reader: &mut R, writer: &mut W, secret: &[u8; 32]) -> Result<Option<(String, String)>>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    match receive(reader, ANSWER_WAIT).await? {
        Message::Ask { secret: given, name, device } if secret_matches(&given, secret) => {
            if !crate::transport::is_device_id(&device) {
                send(writer, &Message::Refuse { reason: "update quire on the new device".into() }).await?;
                return Err(SyncError::Link("the new device runs an older quire: update it first".into()));
            }
            send(writer, &Message::Wait).await?;
            Ok(Some((clean_name(&name), device.trim().to_string())))
        }
        Message::Ask { .. } => {
            send(writer, &Message::Refuse { reason: "wrong or old code".into() }).await?;
            Ok(None)
        }
        _ => Err(SyncError::Protocol("a link request was expected".into())),
    }
}

/// The joining side's first step: asks with the secret, `name` and `device` (its ID); `Ok` once
/// told to wait.
async fn ask<R, W>(reader: &mut R, writer: &mut W, secret: &[u8; 32], name: &str, device: &str) -> Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let ask = Message::Ask { secret: identity::key_to_hex(secret), name: name.to_string(), device: device.to_string() };
    send(writer, &ask).await?;
    match receive(reader, ANSWER_WAIT).await? {
        Message::Wait => Ok(()),
        Message::Refuse { reason } => Err(SyncError::Link(format!("the other device refused: {reason}"))),
        _ => Err(SyncError::Protocol("unexpected link message".into())),
    }
}

/// The joining side's last step: the grant, once the person allowed it (up to [`WAIT`]).
async fn granted<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Grant> {
    match receive(reader, WAIT).await? {
        Message::Give { grant } => Ok(grant),
        Message::Refuse { reason } => Err(SyncError::Link(format!("the other device refused: {reason}"))),
        _ => Err(SyncError::Protocol("unexpected link message".into())),
    }
}

fn expired() -> SyncError {
    SyncError::Link("the link expired: show a new code".into())
}

/// An offer to link a device: a one-time endpoint and its link, open for [`WAIT`].
pub struct Offer {
    endpoint: Endpoint,
    link: DeviceLink,
    deadline: Instant,
    /// Requests with a wrong code so far (shown to the person: someone may have tried).
    wrong: std::sync::atomic::AtomicUsize,
}

impl Offer {
    /// Opens an offer reachable through `relay` (our own; `None`: n0's relays and lookup).
    pub async fn open(relay: Option<&str>) -> Result<Self> {
        let relay = relay.map(transport::parse_relay).transpose()?;
        // A fresh key: the device's own ID is not shown in the code.
        let key = SecretKey::from_bytes(&seal::random()?);
        let endpoint = transport::bind_endpoint(key, relay.as_ref(), vec![ALPN.to_vec()]).await?;
        let link =
            DeviceLink { id: endpoint.id().to_string(), relay: relay.map(|r| r.to_string()), secret: seal::random()? };
        Ok(Self { endpoint, link, deadline: Instant::now() + WAIT, wrong: Default::default() })
    }

    /// What to show (as a QR code or text).
    pub fn link(&self) -> &DeviceLink {
        &self.link
    }

    /// How many requests came with a wrong code.
    pub fn wrong_codes(&self) -> usize {
        self.wrong.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Waits for a request with the right secret, refusing others; fails after [`WAIT`] or
    /// [`MAX_WRONG`] wrong secrets.
    pub async fn request(&self) -> Result<Request> {
        loop {
            let left = self.deadline.saturating_duration_since(Instant::now());
            let incoming = tokio::time::timeout(left, self.endpoint.accept())
                .await
                .map_err(|_| expired())?
                .ok_or_else(|| SyncError::Link("the offer was closed".into()))?;
            let accepting = match incoming.accept() {
                Ok(accepting) => accepting,
                Err(error) => {
                    tracing::debug!(%error, "link connection refused");
                    continue;
                }
            };
            let connection = match accepting.await {
                Ok(connection) => connection,
                Err(error) => {
                    tracing::debug!(%error, "link connection failed");
                    continue;
                }
            };
            let Ok(Ok((mut send, mut receive))) = tokio::time::timeout(ANSWER_WAIT, connection.accept_bi()).await
            else {
                connection.close(1u8.into(), b"no request");
                continue;
            };
            match answer(&mut receive, &mut send, &self.link.secret).await {
                Ok(Some((name, device))) => {
                    let words = check_words(&self.link.secret, &self.endpoint.id(), &connection.remote_id());
                    return Ok(Request { connection, send, name, device, words });
                }
                Ok(None) => {
                    let _ = send.finish();
                    let wrong = self.wrong.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    tracing::warn!(wrong, "a device tried to link with a wrong code");
                    if wrong >= MAX_WRONG {
                        return Err(SyncError::Link("too many wrong codes: the offer is closed".into()));
                    }
                }
                Err(error) => {
                    tracing::debug!(%error, "a link request failed");
                    connection.close(1u8.into(), b"failed");
                }
            }
        }
    }

    /// Stops the offer (the code no longer works).
    pub async fn close(&self) {
        self.endpoint.close().await;
    }
}

/// A request with the right secret, waiting for the person's decision.
pub struct Request {
    connection: Connection,
    send: SendStream,
    /// The new device's name (as it said; only shown).
    pub name: String,
    /// Its device ID, to introduce to the hub before allowing it.
    pub device: String,
    /// The words both devices show.
    pub words: Vec<String>,
}

impl Request {
    /// Sends `grant` to the new device.
    pub async fn allow(mut self, grant: &Grant) -> Result<()> {
        send(&mut self.send, &Message::Give { grant: grant.clone() }).await?;
        self.end().await;
        Ok(())
    }

    pub async fn refuse(mut self) -> Result<()> {
        send(&mut self.send, &Message::Refuse { reason: "not allowed".into() }).await?;
        self.end().await;
        Ok(())
    }

    /// The new device closes once it has read the answer; closing first could drop it.
    async fn end(mut self) {
        let _ = self.send.finish();
        let _ = tokio::time::timeout(ANSWER_WAIT, self.connection.closed()).await;
    }
}

/// The new device's side: connected and told to wait; [`Join::words`] to show.
pub struct Join {
    endpoint: Endpoint,
    connection: Connection,
    receive: RecvStream,
    pub words: Vec<String>,
}

impl Join {
    /// Dials the offering device of `link` and asks, as `name`, this device being `device` (its ID).
    pub async fn connect(link: &DeviceLink, name: &str, device: &str) -> Result<Self> {
        let relay = link.relay.as_deref().map(transport::parse_relay).transpose()?;
        let offering =
            EndpointId::from_str(&link.id).map_err(|e| SyncError::Link(format!("the code is damaged: {e}")))?;
        let endpoint =
            transport::bind_endpoint(SecretKey::from_bytes(&seal::random()?), relay.as_ref(), vec![]).await?;
        let asked = async {
            let connection = transport::dial(&endpoint, relay.as_ref(), offering, ALPN).await?;
            let (mut send, mut receive) = connection.open_bi().await.map_err(|e| SyncError::Network(e.to_string()))?;
            ask(&mut receive, &mut send, &link.secret, name, device).await?;
            let _ = send.finish();
            Ok::<_, SyncError>((connection, receive))
        };
        match asked.await {
            Ok((connection, receive)) => {
                let words = check_words(&link.secret, &offering, &endpoint.id());
                Ok(Self { endpoint, connection, receive, words })
            }
            Err(error) => {
                endpoint.close().await;
                Err(error)
            }
        }
    }

    /// Gives up (the person cancelled on this device).
    pub async fn close(self) {
        self.connection.close(1u8.into(), b"cancelled");
        self.endpoint.close().await;
    }

    /// Waits for the person's decision on the other device: the grant if allowed.
    pub async fn finish(mut self) -> Result<Grant> {
        let grant = granted(&mut self.receive).await;
        self.connection.close(0u8.into(), b"done");
        self.endpoint.close().await;
        grant
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant() -> Grant {
        Grant {
            user_key: "ab".repeat(32),
            hub: Some("hub".into()),
            relay: None,
            library: Some("lib".into()),
            library_keys: "1:cd".into(),
        }
    }

    #[tokio::test]
    async fn the_right_secret_is_told_to_wait_then_gets_the_grant() {
        let secret = [3; 32];
        let (mut a, mut b) = tokio::io::duplex(4096);
        let (mut a_read, mut a_write) = tokio::io::split(&mut a);
        let (mut b_read, mut b_write) = tokio::io::split(&mut b);
        let offering = async {
            let asked = answer(&mut a_read, &mut a_write, &secret).await.unwrap();
            send(&mut a_write, &Message::Give { grant: grant() }).await.unwrap();
            asked
        };
        let device = crate::transport::device_id(&[5; 32]);
        let joining = async {
            ask(&mut b_read, &mut b_write, &secret, "Pixel\u{7} 9", &device).await.unwrap();
            granted(&mut b_read).await.unwrap()
        };
        let (asked, got) = tokio::join!(offering, joining);
        assert_eq!(asked, Some(("Pixel 9".to_string(), device)), "control characters are dropped");
        assert_eq!(got, grant());
    }

    #[tokio::test]
    async fn a_wrong_secret_is_refused() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        let (mut a_read, mut a_write) = tokio::io::split(&mut a);
        let (mut b_read, mut b_write) = tokio::io::split(&mut b);
        let thief = crate::transport::device_id(&[6; 32]);
        let (offered, asked) = tokio::join!(
            answer(&mut a_read, &mut a_write, &[3; 32]),
            ask(&mut b_read, &mut b_write, &[4; 32], "thief", &thief),
        );
        assert_eq!(offered.unwrap(), None);
        assert!(matches!(asked, Err(SyncError::Link(_))), "{asked:?}");
    }

    #[tokio::test]
    async fn a_refusal_reaches_the_new_device() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        send(&mut a, &Message::Refuse { reason: "not allowed".into() }).await.unwrap();
        assert!(matches!(granted(&mut b).await, Err(SyncError::Link(m)) if m.contains("not allowed")));
    }

    #[test]
    fn check_words_depend_on_both_devices_and_the_secret() {
        let ids: Vec<EndpointId> = (1..=3u8).map(|i| SecretKey::from_bytes(&[i; 32]).public()).collect();
        let words = check_words(&[9; 32], &ids[0], &ids[1]);
        assert_eq!(words.len(), 3);
        assert!(words[0] != words[1] && words[1] != words[2] && words[0] != words[2], "{words:?}");
        assert_eq!(words, check_words(&[9; 32], &ids[0], &ids[1]), "both sides compute the same");
        // Someone else connecting (another joining ID) or another code gives other words.
        let others = [check_words(&[9; 32], &ids[0], &ids[2]), check_words(&[8; 32], &ids[0], &ids[1])];
        assert!(others.iter().all(|o| *o != words), "{others:?} vs {words:?}");
    }

    #[test]
    fn the_keys_never_show_in_debug_output() {
        let shown = format!("{:?}", grant());
        assert!(!shown.contains(&"ab".repeat(32)) && !shown.contains("1:cd"), "{shown}");
    }
}
