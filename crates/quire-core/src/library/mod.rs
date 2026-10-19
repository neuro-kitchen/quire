//! Use cases, each a function over a [`Library`](crate::Library) run in one transaction by the
//! engine.
//!
//! - [`attachments`]: files (PDFs) of references.
//! - [`duplicates`]: references that are the same work; merging them.
//! - [`highlights`]: passages marked in the reader.
//! - [`references`]: add (with deduplication and cite key), find, trash and restore.
//! - [`search`]: full-text search (FTS5).
//! - [`projects`]: create, find by path, list as a tree, link references, collect for export.
//! - [`tags`]: tags, notes and related references.
//! - [`update`]: refresh from a source and edit by the user, field by field.

pub mod attachments;
pub mod duplicates;
pub mod highlights;
pub mod projects;
pub mod references;
pub mod search;
pub mod tags;
pub mod update;
