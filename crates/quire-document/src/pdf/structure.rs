//! The text layer as reader Markdown ([`crate::paper_markdown`]): the Text view's fallback when
//! pdf-inspector reads too little. Pages are read in the layer's order; their lines become
//! paragraphs and headings:
//!
//! - a line's size is the median height of its glyphs (each glyph box is one em tall), so a tall
//!   symbol or a superscript does not make it larger; the body size is the median line size;
//! - a short line at least [`HEADING_SIZE`] times the body is a heading, unless it has two commas
//!   or more (an author list); levels by size (largest first, at most three); heading lines of one
//!   size in a row are one heading (a title over two lines);
//! - lines join into a paragraph until a gap of more than [`PARAGRAPH_GAP`] lines, a heading, or a
//!   line that starts further left than the paragraph (a new column);
//! - a word hyphenated at a line's end joins the next line's first word when that starts lower case.
//!
//! [`super::tidy`] runs after it, as after pdf-inspector: running heads and stamps out, numbered and
//! run-in headings found.

use super::types::{Line, PageText};

/// How much larger than the body a heading's line is.
const HEADING_SIZE: f32 = 1.15;
/// Most words in a heading.
const HEADING_WORDS: usize = 16;
/// Vertical gap, in line heights, that ends a paragraph.
const PARAGRAPH_GAP: f32 = 0.5;

/// The pages' text as reader Markdown blocks, separated by blank lines.
pub(crate) fn markdown(pages: &[PageText]) -> String {
    let lines: Vec<(&Line, f32)> =
        pages.iter().flat_map(|p| p.lines.iter().map(move |l| (l, line_size(p, l)))).collect();
    let body = median(lines.iter().map(|(_, size)| *size));
    let rounded = |size: f32| (size * 2.0).round() / 2.0;
    let mut sizes: Vec<f32> =
        lines.iter().filter(|(l, size)| is_heading(l, *size, body)).map(|(_, size)| rounded(*size)).collect();
    sizes.sort_by(|a, b| b.total_cmp(a));
    sizes.dedup();
    let level = |size: f32| 2 + sizes.iter().position(|s| *s == rounded(size)).unwrap_or(0).min(2);

    let mut blocks: Vec<String> = Vec::new();
    let mut paragraph = String::new();
    let mut last: Option<&Line> = None;
    // The last block if it is a heading, with its size, to continue a heading over two lines.
    let mut heading: Option<f32> = None;
    let flush = |paragraph: &mut String, blocks: &mut Vec<String>| {
        if !paragraph.trim().is_empty() {
            blocks.push(std::mem::take(paragraph));
        }
        paragraph.clear();
    };
    for (line, size) in &lines {
        let text = line.text.trim();
        if text.is_empty() {
            continue;
        }
        if is_heading(line, *size, body) {
            flush(&mut paragraph, &mut blocks);
            match (heading, blocks.last_mut()) {
                (Some(previous), Some(block)) if rounded(previous) == rounded(*size) => {
                    block.push(' ');
                    block.push_str(text);
                }
                _ => blocks.push(format!("{} {text}", "#".repeat(level(*size)))),
            }
            heading = Some(*size);
            last = None;
            continue;
        }
        heading = None;
        let continues = last.is_some_and(|previous| {
            let gap = line.rect.top - previous.rect.bottom;
            gap <= PARAGRAPH_GAP * previous.rect.height().max(line.rect.height())
                && gap > -previous.rect.height()
                && line.rect.left >= previous.rect.left - previous.rect.height()
        });
        if !continues {
            flush(&mut paragraph, &mut blocks);
        }
        join(&mut paragraph, text);
        last = Some(line);
    }
    flush(&mut paragraph, &mut blocks);
    let mut out = blocks.join("\n\n");
    out.push('\n');
    out
}

fn is_heading(line: &Line, size: f32, body: f32) -> bool {
    size >= HEADING_SIZE * body
        && line.word_end - line.word_start <= HEADING_WORDS
        && line.text.chars().any(char::is_alphabetic)
        && line.text.matches(',').count() < 2
}

/// The median height of the line's glyphs (separators left out).
fn line_size(page: &PageText, line: &Line) -> f32 {
    let (Some(first), Some(last)) = (page.words.get(line.word_start), page.words.get(line.word_end.saturating_sub(1)))
    else {
        return line.rect.height();
    };
    median((first.glyph_start..last.glyph_end).filter(|&i| !page.is_separator(i)).map(|i| page.glyphs[i].rect.height()))
}

/// `text` after `paragraph`: a space between them, or nothing after a hyphen whose next word
/// starts lower case (`inter-` + `neurons`).
fn join(paragraph: &mut String, text: &str) {
    if paragraph.is_empty() {
        paragraph.push_str(text);
    } else if paragraph.ends_with('-') && text.starts_with(|c: char| c.is_lowercase()) {
        paragraph.pop();
        paragraph.push_str(text);
    } else {
        paragraph.push(' ');
        paragraph.push_str(text);
    }
}

fn median(values: impl Iterator<Item = f32>) -> f32 {
    let mut values: Vec<f32> = values.filter(|v| v.is_finite() && *v > 0.0).collect();
    values.sort_by(f32::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or(10.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::layout::page_text;
    use crate::pdf::types::{Glyph, PageRect};

    /// Lines of (`text`, font size, top); glyphs 0.5 em wide, words 0.3 em apart.
    fn page(lines: &[(&str, f32, f32)]) -> PageText {
        let mut glyphs = Vec::new();
        for (text, size, top) in lines {
            let mut x = 72.0;
            for word in text.split(' ') {
                for c in word.chars() {
                    glyphs.push(Glyph { text: c.into(), rect: PageRect::new(x, *top, x + size * 0.5, top + size) });
                    x += size * 0.5;
                }
                x += size * 0.3;
            }
        }
        page_text(glyphs)
    }

    #[test]
    fn headings_paragraphs_and_hyphens() {
        let text = page(&[
            ("Place cells in rats", 18.0, 40.0),
            ("Introduction", 13.0, 80.0),
            ("Neurons fire when the", 10.0, 100.0),
            ("animal is in one place, inter-", 10.0, 112.0),
            ("neurons do not.", 10.0, 124.0),
            ("A second paragraph starts here.", 10.0, 150.0),
        ]);
        assert_eq!(
            markdown(&[text]),
            "## Place cells in rats\n\n### Introduction\n\nNeurons fire when the animal is in one place, interneurons do not.\n\nA second paragraph starts here.\n"
        );
    }

    /// A degree sign or superscript does not make a body line a heading; an author list and a title
    /// over two lines are what they are.
    #[test]
    fn tall_glyphs_author_lists_and_two_line_titles() {
        let mut text = page(&[
            ("Non-stationarity of EMG activity in", 14.0, 40.0),
            ("rat diaphragm muscle", 14.0, 58.0),
            ("Yasin Seven, Carlos Mantilla, Gary Sieck", 12.0, 80.0),
            ("kept at 37 C with a heating pad.", 10.0, 110.0),
            ("Body text goes on here as usual.", 10.0, 122.0),
            ("And on, so that the body is ten.", 10.0, 134.0),
            ("Then one more line of the body.", 10.0, 146.0),
        ]);
        // A tall glyph in the fourth line (a degree sign drawn large).
        let degree = text.lines[3].word_start + 3;
        let glyph = text.words[degree].glyph_start;
        text.glyphs[glyph].rect.top -= 8.0;
        text.lines[3].rect.top -= 8.0; // its line's box grows with it, as the layout makes it
        assert_eq!(
            markdown(&[text]),
            "## Non-stationarity of EMG activity in rat diaphragm muscle\n\nYasin Seven, Carlos Mantilla, Gary Sieck\n\nkept at 37 C with a heating pad. Body text goes on here as usual. And on, so that the body is ten. Then one more line of the body.\n"
        );
    }

    #[test]
    fn a_paragraph_runs_on_across_pages() {
        let first = page(&[("The text goes on", 10.0, 700.0)]);
        let second = page(&[("over the page.", 10.0, 72.0)]);
        // Different pages: a new paragraph (tidy joins paragraphs cut by a page break).
        assert_eq!(markdown(&[first, second]), "The text goes on\n\nover the page.\n");
    }
}
