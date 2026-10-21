//! The one crate front ends use (CLI, TUI, the apps through `quire-ffi`, `quire serve`): opens a
//! library and offers every operation as an async method, wiring `quire-core`, `quire-store`,
//! `quire-io`, `quire-document`, `quire-fetch` and `quire-sync` together.
//!
//! Operations:
//! - [`engine`]: [`Engine`] and its operations, one module per concept.
//!
//! Infrastructure the operations use:
//! - [`config`]: the user's `config.toml` (git-style keys).
//! - [`editor`]: the user's text editor, for front ends.
//! - [`libraries`]: named libraries and which one to open.
//! - [`files`]: the library's content-addressed file store.
//! - [`folder`]: `quire.toml`, a document folder's library, project and bibliography file.
//! - [`paths`]: where the config file and libraries live.
//! - [`private`]: folders and files readable by the user only.
//! - [`secrets`]: secrets in the system keyring (file fallback).
//! - [`tools`]: programs used when installed (poppler, Tesseract, OCRmyPDF).
//! - [`zotero`]: reading a Zotero data folder (for `import zotero`).
//! - [`error`]: the error type, [`EngineError`].

pub mod config;
pub mod editor;
pub mod engine;
pub mod error;
pub mod files;
pub mod folder;
pub mod libraries;
pub mod paths;
pub mod private;
pub mod secrets;
pub mod tools;
pub mod zotero;

pub use config::Config;
pub use engine::{
    AbstractOutcome, AlertCheck, Attached, AutoExportInfo, AutoExportOutcome, Change, CiteUse, DuplicateGroup, Engine,
    Eviction, ExportRequest, GraphEntry, GraphKind, ManuscriptCheck, OcrOutcome, OpenOptions, OpenedPdf, PaperText,
    PdfOutcome, PdfSource, PdfStatus, ProjectSummary, RetractionCheck, SEARCH_LIMIT, TextSource, UnknownKey,
    ZoteroImport, online_candidate, suggest_keys, write_if_changed,
};
pub use error::{EngineError, Result};
pub use folder::FolderFile;
pub use libraries::{LibraryChoice, LibraryInfo};

// Types front ends receive from the engine.
pub use quire_core::library::duplicates::{Merged, Reason as DuplicateReason};
pub use quire_core::library::update::{FieldChange, Outcome, Updated};
pub use quire_core::{Added, Checked, CoreError, FailedInput, SearchHit};
pub use quire_document::pdf::PdfMatch;
pub use quire_document::{paper_markdown, reading};
pub use quire_fetch::crossref::{Update, UpdateKind};
pub use quire_fetch::{Endpoints, FetchError};
pub use quire_io::{Encoding, ExportFormat, ExportOptions};
pub use quire_model::{
    Alert, AlertSource, Attachment, Field, Highlight, HighlightId, Issue, IssueCode, Note, NoteId, PdfAttempt, PdfKind,
    Project, Reading, ReadingStatus, Reference, SavedSearch, Severity, SlugPath, Tag, TagKind,
};
pub use quire_model::{ProjectMember, Role as MemberRole, UserId};
pub use quire_sync::hub::{HubReport, SyncOptions as HubOptions};
pub use quire_sync::transport::is_device_id;
pub use quire_sync::{Report as SyncReport, Side as SyncSide};
pub use secrets::Store as SecretStore;
