//! Devices on the network with iroh (QUIC): each has a key pair kept by the engine; its public
//! key (`EndpointId`) is how other devices dial it, through n0's address lookup and relays when
//! no direct path exists (`presets::N0`), or through one relay of the user's own and nothing of
//! n0's (setting `sync.relay`). The connection is authenticated by those keys and encrypted end to
//! end; a device serves only the peers the user added.

use std::str::FromStr;

use iroh::endpoint::presets;
use iroh::{Endpoint, EndpointAddr, EndpointId, RelayMode, RelayUrl, SecretKey};

use crate::error::{Result, SyncError};
use crate::hub::protocol::DeviceInfo;
use crate::hub::{HubReplica, HubReport, SyncOptions};
use crate::session::{self, Replica, Report, Side};

/// How long to wait to get online (a relay connection) before giving up.
const ONLINE_WAIT: std::time::Duration = std::time::Duration::from_secs(20);

/// How long to keep looking for a peer's address before giving up, and how often to ask.
const ADDRESS_WAIT: std::time::Duration = std::time::Duration::from_secs(45);
const ADDRESS_RETRY: std::time::Duration = std::time::Duration::from_secs(3);

/// Whether a connect error means the peer's address is not (yet) known.
fn is_not_found(error: &str) -> bool {
    error.contains("No addressing information")
}

/// The application protocol on the QUIC connection.
pub(crate) const ALPN: &[u8] = b"quire/sync/1";

/// A new device key (32 bytes), for tests: the engine creates keys as secrets (`quire_engine::secrets`).
#[cfg(test)]
pub(crate) fn new_secret() -> [u8; 32] {
    SecretKey::generate().to_bytes()
}

/// The public ID of the device with `secret`, as others add it.
pub fn device_id(secret: &[u8; 32]) -> String {
    SecretKey::from_bytes(secret).public().to_string()
}

/// Whether `text` is a device ID.
pub fn is_device_id(text: &str) -> bool {
    EndpointId::from_str(text.trim()).is_ok()
}

/// This device on the network.
pub struct Device {
    endpoint: Endpoint,
    /// Our own relay, if any: the only one used, and where peers are dialed (no address lookup).
    relay: Option<RelayUrl>,
}

/// `text` as a relay URL (`http://host:3340` for a relay in dev mode, `https://…` otherwise).
pub fn parse_relay(text: &str) -> Result<RelayUrl> {
    text.trim().parse().map_err(|e| SyncError::Network(format!("relay `{text}`: {e}")))
}

/// An endpoint with `key` speaking `alpns`, online; `relay`: our own relay, used instead of n0's
/// relays and address lookup (nothing then goes to n0).
pub(crate) async fn bind_endpoint(key: SecretKey, relay: Option<&RelayUrl>, alpns: Vec<Vec<u8>>) -> Result<Endpoint> {
    let builder = match relay {
        Some(url) => Endpoint::builder(presets::Minimal).relay_mode(RelayMode::custom([url.clone()])),
        None => Endpoint::builder(presets::N0),
    };
    let endpoint = builder.secret_key(key).alpns(alpns).bind().await.map_err(|e| SyncError::Network(e.to_string()))?;
    // Reachable once connected to a relay (its address is then published for peers to find).
    if tokio::time::timeout(ONLINE_WAIT, endpoint.online()).await.is_err() {
        let what = relay.map_or("the internet".to_string(), |url| format!("the relay {url}"));
        return Err(SyncError::Network(format!("no connection to {what} (no relay reachable)")));
    }
    Ok(endpoint)
}

/// A connection from `endpoint` to `peer` speaking `alpn`; with our own `relay`, the peer is dialled
/// there (it is the peer's home relay too, and a direct path is found from there), else found
/// through n0's address lookup.
pub(crate) async fn dial(
    endpoint: &Endpoint,
    relay: Option<&RelayUrl>,
    peer: EndpointId,
    alpn: &[u8],
) -> Result<iroh::endpoint::Connection> {
    let addr = match relay {
        Some(url) => EndpointAddr::new(peer).with_relay_url(url.clone()),
        None => EndpointAddr::new(peer),
    };
    // A peer that just came online may not have published its address yet: ask again for a
    // while (seen live: ~10 s after `sync serve` started, none was found yet).
    let started = std::time::Instant::now();
    loop {
        match endpoint.connect(addr.clone(), alpn).await {
            Ok(connection) => return Ok(connection),
            Err(error) if started.elapsed() < ADDRESS_WAIT && is_not_found(&error.to_string()) => {
                tracing::debug!(%error, "no address for the peer yet");
                tokio::time::sleep(ADDRESS_RETRY).await;
            }
            Err(error) => return Err(SyncError::Network(error.to_string())),
        }
    }
}

impl Device {
    /// Binds this device; `relay`: our own relay's URL (see [`bind_endpoint`]).
    pub async fn bind(secret: &[u8; 32], relay: Option<&str>) -> Result<Self> {
        let relay = relay.map(parse_relay).transpose()?;
        let endpoint = bind_endpoint(SecretKey::from_bytes(secret), relay.as_ref(), vec![ALPN.to_vec()]).await?;
        Ok(Self { endpoint, relay })
    }

    pub fn id(&self) -> String {
        self.endpoint.id().to_string()
    }

    /// A connection to `peer` (a device or hub ID) speaking `alpn`.
    async fn connect(&self, peer: &str, alpn: &[u8]) -> Result<(EndpointId, iroh::endpoint::Connection)> {
        let id = EndpointId::from_str(peer.trim()).map_err(|e| SyncError::Network(format!("`{peer}`: {e}")))?;
        Ok((id, dial(&self.endpoint, self.relay.as_ref(), id, alpn).await?))
    }

    /// Syncs `replica` through the hub `hub` (its ID).
    pub async fn sync_with_hub<R: HubReplica>(
        &self,
        replica: &R,
        hub: &str,
        options: &SyncOptions,
    ) -> Result<HubReport> {
        let (id, connection) = self.connect(hub, crate::hub::ALPN).await?;
        let (mut send, mut receive) = connection.open_bi().await.map_err(|e| SyncError::Network(e.to_string()))?;
        let report = crate::hub::sync(replica, &id.to_string(), options, &mut receive, &mut send).await;
        finish(connection, send, report.is_ok()).await;
        report
    }

    /// Downloads the file `hash` from the hub `hub`; `false` if the hub does not have it.
    pub async fn fetch_from_hub<R: HubReplica>(&self, replica: &R, hub: &str, hash: &str) -> Result<bool> {
        let (id, connection) = self.connect(hub, crate::hub::ALPN).await?;
        let (mut send, mut receive) = connection.open_bi().await.map_err(|e| SyncError::Network(e.to_string()))?;
        let found = crate::hub::fetch_file(replica, &id.to_string(), hash, &mut receive, &mut send).await;
        finish(connection, send, found.is_ok()).await;
        found
    }

    /// This person's devices known to the hub `hub`.
    pub async fn hub_devices<R: HubReplica>(&self, replica: &R, hub: &str) -> Result<Vec<DeviceInfo>> {
        let (id, connection) = self.connect(hub, crate::hub::ALPN).await?;
        let (mut send, mut receive) = connection.open_bi().await.map_err(|e| SyncError::Network(e.to_string()))?;
        let devices = crate::hub::devices(replica, &id.to_string(), &mut receive, &mut send).await;
        finish(connection, send, devices.is_ok()).await;
        devices
    }

    /// Makes the hub `hub` refuse `device` of this person from now on.
    pub async fn hub_revoke<R: HubReplica>(&self, replica: &R, hub: &str, device: &str) -> Result<()> {
        let (id, connection) = self.connect(hub, crate::hub::ALPN).await?;
        let (mut send, mut receive) = connection.open_bi().await.map_err(|e| SyncError::Network(e.to_string()))?;
        let revoked = crate::hub::revoke(replica, &id.to_string(), device, &mut receive, &mut send).await;
        finish(connection, send, revoked.is_ok()).await;
        revoked
    }

    /// Syncs `replica` with the device `peer` (its ID).
    pub async fn sync_with<R: Replica>(&self, replica: &R, peer: &str) -> Result<Report> {
        let (id, connection) = self.connect(peer, ALPN).await?;
        let (mut send, mut receive) = connection.open_bi().await.map_err(|e| SyncError::Network(e.to_string()))?;
        let report = session::run(replica, Side::Client, Some(&id.to_string()), &mut receive, &mut send).await;
        let _ = send.finish();
        // The server closes once it has everything; closing first could drop the last bytes.
        if report.is_ok() {
            connection.closed().await;
        } else {
            connection.close(1u8.into(), b"failed");
        }
        report
    }

    /// Serves sync sessions to the devices in `peers` (their IDs), one at a time, until the
    /// endpoint closes; `done` hears about each session.
    pub async fn serve<R: Replica>(
        &self,
        replica: &R,
        peers: &[String],
        mut done: impl FnMut(Result<Report>),
    ) -> Result<()> {
        while let Some(incoming) = self.endpoint.accept().await {
            let connection = match incoming.accept() {
                Ok(accepting) => match accepting.await {
                    Ok(connection) => connection,
                    Err(error) => {
                        tracing::debug!(%error, "incoming connection failed");
                        continue;
                    }
                },
                Err(error) => {
                    tracing::debug!(%error, "incoming connection refused");
                    continue;
                }
            };
            let remote = connection.remote_id().to_string();
            if !peers.iter().any(|p| p.trim() == remote) {
                connection.close(1u8.into(), b"not a peer");
                done(Err(SyncError::NotAPeer(remote)));
                continue;
            }
            let outcome = match connection.accept_bi().await {
                Ok((mut send, mut receive)) => {
                    let report = session::run(replica, Side::Server, Some(&remote), &mut receive, &mut send).await;
                    let _ = send.finish();
                    // The session ends with the client's last message read: all is delivered.
                    connection.close(0u8.into(), b"done");
                    report
                }
                Err(error) => Err(SyncError::Network(error.to_string())),
            };
            done(outcome);
        }
        Ok(())
    }

    pub async fn close(self) {
        self.endpoint.close().await;
    }
}

/// Ends a client's stream: the other side closes once it has read everything (closing first
/// could drop the last bytes); after a failure, closes at once.
async fn finish(connection: iroh::endpoint::Connection, mut send: iroh::endpoint::SendStream, ok: bool) {
    let _ = send.finish();
    if ok {
        connection.closed().await;
    } else {
        connection.close(1u8.into(), b"failed");
    }
}
