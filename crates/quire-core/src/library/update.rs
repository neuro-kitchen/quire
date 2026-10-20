//! Updating a stored reference field by field (`.knowledge/data-foundations.md`, 5):
//!
//! - **refresh** (`ref refetch`): new data from a source replaces fields that came from a source
//!   or a file, never fields the user edited (those are reported as kept). A value the source no
//!   longer sends is kept; identifiers are only ever added.
//! - **edit** (`ref edit`): the user's version replaces every field that differs, including
//!   removals; changed fields become the user's.
//!
//! Both first [`plan`] the changes (shown before writing; `--dry-run` stops there), then apply them.

use quire_model::{CiteKey, Field, Identifier, Issue, Origin, Provenance, Reference, ReferenceData, TextField};

use crate::error::{CoreError, Result};
use crate::library::references::save;
use crate::ports::{Library, RawRecord};

/// How the new data relates to the stored data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Refresh,
    Edit,
}

/// What happens to one field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Takes the new value.
    Updated,
    /// The user's value is kept; the source has another one.
    KeptUserValue,
}

/// A field whose value differs between the stored reference and the new data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    pub field: Field,
    pub old: Option<String>,
    pub new: Option<String>,
    pub outcome: Outcome,
}

/// Every field a reference has, in display order.
fn all_fields() -> impl Iterator<Item = Field> {
    [Field::Kind, Field::Creators, Field::Issued, Field::Identifiers]
        .into_iter()
        .chain(TextField::ALL.iter().map(|f| Field::Text(*f)))
}

/// `field` of `data` as text, for comparing and showing changes.
pub(crate) fn field_text(data: &ReferenceData, field: Field) -> Option<String> {
    let joined = |items: Vec<String>| Some(items.join("; ")).filter(|t| !t.is_empty());
    match field {
        Field::Kind => Some(data.kind.to_string()),
        Field::Creators => joined(data.creators.iter().map(|c| format!("{} ({})", c.display_name(), c.role)).collect()),
        Field::Issued => data.issued.map(|d| d.to_string()),
        Field::Identifiers => joined(data.identifiers.iter().map(ToString::to_string).collect()),
        Field::Text(text) => data.get(text).map(str::to_string),
    }
}

/// The changes that turn `current` into `new` under `mode`.
pub(crate) fn plan(current: &Reference, new: &ReferenceData, mode: Mode) -> Vec<FieldChange> {
    let mut changes = Vec::new();
    for field in all_fields() {
        let old = field_text(&current.data, field);
        let new_text = match (mode, field) {
            (Mode::Refresh, Field::Identifiers) => {
                let merged = ReferenceData {
                    identifiers: merged_identifiers(&current.data, new),
                    ..ReferenceData::new(new.kind)
                };
                field_text(&merged, field)
            }
            _ => field_text(new, field),
        };
        if old == new_text || (mode == Mode::Refresh && new_text.is_none()) {
            continue;
        }
        let user_owned = current.provenance.get(&field).is_some_and(|p| !p.replaceable());
        let outcome = if mode == Mode::Refresh && user_owned { Outcome::KeptUserValue } else { Outcome::Updated };
        changes.push(FieldChange { field, old, new: new_text, outcome });
    }
    changes
}

/// Copies the `Updated` fields of `changes` from `new` into `reference`, attributed to `origin`.
pub fn apply(reference: &mut Reference, new: &ReferenceData, changes: &[FieldChange], mode: Mode, origin: &Origin) {
    let stamp = Provenance::now(origin.clone());
    for change in changes.iter().filter(|c| c.outcome == Outcome::Updated) {
        let data = &mut reference.data;
        match change.field {
            Field::Kind => data.kind = new.kind,
            Field::Creators => data.creators = new.creators.clone(),
            Field::Issued => data.issued = new.issued,
            Field::Identifiers if mode == Mode::Refresh => data.identifiers = merged_identifiers(data, new),
            Field::Identifiers => data.identifiers = new.identifiers.clone(),
            Field::Text(text) => match new.get(text) {
                Some(value) => data.set(text, value),
                None => {
                    data.text.remove(&text);
                }
            },
        }
        if field_text(&reference.data, change.field).is_some() {
            reference.provenance.insert(change.field, stamp.clone());
        } else {
            reference.provenance.remove(&change.field);
        }
    }
    if mode == Mode::Refresh {
        // Source variables quire has no field for: the source's current ones win.
        for (key, value) in &new.extra {
            reference.data.extra.insert(key.clone(), value.clone());
        }
    }
}

/// `current`'s identifiers plus any new kind/key from `new`.
fn merged_identifiers(current: &ReferenceData, new: &ReferenceData) -> Vec<Identifier> {
    let mut all = current.identifiers.clone();
    for id in &new.identifiers {
        if !all.iter().any(|known| known.kind() == id.kind() && known.key() == id.key()) {
            all.push(id.clone());
        }
    }
    all
}

/// What a refresh or edit did (or, dry, would do).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Updated {
    pub reference: Reference,
    pub changes: Vec<FieldChange>,
    /// Old and new cite key, when an edit changed it.
    pub cite_key: Option<(CiteKey, CiteKey)>,
}

impl Updated {
    /// Whether anything was (or would be) written.
    pub fn changed(&self) -> bool {
        self.cite_key.is_some() || self.changes.iter().any(|c| c.outcome == Outcome::Updated)
    }
}

/// Refreshes `reference` from `new` (fetched from `origin`): plans, and unless `dry_run`,
/// applies the changes, stores the raw record and replaces the recorded issues with `issues`
/// (the fetch succeeded, so earlier ones are out of date).
pub fn refresh(
    library: &mut impl Library,
    mut reference: Reference,
    mut new: ReferenceData,
    origin: Origin,
    raw: Option<RawRecord>,
    issues: &[Issue],
    dry_run: bool,
) -> Result<Updated> {
    drop_foreign_identifiers(library, &reference, &mut new)?;
    let changes = plan(&reference, &new, Mode::Refresh);
    if dry_run {
        return Ok(Updated { reference, changes, cite_key: None });
    }
    if changes.iter().any(|c| c.outcome == Outcome::Updated) {
        apply(&mut reference, &new, &changes, Mode::Refresh, &origin);
        reference.meta.touch(library.device());
        save(library, &mut reference)?;
    }
    {
        if let Some(raw) = &raw {
            library.add_raw_record(reference.id, raw)?;
        }
        library.set_recorded_issues(reference.id, issues)?;
    }
    Ok(Updated { reference, changes, cite_key: None })
}

/// Replaces `reference`'s fields with the user's `new` version; a changed cite key is checked
/// (valid, free). Unless `dry_run`, stores it.
pub fn edit(
    library: &mut impl Library,
    reference: Reference,
    new: ReferenceData,
    cite_key: Option<CiteKey>,
    dry_run: bool,
) -> Result<Updated> {
    for id in &new.identifiers {
        if let Some(other) = library.reference_by_identifier(id)?
            && other.id != reference.id
        {
            return Err(CoreError::IdentifierTaken {
                identifier: id.to_string(),
                cite_key: other.cite_key.to_string(),
            });
        }
    }
    let changes = plan(&reference, &new, Mode::Edit);
    let mut key_change = None;
    if let Some(key) = cite_key.filter(|k| *k != reference.cite_key) {
        if library.cite_key_taken(key.as_str())? {
            return Err(CoreError::CiteKeyTaken(key.to_string()));
        }
        key_change = Some((reference.cite_key.clone(), key));
    }
    let mut updated = Updated { reference, changes, cite_key: key_change };
    if !dry_run && updated.changed() {
        if let Some((_, key)) = &updated.cite_key {
            updated.reference.cite_key = key.clone();
        }
        apply(&mut updated.reference, &new, &updated.changes, Mode::Edit, &Origin::User);
        updated.reference.meta.touch(library.device());
        save(library, &mut updated.reference)?;
    }
    Ok(updated)
}

/// Removes identifiers of `new` that belong to another reference (they would clash).
fn drop_foreign_identifiers(library: &impl Library, reference: &Reference, new: &mut ReferenceData) -> Result<()> {
    let mut keep = Vec::new();
    for id in std::mem::take(&mut new.identifiers) {
        match library.reference_by_identifier(&id)? {
            // The source names an identifier stored on another reference: leave it out
            // rather than move it (the user can merge the two later).
            Some(other) if other.id != reference.id => {}
            _ => keep.push(id),
        }
    }
    new.identifiers = keep;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{AddRequest, add};
    use crate::memory::MemoryLibrary;
    use crate::prelude::*;
    use quire_model::{Creator, CreatorRole, Date, EntryKind, Issue, IssueCode};

    fn article(title: &str, year: u16) -> ReferenceData {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, title);
        data.creators.push(Creator::person(CreatorRole::Author, "Kucsko", Some("G.".into())));
        data.issued = Some(Date::year(year));
        data.identifiers.push(Identifier::parse("10.1038/nature12373").unwrap());
        data
    }

    fn fetched() -> Origin {
        Origin::Fetched { source: "doi.org".into() }
    }

    fn stored(lib: &mut MemoryLibrary) -> Reference {
        let mut request = AddRequest::new(article("Old title", 2013), fetched());
        request.issues = vec![Issue::recorded(IssueCode::Fallback, "doi.org: HTTP 503")];
        add(lib, request).unwrap().reference().clone()
    }

    #[test]
    fn refresh_updates_fetched_fields_and_keeps_user_ones() {
        let mut lib = MemoryLibrary::default();
        let mut reference = stored(&mut lib);
        // The user fixed the year by hand.
        reference.data.issued = Some(Date::year(2014));
        reference.provenance.insert(Field::Issued, Provenance::now(Origin::User));

        let mut new = article("New title", 2013);
        new.set(TextField::Volume, "500");
        new.identifiers.push(Identifier::parse("pmid:23903748").unwrap());
        let dry = refresh(&mut lib, reference.clone(), new.clone(), fetched(), None, &[], true).unwrap();
        let fields: Vec<(Field, Outcome)> = dry.changes.iter().map(|c| (c.field, c.outcome)).collect();
        assert_eq!(
            fields,
            [
                (Field::Issued, Outcome::KeptUserValue),
                (Field::Identifiers, Outcome::Updated),
                (Field::Text(TextField::Title), Outcome::Updated),
                (Field::Text(TextField::Volume), Outcome::Updated),
            ]
        );
        assert_eq!(dry.reference.data.title(), Some("Old title"), "dry run changes nothing");

        let done = refresh(&mut lib, reference, new, fetched(), None, &[], false).unwrap();
        let data = &done.reference.data;
        assert_eq!(
            (data.title(), data.issued.unwrap().get_year(), data.identifiers.len()),
            (Some("New title"), 2014, 2)
        );
        assert!(lib.recorded_issues(done.reference.id).unwrap().is_empty(), "fallback issue cleared");
    }

    #[test]
    fn refresh_keeps_values_the_source_no_longer_sends() {
        let mut lib = MemoryLibrary::default();
        let reference = stored(&mut lib);
        let mut new = article("Old title", 2013);
        new.creators.clear();
        assert!(plan(&reference, &new, Mode::Refresh).is_empty());
    }

    #[test]
    fn edit_replaces_removes_and_becomes_the_users() {
        let mut lib = MemoryLibrary::default();
        let reference = stored(&mut lib);
        let mut new = reference.data.clone();
        new.set(TextField::Title, "Edited");
        new.issued = None;
        let key = CiteKey::parse("kucsko2013").unwrap();
        let done = edit(&mut lib, reference, new, Some(key.clone()), false).unwrap();
        assert_eq!(done.reference.cite_key, key);
        assert_eq!(done.reference.data.issued, None);
        assert!(matches!(done.reference.provenance[&Field::Text(TextField::Title)].origin, Origin::User));
        assert!(!done.reference.provenance.contains_key(&Field::Issued));
    }

    #[test]
    fn edit_refuses_an_identifier_of_another_reference() {
        let mut lib = MemoryLibrary::default();
        let first = stored(&mut lib);
        let mut other = ReferenceData::new(EntryKind::Book);
        other.identifiers.push(Identifier::parse("10.1/other").unwrap());
        let second = add(&mut lib, AddRequest::new(other.clone(), fetched())).unwrap().reference().clone();
        other.identifiers = first.data.identifiers.clone();
        assert!(matches!(edit(&mut lib, second, other, None, false), Err(CoreError::IdentifierTaken { .. })));
    }
}
