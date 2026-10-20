//! Saved searches: a search kept by name and run again when opened (a smart project).

use quire_model::{SavedSearch, saved_search::name_key};

use crate::error::{CoreError, Result};
use crate::library::search::Query;
use crate::ports::Library;

/// Saves `query` as `name`, replacing the query of a search with that name.
pub fn save(library: &mut impl Library, name: &str, query: &str) -> Result<SavedSearch> {
    let parsed = Query::parse(query)?;
    if parsed.is_empty() {
        return Err(CoreError::InvalidFilter("an empty search cannot be saved".into()));
    }
    let device = library.device();
    if let Some(mut existing) = library.saved_search_by_key(&name_key(name))? {
        existing.query = query.trim().into();
        existing.meta.touch(device);
        library.update_saved_search(&existing)?;
        return Ok(existing);
    }
    let search = SavedSearch::new(name, query, device)?;
    library.insert_saved_search(&search)?;
    Ok(search)
}

/// The saved search called `name` (ignoring case).
pub fn find(library: &impl Library, name: &str) -> Result<SavedSearch> {
    library.saved_search_by_key(&name_key(name))?.ok_or_else(|| CoreError::SavedSearchNotFound(name.into()))
}

/// Renames the saved search `name` to `new`.
pub fn rename(library: &mut impl Library, name: &str, new: &str) -> Result<SavedSearch> {
    let mut search = find(library, name)?;
    if name_key(name) != name_key(new) && library.saved_search_by_key(&name_key(new))?.is_some() {
        return Err(CoreError::SavedSearchExists(new.into()));
    }
    search.name = SavedSearch::new(new, &search.query, library.device())?.name;
    search.meta.touch(library.device());
    library.update_saved_search(&search)?;
    Ok(search)
}

/// Forgets the saved search `name` (a tombstone, for sync).
pub fn remove(library: &mut impl Library, name: &str) -> Result<SavedSearch> {
    let mut search = find(library, name)?;
    search.meta.delete(library.device());
    library.update_saved_search(&search)?;
    Ok(search)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryLibrary;
    use crate::prelude::*;

    #[test]
    fn save_replace_rename_remove() {
        let mut library = MemoryLibrary::default();
        save(&mut library, "To read", "status:unread").unwrap();
        save(&mut library, "to READ", "status:unread pdf:yes").unwrap();
        assert_eq!(library.saved_searches().unwrap().len(), 1);
        assert_eq!(find(&library, "to read").unwrap().query, "status:unread pdf:yes");
        assert!(save(&mut library, "bad", "status:maybe").is_err());
        assert!(save(&mut library, "empty", "  ").is_err());
        rename(&mut library, "to read", "Queue").unwrap();
        assert!(find(&library, "to read").is_err());
        remove(&mut library, "queue").unwrap();
        assert!(library.saved_searches().unwrap().is_empty());
    }
}
