//! Use cases, each a function over a [`Library`](crate::Library) run in one transaction by the
//! engine.
//!
//! - [`alerts`]: new papers from arXiv, PubMed or Crossref searches.
//! - [`attachments`]: files (PDFs) of references.
//! - [`duplicates`]: references that are the same work; merging them.
//! - [`highlights`]: passages marked in the reader.
//! - [`reading`]: reading status (unread, reading, read).
//! - [`references`]: add (with deduplication and cite key), find, trash and restore.
//! - [`saved`]: saved searches (smart projects).
//! - [`search`]: full-text search (FTS5).
//! - [`sharing`]: projects shared with other people (members, roles).
//! - [`projects`]: create, find by path, list as a tree, link references, collect for export.
//! - [`tags`]: tags, notes and related references.
//! - [`update`]: refresh from a source and edit by the user, field by field.

pub mod alerts;
pub mod attachments;
pub mod duplicates;
pub mod highlights;
pub mod projects;
pub mod reading;
pub mod references;
pub mod saved;
pub mod search;
pub mod sharing;
pub mod tags;
pub mod update;
