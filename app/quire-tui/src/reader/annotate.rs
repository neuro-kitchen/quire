//! Highlighting in the reader. The unit selected is a sentence: a terminal has no mouse
//! selection to rely on, and sentences are what people quote. A [`Selection`] is a run of
//! sentences in one block; a saved highlight keeps its quote and the words before it, and
//! [`locate`] finds those words again in the text (which may have been extracted anew or come
//! from another source since).

use std::ops::Range;

use super::document::{Block, Kind};

/// Words, in a block, that a highlight or selection covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Marked {
    pub block: usize,
    pub words: Range<usize>,
}

/// Words ending with a period that do not end a sentence.
const ABBREVIATIONS: &[&str] = &[
    "al.", "e.g.", "i.e.", "fig.", "figs.", "eq.", "eqs.", "vs.", "cf.", "approx.", "dr.", "mr.", "ms.", "no.", "vol.",
    "pp.", "p.", "ca.", "resp.", "ref.", "refs.", "sec.", "et.", "etc.)",
];
/// Words of context kept before a quote.
const PREFIX_WORDS: usize = 6;

/// The sentences of `block`, as ranges of word indices. Headings, table rows and equations are
/// one sentence.
pub fn sentences(block: &Block) -> Vec<Range<usize>> {
    let count = block.words.len();
    if count == 0 {
        return Vec::new();
    }
    if !matches!(block.kind, Kind::Paragraph | Kind::ListItem) {
        // The whole block, as one sentence.
        return std::iter::once(0..count).collect();
    }
    let mut out = Vec::new();
    let mut start = 0;
    for i in 0..count {
        let next = block.words.get(i + 1).map(|w| w.text.as_str());
        if i + 1 == count || (ends_sentence(&block.words[i].text) && next.is_some_and(starts_sentence)) {
            out.push(start..i + 1);
            start = i + 1;
        }
    }
    out
}

fn ends_sentence(word: &str) -> bool {
    let bare = word.trim_end_matches(['"', '\'', ')', ']', '”', '’']);
    if !bare.ends_with(['.', '!', '?']) {
        return false;
    }
    let lower = bare.to_lowercase();
    let initial = bare.chars().count() == 2 && bare.chars().next().is_some_and(char::is_uppercase);
    !initial && !ABBREVIATIONS.contains(&lower.as_str())
}

fn starts_sentence(word: &str) -> bool {
    word.trim_start_matches(['"', '\'', '(', '[', '“', '‘'])
        .chars()
        .next()
        .is_some_and(|c| c.is_uppercase() || c.is_ascii_digit())
}

/// A run of sentences (`first..=last`) in one block of the paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Selection {
    pub block: usize,
    pub first: usize,
    pub last: usize,
}

impl Selection {
    /// The first sentence of the first block from `block` on that has one.
    pub fn at(blocks: &[Block], block: usize) -> Option<Self> {
        (block..blocks.len()).find(|b| !sentences(&blocks[*b]).is_empty()).map(|b| Self { block: b, first: 0, last: 0 })
    }

    /// The words it covers.
    pub fn marked(&self, blocks: &[Block]) -> Marked {
        let sentences = sentences(&blocks[self.block]);
        let start = sentences.get(self.first).map_or(0, |s| s.start);
        let end = sentences.get(self.last).map_or(start, |s| s.end);
        Marked { block: self.block, words: start..end }
    }

    /// The next sentence (the next block's first after the last one), as a one-sentence selection.
    pub fn next(&self, blocks: &[Block]) -> Self {
        if self.last + 1 < sentences(&blocks[self.block]).len() {
            return Self { block: self.block, first: self.last + 1, last: self.last + 1 };
        }
        Self::at(blocks, self.block + 1).unwrap_or(*self)
    }

    /// The previous sentence (the previous block's last before the first one).
    pub fn previous(&self, blocks: &[Block]) -> Self {
        if self.first > 0 {
            return Self { block: self.block, first: self.first - 1, last: self.first - 1 };
        }
        let before = (0..self.block).rev().find(|b| !sentences(&blocks[*b]).is_empty());
        before.map_or(*self, |b| {
            let last = sentences(&blocks[b]).len() - 1;
            Self { block: b, first: last, last }
        })
    }

    /// One more sentence at the end (within the block).
    pub fn extend(&self, blocks: &[Block]) -> Self {
        let count = sentences(&blocks[self.block]).len();
        Self { last: (self.last + 1).min(count.saturating_sub(1)), ..*self }
    }

    /// One sentence fewer at the end.
    pub fn shrink(&self) -> Self {
        Self { last: self.last.saturating_sub(1).max(self.first), ..*self }
    }
}

/// The quote of `marked` and the words before it (for [`locate`]).
pub(crate) fn quote_of(blocks: &[Block], marked: &Marked) -> (String, String) {
    let words = &blocks[marked.block].words;
    let quote = words[marked.words.clone()].iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ");
    let from = marked.words.start.saturating_sub(PREFIX_WORDS);
    let prefix = words[from..marked.words.start].iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ");
    (quote, prefix)
}

/// Where `quote` is in `blocks` (the occurrence after `prefix` when it repeats), to whole words.
pub fn locate(blocks: &[Block], quote: &str, prefix: &str) -> Option<Marked> {
    let quote = squash(quote);
    let prefix = squash(prefix);
    if quote.is_empty() {
        return None;
    }
    let mut first = None;
    for (index, block) in blocks.iter().enumerate() {
        let text = block.text();
        let mut from = 0;
        while let Some(found) = text[from..].find(&quote) {
            let start = from + found;
            let Some(marked) = words_at(block, start, start + quote.len()) else { break };
            let marked = Marked { block: index, words: marked };
            if prefix.is_empty() || squash(&text[..start]).ends_with(&prefix) {
                return Some(marked);
            }
            first.get_or_insert(marked);
            from = start + quote.len();
        }
    }
    first
}

/// The word range covering byte offsets `start..end` of the block's text (words joined by one
/// space).
fn words_at(block: &Block, start: usize, end: usize) -> Option<Range<usize>> {
    let mut at = 0;
    let (mut first, mut last) = (None, None);
    for (i, word) in block.words.iter().enumerate() {
        let word_end = at + word.text.len();
        if first.is_none() && word_end > start {
            first = Some(i);
        }
        if at < end {
            last = Some(i);
        }
        at = word_end + 1;
    }
    Some(first?..last? + 1)
}

/// Whitespace collapsed to single spaces.
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::document::parse;

    fn paper() -> Vec<Block> {
        parse(
            "## 1 Introduction\n\nEMG is noisy (Smith et al. 2020). Filters help, e.g. a notch at 60 Hz. See Fig. 2 for details.\n\n\
             A second paragraph. It ends here.",
            false,
        )
    }

    #[test]
    fn sentences_respect_abbreviations() {
        let blocks = paper();
        let texts: Vec<String> = sentences(&blocks[1])
            .into_iter()
            .map(|r| blocks[1].words[r].iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" "))
            .collect();
        assert_eq!(
            texts,
            ["EMG is noisy (Smith et al. 2020).", "Filters help, e.g. a notch at 60 Hz.", "See Fig. 2 for details."]
        );
        assert_eq!(sentences(&blocks[0]), std::iter::once(0..2).collect::<Vec<_>>(), "a heading is one sentence");
    }

    #[test]
    fn moving_extending_and_locating() {
        let blocks = paper();
        let start = Selection::at(&blocks, 0).unwrap();
        assert_eq!(start.block, 0, "the heading");
        let s = start.next(&blocks).next(&blocks).extend(&blocks);
        assert_eq!((s.block, s.first, s.last), (1, 1, 2));
        let (quote, prefix) = quote_of(&blocks, &s.marked(&blocks));
        assert!(quote.starts_with("Filters help") && quote.ends_with("details."), "{quote}");
        assert_eq!(prefix, "is noisy (Smith et al. 2020).");
        assert_eq!(locate(&blocks, &quote, &prefix), Some(s.marked(&blocks)));
        let back = Selection { block: 2, first: 0, last: 0 }.previous(&blocks);
        assert_eq!((back.block, back.first), (1, 2));
        assert_eq!(locate(&blocks, "not in the text", ""), None);
    }

    #[test]
    fn repeated_quotes_use_the_prefix() {
        let blocks = parse("It works. It works.\n\nBut then: It works.", false);
        let found = locate(&blocks, "It works.", "But then:").unwrap();
        assert_eq!(found, Marked { block: 1, words: 2..4 });
        assert_eq!(locate(&blocks, "It works.", "").unwrap(), Marked { block: 0, words: 0..2 });
    }
}
