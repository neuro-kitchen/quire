//! quire's sync hub (`.knowledge/sync-hub.md`): an always-on store a person's devices push sealed
//! changes and files to and pull from. Devices encrypt everything first (`quire_sync::seal`); the
//! hub keeps accounts, device IDs, sizes and times, and can read nothing else. One program for a
//! self-hosted container (one owner, set up with a printed code) and for the cloud (many accounts).
//!
//! - [`store`]: what is kept, in the data folder (SQLite and sealed files).
//! - [`session`]: the hub's side of a session (`quire_sync::hub::protocol`).
//! - [`server`]: the iroh endpoint and the concurrent sessions.

pub mod server;
pub mod session;
pub mod store;

pub use server::{HubServer, hub_id, hub_key};
pub use session::Settings;
pub use store::{HubError, Purpose, Store};
