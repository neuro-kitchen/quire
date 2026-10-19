//! Page tags → metadata. Highwire Press `citation_*` tags (Google Scholar inclusion guidelines)
//! first, then Dublin Core, PRISM and Open Graph as fallbacks. An identifier on the page
//! (`citation_doi`, `citation_arxiv_id`, `citation_pmid`, `dc.identifier`) is reported so the
//! caller can fetch the registry's record instead.

use quire_model::{Creator, CreatorRole, Date, EntryKind, Identifier, ReferenceData, TextField};
use url::Url;

use super::meta::Tags;

/// What a page says about the work it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageMetadata {
    /// Built from the tags alone.
    pub data: ReferenceData,
    /// An identifier found on the page, if any.
    pub identifier: Option<Identifier>,
}

/// Text fields read from the first tag present.
const TEXT_TAGS: &[(TextField, &[&str])] = &[
    (TextField::Title, &["citation_title", "dc.title", "og:title"]),
    (TextField::ContainerTitle, &["citation_journal_title", "citation_conference_title", "citation_book_title", "prism.publicationname", "og:site_name"]),
    (TextField::ContainerTitleShort, &["citation_journal_abbrev"]),
    (TextField::Volume, &["citation_volume", "prism.volume"]),
    (TextField::Issue, &["citation_issue", "prism.number"]),
    (TextField::Publisher, &["citation_publisher", "citation_dissertation_institution", "citation_technical_report_institution", "dc.publisher"]),
    (TextField::Number, &["citation_technical_report_number"]),
    (TextField::Issn, &["citation_issn", "prism.issn"]),
    (TextField::Isbn, &["citation_isbn"]),
    (TextField::Language, &["citation_language", "dc.language"]),
    (TextField::Abstract, &["citation_abstract", "dc.description"]),
];

const DATE_TAGS: &[&str] = &["citation_publication_date", "citation_date", "citation_cover_date", "citation_online_date", "dc.date", "prism.publicationdate", "article:published_time"];

const AUTHOR_TAGS: &[&str] = &["citation_author", "dc.creator", "author"];

/// The metadata in `tags` of the page at `url`, or `None` if the page has not even a title.
pub fn from_tags(tags: &Tags, url: &Url) -> Option<PageMetadata> {
    let mut data = ReferenceData::new(kind(tags));
    for (field, names) in TEXT_TAGS {
        if let Some(value) = tags.first(names) {
            data.set(*field, value);
        }
    }
    if data.title().is_none() {
        data.set(TextField::Title, tags.title.clone()?);
    }
    if let Some(first) = tags.first(&["citation_firstpage"]) {
        let page = match tags.first(&["citation_lastpage"]) {
            Some(last) if last != first => format!("{first}-{last}"),
            _ => first.to_string(),
        };
        data.set(TextField::Page, page);
    }
    data.issued = DATE_TAGS.iter().find_map(|name| tags.all(name).find_map(date));
    let authors = AUTHOR_TAGS.iter().map(|name| tags.all(name).collect::<Vec<_>>()).find(|list| !list.is_empty());
    data.creators = authors.unwrap_or_default().into_iter().map(person).collect();
    data.set(TextField::Url, tags.first(&["citation_public_url", "citation_abstract_html_url", "og:url"]).unwrap_or(url.as_str()));
    if let Some(pdf) = tags.first(&["citation_pdf_url"]) {
        data.extra.insert("pdf-url".into(), pdf.into());
    }
    Some(PageMetadata { data, identifier: identifier(tags) })
}

/// Entry type from which container tag is present.
fn kind(tags: &Tags) -> EntryKind {
    let has = |name: &str| tags.first(&[name]).is_some();
    if has("citation_journal_title") {
        EntryKind::ArticleJournal
    } else if has("citation_conference_title") {
        EntryKind::PaperConference
    } else if has("citation_book_title") {
        EntryKind::Chapter
    } else if has("citation_dissertation_institution") {
        EntryKind::Thesis
    } else if has("citation_technical_report_institution") {
        EntryKind::Report
    } else if has("citation_arxiv_id") {
        EntryKind::Article
    } else {
        EntryKind::Webpage
    }
}

/// `citation_doi`, `dc.identifier` / `prism.doi` holding a DOI, `citation_arxiv_id`,
/// `citation_pmid`, in that order.
fn identifier(tags: &Tags) -> Option<Identifier> {
    let doi = ["citation_doi", "prism.doi", "dc.identifier", "dc.identifier.doi"]
        .iter()
        .flat_map(|name| tags.all(name))
        .find_map(|value| match Identifier::parse(value) {
            Ok(id @ (Identifier::Doi(_) | Identifier::Arxiv(_))) => Some(id),
            _ => None,
        });
    let arxiv = || tags.first(&["citation_arxiv_id"]).and_then(|v| Identifier::parse(&format!("arXiv:{v}")).ok());
    let pmid = || tags.first(&["citation_pmid"]).and_then(|v| Identifier::parse(&format!("pmid:{v}")).ok());
    doi.or_else(arxiv).or_else(pmid)
}

/// `2013/07/31`, `2013-07-31`, `2013-07`, `2013`, or an ISO date-time; the longest prefix that
/// is a valid date.
fn date(text: &str) -> Option<Date> {
    let normalised = text.trim().replace('/', "-");
    let head: String = normalised.chars().take(10).collect();
    [10, 7, 4].iter().filter_map(|len| head.get(..*len)).find_map(|prefix| prefix.parse().ok())
}

/// `Last, First` or `First Last` (the last word is the family name).
fn person(name: &str) -> Creator {
    let given = |g: &str| Some(g.trim().to_string()).filter(|g| !g.is_empty());
    match name.split_once(',') {
        Some((family, rest)) => Creator::person(CreatorRole::Author, family.trim(), given(rest)),
        None => match name.trim().rsplit_once(' ') {
            Some((first, family)) => Creator::person(CreatorRole::Author, family, given(first)),
            None => Creator::person(CreatorRole::Author, name.trim(), None),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(pairs: &[(&str, &str)]) -> Tags {
        Tags { pairs: pairs.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect(), title: Some("Page title".into()) }
    }

    fn url() -> Url {
        Url::parse("https://example.org/paper").unwrap()
    }

    #[test]
    fn journal_article_tags() {
        let page = from_tags(
            &tags(&[
                ("citation_title", "A title"),
                ("citation_author", "Doe, Jane"),
                ("citation_author", "John Roe"),
                ("citation_journal_title", "Nature"),
                ("citation_publication_date", "2013/07/31"),
                ("citation_firstpage", "54"),
                ("citation_lastpage", "58"),
                ("citation_doi", "10.1038/nature12373"),
                ("citation_pdf_url", "https://example.org/paper.pdf"),
            ]),
            &url(),
        )
        .unwrap();
        assert_eq!(page.data.kind, EntryKind::ArticleJournal);
        assert_eq!(page.data.creators[1].sort_name(), "Roe");
        assert_eq!(page.data.issued.unwrap().to_string(), "2013-07-31");
        assert_eq!(page.data.get(TextField::Page), Some("54-58"));
        assert_eq!(page.data.get(TextField::Url), Some("https://example.org/paper"));
        assert_eq!(page.data.extra.get("pdf-url").map(String::as_str), Some("https://example.org/paper.pdf"));
        assert_eq!(page.identifier.unwrap().key(), "10.1038/NATURE12373");
    }

    #[test]
    fn plain_page_falls_back_to_title() {
        let page = from_tags(&tags(&[("og:site_name", "A blog")]), &url()).unwrap();
        assert_eq!((page.data.kind, page.data.title()), (EntryKind::Webpage, Some("Page title")));
        assert!(page.identifier.is_none());
        assert!(from_tags(&Tags::default(), &url()).is_none());
    }

    #[test]
    fn dates() {
        let d = |t: &str| date(t).map(|d| d.to_string());
        assert_eq!(d("2017/06/12"), Some("2017-06-12".into()));
        assert_eq!(d("2017-06-12T08:00:00Z"), Some("2017-06-12".into()));
        assert_eq!(d("2017"), Some("2017".into()));
        assert_eq!(d("June 2017"), None);
    }
}
