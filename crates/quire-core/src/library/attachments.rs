//! Attachments: a stored file linked to a reference. The same file is attached to a reference
//! once; removing an attachment keeps its row as a tombstone (and the file, which other
//! references or devices may share).

use quire_model::{Attachment, AttachmentId, ReferenceId};

use crate::error::{CoreError, Result};
use crate::ports::Library;

/// Attaches `attachment` unless its reference already has the same file; returns the stored
/// attachment and whether it is new.
pub fn attach(library: &mut impl Library, attachment: Attachment) -> Result<(Attachment, bool)> {
    if let Some(existing) =
        library.attachments_of(attachment.reference)?.into_iter().find(|a| a.hash == attachment.hash)
    {
        return Ok((existing, false));
    }
    library.insert_attachment(&attachment)?;
    Ok((attachment, true))
}

/// Removes the attachment `id` (a tombstone; the file stays).
pub fn detach(library: &mut impl Library, id: AttachmentId) -> Result<Attachment> {
    let mut attachment = library.attachment(id)?.ok_or_else(|| CoreError::AttachmentNotFound(id.to_string()))?;
    attachment.meta.delete(library.device());
    library.update_attachment(&attachment)?;
    Ok(attachment)
}

/// Removes every attachment of `reference`; returns how many.
pub fn detach_all(library: &mut impl Library, reference: ReferenceId) -> Result<usize> {
    let device = library.device();
    let attachments = library.attachments_of(reference)?;
    for mut attachment in attachments.iter().cloned() {
        attachment.meta.delete(device);
        library.update_attachment(&attachment)?;
    }
    Ok(attachments.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryLibrary;
    use crate::prelude::*;
    use quire_model::DeviceId;

    fn pdf(reference: ReferenceId, hash: &str) -> Attachment {
        Attachment::new(
            reference,
            hash.into(),
            "a.pdf".into(),
            "application/pdf".into(),
            1,
            "test".into(),
            DeviceId::new(),
        )
    }

    #[test]
    fn same_file_once_then_detach() {
        let mut lib = MemoryLibrary::default();
        let reference = ReferenceId::new();
        assert!(attach(&mut lib, pdf(reference, "aa")).unwrap().1);
        assert!(!attach(&mut lib, pdf(reference, "aa")).unwrap().1);
        assert!(attach(&mut lib, pdf(reference, "bb")).unwrap().1);
        assert_eq!(lib.references_with_attachments().unwrap(), [reference]);
        assert_eq!(detach_all(&mut lib, reference).unwrap(), 2);
        assert!(lib.attachments_of(reference).unwrap().is_empty());
    }
}
