//! Lines that are not the paper: download stamps added by a proxy or publisher on every page, and
//! running heads and feet (journal, volume and pages; "Author et al. / Journal…") that the
//! converter left between paragraphs.

use std::collections::HashMap;

/// Lines starting with these are download stamps (case-insensitive).
const STAMPS: &[&str] = &[
    "authorized licensed use limited to",
    "downloaded from ",
    "downloaded on ",
    "downloaded by ",
    "this article is protected by copyright",
    "for personal use only",
    "copyright © 20",
    "terms of use",
];

/// A short line seen this many times is a running head or foot.
const REPEATS: usize = 3;
/// Longest line taken for a running head.
const MAX_HEAD_WORDS: usize = 20;

pub fn remove_stamps_and_running_heads(markdown: &str) -> String {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for line in markdown.lines() {
        *seen.entry(shape(line)).or_default() += 1;
    }
    let keep = |line: &str| {
        let text = line.trim();
        let lower = text.trim_start_matches(['*', '_', '#', ' ']).to_lowercase();
        if STAMPS.iter().any(|stamp| lower.starts_with(stamp)) || is_author_journal_head(text) {
            return false;
        }
        let repeated = seen.get(&shape(line)).copied().unwrap_or(0) >= REPEATS;
        !(repeated && is_head_candidate(text))
    };
    let kept: Vec<&str> = markdown.lines().filter(|l| keep(l)).collect();
    kept.join("\n")
}

/// The line with digits made alike (page numbers change from page to page).
fn shape(line: &str) -> String {
    line.trim().chars().map(|c| if c.is_ascii_digit() { '#' } else { c }).collect()
}

/// A line that can be a running head: a few words (one-word lines such as "where" repeat in
/// mathematics), not a table rule or row, not a list item.
fn is_head_candidate(text: &str) -> bool {
    let words = text.split_whitespace().count();
    !text.starts_with('|')
        && !text.starts_with("- ")
        && !text.starts_with("```")
        && (MIN_HEAD_WORDS..=MAX_HEAD_WORDS).contains(&words)
        && text.chars().any(char::is_alphabetic)
}

/// Shortest line taken for a running head.
const MIN_HEAD_WORDS: usize = 3;

/// `T.I. Arabadzhiev et al. / Journal of Electromyography and Kinesiology 20 (2010) 223–232`.
fn is_author_journal_head(text: &str) -> bool {
    let Some((authors, journal)) = text.split_once(" / ") else { return false };
    authors.ends_with("et al.") && authors.split_whitespace().count() <= 6 && journal.split_whitespace().count() <= 14
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_and_heads() {
        let md = "Authorized licensed use limited to: University of Florida. Downloaded on October 19.\n\
            Real text.\n\
            IEEE TRANSACTIONS ON SIGNAL PROCESSING, VOL. 53, NO. 1, PAGE 12\n\
            More text.\n\
            IEEE TRANSACTIONS ON SIGNAL PROCESSING, VOL. 53, NO. 1, PAGE 13\n\
            IEEE TRANSACTIONS ON SIGNAL PROCESSING, VOL. 53, NO. 1, PAGE 14\n\
            |---|---|\n|---|---|\n|---|---|\n\
            K. Smith et al. / Journal of Things 3 (2001) 1–9\n\
            Last text.";
        assert_eq!(
            remove_stamps_and_running_heads(md),
            "Real text.\nMore text.\n|---|---|\n|---|---|\n|---|---|\nLast text."
        );
    }
}
