//! Adding and finding references. A work is stored once: adding something whose identifier is
//! already in the library returns the existing reference (restored from the trash if needed)
//! and only links it to the project.

use std::str::FromStr;

use quire_model::{CiteKey, Identifier, Issue, Origin, ProjectId, Reference, ReferenceData, ReferenceId};

use crate::citekey::{free_key, unique_key};
use crate::error::{CoreError, Result};
use crate::ports::{Library, RawRecord};

/// What to add.
#[derive(Debug, Clone)]
pub struct AddRequest {
    pub data: ReferenceData,
    pub origin: Origin,
    pub raw: Option<RawRecord>,
    /// Link the reference to this project.
    pub project: Option<ProjectId>,
    /// Cite key to use instead of the formula's (e.g. from an imported `.bib`). Used if valid
    /// and free; if taken, it gets a letter postfix; if invalid, the formula is used.
    pub preferred_key: Option<String>,
    /// Issues to record with a new reference (e.g. a fallback source was used).
    pub issues: Vec<Issue>,
}

impl AddRequest {
    pub fn new(data: ReferenceData, origin: Origin) -> Self {
        Self { data, origin, raw: None, project: None, preferred_key: None, issues: Vec::new() }
    }
}

/// Result of [`add`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Added {
    /// Stored as a new reference.
    New(Reference),
    /// Already in the library (same identifier); nothing was changed but the project link.
    Existing(Reference),
}

impl Added {
    pub fn reference(&self) -> &Reference {
        match self {
            Self::New(r) | Self::Existing(r) => r,
        }
    }
}

/// Writes a changed `reference`, stamping the clocks of the parts that differ from the stored
/// version (sync merges references part by part). Every change to a stored reference goes here.
pub fn save(library: &mut impl Library, reference: &mut Reference) -> Result<()> {
    if let Some(stored) = library.reference(reference.id)? {
        crate::sync::stamp_changes(&stored, reference);
    }
    library.update_reference(reference)
}

/// Adds a reference, or finds the one already holding one of its identifiers.
pub fn add(library: &mut impl Library, request: AddRequest) -> Result<Added> {
    let device = library.device();
    if let Some(mut existing) = find_duplicate(library, &request.data)? {
        if existing.meta.is_deleted() {
            existing.meta.restore(device);
            save(library, &mut existing)?;
        }
        if let Some(project) = request.project {
            library.link(existing.id, project)?;
        }
        return Ok(Added::Existing(existing));
    }

    let taken = |key: &str| library.cite_key_taken(key);
    let cite_key = match request.preferred_key.as_deref().filter(|k| CiteKey::parse(k).is_ok()) {
        Some(key) => free_key(key, taken)?,
        None => unique_key(&request.data, taken)?,
    };
    let reference = Reference::new(cite_key, request.data, request.origin, device);
    library.insert_reference(&reference, request.raw.as_ref())?;
    if !request.issues.is_empty() {
        library.set_recorded_issues(reference.id, &request.issues)?;
    }
    if let Some(project) = request.project {
        library.link(reference.id, project)?;
    }
    Ok(Added::New(reference))
}

/// Adds `reference` from another library, keeping its ID (so the same work is recognisable
/// across libraries), cite key (postfixed if taken here), data and provenance. A reference with
/// the same ID or an identifier already here is returned as [`Added::Existing`] and only linked.
pub fn copy_in(library: &mut impl Library, reference: Reference, project: Option<ProjectId>) -> Result<Added> {
    let existing = match library.reference(reference.id)? {
        Some(found) => Some(found),
        None => find_duplicate(library, &reference.data)?,
    };
    if let Some(existing) = existing {
        if let Some(project) = project {
            library.link(existing.id, project)?;
        }
        return Ok(Added::Existing(existing));
    }
    let cite_key = free_key(reference.cite_key.as_str(), |key| library.cite_key_taken(key))?;
    let mut copy = Reference { cite_key, ..reference };
    copy.meta.restore(library.device());
    library.insert_reference(&copy, None)?;
    if let Some(project) = project {
        library.link(copy.id, project)?;
    }
    Ok(Added::New(copy))
}

/// The reference holding any of `data`'s identifiers.
pub fn find_duplicate(library: &impl Library, data: &ReferenceData) -> Result<Option<Reference>> {
    for identifier in &data.identifiers {
        if let Some(found) = library.reference_by_identifier(identifier)? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

/// The reference the user means by `query`: a reference ID (`ref_…`), a cite key, or an
/// identifier (DOI, PMID, arXiv ID, …), tried in that order.
pub fn find(library: &impl Library, query: &str) -> Result<Reference> {
    let query = query.trim();
    let found = if query.starts_with("ref_") {
        library.reference(ReferenceId::from_str(query)?)?
    } else if let Some(found) =
        CiteKey::parse(query).ok().map(|key| library.reference_by_cite_key(&key)).transpose()?.flatten()
    {
        Some(found)
    } else {
        Identifier::parse(query).ok().map(|id| library.reference_by_identifier(&id)).transpose()?.flatten()
    };
    found.ok_or_else(|| CoreError::ReferenceNotFound(query.into()))
}

/// Moves a reference to the trash. Its project links stay, so restoring brings it back in place.
pub fn trash(library: &mut impl Library, query: &str) -> Result<Reference> {
    let mut reference = find(library, query)?;
    if !reference.meta.is_deleted() {
        reference.meta.delete(library.device());
        save(library, &mut reference)?;
    }
    Ok(reference)
}

/// Takes a reference out of the trash.
pub fn restore(library: &mut impl Library, query: &str) -> Result<Reference> {
    let mut reference = find(library, query)?;
    if reference.meta.is_deleted() {
        reference.meta.restore(library.device());
        save(library, &mut reference)?;
    }
    Ok(reference)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryLibrary;
    use quire_model::{Creator, CreatorRole, Date, EntryKind, TextField};

    fn article(doi: &str, family: &str) -> ReferenceData {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.creators.push(Creator::person(CreatorRole::Author, family, None));
        data.set(TextField::Title, "Attention Is All You Need");
        data.issued = Some(Date::year(2017));
        data.identifiers.push(Identifier::parse(doi).unwrap());
        data
    }

    fn fetched() -> Origin {
        Origin::Fetched { source: "doi.org".into() }
    }

    #[test]
    fn same_identifier_is_stored_once() {
        let mut lib = MemoryLibrary::default();
        let first = add(&mut lib, AddRequest::new(article("10.1/a", "Vaswani"), fetched())).unwrap();
        let again = add(&mut lib, AddRequest::new(article("https://doi.org/10.1/A", "Other"), fetched())).unwrap();
        assert!(matches!(first, Added::New(_)));
        assert!(matches!(&again, Added::Existing(r) if r.id == first.reference().id));
        assert_eq!(first.reference().cite_key.as_str(), "vaswaniAttentionAllYou2017");
    }

    #[test]
    fn same_formula_key_gets_a_postfix() {
        let mut lib = MemoryLibrary::default();
        add(&mut lib, AddRequest::new(article("10.1/a", "Vaswani"), fetched())).unwrap();
        let second = add(&mut lib, AddRequest::new(article("10.1/b", "Vaswani"), fetched())).unwrap();
        assert_eq!(second.reference().cite_key.as_str(), "vaswaniAttentionAllYou2017a");
    }

    #[test]
    fn preferred_key_is_used_when_valid() {
        let mut lib = MemoryLibrary::default();
        let mut request = AddRequest::new(article("10.1/a", "Vaswani"), fetched());
        request.preferred_key = Some("attention".into());
        assert_eq!(add(&mut lib, request).unwrap().reference().cite_key.as_str(), "attention");
        let mut invalid = AddRequest::new(article("10.1/b", "Doe"), fetched());
        invalid.preferred_key = Some("not valid".into());
        assert_eq!(add(&mut lib, invalid).unwrap().reference().cite_key.as_str(), "doeAttentionAllYou2017");
    }

    #[test]
    fn find_by_key_identifier_or_id() {
        let mut lib = MemoryLibrary::default();
        let added = add(&mut lib, AddRequest::new(article("10.1/a", "Vaswani"), fetched())).unwrap();
        let id = added.reference().id;
        for query in ["vaswaniAttentionAllYou2017", "doi:10.1/A", &id.to_string()] {
            assert_eq!(find(&lib, query).unwrap().id, id, "{query}");
        }
        assert!(matches!(find(&lib, "nobody2020"), Err(CoreError::ReferenceNotFound(_))));
    }

    #[test]
    fn copy_keeps_id_and_is_not_duplicated() {
        let mut source = MemoryLibrary::default();
        let original =
            add(&mut source, AddRequest::new(article("10.1/a", "Vaswani"), fetched())).unwrap().reference().clone();
        let mut target = MemoryLibrary::default();
        add(&mut target, AddRequest::new(article("10.1/other", "Vaswani"), fetched())).unwrap();
        let copied = copy_in(&mut target, original.clone(), None).unwrap();
        assert!(matches!(&copied, Added::New(r) if r.id == original.id));
        assert_eq!(copied.reference().cite_key.as_str(), "vaswaniAttentionAllYou2017a");
        assert!(matches!(copy_in(&mut target, original, None).unwrap(), Added::Existing(_)));
    }

    #[test]
    fn trash_and_re_add_restores() {
        let mut lib = MemoryLibrary::default();
        add(&mut lib, AddRequest::new(article("10.1/a", "Vaswani"), fetched())).unwrap();
        assert!(trash(&mut lib, "doi:10.1/a").unwrap().meta.is_deleted());
        assert!(find(&lib, "vaswaniAttentionAllYou2017").unwrap().meta.is_deleted(), "found in the trash by key");
        let again = add(&mut lib, AddRequest::new(article("10.1/a", "Vaswani"), fetched())).unwrap();
        assert!(!again.reference().meta.is_deleted());
    }
}
