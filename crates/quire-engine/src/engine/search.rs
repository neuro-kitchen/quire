//! Full-text search of [`Engine`] (`quire_core::library::search`): metadata, tags, notes and
//! highlights are indexed by the store's triggers; a paper's text when the reader's text is
//! extracted ([`Engine::reader_text`]).

use quire_core::library::search;
use quire_core::{Library, SearchHit};
use quire_model::{Reference, ReferenceId};

use super::Engine;
use crate::error::Result;

/// Results shown at most.
pub const DEFAULT_LIMIT: usize = 50;

impl Engine {
    /// The references `query` finds, best first (see `quire_core::library::search` for the syntax).
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<(Reference, SearchHit)>> {
        let query = query.to_string();
        self.run(move |lib| search::search(lib, &query, limit)).await
    }

    /// Indexes (or with `None`, forgets) the text of `reference`'s paper.
    pub(super) async fn index_paper_text(&self, reference: ReferenceId, text: Option<String>) -> Result<()> {
        self.run(move |lib| lib.index_paper_text(reference, text.as_deref())).await
    }
}
