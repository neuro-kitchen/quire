//! Where quotes appear on the pages, for drawing highlights made in the text view.
//!
//! Words are compared by their letters and digits only (case folded), so punctuation and quote
//! marks never block a match. A word hyphenated at a line's end is joined to the next line's
//! first word (`inter-` + `neuron` reads `interneuron`).

use std::path::Path;
use std::sync::Arc;

use super::cache::DocumentCache;
use super::types::{PageRect, PageText, QuoteMatch};
use crate::error::DocumentError;

/// Words of the fallback search, for quotes split across a page or a figure.
const FALLBACK_WORDS: usize = 8;

/// Locates where `quotes` appear across the document pages (first match per quote).
pub(crate) fn locate_quotes(
    cache: &DocumentCache,
    path: &Path,
    quotes: &[String],
) -> Result<Vec<QuoteMatch>, DocumentError> {
    let pages = cache.get_pdf(path)?.pages().len() as u32;
    locate_in(pages, |page| cache.get_page_text(path, page), quotes)
}

/// [`locate_quotes`] over `pages` pages whose text `page_text` gives: each quote's first match,
/// trying the whole quote on every page before its first [`FALLBACK_WORDS`] words.
pub(crate) fn locate_in(
    pages: u32,
    page_text: impl Fn(u32) -> Result<Arc<PageText>, DocumentError>,
    quotes: &[String],
) -> Result<Vec<QuoteMatch>, DocumentError> {
    let mut tokens: Vec<Option<Vec<Token>>> = vec![None; pages as usize];
    let mut matches = Vec::new();
    'quotes: for (i, quote) in quotes.iter().enumerate() {
        let words: Vec<String> = quote.split_whitespace().map(normalize).filter(|w| !w.is_empty()).collect();
        let mut attempts = vec![words.clone()];
        if words.len() > FALLBACK_WORDS {
            attempts.push(words[..FALLBACK_WORDS].to_vec());
        }
        for attempt in attempts.iter().filter(|a| !a.is_empty()) {
            for page in 0..pages {
                let text = page_text(page)?;
                let page_tokens = tokens[page as usize].get_or_insert_with(|| tokenize(&text));
                if let Some(rects) = find(&text, page_tokens, attempt) {
                    matches.push(QuoteMatch { quote: i as u32, page, rects });
                    continue 'quotes;
                }
            }
        }
    }
    Ok(matches)
}

/// A word of the page as compared: its normalized text and its glyph range.
#[derive(Debug, Clone)]
struct Token {
    text: String,
    glyph_start: usize,
    glyph_end: usize,
}

/// The page's words as tokens, hyphenated line ends joined to the next word.
fn tokenize(page: &PageText) -> Vec<Token> {
    let line_ends: Vec<usize> = page.lines.iter().map(|l| l.word_end).collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut joined = false;
    for (i, word) in page.words.iter().enumerate() {
        let text = normalize(&word.text);
        if joined && let Some(last) = tokens.last_mut() {
            last.text.push_str(&text);
            last.glyph_end = word.glyph_end;
        } else if !text.is_empty() {
            tokens.push(Token { text, glyph_start: word.glyph_start, glyph_end: word.glyph_end });
        }
        joined = word.text.ends_with(['-', '\u{2010}', '\u{00ad}']) && line_ends.contains(&(i + 1));
    }
    tokens
}

/// The rectangles of the first place `words` appear in `tokens`.
fn find(page: &PageText, tokens: &[Token], words: &[String]) -> Option<Vec<PageRect>> {
    if words.len() > tokens.len() {
        return None;
    }
    let start = tokens.windows(words.len()).find(|window| window.iter().zip(words).all(|(t, w)| t.text == *w))?;
    let (_, rects) = page.range_rects(start[0].glyph_start, start[words.len() - 1].glyph_end);
    (!rects.is_empty()).then_some(rects)
}

fn normalize(word: &str) -> String {
    word.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::types::Glyph;

    /// A page of `lines` laid out by the text layer (6 pt glyphs, 3 pt word gaps).
    fn page(lines: &[&str]) -> Arc<PageText> {
        let mut glyphs = Vec::new();
        for (n, line) in lines.iter().enumerate() {
            let (mut x, top) = (72.0, 72.0 + n as f32 * 14.0);
            for word in line.split(' ') {
                for c in word.chars() {
                    glyphs.push(Glyph { text: c.into(), rect: PageRect::new(x, top, x + 6.0, top + 10.0) });
                    x += 6.0;
                }
                x += 3.0;
            }
        }
        Arc::new(crate::pdf::layout::page_text(glyphs))
    }

    fn locate(pages: &[Arc<PageText>], quote: &str) -> Option<u32> {
        let found = locate_in(pages.len() as u32, |p| Ok(pages[p as usize].clone()), &[quote.to_string()]).unwrap();
        found.first().map(|m| m.page)
    }

    #[test]
    fn a_word_hyphenated_at_a_line_end_matches() {
        let pages = [page(&["recorded from inter-", "neurons of the spinal cord"])];
        assert_eq!(locate(&pages, "recorded from interneurons of the"), Some(0));
    }

    #[test]
    fn punctuation_does_not_block_a_match() {
        let pages = [page(&["the cell (fired) “twice”, then rested."])];
        assert_eq!(locate(&pages, "fired twice then"), Some(0));
    }

    /// The whole quote on a later page wins over its first words on an earlier one.
    #[test]
    fn the_whole_quote_is_tried_on_every_page_before_its_first_words() {
        let quote = "one two three four five six seven eight nine ten";
        let pages = [page(&["one two three four five six seven eight"]), page(&[quote])];
        assert_eq!(locate(&pages, quote), Some(1));
        let only_start = [page(&["one two three four five six seven eight"]), page(&["nothing"])];
        assert_eq!(locate(&only_start, quote), Some(0), "the fallback still finds it");
    }
}
