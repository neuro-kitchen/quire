//! The hub on the network: an iroh endpoint (QUIC) speaking `quire/hub/1`, dialled by its key
//! like a device (no port forwarding or domain needed, `.knowledge/sync-hub.md`). Each connection
//! is served in its own task, up to a limit; the rest wait for a free place.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use iroh::endpoint::presets;
use iroh::{Endpoint, RelayMode, SecretKey};
use tokio::sync::Semaphore;

use crate::session::{self, Settings};
use crate::store::Store;

/// How long to wait for a relay before going on without one (a hub on a network with no
/// internet still serves devices that reach it directly).
const ONLINE_WAIT: Duration = Duration::from_secs(20);

/// How long to let a device read the hub's last reply when the hub ended the session.
const CLOSE_WAIT: Duration = Duration::from_secs(5);

/// The hub's network key, in `hub.key` in its data folder (made on first start, readable by the
/// hub's user only). Its public half is the hub's ID, what devices add.
pub fn hub_key(dir: &Path) -> std::io::Result<[u8; 32]> {
    let path = dir.join("hub.key");
    if let Ok(text) = std::fs::read_to_string(&path) {
        return quire_sync::identity::key_from_hex(&text)
            .ok_or_else(|| std::io::Error::other(format!("{} is damaged", path.display())));
    }
    std::fs::create_dir_all(dir)?;
    let key = SecretKey::generate().to_bytes();
    write_private(&path, &quire_sync::identity::key_to_hex(&key))?;
    Ok(key)
}

#[cfg(unix)]
fn write_private(path: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    file.write_all(text.as_bytes())
}

#[cfg(not(unix))]
fn write_private(path: &Path, text: &str) -> std::io::Result<()> {
    std::fs::write(path, text)
}

/// The hub's ID for `key` (what devices add).
pub fn hub_id(key: &[u8; 32]) -> String {
    SecretKey::from_bytes(key).public().to_string()
}

pub struct HubServer {
    endpoint: Endpoint,
}

impl HubServer {
    /// Binds the hub's endpoint; `relay`: our own relay's URL instead of n0's relays and address
    /// lookup (nothing then goes to n0; devices dial the hub at that relay).
    pub async fn bind(key: &[u8; 32], relay: Option<&str>) -> Result<Self, String> {
        let builder = match relay {
            Some(relay) => {
                let url = quire_sync::transport::parse_relay(relay).map_err(|e| e.to_string())?;
                Endpoint::builder(presets::Minimal).relay_mode(RelayMode::custom([url]))
            }
            None => Endpoint::builder(presets::N0),
        };
        let endpoint = builder
            .secret_key(SecretKey::from_bytes(key))
            .alpns(vec![quire_sync::hub::ALPN.to_vec()])
            .bind()
            .await
            .map_err(|e| e.to_string())?;
        if tokio::time::timeout(ONLINE_WAIT, endpoint.online()).await.is_err() {
            tracing::warn!("no relay reachable: only devices that reach this hub directly can sync");
        }
        Ok(Self { endpoint })
    }

    pub fn id(&self) -> String {
        self.endpoint.id().to_string()
    }

    /// Serves sessions until the endpoint closes, at most `max_sessions` at once.
    pub async fn run(&self, store: Arc<Store>, settings: Settings, max_sessions: usize) {
        let places = Arc::new(Semaphore::new(max_sessions.max(1)));
        let settings = Arc::new(settings);
        while let Some(incoming) = self.endpoint.accept().await {
            let Ok(place) = Arc::clone(&places).acquire_owned().await else { break };
            let (store, settings) = (Arc::clone(&store), Arc::clone(&settings));
            tokio::spawn(async move {
                let _place = place;
                let connection = match incoming.accept() {
                    Ok(accepting) => match accepting.await {
                        Ok(connection) => connection,
                        Err(error) => return tracing::debug!(%error, "incoming connection failed"),
                    },
                    Err(error) => return tracing::debug!(%error, "incoming connection refused"),
                };
                let remote = connection.remote_id().to_string();
                let Ok((mut send, mut receive)) = connection.accept_bi().await else { return };
                let ended_by_device =
                    match session::serve(&store, &settings, Some(&remote), &mut receive, &mut send).await {
                        Ok(summary) => {
                            tracing::info!(
                                account = summary.account,
                                batches_in = summary.batches_in,
                                batches_out = summary.batches_out,
                                files_in = summary.files_in,
                                files_out = summary.files_out,
                                "session"
                            );
                            summary.ended_by_device
                        }
                        Err(error) => {
                            tracing::warn!(%error, "session failed");
                            false
                        }
                    };
                let _ = send.finish();
                if !ended_by_device {
                    // The hub ended it (a refusal): closing now would drop the reply not yet
                    // delivered (seen live: a revoked device saw "connection lost"). The device
                    // closes once it has read it; wait for that, a little.
                    let _ = tokio::time::timeout(CLOSE_WAIT, connection.closed()).await;
                }
                // After `Bye` the device has read every reply: closing loses nothing.
                connection.close(0u8.into(), b"done");
            });
        }
    }

    pub async fn close(self) {
        self.endpoint.close().await;
    }
}
