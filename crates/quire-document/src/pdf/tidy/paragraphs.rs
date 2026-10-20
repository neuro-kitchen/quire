//! Paragraph repair: paragraphs cut by a page or column break joined (`…Bigland-Ritchie,` +
//! `1981) or remain…`), words hyphenated at a line end mended (`deter- mined` → `determined`,
//! `self- attention` → `self-attention`), and Elsevier's MathPi characters read as the symbols
//! they draw (`ð1Þ` → `(1)`, `¼` → `=`, `þ` → `+`).

use super::{is_heading, is_table};
use crate::paper_markdown::caption_label;

/// First parts of compounds whose hyphen stays (`self-attention`, `two-dimensional`).
const COMPOUND_FIRST: &[&str] = &[
    "self",
    "non",
    "well",
    "multi",
    "co",
    "anti",
    "pre",
    "post",
    "inter",
    "cross",
    "high",
    "low",
    "long",
    "short",
    "real",
    "time",
    "two",
    "three",
    "four",
    "five",
    "one",
    "single",
    "double",
    "sub",
    "semi",
    "over",
    "under",
    "half",
    "full",
    "state",
    "end",
    "user",
    "data",
    "on",
    "off",
    "in",
    "out",
    "up",
    "down",
    "large",
    "small",
    "open",
    "closed",
    "linear",
    "zero",
    "first",
    "second",
    "third",
    "higher",
    "lower",
    "wide",
    "narrow",
    "auto",
    "least",
    "mean",
    "root",
    "signal",
    "noise",
    "frequency",
    "band",
    "all",
    "non",
    "best",
    "worst",
    "fine",
];
/// Words after a hyphen that mean it is a suspended compound (`pre- and post-`).
const CONJUNCTIONS: &[&str] = &["and", "or", "to", "nor"];
/// Characters of Elsevier's MathPi font as the converter reads them, and what they show.
const MATH_PI: &[(char, &str)] = &[('ð', "("), ('Þ', ")"), ('¼', "="), ('þ', "+"), ('\u{f02d}', "−")];

pub(super) fn repair(blocks: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(blocks.len());
    for block in blocks {
        let block = mend_hyphens(&math_pi(&block));
        if let Some(previous) = out.last_mut()
            && continues(previous, &block)
        {
            if previous.ends_with('-') && block.starts_with(char::is_lowercase) {
                previous.pop();
            } else {
                previous.push(' ');
            }
            previous.push_str(&block);
            continue;
        }
        out.push(block);
    }
    out
}

/// Whether `next` continues the paragraph `previous` (cut mid-sentence).
fn continues(previous: &str, next: &str) -> bool {
    let is_text = |b: &str| {
        !is_heading(b) && !is_table(b) && !b.starts_with("- ") && !b.starts_with("<!--") && !b.starts_with("```")
    };
    if !is_text(previous) || !is_text(next) || is_caption(previous) || is_caption(next) {
        return false;
    }
    let end = previous.trim_end_matches(['*', '_', '"', '\'', '”', '’', ')', ']']);
    let finished = end.ends_with(['.', '!', '?', ':']);
    let starts_lower = next.starts_with(char::is_lowercase);
    let starts_continuation = next.starts_with(|c: char| c.is_ascii_digit() || c == '(') && end.ends_with([',', ';']);
    !finished && (starts_lower || starts_continuation || previous.ends_with('-'))
}

fn is_caption(block: &str) -> bool {
    caption_label(block).is_some()
}

/// `deter- mined` → `determined`; `self- attention` → `self-attention`; `pre- and post-` kept.
fn mend_hyphens(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("- ") {
        let (before, after) = (&rest[..i], &rest[i + 2..]);
        let left: String =
            before.chars().rev().take_while(|c| c.is_alphabetic()).collect::<Vec<_>>().into_iter().rev().collect();
        let right: String = after.chars().take_while(|c| c.is_alphabetic()).collect();
        out.push_str(before);
        let broken = left.chars().count() >= 2
            && left.chars().all(char::is_lowercase)
            && right.chars().next().is_some_and(char::is_lowercase)
            && !CONJUNCTIONS.contains(&right.as_str());
        if broken && COMPOUND_FIRST.contains(&left.as_str()) {
            out.push('-');
        } else if !broken {
            out.push_str("- ");
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// MathPi characters replaced, in blocks that have at least two of them (one alone may be a
/// real letter, as in a name).
fn math_pi(block: &str) -> String {
    let count = block.chars().filter(|c| MATH_PI.iter().any(|(m, _)| m == c)).count();
    if count < 2 {
        return block.to_string();
    }
    block
        .chars()
        .map(|c| MATH_PI.iter().find(|(m, _)| *m == c).map_or_else(|| c.to_string(), |(_, s)| s.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(blocks: &[&str]) -> Vec<String> {
        repair(blocks.iter().map(|b| b.to_string()).collect())
    }

    #[test]
    fn joins_cut_paragraphs() {
        assert_eq!(
            run(&["decrease (Stephens and Taylor, 1972; Bigland-Ritchie,", "1981) or remain unchanged."]),
            ["decrease (Stephens and Taylor, 1972; Bigland-Ritchie, 1981) or remain unchanged."]
        );
        assert_eq!(run(&["For many", "years there has been interest."]), ["For many years there has been interest."]);
        assert_eq!(run(&["the deter-", "mination of fatigue."]), ["the determination of fatigue."]);
        // Finished sentences, headings and captions are not joined.
        assert_eq!(run(&["A sentence.", "another starts lowercase"]).len(), 2);
        assert_eq!(run(&["## Methods", "the text"]).len(), 2);
        assert_eq!(run(&["Figure 2: The device", "with labels"]).len(), 2);
    }

    #[test]
    fn hyphens_and_math_font() {
        assert_eq!(
            mend_hyphens("a modifica- tion of the self- attention in pre- and post- tests"),
            "a modification of the self-attention in pre- and post-tests"
        );
        assert_eq!(math_pi("MFPVðm=sÞ¼0:7 v þ 0:01 ð2Þ"), "MFPV(m=s)=0:7 v + 0:01 (2)");
        assert_eq!(math_pi("Þórsson wrote"), "Þórsson wrote");
    }
}
