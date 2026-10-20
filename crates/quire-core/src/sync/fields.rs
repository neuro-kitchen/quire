//! References merged part by part (`quire_model::reference::parts`): each part keeps the value
//! of its latest change ([`newer`] on the part's clock and the row's device), so two people (or
//! devices) editing different fields both keep their edit. Local writes stamp the clocks of
//! the parts they changed ([`stamp_changes`]).

use quire_model::reference::parts;
use quire_model::{Reference, Timestamp};

use super::newer;

/// When `part` of `reference` last changed: its clock, else (a reference stored before clocks)
/// the row's last change.
fn clock(reference: &Reference, part: &str) -> Timestamp {
    reference.clocks.get(part).copied().unwrap_or(reference.meta.updated_at)
}

/// Sets the clocks of the parts `new` changes from `stored` (the version in the library) to
/// `new`'s change time, and makes that time later than the stored row's.
pub fn stamp_changes(stored: &Reference, new: &mut Reference) {
    let at = if new.meta.updated_at > stored.meta.updated_at { new.meta.updated_at } else { Timestamp::now() };
    new.meta.updated_at = at;
    for part in parts::names() {
        let changed = !stored.same_part(new, &part);
        let known = new.clocks.contains_key(&part);
        if changed {
            new.clocks.insert(part, at);
        } else if !known {
            // A reference from before clocks: what is unchanged dates from its last change.
            let since = clock(stored, &part);
            new.clocks.insert(part, since);
        }
    }
}

/// `local` with every part `incoming` changed later taken from it. The result's row stamp is
/// the later of the two.
pub fn merge_reference(local: &Reference, incoming: &Reference) -> Reference {
    let mut merged = local.clone();
    for part in parts::names() {
        let (mine, theirs) = (clock(local, &part), clock(incoming, &part));
        if newer((theirs, incoming.meta.updated_by), (mine, local.meta.updated_by)) {
            if !local.same_part(incoming, &part) {
                merged.copy_part(incoming, &part);
            }
            merged.clocks.insert(part, theirs);
        } else {
            merged.clocks.insert(part, mine);
        }
    }
    if newer((incoming.meta.updated_at, incoming.meta.updated_by), (local.meta.updated_at, local.meta.updated_by)) {
        merged.meta.updated_at = incoming.meta.updated_at;
        merged.meta.updated_by = incoming.meta.updated_by;
    }
    merged.meta.created_at = local.meta.created_at.min(incoming.meta.created_at);
    // The trash state follows its part; its time is the row's.
    if merged.meta.deleted_at.is_some() {
        merged.meta.deleted_at = Some(clock(&merged, parts::TRASH));
    }
    merged
}

#[cfg(test)]
mod tests {
    use quire_model::{CiteKey, DeviceId, EntryKind, Origin, ReferenceData, TextField};

    use super::*;

    fn later(base: &Reference, millis: i64) -> Timestamp {
        Timestamp::from_millis(base.meta.updated_at.as_millis() + millis)
    }

    #[test]
    fn edits_of_different_fields_both_survive() {
        let (a, b) = (DeviceId::new(), DeviceId::new());
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "attention is all you need");
        data.set(TextField::Volume, "30");
        let original = Reference::new(CiteKey::parse("vaswani2017").unwrap(), data, Origin::User, a);

        // A fixes the title; B, not having seen that, the volume.
        let mut on_a = original.clone();
        on_a.data.set(TextField::Title, "Attention Is All You Need");
        on_a.meta.updated_at = later(&original, 10);
        stamp_changes(&original, &mut on_a);
        let mut on_b = original.clone();
        on_b.data.set(TextField::Volume, "31");
        on_b.meta.updated_at = later(&original, 20);
        on_b.meta.updated_by = b;
        stamp_changes(&original, &mut on_b);

        let merged_on_a = merge_reference(&on_a, &on_b);
        let merged_on_b = merge_reference(&on_b, &on_a);
        for merged in [&merged_on_a, &merged_on_b] {
            assert_eq!(merged.data.title(), Some("Attention Is All You Need"));
            assert_eq!(merged.data.get(TextField::Volume), Some("31"));
        }
        assert_eq!(merged_on_a.data, merged_on_b.data, "the same on both devices");
        assert_eq!(merged_on_a.meta.updated_at, on_b.meta.updated_at);
    }

    #[test]
    fn the_same_field_takes_the_later_edit_and_removals_count() {
        let a = DeviceId::new();
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "A title");
        data.set(TextField::Note, "to remove");
        let original = Reference::new(CiteKey::parse("x2020").unwrap(), data, Origin::User, a);
        let mut first = original.clone();
        first.data.set(TextField::Title, "First");
        first.meta.updated_at = later(&original, 10);
        stamp_changes(&original, &mut first);
        let mut second = original.clone();
        second.data.set(TextField::Title, "Second");
        second.data.text.remove(&TextField::Note);
        second.meta.updated_at = later(&original, 20);
        stamp_changes(&original, &mut second);
        let merged = merge_reference(&first, &second);
        assert_eq!(merged.data.title(), Some("Second"));
        assert_eq!(merged.data.get(TextField::Note), None, "the removal is a change too");
    }

    #[test]
    fn rows_from_before_clocks_merge_as_whole_rows() {
        let a = DeviceId::new();
        let mut old =
            Reference::new(CiteKey::parse("x2020").unwrap(), ReferenceData::new(EntryKind::Book), Origin::User, a);
        old.clocks.clear();
        let mut newer_row = old.clone();
        newer_row.data.set(TextField::Title, "Later");
        newer_row.meta.updated_at = later(&old, 5);
        newer_row.clocks.clear();
        assert_eq!(merge_reference(&old, &newer_row).data.title(), Some("Later"));
        assert_eq!(merge_reference(&newer_row, &old).data.title(), Some("Later"));
    }
}
