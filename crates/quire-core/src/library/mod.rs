//! Use cases, each a function over a [`Library`](crate::Library) run in one transaction by the
//! engine.
//!
//! - [`references`]: add (with deduplication and cite key), find, trash and restore.
//! - [`projects`]: create, find by path, list as a tree, link references, collect for export.

pub mod projects;
pub mod references;
