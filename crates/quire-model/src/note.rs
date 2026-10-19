//! [`Note`]: Markdown text the user writes, on a reference (Zotero's child notes) or on its own
//! (standalone notes).

use serde::{Deserialize, Serialize};

use crate::id::{DeviceId, NoteId, ReferenceId};
use crate::record::RecordMeta;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    /// The reference it is about; `None` for a standalone note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<ReferenceId>,
    /// Markdown.
    pub body: String,
    pub meta: RecordMeta,
}

impl Note {
    pub fn new(reference: Option<ReferenceId>, body: &str, device: DeviceId) -> Self {
        Self { id: NoteId::new(), reference, body: body.trim_end().to_string(), meta: RecordMeta::new(device) }
    }

    /// The first non-empty line without Markdown heading marks, cut to 80 characters.
    pub fn title(&self) -> String {
        let line = self.body.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("(empty)");
        let line = line.trim_start_matches('#').trim();
        let mut title: String = line.chars().take(80).collect();
        if line.chars().count() > 80 {
            title.push('…');
        }
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_is_the_first_line() {
        let note = Note::new(None, "\n## Methods idea\n\nUse adaptive subtraction.", DeviceId::new());
        assert_eq!(note.title(), "Methods idea");
    }
}
