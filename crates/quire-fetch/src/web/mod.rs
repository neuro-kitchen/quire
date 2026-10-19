//! Metadata from a web page (`.knowledge/metadata-sources.md`): publishers embed it in `<meta>`
//! tags for Google Scholar (Highwire `citation_*`, Dublin Core `dc.*`, PRISM, Open Graph), and
//! many sites describe the work in schema.org JSON-LD. Tags come first; JSON-LD fills what they
//! lack.
//!
//! - [`meta`]: reading the tags from HTML.
//! - [`highwire`]: the tags → metadata and an identifier.
//! - [`jsonld`]: JSON-LD → metadata and an identifier.

mod highwire;
mod jsonld;
mod meta;

use quire_core::RawRecord;
use quire_model::{EntryKind, Identifier, ReferenceData, Timestamp};
use url::Url;

pub(crate) use highwire::date;
pub use highwire::{PageMetadata, from_tags};
pub use jsonld::JsonLd;
pub use meta::{Tags, read_tags};

use crate::Fetched;
use crate::error::{FetchError, Result};
use crate::http::Http;

/// What a page gave.
#[derive(Debug)]
pub enum PageResult {
    /// The page names an identifier (DOI, arXiv, PMID): resolve that for better metadata.
    Identifier { identifier: Identifier, fallback: Fetched },
    /// Only the page's own metadata.
    Page(Fetched),
}

/// Fetches `url` and reads its metadata.
pub async fn fetch(http: &Http, url: &Url) -> Result<PageResult> {
    let html = http.get_text(url, "text/html,application/xhtml+xml").await?;
    read(&html, url)
}

/// The metadata in `html` of the page at `url`.
pub fn read(html: &str, url: &Url) -> Result<PageResult> {
    let tags = read_tags(html);
    let page =
        merge(from_tags(&tags, url), jsonld::read(html)).ok_or_else(|| FetchError::NoMetadata(url.to_string()))?;
    let body = serde_json::to_string(&tags.pairs).expect("string pairs serialise");
    let raw = RawRecord { source: url.to_string(), format: "html-meta".into(), body, received_at: Timestamp::now() };
    let mut data = page.data;
    if let Some(identifier) = &page.identifier {
        data.identifiers.push(identifier.clone());
    }
    let fetched = Fetched { data, raw, issues: Vec::new() };
    Ok(match page.identifier {
        Some(identifier) => PageResult::Identifier { identifier, fallback: fetched },
        None => PageResult::Page(fetched),
    })
}

/// Where a redirect page sends the browser: Elsevier's linkinghub keeps the target in a hidden
/// `redirectURL` field (and a delayed meta refresh); other pages use `<meta http-equiv=refresh>`.
pub fn html_redirect(html: &str, page: &Url) -> Option<Url> {
    let document = scraper::Html::parse_document(html);
    let hidden = scraper::Selector::parse(r#"input[name="redirectURL"]"#).expect("valid selector");
    if let Some(value) = document.select(&hidden).next().and_then(|input| input.value().attr("value")) {
        let decoded: String =
            url::form_urlencoded::parse(format!("u={value}").as_bytes()).map(|(_, v)| v.into_owned()).next()?;
        return page.join(&decoded).ok();
    }
    let meta = scraper::Selector::parse("meta").expect("valid selector");
    let refresh = document
        .select(&meta)
        .find(|m| m.value().attr("http-equiv").is_some_and(|v| v.eq_ignore_ascii_case("refresh")))?;
    let content = refresh.value().attr("content")?;
    let target = content.split_once(';')?.1.trim();
    let target = target.strip_prefix("url=").or_else(|| target.strip_prefix("URL=")).unwrap_or(target);
    page.join(target.trim_matches(['\'', '"'])).ok()
}

/// Tags first; JSON-LD supplies a better type than `webpage` and any missing field.
fn merge(tags: Option<PageMetadata>, jsonld: Option<JsonLd>) -> Option<PageMetadata> {
    let Some(jsonld) = jsonld else { return tags };
    let Some(mut page) = tags else { return Some(PageMetadata { data: jsonld.data, identifier: jsonld.identifier }) };
    fill(&mut page.data, jsonld.data);
    page.identifier = page.identifier.or(jsonld.identifier);
    Some(page)
}

fn fill(data: &mut ReferenceData, other: ReferenceData) {
    if data.kind == EntryKind::Webpage {
        data.kind = other.kind;
    }
    if data.creators.is_empty() {
        data.creators = other.creators;
    }
    if data.issued.is_none() {
        data.issued = other.issued;
    }
    for (field, value) in other.text {
        data.text.entry(field).or_insert(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linkinghub_redirect() {
        let html = r#"<meta HTTP-EQUIV="REFRESH" content="2; url='/retrieve/articleSelectPrefsTemp?Redirect=x'"/>
            <input type="hidden" name="redirectURL" value="https%3A%2F%2Fwww.sciencedirect.com%2Fscience%2Farticle%2Fpii%2FS0092867420302294%3Fvia%253Dihub" id="redirectURL"/>"#;
        let page = Url::parse("https://linkinghub.elsevier.com/retrieve/pii/S0092867420302294").unwrap();
        let next = html_redirect(html, &page).unwrap();
        assert_eq!(next.as_str(), "https://www.sciencedirect.com/science/article/pii/S0092867420302294?via%3Dihub");
        let meta_only = r#"<meta http-equiv="refresh" content="0; url=/next">"#;
        assert_eq!(html_redirect(meta_only, &page).unwrap().path(), "/next");
        assert!(html_redirect("<html></html>", &page).is_none());
    }
}
