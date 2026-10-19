//! Duplicate references of [`Engine`]: finding them, merging, and recording distinct pairs
//! (`quire_core::library::duplicates`).

use std::collections::HashSet;

use quire_core::Library;
use quire_core::library::duplicates::{self, Merged, Reason};
use quire_model::{Reference, ReferenceId};

use super::Engine;
use crate::error::Result;

/// A reference and the others that look like the same work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub reference: Reference,
    pub duplicates: Vec<(Reference, Reason)>,
}

impl Engine {
    /// Every pair of references that look like the same work, as groups by first reference (each
    /// pair once), ordered by cite key.
    pub async fn duplicates(&self) -> Result<Vec<DuplicateGroup>> {
        self.run(|lib| {
            let found = duplicates::library_index(lib)?;
            let mut groups = Vec::new();
            let mut shown: HashSet<ReferenceId> = HashSet::new();
            let load = |id| lib.reference(id).map(|r| r.expect("indexed reference exists"));
            let mut references = found.keys().map(|id| load(*id)).collect::<quire_core::Result<Vec<_>>>()?;
            references.sort_by(|a, b| a.cite_key.cmp(&b.cite_key));
            for reference in references {
                if shown.contains(&reference.id) {
                    continue;
                }
                shown.insert(reference.id);
                let mut others = Vec::new();
                for m in &found[&reference.id] {
                    if shown.insert(m.other) {
                        others.push((load(m.other)?, m.reason.clone()));
                    }
                }
                if !others.is_empty() {
                    groups.push(DuplicateGroup { reference, duplicates: others });
                }
            }
            Ok(groups)
        })
        .await
    }

    /// Merges the reference `other` into `keep` (`duplicates::merge`).
    pub async fn merge(&self, keep: &str, other: &str) -> Result<Merged> {
        let (keep, other) = (keep.to_string(), other.to_string());
        let merged = self.run(move |lib| duplicates::merge(lib, &keep, &other)).await?;
        self.mark_changed();
        Ok(merged)
    }

    /// Records that `a` and `b` are distinct works; `false` if already recorded.
    pub async fn dismiss_duplicate(&self, a: &str, b: &str) -> Result<bool> {
        let (a, b) = (a.to_string(), b.to_string());
        let done = self.run(move |lib| duplicates::dismiss(lib, &a, &b)).await?;
        self.mark_changed();
        Ok(done)
    }
}
