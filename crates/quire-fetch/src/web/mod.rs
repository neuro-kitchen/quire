//! Metadata from a web page (`.knowledge/metadata-sources.md`): publishers embed it in `<meta>`
//! tags for Google Scholar (Highwire `citation_*`, Dublin Core `dc.*`, PRISM, Open Graph).
//!
//! - [`meta`]: reading the tags from HTML.
//! - [`highwire`]: the tags → an identifier to resolve (preferred) or a reference built from them.

mod highwire;
mod meta;

use quire_core::RawRecord;
use quire_model::{Identifier, Timestamp};
use url::Url;

pub use highwire::{from_tags, PageMetadata};
pub use meta::{read_tags, Tags};

use crate::error::{FetchError, Result};
use crate::http::Http;
use crate::Fetched;

/// What a page gave.
#[derive(Debug)]
pub enum PageResult {
    /// The page names an identifier (DOI, arXiv, PMID): resolve that for better metadata.
    Identifier { identifier: Identifier, fallback: Fetched },
    /// Only the page's own tags.
    Page(Fetched),
}

/// Fetches `url` and reads its metadata tags.
pub async fn fetch(http: &Http, url: &Url) -> Result<PageResult> {
    let html = http.get_text(url, "text/html,application/xhtml+xml").await?;
    let tags = read_tags(&html);
    let page = from_tags(&tags, url).ok_or_else(|| FetchError::NoMetadata(url.to_string()))?;
    let body = serde_json::to_string(&tags.pairs).expect("string pairs serialise");
    let raw = RawRecord { source: url.to_string(), format: "html-meta".into(), body, received_at: Timestamp::now() };
    let fetched = Fetched { data: page.data, raw };
    Ok(match page.identifier {
        Some(identifier) => PageResult::Identifier { identifier, fallback: fetched },
        None => PageResult::Page(fetched),
    })
}
