//! Sync between a user's devices (`.knowledge/sync.md`): each device keeps its own SQLite
//! library; they exchange the rows that changed (`quire_core::sync`) and the PDFs one lacks,
//! directly over QUIC with iroh (devices are dialled by their public key; NAT traversal with a
//! relay fallback; end-to-end encrypted). No server holds the data.
//!
//! - [`session`]: one exchange between two copies, over any byte stream ([`session::Replica`]
//!   is the library copy).
//! - [`protocol`]: the messages and their framing.
//! - [`transport`]: iroh endpoints, accepting only known peers.
//! - [`error`]: the errors ([`SyncError`]).

pub mod error;
pub mod protocol;
pub mod session;
pub mod transport;

pub use error::{Result, SyncError};
pub use session::{Replica, Report, Role};
