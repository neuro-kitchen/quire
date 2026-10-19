//! The one crate front ends use (CLI now; desktop and Python later): opens a library and
//! offers every operation as an async method, wiring `quire-store`, `quire-fetch`, `quire-io`
//! and `quire-core` together.
//!
//! - [`engine`]: [`Engine`] and its operations.
//! - [`config`]: the user's `config.toml` (git-style keys).
//! - [`paths`]: where the config file and the library live.
//! - [`error`]: the error type, [`EngineError`].

pub mod config;
pub mod engine;
pub mod error;
pub mod paths;

pub use config::Config;
pub use engine::{Engine, ExportFormat, OpenOptions};
pub use error::{EngineError, Result};

// Types front ends receive from the engine.
pub use quire_core::{Added, CoreError};
pub use quire_fetch::{Endpoints, FetchError};
pub use quire_model::{Project, Reference, SlugPath};
