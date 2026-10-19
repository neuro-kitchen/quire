//! The one crate front ends use (CLI now; desktop and Python later): opens a library and
//! offers every operation as an async method, wiring `quire-store`, `quire-fetch`, `quire-io`
//! and `quire-core` together.
//!
//! - [`engine`]: [`Engine`] and its operations.
//! - [`config`]: the user's `config.toml` (git-style keys).
//! - [`libraries`]: named libraries and which one to open.
//! - [`files`]: the library's content-addressed file store.
//! - [`folder`]: `quire.toml`, a document folder's library, project and bibliography file.
//! - [`paths`]: where the config file and libraries live.
//! - [`private`]: folders and files readable by the user only.
//! - [`secrets`]: secrets in the system keyring (file fallback).
//! - [`error`]: the error type, [`EngineError`].

pub mod config;
pub mod engine;
pub mod error;
pub mod files;
pub mod folder;
pub mod libraries;
pub mod paths;
pub mod private;
pub mod secrets;

pub use config::Config;
pub use engine::{
    AbstractOutcome, Attached, AutoExportInfo, AutoExportOutcome, Change, Engine, ExportRequest, OpenOptions,
    PaperText, PdfOutcome, PdfStatus, ProjectSummary, TextSource, online_candidate,
};
pub use error::{EngineError, Result};
pub use folder::FolderFile;
pub use libraries::{LibraryChoice, LibraryInfo};

// Types front ends receive from the engine.
pub use quire_core::library::update::{FieldChange, Outcome, Updated};
pub use quire_core::{Added, Checked, CoreError, FailedInput};
pub use quire_fetch::{Endpoints, FetchError};
pub use quire_io::pdf::PdfMatch;
pub use quire_io::{ExportFormat, ExportOptions};
pub use quire_model::{
    Attachment, Issue, IssueCode, Note, NoteId, PdfAttempt, PdfKind, Project, Reference, Severity, SlugPath, Tag,
    TagKind,
};
pub use secrets::Store as SecretStore;
