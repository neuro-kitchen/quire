//! [`SqliteLibrary`]: the `quire-core` [`Library`] port on a SQLite connection (usually a
//! transaction, so one use case commits or rolls back as a whole).
//!
//! - [`codec`]: IDs, times and JSON ↔ SQLite values; [`core`]: what the query modules share.
//! - [`references`], [`projects`], [`links`], [`issues`], [`auto_export`], [`attachments`], [`tags`],
//!   [`notes`], [`relations`], [`highlights`], [`search`], [`imports`], [`reading`], [`alerts`], [`members`]: the queries, one module per table group,
//!   each implementing its `quire-core` store trait.

mod alerts;
mod attachments;
mod auto_export;
pub(crate) mod codec;
mod core;
mod highlights;
mod imports;
mod issues;
mod links;
mod local_files;
pub use local_files::{FileState, KeepTarget};
mod members;
mod notes;
mod projects;
mod reading;
mod references;
mod relations;
mod search;
mod sync;
mod tags;

use quire_core::ports::Device;
use quire_model::DeviceId;
use rusqlite::Connection;

/// A library on `conn`, recording changes as made by `device`.
pub struct SqliteLibrary<'c> {
    conn: &'c Connection,
    device: DeviceId,
}

impl<'c> SqliteLibrary<'c> {
    pub fn new(conn: &'c Connection, device: DeviceId) -> Self {
        Self { conn, device }
    }
}

impl Device for SqliteLibrary<'_> {
    fn device(&self) -> DeviceId {
        self.device
    }
}
