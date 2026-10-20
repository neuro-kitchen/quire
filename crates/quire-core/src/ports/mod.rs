//! The storage port: what the use cases need from a library store. `quire-store` implements it
//! on SQLite; tests use the in-memory [`MemoryLibrary`](crate::memory::MemoryLibrary).
//!
//! One trait per group of tables ([`stores`]), so a use case's bound says what it touches and
//! each store module implements its own; [`Library`] is all of them.
//!
//! Calls are synchronous: the engine runs them on the database thread, inside one transaction
//! per use case.
//!
//! - [`records`]: data the store keeps that is not a domain type ([`RawRecord`], …).
//! - [`stores`]: the traits.
//! - [`prelude`]: every trait, for calling methods on a concrete store.

pub mod records;
pub mod stores;

pub use records::{AutoExport, FailedInput, RawRecord, ReferenceFilter, SearchHit};
pub use stores::{
    AlertStore, AttachmentStore, Device, ExportStore, HighlightStore, ImportStore, IssueStore, MemberStore, NoteStore,
    ProjectStore, ReadingStore, ReferenceStore, RelationStore, SavedSearchStore, SearchStore, TagStore,
};

/// A whole library store: every trait in [`stores`].
pub trait Library:
    Device
    + ReferenceStore
    + ProjectStore
    + IssueStore
    + ExportStore
    + AttachmentStore
    + TagStore
    + NoteStore
    + RelationStore
    + HighlightStore
    + SearchStore
    + ImportStore
    + ReadingStore
    + SavedSearchStore
    + AlertStore
    + MemberStore
{
}

impl<T> Library for T where
    T: Device
        + ReferenceStore
        + ProjectStore
        + IssueStore
        + ExportStore
        + AttachmentStore
        + TagStore
        + NoteStore
        + RelationStore
        + HighlightStore
        + SearchStore
        + ImportStore
        + ReadingStore
        + SavedSearchStore
        + AlertStore
        + MemberStore
{
}

/// Every store trait in scope (`use quire_core::prelude::*`), for code holding a concrete store.
pub mod prelude {
    pub use super::Library;
    pub use super::stores::{
        AlertStore as _, AttachmentStore as _, Device as _, ExportStore as _, HighlightStore as _, ImportStore as _,
        IssueStore as _, MemberStore as _, NoteStore as _, ProjectStore as _, ReadingStore as _, ReferenceStore as _,
        RelationStore as _, SavedSearchStore as _, SearchStore as _, TagStore as _,
    };
}
