//! Alerts: a search of arXiv, PubMed or Crossref checked for new papers. The engine fetches; this
//! keeps only what is new (not shown before by this alert, not already in the library) and
//! records the check.

use quire_model::saved_search::name_key;
use quire_model::{Alert, AlertSource, ReferenceData, Timestamp};

use crate::error::{CoreError, Result};
use crate::ports::Library;

/// Adds an alert (an existing one with the name gets the new source and query).
pub fn create(library: &mut impl Library, name: &str, source: AlertSource, query: &str) -> Result<Alert> {
    let device = library.device();
    if let Some(mut existing) = library.alert_by_key(&name_key(name))? {
        existing.source = source;
        existing.query = query.trim().into();
        existing.meta.touch(device);
        library.update_alert(&existing)?;
        return Ok(existing);
    }
    let alert = Alert::new(name, source, query, device)?;
    library.insert_alert(&alert)?;
    Ok(alert)
}

pub fn find(library: &impl Library, name: &str) -> Result<Alert> {
    library.alert_by_key(&name_key(name))?.ok_or_else(|| CoreError::AlertNotFound(name.into()))
}

pub fn remove(library: &mut impl Library, name: &str) -> Result<Alert> {
    let mut alert = find(library, name)?;
    alert.meta.delete(library.device());
    library.update_alert(&alert)?;
    Ok(alert)
}

/// Of `found` (what the source answered at `checked`), the works `alert` has not shown and the
/// library lacks; all of them are marked shown and the check is recorded.
pub fn record_check(
    library: &mut impl Library,
    alert: &Alert,
    found: Vec<ReferenceData>,
    checked: Timestamp,
) -> Result<Vec<ReferenceData>> {
    let keys: Vec<String> = found.iter().map(work_key).collect();
    let seen = library.seen(alert.id, &keys)?;
    let mut new = Vec::new();
    for (data, key) in found.into_iter().zip(&keys) {
        if seen.contains(key) || in_library(library, &data)? {
            continue;
        }
        new.push(data);
    }
    library.mark_seen(alert.id, &keys)?;
    let mut alert = alert.clone();
    alert.checked_at = Some(checked);
    alert.meta.touch(library.device());
    library.update_alert(&alert)?;
    Ok(new)
}

/// What identifies a found work across checks: its first identifier, else its title.
fn work_key(data: &ReferenceData) -> String {
    match data.identifiers.first() {
        Some(id) => format!("{}:{}", id.kind().as_str(), id.key()),
        None => format!("title:{}", data.title().unwrap_or_default().to_lowercase()),
    }
}

fn in_library(library: &impl Library, data: &ReferenceData) -> Result<bool> {
    for identifier in &data.identifiers {
        if library.reference_by_identifier(identifier)?.is_some_and(|r| !r.meta.is_deleted()) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{AddRequest, add};
    use crate::memory::MemoryLibrary;
    use quire_model::{EntryKind, Identifier, Origin, TextField};

    fn paper(title: &str, doi: &str) -> ReferenceData {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, title);
        data.identifiers.push(Identifier::parse(doi).unwrap());
        data
    }

    #[test]
    fn only_new_works_once() {
        let mut library = MemoryLibrary::default();
        let alert = create(&mut library, "NV", AlertSource::Crossref, "nanodiamond thermometry").unwrap();
        add(&mut library, AddRequest::new(paper("Known", "10.1/known"), Origin::User)).unwrap();
        let found = vec![paper("Known", "10.1/known"), paper("Fresh", "10.1/fresh")];
        let new = record_check(&mut library, &alert, found.clone(), Timestamp::now()).unwrap();
        assert_eq!(new.len(), 1);
        assert_eq!(new[0].title(), Some("Fresh"));
        let again = record_check(&mut library, &alert, found, Timestamp::now()).unwrap();
        assert!(again.is_empty(), "shown once");
        assert!(find(&library, "nv").unwrap().checked_at.is_some());
        remove(&mut library, "NV").unwrap();
        assert!(find(&library, "nv").is_err());
    }
}
