//! Reading status: unread, reading, read, with when it started and finished.

use std::collections::HashMap;

use quire_model::{Reading, ReadingStatus, Reference, ReferenceId};

use crate::error::Result;
use crate::library::references::find;
use crate::ports::Library;

/// Sets the paper `query` names to `status`; `false` in the result if it already was.
pub fn set(library: &mut impl Library, query: &str, status: ReadingStatus) -> Result<(Reference, Reading, bool)> {
    let reference = find(library, query)?;
    let device = library.device();
    let mut reading = library.reading(reference.id)?.unwrap_or_else(|| Reading::unread(reference.id, device));
    let changed = reading.set(status, device);
    if changed {
        library.set_reading(&reading)?;
    }
    Ok((reference, reading, changed))
}

/// Marks the paper reading if it is unread (opening it in the reader); its status otherwise.
pub fn opened(library: &mut impl Library, reference: ReferenceId) -> Result<ReadingStatus> {
    let current = library.reading(reference)?;
    match current {
        Some(reading) if reading.status != ReadingStatus::Unread => Ok(reading.status),
        _ => {
            let device = library.device();
            let mut reading = current.unwrap_or_else(|| Reading::unread(reference, device));
            reading.set(ReadingStatus::Reading, device);
            library.set_reading(&reading)?;
            Ok(ReadingStatus::Reading)
        }
    }
}

/// Every reference's status that is not unread.
pub fn statuses(library: &impl Library) -> Result<HashMap<ReferenceId, Reading>> {
    Ok(library
        .readings()?
        .into_iter()
        .filter(|r| r.status != ReadingStatus::Unread)
        .map(|r| (r.reference, r))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{AddRequest, add};
    use crate::memory::MemoryLibrary;
    use quire_model::{EntryKind, Origin, ReferenceData, TextField};

    #[test]
    fn reading_then_read() {
        let mut library = MemoryLibrary::default();
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "A paper");
        let id = add(&mut library, AddRequest::new(data, Origin::User)).unwrap().reference().id;
        assert_eq!(opened(&mut library, id).unwrap(), ReadingStatus::Reading);
        let (_, reading, changed) = set(&mut library, &id.to_string(), ReadingStatus::Read).unwrap();
        assert!(changed && reading.finished_at.is_some());
        assert_eq!(opened(&mut library, id).unwrap(), ReadingStatus::Read, "opening a read paper keeps it read");
        assert_eq!(statuses(&library).unwrap()[&id].status, ReadingStatus::Read);
    }
}
