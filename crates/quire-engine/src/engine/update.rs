//! Changing stored references: refetch from their source, and edit by the user.

use quire_core::library::references;
use quire_core::library::update::{self, Updated};
use quire_fetch::Lookup;
use quire_model::{Identifier, Origin, Reference, ReferenceId, TextField};
use url::Url;

use super::Engine;
use crate::error::{EngineError, Result};

impl Engine {
    /// Looks the reference `query` names up again (by its DOI, arXiv ID, PMID or PMCID, else its
    /// URL) and updates the fields that came from a source; fields the user edited are kept. With
    /// `dry_run`, only reports what would change.
    pub async fn refetch(&self, query: &str, dry_run: bool) -> Result<Updated> {
        let (reference, _) = self.show(query).await?;
        let lookup =
            lookup_for(&reference).ok_or_else(|| EngineError::Unrefetchable(reference.cite_key.to_string()))?;
        let fetched = self.fetcher.fetch(&lookup).await?;
        let origin = Origin::Fetched { source: fetched.raw.source.clone() };
        let updated = self
            .run(move |lib| {
                update::refresh(lib, reference, fetched.data, origin, Some(fetched.raw), &fetched.issues, dry_run)
            })
            .await?;
        if updated.changed() && !dry_run {
            self.mark_changed();
        }
        Ok(updated)
    }

    /// The reference `query` names as an editable TOML document, with its ID to apply the edit to.
    pub async fn edit_document(&self, query: &str) -> Result<(ReferenceId, String)> {
        let (reference, _) = self.show(query).await?;
        Ok((reference.id, quire_io::to_edit_toml(&reference)))
    }

    /// Applies an edited document to the reference `id`: changed fields become the user's. With
    /// `dry_run`, only reports what would change.
    pub async fn apply_edit(&self, id: ReferenceId, text: &str, dry_run: bool) -> Result<Updated> {
        let (cite_key, mut data) = quire_io::from_edit_toml(text)?;
        let updated = self
            .run(move |lib| {
                let reference = references::find(lib, &id.to_string())?;
                // `extra` is not in the document; keep it.
                data.extra = reference.data.extra.clone();
                update::edit(lib, reference, data, Some(cite_key), dry_run)
            })
            .await?;
        if updated.changed() && !dry_run {
            self.mark_changed();
        }
        Ok(updated)
    }
}

/// What to look `reference` up by: the most specific identifier, else its URL.
fn lookup_for(reference: &Reference) -> Option<Lookup> {
    let rank = |id: &Identifier| match id {
        Identifier::Doi(_) => 0,
        Identifier::Arxiv(_) => 1,
        Identifier::Pmid(_) => 2,
        Identifier::Pmcid(_) => 3,
    };
    if let Some(best) = reference.data.identifiers.iter().min_by_key(|id| rank(id)) {
        return Some(Lookup::Identifier(best.clone()));
    }
    let url = Url::parse(reference.data.get(TextField::Url)?).ok()?;
    matches!(url.scheme(), "http" | "https").then_some(Lookup::Url(url))
}
