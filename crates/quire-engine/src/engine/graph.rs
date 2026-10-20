//! The citation graph from OpenAlex: the works a reference cites, the works citing it, and works
//! OpenAlex finds related; each marked with the library's reference for it when there is one, so
//! front ends can offer to add the rest.

use quire_core::prelude::*;
use quire_fetch::openalex::GraphWork;
use quire_model::{Identifier, Reference};

use super::Engine;
use crate::error::Result;

/// Which neighbours of a work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphKind {
    /// The works it cites (its reference list).
    References,
    /// Works citing it, most cited first.
    Citing,
    /// Works OpenAlex computes as related (shared concepts and citations).
    Related,
}

/// A neighbouring work and the library's reference for it, if any.
#[derive(Debug, Clone)]
pub struct GraphEntry {
    pub work: GraphWork,
    pub in_library: Option<Reference>,
}

impl Engine {
    /// The neighbours of the reference `query` names (at most `limit` citing works).
    pub async fn graph(&self, query: &str, kind: GraphKind, limit: usize) -> Result<(Reference, Vec<GraphEntry>)> {
        let (reference, _) = self.show(query).await?;
        let links = self.fetcher.openalex_work(&reference.data.identifiers).await?;
        let works = match kind {
            GraphKind::References => self.fetcher.openalex_works(&links.references).await?,
            GraphKind::Related => self.fetcher.openalex_works(&links.related).await?,
            GraphKind::Citing => self.fetcher.openalex_citing(&links.id, limit).await?,
        };
        let entries = self
            .run(move |lib| {
                works
                    .into_iter()
                    .map(|work| {
                        let identifier = work.doi.as_deref().and_then(|doi| Identifier::parse(doi).ok());
                        let in_library = match identifier {
                            Some(id) => lib.reference_by_identifier(&id)?.filter(|r| !r.meta.is_deleted()),
                            None => None,
                        };
                        Ok(GraphEntry { work, in_library })
                    })
                    .collect::<quire_core::Result<Vec<_>>>()
            })
            .await?;
        Ok((reference, entries))
    }
}
