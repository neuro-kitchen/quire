//! References that are the same work. The store refuses two references with the same identifier,
//! so duplicates are mostly one work under different identifiers: the arXiv preprint and the
//! journal version, a PubMed record and a DOI added apart, a `.bib` import without identifiers.
//!
//! - [`index`]: every reference with the ones it duplicates and why (same identifier, or the same
//!   title with a compatible first author and year); pairs the user called distinct are left out.
//! - [`merge`]: moves everything of one reference into another (missing fields, identifiers,
//!   projects, tags, notes, PDFs, highlights, relations) and puts the emptied one in the trash.
//! - [`dismiss`]: records that two references are distinct works.

use std::collections::{BTreeMap, HashMap, HashSet};

use deunicode::deunicode;
use quire_model::{Field, Reference, ReferenceData, ReferenceId, TextField};

use crate::error::{CoreError, Result};
use crate::library::references::{find, save};
use crate::ports::{Library, ReferenceFilter};

/// Why two references look like one work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// They share an identifier (shown as written).
    SameIdentifier(String),
    /// Their titles are the same once case, accents and punctuation are ignored.
    SameTitle,
}

impl Reason {
    pub fn describe(&self) -> String {
        match self {
            Self::SameIdentifier(id) => format!("same identifier {id}"),
            Self::SameTitle => "same title".into(),
        }
    }
}

/// A reference another one duplicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub other: ReferenceId,
    pub reason: Reason,
}

/// Shortest normalised title compared (short titles such as "Editorial" repeat legitimately).
const MIN_TITLE_CHARS: usize = 20;
/// Largest gap in years between a preprint and its published version.
const MAX_YEAR_GAP: u16 = 2;

/// For each of `references` (live ones), the others that look like the same work, minus the
/// `dismissed` pairs.
pub fn index(
    references: &[Reference],
    dismissed: &HashSet<(ReferenceId, ReferenceId)>,
) -> HashMap<ReferenceId, Vec<Match>> {
    let live: Vec<&Reference> = references.iter().filter(|r| !r.meta.is_deleted()).collect();
    let mut by_key: BTreeMap<String, Vec<(usize, Option<Reason>)>> = BTreeMap::new();
    for (i, reference) in live.iter().enumerate() {
        for id in &reference.data.identifiers {
            by_key
                .entry(format!("id:{}", id.to_string().to_lowercase()))
                .or_default()
                .push((i, Some(Reason::SameIdentifier(id.to_string()))));
        }
        if let Some(title) = title_key(&reference.data) {
            by_key.entry(format!("title:{title}")).or_default().push((i, None));
        }
    }
    let mut found: HashMap<ReferenceId, Vec<Match>> = HashMap::new();
    let mut seen: HashSet<(ReferenceId, ReferenceId)> = HashSet::new();
    for group in by_key.values().filter(|g| g.len() > 1) {
        for (x, (i, reason)) in group.iter().enumerate() {
            for (j, _) in &group[x + 1..] {
                let (a, b) = (live[*i], live[*j]);
                let pair = ordered(a.id, b.id);
                if a.id == b.id || dismissed.contains(&pair) || seen.contains(&pair) {
                    continue;
                }
                let reason = match reason {
                    Some(reason) => reason.clone(),
                    None if compatible(&a.data, &b.data) => Reason::SameTitle,
                    None => continue,
                };
                seen.insert(pair);
                found.entry(a.id).or_default().push(Match { other: b.id, reason: reason.clone() });
                found.entry(b.id).or_default().push(Match { other: a.id, reason });
            }
        }
    }
    found
}

/// The duplicates of the whole library (live references, dismissed pairs left out).
pub fn library_index(library: &impl Library) -> Result<HashMap<ReferenceId, Vec<Match>>> {
    let references = library.references(&ReferenceFilter::default())?;
    let dismissed: HashSet<_> = library.dismissed_duplicates()?.into_iter().map(|(a, b)| ordered(a, b)).collect();
    Ok(index(&references, &dismissed))
}

/// The pair in a fixed order (dismissals are stored that way).
pub fn ordered(a: ReferenceId, b: ReferenceId) -> (ReferenceId, ReferenceId) {
    if a < b { (a, b) } else { (b, a) }
}

/// The title lowercased, without accents and punctuation, words single-spaced; `None` when
/// too short to tell works apart.
pub fn title_key(data: &ReferenceData) -> Option<String> {
    let title = deunicode(data.title()?).to_lowercase();
    let words: Vec<String> =
        title.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect();
    let key = words.join(" ");
    (key.len() >= MIN_TITLE_CHARS).then_some(key)
}

/// Same first author (family name) and close years, where both are known.
fn compatible(a: &ReferenceData, b: &ReferenceData) -> bool {
    let family = |d: &ReferenceData| d.creators.first().map(|c| deunicode(&c.sort_name()).to_lowercase());
    let first_word = |name: String| name.split([',', ' ']).next().unwrap_or_default().to_string();
    let authors_agree = match (family(a).map(first_word), family(b).map(first_word)) {
        (Some(x), Some(y)) => x == y,
        _ => true,
    };
    let years_agree = match (a.issued.map(|d| d.get_year()), b.issued.map(|d| d.get_year())) {
        (Some(x), Some(y)) => x.abs_diff(y) <= MAX_YEAR_GAP,
        _ => true,
    };
    authors_agree && years_agree
}

/// What [`merge`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merged {
    /// The kept reference, with the other's fields filled in.
    pub kept: Reference,
    /// The merged-away reference, now in the trash.
    pub removed: Reference,
    /// Fields copied into the kept reference.
    pub fields: Vec<Field>,
    pub projects: usize,
    pub tags: usize,
    pub notes: usize,
    pub attachments: usize,
    pub highlights: usize,
    pub relations: usize,
}

/// Merges the reference `other` names into the one `keep` names: the kept one gains the other's
/// identifiers and every field it lacks, its projects, tags, notes, PDFs and relations; the other
/// goes to the trash (restoring it brings back an empty record, its identifiers stay moved).
pub fn merge(library: &mut impl Library, keep: &str, other: &str) -> Result<Merged> {
    let mut kept = find(library, keep)?;
    let mut removed = find(library, other)?;
    if kept.id == removed.id {
        return Err(CoreError::SameReference(keep.into(), other.into()));
    }
    let device = library.device();

    // Identifiers are unique in the store: take them off the other one first.
    let moved_ids: Vec<_> = std::mem::take(&mut removed.data.identifiers);
    removed.meta.delete(device);
    save(library, &mut removed)?;

    let fields = fill_missing(&mut kept, &removed, moved_ids);
    kept.meta.touch(device);
    save(library, &mut kept)?;

    let mut merged =
        Merged { kept, removed, fields, projects: 0, tags: 0, notes: 0, attachments: 0, highlights: 0, relations: 0 };
    let (kept, removed) = (merged.kept.clone(), merged.removed.clone());
    for project in library.projects_of(removed.id)? {
        merged.projects += usize::from(library.link(kept.id, project)?);
    }
    for (tag, kind) in library.tags_of(removed.id)? {
        merged.tags += usize::from(library.tag_reference(kept.id, tag.id, kind)?);
    }
    for mut note in library.notes_of(Some(removed.id))? {
        note.reference = Some(kept.id);
        note.meta.touch(device);
        library.update_note(&note)?;
        merged.notes += 1;
    }
    for mut attachment in library.attachments_of(removed.id)? {
        attachment.reference = kept.id;
        attachment.meta.touch(device);
        library.update_attachment(&attachment)?;
        merged.attachments += 1;
    }
    for mut highlight in library.highlights_of(removed.id)? {
        highlight.reference = kept.id;
        highlight.meta.touch(device);
        library.update_highlight(&highlight)?;
        merged.highlights += 1;
    }
    for related in library.related(removed.id)? {
        library.unrelate(removed.id, related)?;
        if related != kept.id {
            merged.relations += usize::from(library.relate(kept.id, related)?);
        }
    }
    Ok(merged)
}

/// Copies into `kept` what it lacks from `other` (with provenance), plus `identifiers`; the
/// fields copied.
fn fill_missing(kept: &mut Reference, other: &Reference, identifiers: Vec<quire_model::Identifier>) -> Vec<Field> {
    let mut copied = Vec::new();
    let mut take = |field: Field, kept: &mut Reference| {
        if let Some(stamp) = other.provenance.get(&field) {
            kept.provenance.insert(field, stamp.clone());
        }
        copied.push(field);
    };
    let before = kept.data.identifiers.len();
    for id in identifiers {
        if !kept.data.identifiers.contains(&id) {
            kept.data.identifiers.push(id);
        }
    }
    if kept.data.identifiers.len() > before {
        take(Field::Identifiers, kept);
    }
    if kept.data.creators.is_empty() && !other.data.creators.is_empty() {
        kept.data.creators = other.data.creators.clone();
        take(Field::Creators, kept);
    }
    if kept.data.issued.is_none() && other.data.issued.is_some() {
        kept.data.issued = other.data.issued;
        take(Field::Issued, kept);
    }
    for field in TextField::ALL {
        if kept.data.get(*field).is_none()
            && let Some(value) = other.data.get(*field)
        {
            kept.data.set(*field, value.to_string());
            take(Field::Text(*field), kept);
        }
    }
    copied
}

/// Records that the references `a` and `b` name are distinct works; `false` if already recorded.
pub fn dismiss(library: &mut impl Library, a: &str, b: &str) -> Result<bool> {
    let (a, b) = (find(library, a)?, find(library, b)?);
    if a.id == b.id {
        return Err(CoreError::SameReference(a.cite_key.to_string(), b.cite_key.to_string()));
    }
    library.dismiss_duplicate(a.id, b.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{AddRequest, add};
    use crate::memory::MemoryLibrary;
    use crate::prelude::*;
    use quire_model::{Creator, CreatorRole, Date, EntryKind, Identifier, Origin};

    fn data(title: &str, family: &str, year: u16, id: Option<&str>) -> ReferenceData {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, title);
        data.creators.push(Creator::person(CreatorRole::Author, family, Some("A.".into())));
        data.issued = Some(Date::year(year));
        if let Some(id) = id {
            data.identifiers.push(Identifier::parse(id).unwrap());
        }
        data
    }

    fn added(library: &mut MemoryLibrary, data: ReferenceData) -> Reference {
        add(library, AddRequest::new(data, Origin::User)).unwrap().reference().clone()
    }

    #[test]
    fn preprint_and_journal_version() {
        let mut library = MemoryLibrary::default();
        let preprint =
            added(&mut library, data("Attention Is All You Need", "Vaswani", 2017, Some("arXiv:1706.03762")));
        let journal = added(&mut library, data("Attention is all you need!", "Vaswani", 2018, Some("10.5555/3295222")));
        let other = added(&mut library, data("Attention is all you need", "Smith", 2017, None));
        let old = added(&mut library, data("Attention is all you need", "Vaswani", 1990, None));
        let found = library_index(&library).unwrap();
        assert_eq!(found[&preprint.id], [Match { other: journal.id, reason: Reason::SameTitle }]);
        assert!(!found.contains_key(&other.id), "another first author");
        assert!(!found.contains_key(&old.id), "years too far apart");

        assert!(dismiss(&mut library, preprint.cite_key.as_str(), journal.cite_key.as_str()).unwrap());
        assert!(library_index(&library).unwrap().is_empty(), "dismissed");
    }

    #[test]
    fn merge_moves_everything() {
        let mut library = MemoryLibrary::default();
        let keep = added(&mut library, data("Attention Is All You Need", "Vaswani", 2017, Some("arXiv:1706.03762")));
        let mut journal = data("Attention is all you need", "Vaswani", 2017, Some("10.5555/3295222"));
        journal.set(TextField::ContainerTitle, "NeurIPS");
        let other = added(&mut library, journal);
        library.relate(other.id, keep.id).unwrap();
        let note = quire_model::Note::new(Some(other.id), "key idea", library.device());
        library.insert_note(&note).unwrap();

        let merged = merge(&mut library, keep.cite_key.as_str(), other.cite_key.as_str()).unwrap();
        assert!(merged.fields.contains(&Field::Text(TextField::ContainerTitle)));
        assert_eq!(merged.kept.data.identifiers.len(), 2);
        assert_eq!(merged.notes, 1);
        let stored = library.reference(other.id).unwrap().unwrap();
        assert!(stored.meta.is_deleted() && stored.data.identifiers.is_empty());
        let doi = Identifier::parse("10.5555/3295222").unwrap();
        assert_eq!(library.reference_by_identifier(&doi).unwrap().unwrap().id, keep.id);
        assert!(library.related(keep.id).unwrap().is_empty(), "the relation to itself is gone");
    }
}
