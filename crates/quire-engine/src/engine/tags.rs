//! Tags, notes and related references of [`Engine`] (as in Zotero).

use std::collections::HashMap;

use quire_core::Library;
use quire_core::library::{references, tags};
use quire_model::{Note, NoteId, Reference, ReferenceId, Tag, TagKind};

use super::Engine;
use crate::error::Result;

impl Engine {
    /// Puts each of `names` on the reference `query` (created if new); returns which were new on it.
    pub async fn tag(&self, query: &str, names: &[String]) -> Result<(Reference, Vec<(String, bool)>)> {
        let (query, names) = (query.to_string(), names.to_vec());
        let result = self
            .run(move |lib| {
                let mut reference = None;
                let mut outcome = Vec::new();
                for name in &names {
                    let (found, added) = tags::add(lib, &query, name, TagKind::Manual)?;
                    reference = Some(found);
                    outcome.push((name.clone(), added));
                }
                Ok((reference.map_or_else(|| references::find(lib, &query), Ok)?, outcome))
            })
            .await?;
        self.mark_changed();
        Ok(result)
    }

    /// Takes each of `names` off the reference `query`; returns which it had.
    pub async fn untag(&self, query: &str, names: &[String]) -> Result<(Reference, Vec<(String, bool)>)> {
        let (query, names) = (query.to_string(), names.to_vec());
        let result = self
            .run(move |lib| {
                let reference = references::find(lib, &query)?;
                let mut outcome = Vec::new();
                for name in &names {
                    outcome.push((name.clone(), tags::remove(lib, &query, name)?.1));
                }
                Ok((reference, outcome))
            })
            .await?;
        self.mark_changed();
        Ok(result)
    }

    /// Every tag with how many references carry it; colored first.
    pub async fn tags(&self) -> Result<Vec<(Tag, usize)>> {
        self.run(|lib| tags::counts(lib)).await
    }

    /// Renames a tag (merging into an existing one with that name); `true` if merged.
    pub async fn rename_tag(&self, old: &str, new: &str) -> Result<(Tag, bool)> {
        let (old, new) = (old.to_string(), new.to_string());
        let result = self.run(move |lib| tags::rename(lib, &old, &new)).await?;
        self.mark_changed();
        Ok(result)
    }

    /// Deletes a tag everywhere; returns how many references had it.
    pub async fn delete_tag(&self, name: &str) -> Result<usize> {
        let name = name.to_string();
        let count = self.run(move |lib| tags::delete(lib, &name)).await?;
        self.mark_changed();
        Ok(count)
    }

    /// Colors a tag (`None` removes the color), at `position` 1–9 or the first free one.
    pub async fn color_tag(&self, name: &str, color: Option<&str>, position: Option<u8>) -> Result<Tag> {
        let (name, color) = (name.to_string(), color.map(str::to_string));
        self.run(move |lib| tags::color(lib, &name, color.as_deref(), position)).await
    }

    /// The tags of each reference in `ids` (references without tags are left out).
    pub async fn tags_of(&self, ids: Vec<ReferenceId>) -> Result<HashMap<ReferenceId, Vec<(Tag, TagKind)>>> {
        self.run(move |lib| {
            let mut out = HashMap::new();
            for id in ids {
                let found = lib.tags_of(id)?;
                if !found.is_empty() {
                    out.insert(id, found);
                }
            }
            Ok(out)
        })
        .await
    }

    /// Adds a note to the reference `query`, or a standalone note (`None`).
    pub async fn add_note(&self, query: Option<&str>, body: &str) -> Result<Note> {
        let (query, body) = (query.map(str::to_string), body.to_string());
        let note = self.run(move |lib| tags::add_note(lib, query.as_deref(), &body)).await?;
        self.mark_changed();
        Ok(note)
    }

    /// Replaces a note's text; empty text removes it.
    pub async fn edit_note(&self, id: NoteId, body: &str) -> Result<Note> {
        let body = body.to_string();
        let note = self.run(move |lib| tags::edit_note(lib, id, &body)).await?;
        self.mark_changed();
        Ok(note)
    }

    /// Removes a note.
    pub async fn remove_note(&self, id: NoteId) -> Result<Note> {
        let note = self.run(move |lib| tags::remove_note(lib, id)).await?;
        self.mark_changed();
        Ok(note)
    }

    /// The note `id` (also a removed one, for messages).
    pub async fn note(&self, id: NoteId) -> Result<Option<Note>> {
        self.run(move |lib| lib.note(id)).await
    }

    /// Notes of the reference `query`, or the standalone notes (`None`).
    pub async fn notes(&self, query: Option<&str>) -> Result<Vec<Note>> {
        let query = query.map(str::to_string);
        self.run(move |lib| {
            let reference = query.map(|q| references::find(lib, &q)).transpose()?;
            lib.notes_of(reference.map(|r| r.id))
        })
        .await
    }

    /// How many notes each reference has.
    pub async fn note_counts(&self) -> Result<HashMap<ReferenceId, usize>> {
        Ok(self.run(|lib| lib.note_counts()).await?.into_iter().collect())
    }

    /// Relates two references both ways; `false` if they already were.
    pub async fn relate(&self, a: &str, b: &str) -> Result<(Reference, Reference, bool)> {
        let (a, b) = (a.to_string(), b.to_string());
        let result = self.run(move |lib| tags::relate(lib, &a, &b)).await?;
        self.mark_changed();
        Ok(result)
    }

    /// Unrelates two references; `false` if they were not related.
    pub async fn unrelate(&self, a: &str, b: &str) -> Result<(Reference, Reference, bool)> {
        let (a, b) = (a.to_string(), b.to_string());
        let result = self.run(move |lib| tags::unrelate(lib, &a, &b)).await?;
        self.mark_changed();
        Ok(result)
    }

    /// The references related to the reference `query`.
    pub async fn related(&self, query: &str) -> Result<Vec<Reference>> {
        let query = query.to_string();
        self.run(move |lib| {
            let reference = references::find(lib, &query)?;
            let ids = lib.related(reference.id)?;
            ids.into_iter().filter_map(|id| lib.reference(id).transpose()).collect::<quire_core::Result<Vec<_>>>()
        })
        .await
    }
}
