//! Devices on the network with iroh (QUIC): each has a key pair kept by the engine; its public
//! key (`EndpointId`) is how other devices dial it, through n0's address lookup and relays when
//! no direct path exists (`presets::N0`). The connection is authenticated by those keys and
//! encrypted end to end; a device serves only the peers the user added.

use std::str::FromStr;

use iroh::endpoint::presets;
use iroh::{Endpoint, EndpointId, SecretKey};

use crate::error::{Result, SyncError};
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
}

impl Device {
    pub async fn bind(secret: &[u8; 32]) -> Result<Self> {
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(SecretKey::from_bytes(secret))
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;
        // Reachable once connected to a relay (its address is then published for peers to find).
        if tokio::time::timeout(ONLINE_WAIT, endpoint.online()).await.is_err() {
            return Err(SyncError::Network("no connection to the internet (no relay reachable)".into()));
        }
        Ok(Self { endpoint })
    }

    pub fn id(&self) -> String {
        self.endpoint.id().to_string()
    }

    /// Syncs `replica` with the device `peer` (its ID).
    pub async fn sync_with<R: Replica>(&self, replica: &R, peer: &str) -> Result<Report> {
        let id = EndpointId::from_str(peer.trim()).map_err(|e| SyncError::Network(format!("`{peer}`: {e}")))?;
        // A peer that just came online may not have published its address yet: ask again for a
        // while (seen live: ~10 s after `sync serve` started, none was found yet).
        let started = std::time::Instant::now();
        let connection = loop {
            match self.endpoint.connect(id, ALPN).await {
                Ok(connection) => break connection,
                Err(error) if started.elapsed() < ADDRESS_WAIT && is_not_found(&error.to_string()) => {
                    tracing::debug!(%error, "no address for the peer yet");
                    tokio::time::sleep(ADDRESS_RETRY).await;
                }
                Err(error) => return Err(SyncError::Network(error.to_string())),
            }
        };
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
