//! Item type of a CSL-JSON record. Crossref's content negotiation returns its own type names
//! (`journal-article`) instead of CSL's (`article-journal`); the first five mappings and the
//! `posted-content` rules come from citation-js `@citation-js/plugin-doi` 0.9.0 (`type.js`, MIT,
//! © Lars Willighagen); the book-like ones below them are quire's.

use quire_model::EntryKind;
use serde_json::{Map, Value};

/// Crossref type → CSL type.
const CROSSREF_TYPES: &[(&str, EntryKind)] = &[
    ("journal-article", EntryKind::ArticleJournal),
    ("book-chapter", EntryKind::Chapter),
    ("posted-content", EntryKind::Manuscript),
    ("proceedings-article", EntryKind::PaperConference),
    ("dissertation", EntryKind::Thesis),
    // quire additions
    ("monograph", EntryKind::Book),
    ("edited-book", EntryKind::Book),
    ("reference-book", EntryKind::Book),
    ("book-set", EntryKind::Book),
    ("book-series", EntryKind::Book),
    ("book-part", EntryKind::Chapter),
    ("book-section", EntryKind::Chapter),
    ("reference-entry", EntryKind::Entry),
    ("proceedings", EntryKind::Book),
    ("journal", EntryKind::Periodical),
    ("peer-review", EntryKind::Review),
];

/// The CSL type of `item`, and whether the source name was not understood (the caller keeps it
/// in `extra` then). Unknown or missing types become `document`.
pub fn entry_kind(item: &Map<String, Value>) -> (EntryKind, bool) {
    let Some(name) = item.get("type").and_then(Value::as_str) else {
        return (EntryKind::Document, false);
    };
    if name == "posted-content" {
        match item.get("subtype").and_then(Value::as_str) {
            Some("preprint") => return (EntryKind::Article, false),
            Some("other") if item.get("member").and_then(Value::as_str) == Some("31795") => {
                return (EntryKind::PostWeblog, false);
            }
            _ => {}
        }
    }
    if let Ok(kind) = name.parse::<EntryKind>() {
        return (kind, false);
    }
    match CROSSREF_TYPES.iter().find(|(crossref, _)| *crossref == name) {
        Some((_, kind)) => (*kind, false),
        None => (EntryKind::Document, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn kind_of(value: Value) -> (EntryKind, bool) {
        entry_kind(value.as_object().unwrap())
    }

    #[test]
    fn csl_crossref_and_unknown_types() {
        assert_eq!(kind_of(json!({"type": "article-journal"})), (EntryKind::ArticleJournal, false));
        assert_eq!(kind_of(json!({"type": "journal-article"})), (EntryKind::ArticleJournal, false));
        assert_eq!(kind_of(json!({"type": "posted-content", "subtype": "preprint"})), (EntryKind::Article, false));
        assert_eq!(kind_of(json!({"type": "posted-content"})), (EntryKind::Manuscript, false));
        assert_eq!(kind_of(json!({"type": "something-new"})), (EntryKind::Document, true));
        assert_eq!(kind_of(json!({})), (EntryKind::Document, false));
    }
}
