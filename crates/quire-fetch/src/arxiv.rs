//! The arXiv API (`https://export.arxiv.org/api/query`, Atom XML): the fallback when the arXiv
//! DOI's DataCite record is unavailable (new papers can take a while to register), and searches
//! for alerts (newest first). Terms: one request every 3 seconds, one connection
//! (`.knowledge/metadata-sources.md`).

use quire_core::RawRecord;
use quire_io::text::squash;
use quire_model::{ArxivId, EntryKind, Identifier, ReferenceData, TextField, Timestamp};
use roxmltree::{Document, Node};
use url::Url;

use crate::Fetched;
use crate::error::{FetchError, Result};
use crate::http::Http;

const ATOM: &str = "http://www.w3.org/2005/Atom";
const ARXIV: &str = "http://arxiv.org/schemas/atom";

/// Fetches `id` from the API at `base` (normally `https://export.arxiv.org/api/`).
pub async fn fetch(http: &Http, base: &Url, id: &ArxivId) -> Result<Fetched> {
    let mut url = base.join("query").map_err(|_| FetchError::InvalidUrl(base.to_string()))?;
    url.query_pairs_mut().append_pair("id_list", id.id());
    let body = http.get_text(&url, "application/atom+xml").await?;
    let data = parse(&body, id)
        .map_err(|e| FetchError::parse(&url, e))?
        .ok_or_else(|| FetchError::NotFound(format!("arXiv:{}", id.id())))?;
    let raw = RawRecord {
        source: "export.arxiv.org".into(),
        format: "arxiv-atom".into(),
        body,
        received_at: Timestamp::now(),
    };
    Ok(Fetched { data, raw, issues: Vec::new() })
}

/// The entry of a feed, `None` when the feed has none (unknown ID).
pub fn parse(xml: &str, id: &ArxivId) -> std::result::Result<Option<ReferenceData>, roxmltree::Error> {
    let document = Document::parse(xml)?;
    Ok(entry_nodes(&document).next().map(|entry| entry_data(entry, id)))
}

/// Every entry of a feed (search results), each with the arXiv ID its `<id>` gives.
pub fn entries(xml: &str) -> std::result::Result<Vec<ReferenceData>, roxmltree::Error> {
    let document = Document::parse(xml)?;
    Ok(entry_nodes(&document)
        .filter_map(|entry| {
            // The work, not the version listed (`2609.22661v1`).
            let id = match Identifier::parse(&child_text(entry, ATOM, "id")?).ok()? {
                Identifier::Arxiv(id) => ArxivId::parse(id.id()).ok()?,
                _ => return None,
            };
            Some(entry_data(entry, &id))
        })
        .collect())
}

/// Up to `limit` papers matching `query`, newest first. Plain words are all required anywhere
/// (`all:`); a query with arXiv's own field prefixes (`au:`, `ti:`, `abs:`, `cat:`) is sent as is.
pub async fn search(http: &Http, base: &Url, query: &str, limit: usize) -> Result<Vec<ReferenceData>> {
    let mut url = base.join("query").map_err(|_| FetchError::InvalidUrl(base.to_string()))?;
    url.query_pairs_mut()
        .append_pair("search_query", &search_query(query))
        .append_pair("sortBy", "submittedDate")
        .append_pair("sortOrder", "descending")
        .append_pair("max_results", &limit.to_string());
    let body = http.get_text(&url, "application/atom+xml").await?;
    entries(&body).map_err(|e| FetchError::parse(&url, e))
}

/// arXiv's `search_query` for what the user typed.
fn search_query(query: &str) -> String {
    let has_fields = ["au:", "ti:", "abs:", "cat:", "all:", "co:", "jr:"].iter().any(|f| query.contains(f));
    if has_fields {
        return query.trim().to_string();
    }
    query.split_whitespace().map(|word| format!("all:{word}")).collect::<Vec<_>>().join(" AND ")
}

/// The `<entry>` elements, without the error entry the API sends for a bad query.
fn entry_nodes<'a>(document: &'a Document<'a>) -> impl Iterator<Item = Node<'a, 'a>> {
    document
        .root_element()
        .children()
        .filter(|n| n.has_tag_name((ATOM, "entry")))
        // An error is reported as an entry whose id is http://arxiv.org/api/errors#…
        .filter(|entry| !child_text(*entry, ATOM, "id").is_some_and(|t| t.contains("/api/errors")))
}

fn entry_data(entry: Node, id: &ArxivId) -> ReferenceData {
    let mut data = ReferenceData::new(EntryKind::Article);
    if let Some(title) = child_text(entry, ATOM, "title") {
        data.set(TextField::Title, squash(&title));
    }
    if let Some(summary) = child_text(entry, ATOM, "summary") {
        data.set(TextField::Abstract, squash(&summary));
    }
    data.creators = entry
        .children()
        .filter(|n| n.has_tag_name((ATOM, "author")))
        .filter_map(|author| child_text(author, ATOM, "name"))
        .map(|name| crate::names::person(&squash(&name)))
        .collect();
    data.issued = child_text(entry, ATOM, "published").and_then(|t| crate::web::date(&t));
    data.set(TextField::Publisher, "arXiv");
    data.set(TextField::Url, format!("https://arxiv.org/abs/{}", id.id()));
    if let Some(comment) = child_text(entry, ARXIV, "comment") {
        data.set(TextField::Note, squash(&comment));
    }
    if let Some(category) =
        entry.children().find(|n| n.has_tag_name((ARXIV, "primary_category"))).and_then(|n| n.attribute("term"))
    {
        data.extra.insert("arxiv-category".into(), category.into());
    }
    // A published version's DOI and citation, kept as extra: the reference stays the preprint.
    for (tag, key) in [("doi", "published-doi"), ("journal_ref", "journal-ref")] {
        if let Some(value) = child_text(entry, ARXIV, tag) {
            data.extra.insert(key.into(), squash(&value));
        }
    }
    data.identifiers.push(Identifier::Arxiv(id.clone()));
    data
}

fn child_text(node: Node, namespace: &str, name: &str) -> Option<String> {
    node.children().find(|n| n.has_tag_name((namespace, name))).and_then(|n| n.text()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-written (the API rate-limited us when saving a real one): an old-style ID, and the
    /// published version's DOI and journal reference.
    const OLD_STYLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
  <entry>
    <id>http://arxiv.org/abs/hep-th/9711200v3</id>
    <published>1997-11-27T19:13:44Z</published>
    <title>The Large N Limit of Superconformal Field Theories
      and Supergravity</title>
    <summary>  We show that the large N limit of certain conformal field theories ...  </summary>
    <author><name>Juan M. Maldacena</name></author>
    <arxiv:doi>10.1023/A:1026654312961</arxiv:doi>
    <arxiv:journal_ref>Adv.Theor.Math.Phys.2:231-252,1998</arxiv:journal_ref>
    <arxiv:primary_category term="hep-th"/>
  </entry>
</feed>"#;

    #[test]
    fn old_style_entry() {
        let id = ArxivId::parse("hep-th/9711200").unwrap();
        let data = parse(OLD_STYLE, &id).unwrap().unwrap();
        assert_eq!(data.title(), Some("The Large N Limit of Superconformal Field Theories and Supergravity"));
        assert_eq!(data.creators[0].display_name(), "Maldacena, Juan M.");
        assert_eq!(data.issued.unwrap().to_string(), "1997-11-27");
        assert_eq!(data.get(TextField::Url), Some("https://arxiv.org/abs/hep-th/9711200"));
        assert_eq!(data.extra.get("published-doi").map(String::as_str), Some("10.1023/A:1026654312961"));
        assert_eq!(data.extra.get("arxiv-category").map(String::as_str), Some("hep-th"));
    }

    #[test]
    fn error_entry_is_none() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom"><entry><id>http://arxiv.org/api/errors#incorrect_id_format</id><title>Error</title></entry></feed>"#;
        assert_eq!(parse(xml, &ArxivId::parse("2101.00001").unwrap()).unwrap(), None);
    }

    #[test]
    fn search_results() {
        let xml = &quire_testkit::fixture("arxiv-api-search-nv-thermometry.xml");
        let found = entries(xml).unwrap();
        assert_eq!(found.len(), 3);
        assert_eq!(found[0].identifiers[0].to_string(), "arXiv:2609.22661");
        assert!(found[0].title().is_some() && found[0].issued.is_some());
        assert_eq!(search_query("spin defects"), "all:spin AND all:defects");
        assert_eq!(search_query("au:Lukin AND ti:thermometry"), "au:Lukin AND ti:thermometry");
    }
}
