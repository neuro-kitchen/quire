//! schema.org metadata in `<script type="application/ld+json">` blocks: many sites (blogs, news,
//! some publishers) describe the page's work there, often without `citation_*` tags. Nodes may
//! be one object, an array, or an `@graph` whose nodes refer to each other by `@id` (as Yoast
//! writes them); references are followed.

use std::collections::HashMap;

use quire_model::{Creator, CreatorRole, EntryKind, Identifier, ReferenceData, TextField};
use scraper::{Html, Selector};
use serde_json::Value;

/// schema.org types of works, most specific first, with their CSL type.
const TYPES: &[(&str, EntryKind)] = &[
    ("ScholarlyArticle", EntryKind::ArticleJournal),
    ("MedicalScholarlyArticle", EntryKind::ArticleJournal),
    ("Chapter", EntryKind::Chapter),
    ("Book", EntryKind::Book),
    ("Thesis", EntryKind::Thesis),
    ("Report", EntryKind::Report),
    ("Dataset", EntryKind::Dataset),
    ("SoftwareSourceCode", EntryKind::Software),
    ("SoftwareApplication", EntryKind::Software),
    ("NewsArticle", EntryKind::ArticleNewspaper),
    ("BlogPosting", EntryKind::PostWeblog),
    ("TechArticle", EntryKind::Webpage),
    ("Article", EntryKind::Webpage),
];

/// What a page's JSON-LD says about its work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonLd {
    pub data: ReferenceData,
    pub identifier: Option<Identifier>,
}

/// The work described in `html`'s JSON-LD, if any block names one with a title.
pub(super) fn read(html: &str) -> Option<JsonLd> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    let blocks: Vec<Value> = document
        .select(&selector)
        .filter_map(|script| serde_json::from_str(&script.text().collect::<String>()).ok())
        .collect();
    let mut nodes = Vec::new();
    for value in &blocks {
        collect(value, &mut nodes);
    }
    let by_id: HashMap<&str, &Value> = nodes.iter().filter_map(|n| Some((n.get("@id")?.as_str()?, *n))).collect();
    let (node, kind) =
        TYPES.iter().find_map(|(name, kind)| nodes.iter().find(|n| has_type(n, name)).map(|n| (*n, *kind)))?;
    let graph = Graph { by_id };
    from_node(&graph, node, kind)
}

/// Every object node: the value itself, array items, `@graph` members.
fn collect<'v>(value: &'v Value, out: &mut Vec<&'v Value>) {
    match value {
        Value::Array(items) => items.iter().for_each(|item| collect(item, out)),
        Value::Object(object) => {
            if let Some(graph) = object.get("@graph") {
                collect(graph, out);
            }
            if object.contains_key("@type") {
                out.push(value);
            }
        }
        _ => {}
    }
}

fn has_type(node: &Value, name: &str) -> bool {
    match node.get("@type") {
        Some(Value::String(t)) => t == name,
        Some(Value::Array(types)) => types.iter().any(|t| t.as_str() == Some(name)),
        _ => false,
    }
}

struct Graph<'v> {
    by_id: HashMap<&'v str, &'v Value>,
}

impl<'v> Graph<'v> {
    /// `value`, or the node it refers to when it is only `{"@id": …}`.
    fn resolve(&self, value: &'v Value) -> &'v Value {
        match value.as_object() {
            Some(object) if object.len() == 1 => {
                object.get("@id").and_then(Value::as_str).and_then(|id| self.by_id.get(id).copied()).unwrap_or(value)
            }
            _ => value,
        }
    }
}

fn from_node(graph: &Graph, node: &Value, kind: EntryKind) -> Option<JsonLd> {
    let mut data = ReferenceData::new(kind);
    data.set(TextField::Title, text(node.get("headline").or_else(|| node.get("name"))?)?);
    for author in list(node.get("author").or_else(|| node.get("creator"))) {
        data.creators.extend(creator(graph.resolve(author)));
    }
    data.issued = ["datePublished", "dateCreated"]
        .iter()
        .find_map(|key| node.get(*key).and_then(text))
        .and_then(|t| super::highwire::date(&t));
    if let Some(publisher) = node.get("publisher").map(|p| graph.resolve(p)).and_then(|p| p.get("name")).and_then(text)
    {
        data.set(TextField::Publisher, publisher);
    }
    for (key, field) in [
        ("description", TextField::Abstract),
        ("abstract", TextField::Abstract),
        ("url", TextField::Url),
        ("inLanguage", TextField::Language),
        ("pagination", TextField::Page),
    ] {
        if data.get(field).is_none()
            && let Some(value) = node.get(key).and_then(text)
        {
            data.set(field, value);
        }
    }
    container(graph, node, &mut data);
    Some(JsonLd { data, identifier: identifier(node) })
}

/// Walks `isPartOf`: PublicationIssue (issue) → PublicationVolume (volume) → Periodical (journal
/// title, ISSN), or a Book (book title). Web pages and sites are not containers.
fn container(graph: &Graph, node: &Value, data: &mut ReferenceData) {
    let mut next = node.get("isPartOf").map(|v| graph.resolve(v));
    for _ in 0..4 {
        let Some(part) = next else { return };
        if let Some(issue) = part.get("issueNumber").and_then(text) {
            data.set(TextField::Issue, issue);
        }
        if let Some(volume) = part.get("volumeNumber").and_then(text) {
            data.set(TextField::Volume, volume);
        }
        if has_type(part, "Periodical")
            || has_type(part, "Book")
            || has_type(part, "PublicationIssue") && part.get("name").is_some()
        {
            if let Some(name) = part.get("name").and_then(text) {
                data.set(TextField::ContainerTitle, name);
            }
            if let Some(issn) = part.get("issn").and_then(text) {
                data.set(TextField::Issn, issn);
            }
        }
        next = part.get("isPartOf").map(|v| graph.resolve(v));
    }
}

/// A DOI from `identifier` (text or `PropertyValue`), `sameAs` or `@id`.
fn identifier(node: &Value) -> Option<Identifier> {
    let mut candidates: Vec<String> = Vec::new();
    for value in list(node.get("identifier")) {
        match value {
            Value::Object(object) => candidates.extend(object.get("value").and_then(text)),
            other => candidates.extend(text(other)),
        }
    }
    candidates.extend(list(node.get("sameAs")).into_iter().filter_map(text));
    candidates.extend(node.get("@id").and_then(text));
    candidates.iter().filter(|c| c.contains("10.")).find_map(|c| Identifier::parse(c).ok())
}

fn creator(value: &Value) -> Option<Creator> {
    let family = value.get("familyName").and_then(text);
    let given = value.get("givenName").and_then(text);
    match (family, value.get("name").and_then(text)) {
        (Some(family), _) => Some(Creator::person(CreatorRole::Author, family, given)),
        (None, Some(name)) if has_type(value, "Organization") => Some(Creator::literal(CreatorRole::Author, name)),
        (None, Some(name)) => Some(crate::names::person(&name)),
        (None, None) => text(value).map(|name| crate::names::person(&name)),
    }
}

/// A value or each item of an array.
fn list(value: Option<&Value>) -> Vec<&Value> {
    match value {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(other) => vec![other],
        None => Vec::new(),
    }
}

fn text(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(s) => s.split_whitespace().collect::<Vec<_>>().join(" "),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scholarly_article_with_graph_references() {
        let html = r##"<script type="application/ld+json">{"@context":"https://schema.org","@graph":[
            {"@type":"ScholarlyArticle","headline":"A result","author":[{"@id":"#p1"},{"@type":"Organization","name":"The Consortium"}],
             "datePublished":"2021-03-04","isPartOf":{"@type":"PublicationIssue","issueNumber":"2",
               "isPartOf":{"@type":"PublicationVolume","volumeNumber":"7","isPartOf":{"@type":"Periodical","name":"J. Things","issn":"1234-5678"}}},
             "identifier":{"@type":"PropertyValue","propertyID":"DOI","value":"10.1000/xyz"}},
            {"@type":"Person","@id":"#p1","givenName":"Jane","familyName":"Doe"}]}</script>"##;
        let found = read(html).unwrap();
        let data = &found.data;
        assert_eq!((data.kind, data.title()), (EntryKind::ArticleJournal, Some("A result")));
        let names: Vec<String> = data.creators.iter().map(Creator::display_name).collect();
        assert_eq!(names, ["Doe, Jane", "The Consortium"]);
        assert_eq!(data.get(TextField::ContainerTitle), Some("J. Things"));
        assert_eq!((data.get(TextField::Volume), data.get(TextField::Issue)), (Some("7"), Some("2")));
        assert_eq!(data.issued.unwrap().to_string(), "2021-03-04");
        assert_eq!(found.identifier.unwrap().key(), "10.1000/XYZ");
    }

    #[test]
    fn no_work_no_result() {
        assert!(read(r#"<script type="application/ld+json">{"@type":"WebSite","name":"x"}</script>"#).is_none());
        assert!(read("<script type=\"application/ld+json\">not json</script>").is_none());
    }
}
