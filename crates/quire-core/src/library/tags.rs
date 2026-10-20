//! Tags, notes and related references (as in Zotero, `.knowledge/library-model.md`).
//!
//! - A tag is created the first time it is used; names are unique ignoring case.
//! - Renaming a tag to the name of another merges them (Zotero's way of merging tags).
//! - Up to nine tags are colored, each at a position 1–9 (its key in the TUI).

use quire_model::tag::{self, MAX_COLORED};
use quire_model::{Note, NoteId, Reference, Tag, TagKind};

use crate::error::{CoreError, Result};
use crate::library::references::find;
use crate::ports::Library;

/// The live tag named `name` (any case).
pub fn find_tag(library: &impl Library, name: &str) -> Result<Tag> {
    library.tag_by_key(&tag::name_key(name))?.ok_or_else(|| CoreError::TagNotFound(name.trim().into()))
}

/// The tag named `name`, created if it does not exist.
pub fn get_or_create(library: &mut impl Library, name: &str) -> Result<Tag> {
    let name = tag::check_name(name)?;
    if let Some(found) = library.tag_by_key(&tag::name_key(&name))? {
        return Ok(found);
    }
    let created = Tag::new(&name, library.device())?;
    library.insert_tag(&created)?;
    Ok(created)
}

/// Puts the tag `name` on the reference `query`; `false` if it already had it.
pub fn add(library: &mut impl Library, query: &str, name: &str, kind: TagKind) -> Result<(Reference, bool)> {
    let reference = find(library, query)?;
    let tag = get_or_create(library, name)?;
    let added = library.tag_reference(reference.id, tag.id, kind)?;
    Ok((reference, added))
}

/// Takes the tag `name` off the reference `query`; `false` if it did not have it.
pub fn remove(library: &mut impl Library, query: &str, name: &str) -> Result<(Reference, bool)> {
    let reference = find(library, query)?;
    let tag = find_tag(library, name)?;
    let removed = library.untag_reference(reference.id, tag.id)?;
    Ok((reference, removed))
}

/// Every live tag with how many references carry it.
pub fn counts(library: &impl Library) -> Result<Vec<(Tag, usize)>> {
    library
        .tags()?
        .into_iter()
        .map(|t| Ok((library.tagged(t.id)?.len(), t)))
        .map(|r: Result<_>| r.map(|(n, t)| (t, n)))
        .collect()
}

/// Renames a tag. If another tag already has the new name, the two are merged: references move to
/// it and this tag is removed. Returns the resulting tag and whether a merge happened.
pub fn rename(library: &mut impl Library, old: &str, new: &str) -> Result<(Tag, bool)> {
    let mut source = find_tag(library, old)?;
    let new = tag::check_name(new)?;
    let device = library.device();
    match library.tag_by_key(&tag::name_key(&new))? {
        Some(target) if target.id != source.id => {
            for reference in library.tagged(source.id)? {
                library.tag_reference(reference, target.id, TagKind::Manual)?;
                library.untag_reference(reference, source.id)?;
            }
            source.meta.delete(device);
            library.update_tag(&source)?;
            Ok((target, true))
        }
        _ => {
            source.name = new;
            source.meta.touch(device);
            library.update_tag(&source)?;
            Ok((source, false))
        }
    }
}

/// Deletes a tag everywhere (references keep everything else). Returns how many it was on.
pub fn delete(library: &mut impl Library, name: &str) -> Result<usize> {
    let mut found = find_tag(library, name)?;
    let references = library.tagged(found.id)?;
    for reference in &references {
        library.untag_reference(*reference, found.id)?;
    }
    found.meta.delete(library.device());
    library.update_tag(&found)?;
    Ok(references.len())
}

/// Colors a tag (`color` = `None` removes it). A colored tag gets `position` (1–9), or the first
/// free one; another tag at that position gives it up.
pub fn color(library: &mut impl Library, name: &str, color: Option<&str>, position: Option<u8>) -> Result<Tag> {
    let mut target = find_tag(library, name)?;
    let device = library.device();
    let Some(color) = color else {
        target.color = None;
        target.position = None;
        target.meta.touch(device);
        library.update_tag(&target)?;
        return Ok(target);
    };
    let color = tag::parse_color(color)?;
    let taken: Vec<Tag> = library.tags()?.into_iter().filter(|t| t.position.is_some() && t.id != target.id).collect();
    let position = match position {
        Some(p) if (1..=MAX_COLORED).contains(&p) => p,
        Some(p) => return Err(CoreError::TagPosition(p)),
        None => target
            .position
            .or_else(|| (1..=MAX_COLORED).find(|p| !taken.iter().any(|t| t.position == Some(*p))))
            .ok_or(CoreError::TagPosition(0))?,
    };
    if let Some(mut holder) = taken.into_iter().find(|t| t.position == Some(position)) {
        holder.position = None;
        holder.color = None;
        holder.meta.touch(device);
        library.update_tag(&holder)?;
    }
    target.color = Some(color);
    target.position = Some(position);
    target.meta.touch(device);
    library.update_tag(&target)?;
    Ok(target)
}

/// Adds a note to the reference `query` (`None`: a standalone note).
pub fn add_note(library: &mut impl Library, query: Option<&str>, body: &str) -> Result<Note> {
    if body.trim().is_empty() {
        return Err(CoreError::EmptyNote);
    }
    let reference = query.map(|q| find(library, q)).transpose()?;
    let note = Note::new(reference.map(|r| r.id), body, library.device());
    library.insert_note(&note)?;
    Ok(note)
}

/// Replaces a note's text (an empty text removes the note).
pub fn edit_note(library: &mut impl Library, id: NoteId, body: &str) -> Result<Note> {
    let mut note =
        library.note(id)?.filter(|n| !n.meta.is_deleted()).ok_or_else(|| CoreError::NoteNotFound(id.to_string()))?;
    let device = library.device();
    if body.trim().is_empty() {
        note.meta.delete(device);
    } else {
        note.body = body.trim_end().to_string();
        note.meta.touch(device);
    }
    library.update_note(&note)?;
    Ok(note)
}

/// Removes a note (kept as a tombstone).
pub fn remove_note(library: &mut impl Library, id: NoteId) -> Result<Note> {
    edit_note(library, id, "")
}

/// Relates two references (both ways); `false` if they already were.
pub fn relate(library: &mut impl Library, a: &str, b: &str) -> Result<(Reference, Reference, bool)> {
    let (first, second) = (find(library, a)?, find(library, b)?);
    if first.id == second.id {
        return Err(CoreError::SelfRelation(first.cite_key.to_string()));
    }
    let related = library.relate(first.id, second.id)?;
    Ok((first, second, related))
}

/// Unrelates two references; `false` if they were not related.
pub fn unrelate(library: &mut impl Library, a: &str, b: &str) -> Result<(Reference, Reference, bool)> {
    let (first, second) = (find(library, a)?, find(library, b)?);
    let removed = library.unrelate(first.id, second.id)?;
    Ok((first, second, removed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{AddRequest, add as add_reference};
    use crate::memory::MemoryLibrary;
    use crate::ports::ReferenceFilter;
    use crate::prelude::*;
    use quire_model::{EntryKind, Identifier, Origin, ReferenceData};

    fn reference(lib: &mut MemoryLibrary, doi: &str) -> Reference {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.identifiers.push(Identifier::parse(doi).unwrap());
        add_reference(lib, AddRequest::new(data, Origin::User)).unwrap().reference().clone()
    }

    #[test]
    fn tags_are_created_once_ignoring_case_and_filter_with_and() {
        let mut lib = MemoryLibrary::default();
        let a = reference(&mut lib, "10.1/a");
        let b = reference(&mut lib, "10.1/b");
        assert!(add(&mut lib, "doi:10.1/a", "To Read", TagKind::Manual).unwrap().1);
        assert!(!add(&mut lib, "doi:10.1/a", "to read", TagKind::Manual).unwrap().1);
        add(&mut lib, "doi:10.1/b", "to read", TagKind::Manual).unwrap();
        add(&mut lib, "doi:10.1/a", "EMG", TagKind::Automatic).unwrap();
        assert_eq!(lib.tags().unwrap().len(), 2);
        let both = ReferenceFilter {
            tags: vec![find_tag(&lib, "to read").unwrap().id, find_tag(&lib, "emg").unwrap().id],
            ..ReferenceFilter::default()
        };
        assert_eq!(lib.references(&both).unwrap().iter().map(|r| r.id).collect::<Vec<_>>(), [a.id]);
        assert_eq!(
            counts(&lib).unwrap().iter().map(|(t, n)| (t.name.as_str(), *n)).collect::<Vec<_>>(),
            [("EMG", 1), ("To Read", 2)]
        );
        assert!(remove(&mut lib, "doi:10.1/b", "TO READ").unwrap().1);
        let _ = b;
    }

    #[test]
    fn rename_merges_and_delete_removes() {
        let mut lib = MemoryLibrary::default();
        reference(&mut lib, "10.1/a");
        reference(&mut lib, "10.1/b");
        add(&mut lib, "doi:10.1/a", "emg", TagKind::Manual).unwrap();
        add(&mut lib, "doi:10.1/b", "EMG signals", TagKind::Manual).unwrap();
        let (tag, merged) = rename(&mut lib, "EMG signals", "Emg").unwrap();
        assert!(merged && tag.name == "emg");
        assert_eq!(counts(&lib).unwrap()[0].1, 2);
        let (renamed, merged) = rename(&mut lib, "emg", "Electromyography").unwrap();
        assert!(!merged && renamed.name == "Electromyography");
        assert_eq!(delete(&mut lib, "electromyography").unwrap(), 2);
        assert!(lib.tags().unwrap().is_empty());
    }

    #[test]
    fn colors_take_free_positions() {
        let mut lib = MemoryLibrary::default();
        reference(&mut lib, "10.1/a");
        for name in ["important", "to read", "methods"] {
            add(&mut lib, "doi:10.1/a", name, TagKind::Manual).unwrap();
        }
        assert_eq!(color(&mut lib, "important", Some("red"), None).unwrap().position, Some(1));
        assert_eq!(color(&mut lib, "to read", Some("blue"), None).unwrap().position, Some(2));
        color(&mut lib, "methods", Some("green"), Some(1)).unwrap();
        assert_eq!(find_tag(&lib, "important").unwrap().position, None, "gave position 1 up");
        assert!(color(&mut lib, "methods", Some("green"), Some(10)).is_err());
        assert_eq!(lib.tags().unwrap()[0].name, "methods", "colored first, by position");
    }

    #[test]
    fn notes_and_relations() {
        let mut lib = MemoryLibrary::default();
        let a = reference(&mut lib, "10.1/a");
        reference(&mut lib, "10.1/b");
        let note = add_note(&mut lib, Some("doi:10.1/a"), "# Idea\nCompare with b").unwrap();
        add_note(&mut lib, None, "A standalone thought").unwrap();
        assert_eq!(lib.notes_of(Some(a.id)).unwrap().len(), 1);
        assert_eq!(lib.notes_of(None).unwrap().len(), 1);
        edit_note(&mut lib, note.id, "Changed").unwrap();
        remove_note(&mut lib, note.id).unwrap();
        assert!(lib.notes_of(Some(a.id)).unwrap().is_empty());
        assert!(add_note(&mut lib, None, "  ").is_err());

        assert!(relate(&mut lib, "doi:10.1/a", "doi:10.1/b").unwrap().2);
        assert!(!relate(&mut lib, "doi:10.1/b", "doi:10.1/a").unwrap().2, "both ways");
        assert_eq!(lib.related(a.id).unwrap().len(), 1);
        assert!(relate(&mut lib, "doi:10.1/a", "doi:10.1/a").is_err());
        assert!(unrelate(&mut lib, "doi:10.1/b", "doi:10.1/a").unwrap().2);
    }
}
