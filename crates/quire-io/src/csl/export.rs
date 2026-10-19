//! Stored references → CSL-JSON, e.g. for Pandoc. The cite key becomes `id` and `citation-key`.

use quire_model::{Identifier, Reference};
use serde_json::{Map, Value};

use super::values::{date_value, name_value};

/// A pretty-printed CSL-JSON array of `references`.
pub fn to_csl_json(references: &[Reference]) -> String {
    let items: Vec<Value> = references.iter().map(to_csl_value).collect();
    serde_json::to_string_pretty(&items).expect("JSON values always serialise")
}

/// One CSL-JSON item. `extra` values are written back unless a modelled field has the same name.
pub fn to_csl_value(reference: &Reference) -> Value {
    let data = &reference.data;
    let mut item = Map::new();
    for (key, value) in &data.extra {
        item.insert(key.clone(), value.clone().into());
    }
    item.insert("id".into(), reference.cite_key.as_str().into());
    item.insert("citation-key".into(), reference.cite_key.as_str().into());
    item.insert("type".into(), data.kind.as_str().into());

    for creator in &data.creators {
        let list = item.entry(creator.role.as_str()).or_insert_with(|| Value::Array(Vec::new()));
        if let Value::Array(names) = list {
            names.push(name_value(creator));
        }
    }
    if let Some(issued) = data.issued {
        item.insert("issued".into(), date_value(issued));
    }
    for (field, value) in &data.text {
        item.insert(field.as_str().into(), value.clone().into());
    }
    for id in &data.identifiers {
        // CSL has no arXiv variable: an arXiv ID is written as its DOI, unless the work has
        // another DOI (a published version), which then wins.
        let (key, value, replaces) = match id {
            Identifier::Doi(doi) => ("DOI", doi.to_string(), true),
            Identifier::Pmid(pmid) => ("PMID", pmid.to_string(), true),
            Identifier::Pmcid(pmcid) => ("PMCID", pmcid.to_string(), true),
            Identifier::Arxiv(arxiv) => ("DOI", arxiv.doi().to_string(), false),
        };
        if replaces || !item.contains_key(key) {
            item.insert(key.into(), value.into());
        }
    }
    Value::Object(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csl::from_csl_value;
    use quire_model::{CiteKey, DeviceId, Origin};
    use serde_json::json;

    #[test]
    fn round_trip_through_csl() {
        let source = json!({
            "type": "article-journal",
            "title": "A title",
            "author": [{"family": "Doe", "given": "Jane"}, {"literal": "WHO"}],
            "editor": [{"family": "Roe"}],
            "issued": {"date-parts": [[2020, 5]]},
            "DOI": "10.1000/xyz",
            "PMID": "123"
        });
        let data = from_csl_value(&source).unwrap();
        let reference = Reference::new(CiteKey::parse("doe2020").unwrap(), data.clone(), Origin::User, DeviceId::new());
        let exported = to_csl_value(&reference);
        assert_eq!(exported["id"], "doe2020");
        assert_eq!(exported["author"][1]["literal"], "WHO");
        let mut back = from_csl_value(&exported).unwrap();
        back.extra.clear();
        assert_eq!(back, data);
    }

    #[test]
    fn arxiv_id_is_written_as_its_doi() {
        let data = from_csl_value(&json!({"type": "article", "DOI": "10.48550/arXiv.1706.03762"})).unwrap();
        let reference = Reference::new(CiteKey::parse("v2017").unwrap(), data, Origin::User, DeviceId::new());
        assert_eq!(to_csl_value(&reference)["DOI"], "10.48550/arXiv.1706.03762");
    }
}
