//! Highlights: passages of a paper marked in the reader, with an optional comment.

use quire_model::{Highlight, HighlightId, Reference};

use crate::error::{CoreError, Result};
use crate::library::references::find;
use crate::ports::Library;

/// Highlights `quote` (after `prefix`) in the paper `query` names, with an optional comment.
pub fn add(
    library: &mut impl Library,
    query: &str,
    quote: &str,
    prefix: &str,
    comment: Option<&str>,
) -> Result<Highlight> {
    if quote.trim().is_empty() {
        return Err(CoreError::EmptyHighlight);
    }
    let reference = find(library, query)?;
    let highlight = Highlight::new(reference.id, quote, prefix, comment, library.device());
    // The same passage again: one highlight, its comment updated if a new one is given.
    let existing = library.highlights_of(reference.id)?;
    if let Some(mut same) = existing.into_iter().find(|h| h.quote == highlight.quote && h.prefix == highlight.prefix) {
        if highlight.comment.is_some() && highlight.comment != same.comment {
            same.comment = highlight.comment;
            same.meta.touch(library.device());
            library.update_highlight(&same)?;
        }
        return Ok(same);
    }
    library.insert_highlight(&highlight)?;
    Ok(highlight)
}

/// The paper `query` names and its highlights, oldest first.
pub fn of(library: &impl Library, query: &str) -> Result<(Reference, Vec<Highlight>)> {
    let reference = find(library, query)?;
    let highlights = library.highlights_of(reference.id)?;
    Ok((reference, highlights))
}

/// Sets (or with an empty text, removes) a highlight's comment.
pub fn comment(library: &mut impl Library, id: HighlightId, text: &str) -> Result<Highlight> {
    let mut highlight = live(library, id)?;
    highlight.comment = Some(text.trim().to_string()).filter(|c| !c.is_empty());
    highlight.meta.touch(library.device());
    library.update_highlight(&highlight)?;
    Ok(highlight)
}

/// Removes a highlight (kept as a tombstone for sync).
pub fn remove(library: &mut impl Library, id: HighlightId) -> Result<Highlight> {
    let mut highlight = live(library, id)?;
    highlight.meta.delete(library.device());
    library.update_highlight(&highlight)?;
    Ok(highlight)
}

fn live(library: &impl Library, id: HighlightId) -> Result<Highlight> {
    library.highlight(id)?.filter(|h| !h.meta.is_deleted()).ok_or_else(|| CoreError::HighlightNotFound(id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{AddRequest, add as add_reference};
    use crate::memory::MemoryLibrary;
    use quire_model::{EntryKind, Origin, ReferenceData, TextField};

    #[test]
    fn add_comment_remove() {
        let mut library = MemoryLibrary::default();
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "A paper about attention");
        let key =
            add_reference(&mut library, AddRequest::new(data, Origin::User)).unwrap().reference().cite_key.to_string();
        assert!(matches!(add(&mut library, &key, "  ", "", None), Err(CoreError::EmptyHighlight)));
        let h = add(&mut library, &key, "Attention is all you need.", "We show that", None).unwrap();
        let again = add(&mut library, &key, "Attention is all you need.", "We show that", Some("again")).unwrap();
        assert_eq!((again.id, again.comment.as_deref()), (h.id, Some("again")), "the same passage: one highlight");
        assert_eq!(comment(&mut library, h.id, "key claim").unwrap().comment.as_deref(), Some("key claim"));
        assert_eq!(of(&library, &key).unwrap().1.len(), 1);
        assert_eq!(of(&library, &key).unwrap().1.len(), 1);
        remove(&mut library, h.id).unwrap();
        assert!(of(&library, &key).unwrap().1.is_empty());
        assert!(matches!(remove(&mut library, h.id), Err(CoreError::HighlightNotFound(_))));
    }
}
