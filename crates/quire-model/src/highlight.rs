//! [`Highlight`]: a passage of a paper's text the user marked in the reader, with an optional
//! comment. It is anchored by the quoted text and the few words before it (as a W3C text-quote
//! selector), not by a position: positions change when the text is extracted again or comes
//! from another source (PDF, arXiv HTML, Europe PMC), the words do not.

use serde::{Deserialize, Serialize};

use crate::id::{DeviceId, HighlightId, ReferenceId};
use crate::record::RecordMeta;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Highlight {
    pub id: HighlightId,
    pub reference: ReferenceId,
    /// The highlighted text, as shown.
    pub quote: String,
    /// Text just before the quote, to tell repeated quotes apart (may be empty).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prefix: String,
    /// The user's comment (Markdown), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub meta: RecordMeta,
}

impl Highlight {
    pub fn new(reference: ReferenceId, quote: &str, prefix: &str, comment: Option<&str>, device: DeviceId) -> Self {
        let comment = comment.map(str::trim).filter(|c| !c.is_empty()).map(str::to_string);
        Self {
            id: HighlightId::new(),
            reference,
            quote: quote.trim().to_string(),
            prefix: prefix.to_string(),
            comment,
            meta: RecordMeta::new(device),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trims_and_drops_empty_comments() {
        let h = Highlight::new(ReferenceId::new(), "  a passage ", "before", Some("  "), DeviceId::new());
        assert_eq!((h.quote.as_str(), h.comment), ("a passage", None));
        assert!(h.id.to_string().starts_with("hl_"));
    }
}
