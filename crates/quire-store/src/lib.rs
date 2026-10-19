//! quire's library storage: one SQLite file per library.
//!
//! - [`open`](mod@open): opening, connection settings, migrations, device ID.
//! - [`schema`]: the migrations (plain SQL files).
//! - [`repo`]: [`SqliteLibrary`], the `quire-core` `Library` port on a connection.
//! - [`error`]: the error type, [`StoreError`].

pub mod error;
pub mod open;
pub mod repo;
pub mod schema;

pub use error::{StoreError, Result};
pub use open::{configure, device_id, open, open_in_memory, prepare, prepare_in_place};
pub use repo::SqliteLibrary;
