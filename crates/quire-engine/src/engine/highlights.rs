//! Highlights of [`Engine`]: passages marked in the reader (`quire_core::library::highlights`).

use quire_core::library::highlights;
use quire_model::{Highlight, HighlightId, Reference};

use super::Engine;
use crate::error::Result;

impl Engine {
    /// Highlights `quote` (after `prefix`) in the paper `query` names.
    pub async fn add_highlight(
        &self,
        query: &str,
        quote: &str,
        prefix: &str,
        comment: Option<&str>,
    ) -> Result<Highlight> {
        let (query, quote, prefix) = (query.to_string(), quote.to_string(), prefix.to_string());
        let comment = comment.map(str::to_string);
        let highlight = self.run(move |lib| highlights::add(lib, &query, &quote, &prefix, comment.as_deref())).await?;
        self.mark_changed();
        Ok(highlight)
    }

    /// The paper `query` names and its highlights, oldest first.
    pub async fn highlights(&self, query: &str) -> Result<(Reference, Vec<Highlight>)> {
        let query = query.to_string();
        self.run(move |lib| highlights::of(lib, &query)).await
    }

    /// Sets a highlight's comment (empty: removes it).
    pub async fn comment_highlight(&self, id: HighlightId, text: &str) -> Result<Highlight> {
        let text = text.to_string();
        let highlight = self.run(move |lib| highlights::comment(lib, id, &text)).await?;
        self.mark_changed();
        Ok(highlight)
    }

    pub async fn remove_highlight(&self, id: HighlightId) -> Result<Highlight> {
        let highlight = self.run(move |lib| highlights::remove(lib, id)).await?;
        self.mark_changed();
        Ok(highlight)
    }
}
