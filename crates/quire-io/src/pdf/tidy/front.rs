//! The front matter: everything before the first section (journal header, title, authors,
//! affiliations, abstract, keywords) and the first page's footnotes, which the converter places
//! wherever the page ended (corresponding author, received/accepted dates, ISSN and copyright).

use super::{is_heading, plain};

/// Section names that start the body.
const OPENING_SECTIONS: &[&str] = &["introduction", "background", "motivation", "overview"];
/// Blocks that end the front matter: the first heading after one of these starts the body.
const FRONT_LAST: &[&str] = &["abstract", "summary", "keywords", "key words", "index terms"];
/// The front matter is at most this share of the blocks (a body start found later is a mistake).
const MAX_FRONT_SHARE: f32 = 0.6;
/// Longest line moved as a footnote of the first page.
const MAX_FOOTNOTE_WORDS: usize = 60;

/// Lines of `text` that are front-page footnotes, taken out; the rest of the text.
pub fn take_front_lines(text: &str) -> (String, Vec<String>) {
    let mut kept = Vec::new();
    let mut taken = Vec::new();
    for line in text.lines() {
        if is_front_line(line) {
            taken.push(line.trim().to_string());
        } else {
            kept.push(line);
        }
    }
    (kept.join("\n"), taken)
}

fn is_front_line(line: &str) -> bool {
    let text = line.trim().trim_start_matches(['*', '_', ' ']);
    if text.split_whitespace().count() > MAX_FOOTNOTE_WORDS || text.is_empty() {
        return false;
    }
    let lower = text.to_lowercase();
    let starts = [
        "corresponding author",
        "e-mail address",
        "email address",
        "e-mail:",
        "tel.:",
        "tel:",
        "fax:",
        "available online",
        "published online",
        "article history",
        "contents lists available at",
        "journal homepage",
        "manuscript received",
        "© ",
        "(c) 19",
        "(c) 20",
        "doi:",
        "https://doi.org/",
        "http://dx.doi.org/",
    ];
    let contains = ["see front matter", "all rights reserved", "published by elsevier", "elsevier ltd", "elsevier b.v"];
    let dated = |word: &str| lower.starts_with(word) && lower.chars().take(40).any(|c| c.is_ascii_digit());
    starts.iter().any(|s| lower.starts_with(s))
        || contains.iter().any(|s| lower.contains(s))
        || dated("received")
        || dated("accepted")
        || is_issn_line(text)
}

/// `1050-6411/$ - see front matter`, `0031-3998/11/7006-0607`.
fn is_issn_line(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() > 10
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..8].iter().all(u8::is_ascii_digit)
        && (b[8].is_ascii_digit() || b[8] == b'X')
        && b[9] == b'/'
}

/// The blocks before the body's first section, and the rest; all body when no start is found.
pub fn split_opening(blocks: Vec<String>) -> (Vec<String>, Vec<String>) {
    let Some(start) = body_start(&blocks) else { return (Vec::new(), blocks) };
    let mut front = blocks;
    let body = front.split_off(start);
    (front, body)
}

fn body_start(blocks: &[String]) -> Option<usize> {
    let limit = (blocks.len() as f32 * MAX_FRONT_SHARE).ceil() as usize;
    let opening = blocks.iter().position(|b| is_heading(b) && OPENING_SECTIONS.contains(&section_name(b).as_str()));
    let after_abstract = || {
        let last = blocks.iter().position(|b| FRONT_LAST.iter().any(|w| section_name(b).starts_with(w)))?;
        blocks.iter().enumerate().skip(last + 1).find(|(_, b)| is_heading(b)).map(|(i, _)| i)
    };
    opening.or_else(after_abstract).filter(|i| *i > 0 && *i <= limit)
}

/// A heading's or block's leading words without numbering, lowercase: `## 1. Introduction` →
/// `introduction`.
fn section_name(block: &str) -> String {
    let text = plain(block);
    let mut words = text.split_whitespace().peekable();
    if words.peek().is_some_and(|w| is_numbering(w)) {
        words.next();
    }
    words.collect::<Vec<_>>().join(" ").to_lowercase()
}

/// `1`, `1.`, `2.1.`, `I.`, `IV`.
pub(super) fn is_numbering(word: &str) -> bool {
    let word = word.trim_end_matches('.');
    !word.is_empty()
        && (word.split('.').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
            || word.chars().all(|c| matches!(c, 'I' | 'V' | 'X')))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(text: &str) -> Vec<String> {
        text.split("\n\n").map(str::to_string).collect()
    }

    #[test]
    fn footnote_lines() {
        let (text, taken) = take_front_lines(
            "Body line.\n* Corresponding author. Tel.: +1 555\nReceived 18 July 2008; accepted 5 Jan 2009\n\
             accepted values were higher\n1050-6411/$ - see front matter\nMore body.",
        );
        assert_eq!(text, "Body line.\naccepted values were higher\nMore body.");
        assert_eq!(taken.len(), 3);
    }

    #[test]
    fn opening_until_the_first_section() {
        let (front, body) = split_opening(blocks(
            "Title\n\nAuthors\n\nAbstract text\n\n## 1. Introduction\n\nText\n\n## 2. Methods\n\nMore",
        ));
        assert_eq!(front.len(), 3);
        assert_eq!(body[0], "## 1. Introduction");
        // No introduction: the first heading after the abstract.
        let (front, body) =
            split_opening(blocks("Title\n\n**Abstract** Text\n\n## Neural control\n\nText\n\nMore\n\nEnd"));
        assert_eq!((front.len(), body[0].as_str()), (2, "## Neural control"));
        // Nothing found: all body.
        let (front, _) = split_opening(blocks("Just\n\ntext"));
        assert!(front.is_empty());
    }
}
