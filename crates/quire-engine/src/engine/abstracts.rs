//! Filling in missing abstracts: many publishers do not send them to Crossref, and references
//! imported from a `.bib` file often have none. Sources, in order: Europe PMC (by DOI or PMID),
//! then the paper's own text (its stored Markdown, `paper_text`).

use quire_core::library::update;
use quire_model::{Origin, Reference, TextField};

use super::Engine;
use crate::error::Result;

/// What [`Engine::fill_abstract`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbstractOutcome {
    /// It already had one.
    Present,
    /// Found at `source` (`europepmc` or `pdf`); stored unless it was a dry run.
    Found { source: &'static str, text: String },
    /// No source had one.
    NotFound,
}

impl Engine {
    /// Fills the abstract of the reference `query` names if it has none: Europe PMC, then (with
    /// `from_pdf`) its PDF's text. With `dry_run` nothing is stored.
    pub async fn fill_abstract(
        &self,
        query: &str,
        from_pdf: bool,
        dry_run: bool,
    ) -> Result<(Reference, AbstractOutcome)> {
        let (reference, _) = self.show(query).await?;
        if reference.data.get(TextField::Abstract).is_some() {
            return Ok((reference, AbstractOutcome::Present));
        }
        let mut found = match self.fetcher.abstract_for(&reference.data.identifiers).await {
            Ok(Some(text)) => Some(("europepmc", text)),
            Ok(None) => None,
            Err(error) => {
                tracing::debug!(%error, "Europe PMC");
                None
            }
        };
        if found.is_none() && from_pdf && !self.attachments(&reference).await?.is_empty() {
            let (_, markdown, _) = self.paper_text(&reference.id.to_string()).await?;
            found = quire_document::abstracts::from_markdown(&markdown).map(|text| ("pdf", text));
        }
        let Some((source, text)) = found else { return Ok((reference, AbstractOutcome::NotFound)) };
        if dry_run {
            return Ok((reference, AbstractOutcome::Found { source, text }));
        }
        let mut new = reference.data.clone();
        new.set(TextField::Abstract, text.clone());
        let origin = Origin::Fetched { source: source.into() };
        let updated = self.run(move |lib| update::refresh(lib, reference, new, origin, None, &[], false)).await?;
        self.mark_changed();
        Ok((updated.reference, AbstractOutcome::Found { source, text }))
    }
}
