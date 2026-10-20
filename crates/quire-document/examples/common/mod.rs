//! Scoring text against a reference (`pdftotext`), shared by the corpus tools.

use std::collections::HashMap;

/// `text`'s words, lower case, without surrounding punctuation, ligatures (`ﬁ`) as their letters
/// (`pdftotext` keeps them; the text layer expands them).
pub fn words(text: &str) -> Vec<String> {
    let text: String = text
        .chars()
        .flat_map(|c| match c {
            '\u{FB00}' => "ff".chars().collect(),
            '\u{FB01}' => "fi".chars().collect(),
            '\u{FB02}' => "fl".chars().collect(),
            '\u{FB03}' => "ffi".chars().collect(),
            '\u{FB04}' => "ffl".chars().collect(),
            '\u{FB05}' | '\u{FB06}' => "st".chars().collect(),
            c => vec![c],
        })
        .collect();
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Consecutive word pairs: they measure reading order.
pub fn pairs(text: &str) -> Vec<String> {
    words(text).windows(2).map(|w| format!("{} {}", w[0], w[1])).collect()
}

/// F1 of two bags of tokens.
pub fn f1(ours: &[String], theirs: &[String]) -> f64 {
    if ours.is_empty() && theirs.is_empty() {
        return 1.0;
    }
    let mut counts: HashMap<&str, i64> = HashMap::new();
    for t in theirs {
        *counts.entry(t).or_default() += 1;
    }
    let mut common = 0;
    for t in ours {
        if let Some(c) = counts.get_mut(t.as_str())
            && *c > 0
        {
            *c -= 1;
            common += 1;
        }
    }
    if common == 0 {
        return 0.0;
    }
    let (p, r) = (common as f64 / ours.len() as f64, common as f64 / theirs.len() as f64);
    2.0 * p * r / (p + r)
}
