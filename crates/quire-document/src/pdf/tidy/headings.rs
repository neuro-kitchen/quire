//! Section headings the converter ran into their paragraph or left as plain text:
//!
//! - `**1. Introduction** The myoelectric signal…` (a bold heading, then the text);
//! - `1. Introduction Failure to maintain…`, `2.1. Simulation of muscle morphology and recording
//!   conditions The muscle was…` (the heading ends before the first sentence: a capitalised word
//!   followed by a lowercase one);
//! - `V. INVARIANCE PROPERTIES OF THE ALGORITHM The invariance…` (IEEE capitals);
//! - `Introduction`, `Materials and methods` … without a number;
//! - `I. INTRODUCTION`, `3. Results` alone in their block.
//!
//! Reference lists look alike (`3. Andreassen, S., and A. Rosenfalck…`, `1. J Biomed Mater Res
//! 1989;23:535`): heading words never have commas, semicolons or digits, and nothing is split
//! after a References heading.

use super::front::is_numbering;
use super::{is_heading, is_table, plain};
use crate::paper_markdown::{is_references_title, starts_with_caption_name};

/// Longest heading, in words.
const MAX_WORDS: usize = 12;
/// Section names recognised without a number (lowercase; the longest first).
const NAMES: &[&str] = &[
    "materials and methods",
    "results and discussion",
    "methods and materials",
    "conclusions and future work",
    "related work",
    "acknowledgments",
    "acknowledgements",
    "introduction",
    "background",
    "methodology",
    "methods",
    "results",
    "discussion",
    "conclusions",
    "conclusion",
    "references",
];

pub(super) fn split(blocks: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(blocks.len());
    let mut in_references = false;
    for block in blocks {
        if is_heading(&block) {
            in_references = is_references_title(&block);
            out.push(block);
            continue;
        }
        if in_references || is_table(&block) || block.starts_with("<!--") || block.starts_with("```") {
            out.push(block);
            continue;
        }
        match heading_of(&block) {
            Some((level, heading, rest)) => {
                in_references = is_references_title(&heading);
                out.push(format!("{} {heading}", "#".repeat(level)));
                if !rest.is_empty() {
                    out.push(rest);
                }
            }
            None => out.push(block),
        }
    }
    out
}

/// (level, heading, the text after it) if `block` starts with a section heading.
fn heading_of(block: &str) -> Option<(usize, String, String)> {
    // A bold run at the start: the heading if it looks like one.
    if let Some(inner) = block.strip_prefix("**")
        && let Some(end) = inner.find("**")
    {
        let bold = inner[..end].trim();
        let rest = inner[end + 2..].trim();
        if let Some(level) = bold_heading_level(bold, rest) {
            return Some((level, bold.trim_end_matches([':', '.']).to_string(), rest.to_string()));
        }
        return None;
    }
    let text = block.replace('\n', " ");
    let words: Vec<&str> = text.split_whitespace().collect();
    let first = *words.first()?;
    if is_numbering(first) && (first.ends_with('.') || first.contains('.')) && words.len() > 1 {
        // `last`: index of the heading's last word, counted after the number.
        let last = title_words(&words[1..])? + 1;
        let heading = format!("{first} {}", words[1..=last].join(" "));
        let rest = words[last + 1..].join(" ");
        return Some((level_of(first), heading, rest));
    }
    let lower = plain(&text).to_lowercase();
    let name =
        NAMES.iter().find(|n| lower.starts_with(*n) && lower[n.len()..].chars().next().is_none_or(|c| c == ' '))?;
    let count = name.split_whitespace().count();
    let rest = words[count.min(words.len())..].join(" ");
    // A name alone, or followed by a sentence.
    let sentence = rest.chars().next().is_none_or(char::is_uppercase);
    sentence.then(|| (2, words[..count].join(" "), rest))
}

/// The level of a bold lead-in that is a heading: numbered or a section name; a short bold
/// phrase before a sentence (`**Neural triggering** Current methods…`) is a minor heading.
fn bold_heading_level(bold: &str, rest: &str) -> Option<usize> {
    let words: Vec<&str> = bold.split_whitespace().collect();
    let first = *words.first()?;
    if words.len() > MAX_WORDS
        || starts_with_caption_name(bold)
        || !clean_words(&words[usize::from(is_numbering(first))..])
    {
        return None;
    }
    if is_numbering(first) && words.len() > 1 {
        return Some(level_of(first));
    }
    let lower = bold.to_lowercase();
    let lower = lower.trim_end_matches([':', '.']);
    if NAMES.contains(&lower) {
        return Some(2);
    }
    let starts_sentence = rest.chars().next().is_some_and(char::is_uppercase);
    (words.len() <= 8 && !bold.ends_with([':', '.', ',']) && starts_sentence && !rest.is_empty()).then_some(4)
}

/// Index of the last heading word in `words` (after the number), if they start a heading.
fn title_words(words: &[&str]) -> Option<usize> {
    let first = words.first()?;
    if !first.starts_with(char::is_uppercase) {
        return None;
    }
    let capitals = |w: &str| w.chars().filter(|c| c.is_alphabetic()).all(char::is_uppercase);
    let last = if capitals(first) && first.chars().filter(|c| c.is_alphabetic()).count() > 1 {
        // IEEE: the run of capitals.
        words.iter().take_while(|w| capitals(w)).count() - 1
    } else {
        // Up to the first capitalised word followed by a lowercase one (a sentence starts).
        let starts_sentence = |i: usize| {
            words[i].starts_with(char::is_uppercase)
                && words.get(i + 1).is_some_and(|n| n.starts_with(char::is_lowercase))
        };
        (1..words.len()).find(|i| starts_sentence(*i)).map_or(words.len() - 1, |i| i - 1)
    };
    (last < MAX_WORDS && clean_words(&words[..=last])).then_some(last)
}

/// Heading words: no commas, semicolons or digits (those are citations), not a whole sentence.
fn clean_words(words: &[&str]) -> bool {
    words.iter().all(|w| !w.contains([',', ';']) && !w.chars().any(|c| c.is_ascii_digit()))
        && !words.last().is_some_and(|w| w.ends_with('.') && words.len() > 3)
}

/// `1.` → 2, `2.1.` → 3, `2.1.1` → 4; Roman numerals → 2.
fn level_of(number: &str) -> usize {
    let depth = number.trim_end_matches('.').split('.').count();
    (depth + 1).min(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(blocks: &[&str]) -> Vec<String> {
        split(blocks.iter().map(|b| b.to_string()).collect())
    }

    #[test]
    fn run_in_headings() {
        assert_eq!(
            run(&[
                "1. Introduction Failure to maintain the force.",
                "2.1. Simulation of muscle morphology and recording conditions The muscle was assumed."
            ]),
            [
                "## 1. Introduction",
                "Failure to maintain the force.",
                "### 2.1. Simulation of muscle morphology and recording conditions",
                "The muscle was assumed."
            ]
        );
        assert_eq!(
            run(&["**1. Introduction** The myoelectric signal."]),
            ["## 1. Introduction", "The myoelectric signal."]
        );
        assert_eq!(
            run(&["**Neural triggering** Current methods can fail."]),
            ["#### Neural triggering", "Current methods can fail."]
        );
        assert_eq!(
            run(&["V. INVARIANCE PROPERTIES OF THE ALGORITHM The invariance of an algorithm."]),
            ["## V. INVARIANCE PROPERTIES OF THE ALGORITHM", "The invariance of an algorithm."]
        );
        assert_eq!(run(&["I. INTRODUCTION", "3. Results"]), ["## I. INTRODUCTION", "## 3. Results"]);
        assert_eq!(
            run(&["Materials and methods Twelve subjects took part."]),
            ["## Materials and methods", "Twelve subjects took part."]
        );
    }

    #[test]
    fn not_headings() {
        for block in [
            "3. Andreassen, S., and A. Rosenfalck. Recording from a single motor unit.",
            "1. J Biomed Mater Res 1989;23(6):535–48.",
            "1 In general, if the original signal has few singularities.",
            "**Fig. 1** Adaptive mouse for users.",
            "**Encoder:** The encoder is composed of a stack.",
            "Results were similar in both groups.",
            "|a|b|",
        ] {
            assert_eq!(run(&[block]), [block], "{block}");
        }
        // Inside the references nothing is split.
        assert_eq!(run(&["## References", "1. Patzak A. Time of measurement."]).len(), 2);
    }
}
