//! CSL-JSON → [`ReferenceData`]. Known variables go to their fields; other scalar values are kept
//! in `extra` as text; nested objects that quire does not model (Crossref's `reference` list,
//! `license`, `indexed`, …) are left out here — the raw record is stored separately.

use quire_model::{CreatorRole, Doi, Identifier, Pmcid, Pmid, ReferenceData, TextField};
use serde_json::{Map, Value};

use super::types::entry_kind;
use super::values::{date, names, text};
use crate::error::{IoError, Result};

/// Variables read into structured fields (not copied to `extra`).
const CONSUMED: &[&str] = &["type", "id", "citation-key", "issued", "DOI", "PMID", "PMCID"];

/// Non-standard names for CSL text variables, used when the standard one is missing.
const ALIASES: &[(&str, TextField)] = &[
    ("short-title", TextField::TitleShort),
    ("shortTitle", TextField::TitleShort),
    ("journalAbbreviation", TextField::ContainerTitleShort),
];

/// Reads one item (an object) or several (an array of objects).
pub fn from_csl_json(text: &str) -> Result<Vec<ReferenceData>> {
    match serde_json::from_str::<Value>(text)? {
        Value::Array(items) => items.iter().map(from_csl_value).collect(),
        item @ Value::Object(_) => Ok(vec![from_csl_value(&item)?]),
        _ => Err(IoError::NotCslItems),
    }
}

/// Reads one CSL-JSON item.
pub fn from_csl_value(item: &Value) -> Result<ReferenceData> {
    let item = item.as_object().ok_or(IoError::NotCslItems)?;
    let (kind, unknown_type) = entry_kind(item);
    let mut data = ReferenceData::new(kind);
    let mut used: Vec<&str> = CONSUMED.to_vec();

    for role in CreatorRole::ALL {
        if let Some(value) = item.get(role.as_str()) {
            data.creators.extend(names(*role, value));
            used.push(role.as_str());
        }
    }
    data.issued = item.get("issued").and_then(date);
    data.identifiers = identifiers(item);

    for field in TextField::ALL {
        if let Some(value) = item.get(field.as_str()).and_then(text) {
            data.set(*field, value);
        }
        used.push(field.as_str());
    }
    for (alias, field) in ALIASES {
        if data.get(*field).is_none()
            && let Some(value) = item.get(*alias).and_then(text)
        {
            data.set(*field, value);
        }
        used.push(alias);
    }

    for (key, value) in item {
        if !used.contains(&key.as_str())
            && let Some(value) = extra_text(value)
        {
            data.extra.insert(key.clone(), value);
        }
    }
    if unknown_type && let Some(name) = item.get("type").and_then(Value::as_str) {
        data.extra.insert("type".into(), name.into());
    }
    Ok(data)
}

/// DOI (an arXiv DOI becomes an arXiv ID), PMID and PMCID; malformed values are skipped.
fn identifiers(item: &Map<String, Value>) -> Vec<Identifier> {
    let get = |key: &str| item.get(key).and_then(text);
    let mut ids = Vec::new();
    if let Some(doi) = get("DOI").and_then(|t| Doi::parse(&t).ok()) {
        ids.push(doi.arxiv_id().map_or(Identifier::Doi(doi), Identifier::Arxiv));
    }
    ids.extend(get("PMID").and_then(|t| Pmid::parse(&t).ok()).map(Identifier::Pmid));
    ids.extend(get("PMCID").and_then(|t| Pmcid::parse(&t).ok()).map(Identifier::Pmcid));
    ids
}

/// Scalars and lists of scalars as text (lists joined with `; `); `None` for objects.
fn extra_text(value: &Value) -> Option<String> {
    match value {
        Value::Array(items) if items.iter().all(|v| !v.is_object() && !v.is_array()) => {
            let parts: Vec<String> = items.iter().filter_map(text).collect();
            (!parts.is_empty()).then(|| parts.join("; "))
        }
        Value::Bool(b) => Some(b.to_string()),
        Value::String(_) | Value::Number(_) => text(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_model::{EntryKind, IdentifierKind};
    use serde_json::json;

    #[test]
    fn reads_fields_aliases_and_extra() {
        let item = json!({
            "type": "journal-article",
            "title": ["Nanometre-scale thermometry in a living cell"],
            "short-title": "Thermometry",
            "author": [{"family": "Kucsko", "given": "G."}],
            "issued": {"date-parts": [[2013, 7, 31]]},
            "DOI": "10.1038/nature12373",
            "volume": 500,
            "subject": ["Multidisciplinary"],
            "license": [{"URL": "http://www.springer.com/tdm"}],
            "is-referenced-by-count": 1851
        });
        let data = from_csl_value(&item).unwrap();
        assert_eq!(data.kind, EntryKind::ArticleJournal);
        assert_eq!(data.title(), Some("Nanometre-scale thermometry in a living cell"));
        assert_eq!(data.get(TextField::TitleShort), Some("Thermometry"));
        assert_eq!(data.get(TextField::Volume), Some("500"));
        assert_eq!(data.issued.unwrap().to_string(), "2013-07-31");
        assert_eq!(data.identifiers[0].kind(), IdentifierKind::Doi);
        assert_eq!(data.extra.get("subject").map(String::as_str), Some("Multidisciplinary"));
        assert_eq!(data.extra.get("is-referenced-by-count").map(String::as_str), Some("1851"));
        assert!(!data.extra.contains_key("license") && !data.extra.contains_key("short-title"));
    }

    #[test]
    fn unknown_type_is_kept_in_extra() {
        let data = from_csl_value(&json!({"type": "grant"})).unwrap();
        assert_eq!((data.kind, data.extra.get("type").map(String::as_str)), (EntryKind::Document, Some("grant")));
    }

    #[test]
    fn arxiv_doi_becomes_arxiv_id() {
        let data = from_csl_value(&json!({"type": "article", "DOI": "10.48550/arXiv.1706.03762"})).unwrap();
        assert_eq!(data.identifiers[0].kind(), IdentifierKind::Arxiv);
    }

    #[test]
    fn rejects_non_items() {
        assert!(matches!(from_csl_json("42"), Err(IoError::NotCslItems)));
        assert!(matches!(from_csl_json("[1]"), Err(IoError::NotCslItems)));
        assert!(matches!(from_csl_json("{"), Err(IoError::Json(_))));
    }
}
