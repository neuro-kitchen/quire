//! The abstract of a paper, read from its text (`pdf::to_markdown`) when no metadata service has
//! it. Seen in real papers (2026-10-19):
//!
//! - an `Abstract` / `ABSTRACT` / `Summary` heading, then paragraphs (most journals);
//! - a paragraph opening with the word (`Abstract Neurotechnology is…`, IEEE's `Abstract—…` in bold
//!   over several lines);
//! - two columns interleaved under the heading (text starting mid-word: "ring. In this…"):
//!   refused rather than stored garbled.

/// Shortest and longest text taken for an abstract.
const MIN_CHARS: usize = 150;
const MAX_CHARS: usize = 5000;

/// Words that end the abstract when a paragraph starts with them.
const ENDS: &[&str] =
    &["keywords", "key words", "index terms", "introduction", "1 introduction", "1. introduction", "background"];

/// Text that starts what follows an abstract inside its paragraph.
const TRAILERS: &[&str] =
    &["©", "Copyright", "Published by", "All rights reserved", "Index Terms", "Keywords:", "Key words:", "KEYWORDS"];

/// The abstract in `markdown`, if one can be found and looks clean.
pub fn from_markdown(markdown: &str) -> Option<String> {
    let blocks: Vec<&str> = markdown.split("\n\n").map(str::trim).filter(|b| !b.is_empty()).collect();
    let (start, inline) = blocks.iter().enumerate().find_map(|(i, block)| {
        let bare = plain(block);
        let lower = bare.to_lowercase();
        let word = ["abstract", "summary"].into_iter().find(|w| lower.starts_with(w))?;
        let rest = bare[word.len()..].trim_start_matches([' ', '—', '–', ':', '.', '-']).trim();
        // A heading alone, or the paragraph that starts with the word.
        if rest.is_empty() {
            Some((i + 1, None))
        } else if bare.len() > word.len() + 20 {
            Some((i + 1, Some(rest.to_string())))
        } else {
            None
        }
    })?;
    let mut parts: Vec<String> = inline.clone().into_iter().collect();
    // A bold inline abstract (IEEE) continues in the following bold paragraphs.
    let bold = inline.is_some() && blocks[start - 1].trim_start_matches('*').len() + 2 <= blocks[start - 1].len();
    for block in &blocks[start..] {
        let text = plain(block);
        let lower = text.to_lowercase();
        let heading = block.starts_with('#');
        if heading || ENDS.iter().any(|end| lower.starts_with(end)) {
            break;
        }
        if inline.is_some() && !(bold && block.starts_with("**")) {
            break;
        }
        parts.push(text);
        if parts.iter().map(String::len).sum::<usize>() > MAX_CHARS {
            break;
        }
    }
    let mut text = parts.join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
    // What follows an abstract in the same paragraph: copyright lines and running headers
    // (Elsevier), index terms (IEEE), keywords.
    if let Some(cut) = TRAILERS.iter().filter_map(|marker| text.find(marker)).min() {
        text.truncate(cut);
        text = text.trim_end().to_string();
    }
    let starts_well = text.chars().next().is_some_and(|c| c.is_uppercase() || c.is_ascii_digit());
    (starts_well && (MIN_CHARS..=MAX_CHARS).contains(&text.chars().count())).then_some(text)
}

/// A Markdown block as plain text: no heading marks, emphasis, `<sup>`/`<sub>` tags or link targets.
fn plain(block: &str) -> String {
    let mut text = block.trim_start_matches('#').trim().to_string();
    for tag in ["<sup>", "</sup>", "<sub>", "</sub>"] {
        text = text.replace(tag, "");
    }
    // [label](target) → label
    while let Some(open) = text.find("](") {
        let Some(close) = text[open..].find(')') else { break };
        text.replace_range(open..open + close + 1, "");
    }
    text.replace(['*', '_', '['], "").split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = "Respiratory deficits after C2 hemisection have been well documented through single-sex investigations. Although ovarian sex hormones enable enhanced recovery, it remains unknown if sex impacts spontaneous recovery at chronic time points.";

    #[test]
    fn heading_then_paragraphs() {
        let md = format!(
            "Research Center, Milwaukee\n\n### Abstract\n\n{BODY}\n\nSecond <sup>2</sup> part.\n\n#### Keywords\n\nEMG"
        );
        let found = from_markdown(&md).unwrap();
        assert!(found.starts_with("Respiratory deficits") && found.ends_with("Second 2 part."), "{found}");
    }

    #[test]
    fn inline_and_ieee_bold() {
        let md = format!("### Kevin Woeppel\n\nAbstract {BODY}\n\nIntroduction text.");
        assert!(from_markdown(&md).unwrap().starts_with("Respiratory"));
        let ieee = format!("***Abstract*—{}**\n\n**{}**\n\n***Index Terms*—EMG**", &BODY[..120], &BODY[120..]);
        let found = from_markdown(&ieee).unwrap();
        assert!(found.starts_with("Respiratory") && found.contains("chronic time points"), "{found}");
    }

    #[test]
    fn trailers_are_cut() {
        let elsevier = format!(
            "### Abstract\n\n{BODY} © 2023 Nałęcz Institute. Published by Elsevier B.V. All rights reserved. journal 43 (2023)"
        );
        assert!(from_markdown(&elsevier).unwrap().ends_with("chronic time points."));
        let ieee = format!("***Abstract*—{BODY} Index Terms—Onset detection, EMG.**");
        assert!(from_markdown(&ieee).unwrap().ends_with("chronic time points."));
    }

    #[test]
    fn garbled_or_missing_is_none() {
        let garbled = format!("## ABSTRACT\n\nring. In this condition the average activity {BODY}");
        assert_eq!(from_markdown(&garbled), None, "starts mid-word");
        assert_eq!(from_markdown("## Introduction\n\nNo abstract here at all."), None);
        assert_eq!(from_markdown("### Abstract\n\nToo short."), None);
    }
}
