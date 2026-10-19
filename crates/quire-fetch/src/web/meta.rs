//! Reading `<meta name|property="…" content="…">` tags and `<title>` from an HTML page.

use scraper::{Html, Selector};

/// A page's metadata tags, names lowercased, in document order (names may repeat, e.g. one
/// `citation_author` per author).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tags {
    pub pairs: Vec<(String, String)>,
    /// Text of `<title>`.
    pub title: Option<String>,
}

impl Tags {
    /// The first non-empty value of any of `names`, in the order of `names`.
    pub fn first(&self, names: &[&str]) -> Option<&str> {
        let value_of = |name: &&str| self.pairs.iter().find(|(n, v)| n == name && !v.is_empty());
        names.iter().find_map(value_of).map(|(_, v)| v.as_str())
    }

    /// Every non-empty value of `name`.
    pub fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.pairs.iter().filter(move |(n, v)| n == name && !v.is_empty()).map(|(_, v)| v.as_str())
    }
}

/// The `<meta>` tags and title of `html`.
pub fn read_tags(html: &str) -> Tags {
    let document = Html::parse_document(html);
    let meta = Selector::parse("meta").expect("valid selector");
    let pairs = document
        .select(&meta)
        .filter_map(|element| {
            let attrs = element.value();
            let name = attrs.attr("name").or_else(|| attrs.attr("property"))?;
            let content = attrs.attr("content")?;
            Some((name.trim().to_ascii_lowercase(), squash(content)))
        })
        .collect();
    let title_selector = Selector::parse("title").expect("valid selector");
    let title = document
        .select(&title_selector)
        .next()
        .map(|t| squash(&t.text().collect::<String>()))
        .filter(|t| !t.is_empty());
    Tags { pairs, title }
}

/// Trims and collapses runs of whitespace.
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_name_and_property_tags() {
        let html = r#"<html><head><title> A  page </title>
            <meta name="citation_author" content="Doe, Jane">
            <meta name="Citation_Author" content=" Roe,
                John ">
            <meta property="og:title" content="OG">
            <meta charset="utf-8"></head></html>"#;
        let tags = read_tags(html);
        assert_eq!(tags.title.as_deref(), Some("A page"));
        assert_eq!(tags.all("citation_author").collect::<Vec<_>>(), ["Doe, Jane", "Roe, John"]);
        assert_eq!(tags.first(&["dc.title", "og:title"]), Some("OG"));
    }
}
