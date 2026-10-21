//! Sync through a hub (`.knowledge/sync-hub.md`): an always-on store the user's devices push
//! sealed changes and files to and pull from, which can read none of it (devices encrypt first,
//! [`crate::seal`]). Reached over iroh like a device (ALPN `quire/hub/1`).
//!
//! - [`protocol`]: the messages (Protocol Buffers) and their framing.
//! - [`client`]: a device's side of a session.

pub mod client;
pub mod protocol;

pub use client::{HubReplica, HubReport, SyncOptions, devices, fetch_file, hub_peer, revoke, sync};

/// The application protocol of hub connections.
pub const ALPN: &[u8] = b"quire/hub/1";
