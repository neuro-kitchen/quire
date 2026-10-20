//! Tags beyond putting them on (in the crate root): taking them off, renaming, deleting,
//! colouring; related references; standalone notes (on no reference).

use crate::reader::{self, NoteRecord};
use crate::records::{ReferenceRecord, TagRecord};
use crate::{QuireLibrary, Result, query};

#[uniffi::export]
impl QuireLibrary {
    /// Takes tags off a reference.
    pub async fn untag(&self, id: String, names: Vec<String>) -> Result<()> {
        self.with(|engine| async move {
            engine.untag(&id, &names).await?;
            Ok(())
        })
        .await
    }

    /// Renames a tag everywhere; `true` if it merged into an existing tag of that name.
    pub async fn rename_tag(&self, old: String, new: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.rename_tag(&old, &new).await?.1) }).await
    }

    /// Deletes a tag everywhere; returns how many references had it.
    pub async fn delete_tag(&self, name: String) -> Result<u32> {
        self.with(|engine| async move { Ok(engine.delete_tag(&name).await? as u32) }).await
    }

    /// Colours a tag (`#rrggbb`; `None` removes the colour), at `position` 1–9 or the first free one.
    pub async fn color_tag(&self, name: String, color: Option<String>, position: Option<u8>) -> Result<TagRecord> {
        self.with(|engine| async move {
            let tag = engine.color_tag(&name, color.as_deref(), position).await?;
            let count = engine.tags().await?.into_iter().find(|(t, _)| t.id == tag.id).map_or(0, |(_, n)| n as u64);
            Ok(TagRecord { name: tag.name, color: tag.color, count })
        })
        .await
    }

    /// Relates two references both ways; `false` if they already were.
    pub async fn relate(&self, a: String, b: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.relate(&a, &b).await?.2) }).await
    }

    /// `false` if they were not related.
    pub async fn unrelate(&self, a: String, b: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.unrelate(&a, &b).await?.2) }).await
    }

    /// The references related to a reference.
    pub async fn related(&self, id: String) -> Result<Vec<ReferenceRecord>> {
        self.with(|engine| async move {
            let mut records = Vec::new();
            for reference in engine.related(&id).await? {
                records.push(query::reference(&engine, &reference.id.to_string()).await?);
            }
            Ok(records)
        })
        .await
    }

    /// Notes on no reference, oldest first.
    pub async fn standalone_notes(&self) -> Result<Vec<NoteRecord>> {
        self.with(|engine| async move { Ok(engine.notes(None).await?.into_iter().map(reader::note).collect()) }).await
    }

    pub async fn add_standalone_note(&self, body: String) -> Result<NoteRecord> {
        self.with(|engine| async move { Ok(reader::note(engine.add_note(None, &body).await?)) }).await
    }
}
