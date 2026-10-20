//! From positioned glyphs to the page's text in reading order: words, lines, columns.
//!
//! 1. Glyphs on the same baseline form rows; a row splits into words at drawn spaces and at gaps
//!    wider than [`WORD_GAP`] (most publishers position words without drawing a space, and some
//!    that draw them set words closer than that).
//! 2. Words are ordered by recursive XY-cut: a region splits at its widest column gutter (a vertical
//!    gap of [`GUTTER`] with [`GUTTER_LINES`] rows on each side), else at its widest horizontal
//!    gap, until no gap is left; the leaves are lines, read top to bottom.
//! 3. The text gets separators as glyphs of their own: a space between words, a newline after
//!    each line. They are whitespace with an empty rectangle, so hit-testing and selection
//!    rectangles skip them while selected text reads naturally.
//!
//! Distances are in ems: the median glyph height on the page (a glyph box is one em tall). Text that
//! does not run left to right (rotated pages, axis labels) is laid out the same way in a frame
//! turned to its direction ([`page_text_directed`]).

use super::types::{Glyph, GlyphBox, Line, PageRect, PageText, WordBox};

/// Gap between two glyphs of a row that starts a new word.
const WORD_GAP: f32 = 0.15;
/// Narrowest vertical gap taken as a column gutter (0.8 em measured best on the corpus: 1.2 missed
/// the narrow gutters of older scans, 0.5 split non-columns; `.knowledge/pdf-engine.md`).
const GUTTER: f32 = 0.8;
/// Rows a gutter needs on each side (a centred two-line title has coinciding word gaps).
const GUTTER_LINES: usize = 3;
/// Most a glyph's centre may sit from its row's centre, in the taller one's heights.
const SAME_ROW: f32 = 0.35;

/// A word before ordering: its glyphs (sorted left to right) and the row it came from.
struct Draft {
    glyphs: Vec<Glyph>,
    rect: PageRect,
    row: usize,
}

/// Which way a glyph's text runs on the page (y grows downwards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Direction {
    Right,
    Down,
    Left,
    Up,
    /// More than [`SLANT`] off every axis: watermarks and stamps ("Accepted Manuscript" across the
    /// page), laid out apart so their large boxes never join lines of the text.
    Slanted,
}

/// Sine of the angle off an axis past which a glyph is [`Direction::Slanted`] (15°).
const SLANT: f64 = 0.2588;

impl Direction {
    /// From a glyph's advance on the page (`dx`, `dy`): the nearest quarter turn, unless slanted.
    pub(crate) fn of(dx: f64, dy: f64) -> Self {
        let length = dx.hypot(dy);
        if length > 0.0 && dx.abs().min(dy.abs()) / length > SLANT {
            return Self::Slanted;
        }
        match (dx.abs() >= dy.abs(), dx >= 0.0, dy > 0.0) {
            (true, true, _) => Self::Right,
            (true, false, _) => Self::Left,
            (false, _, true) => Self::Down,
            (false, _, false) => Self::Up,
        }
    }

    /// A page point in this direction's frame, where its text runs left to right and its lines
    /// follow downwards (text running down has its next line to the left: below, in its frame).
    fn to_frame(self, (x, y): (f32, f32)) -> (f32, f32) {
        match self {
            Self::Right | Self::Slanted => (x, y),
            Self::Down => (y, -x),
            Self::Left => (-x, -y),
            Self::Up => (-y, x),
        }
    }

    fn to_page(self, (x, y): (f32, f32)) -> (f32, f32) {
        match self {
            Self::Right | Self::Slanted => (x, y),
            Self::Down => (-y, x),
            Self::Left => (-x, -y),
            Self::Up => (y, -x),
        }
    }

    fn map(rect: PageRect, point: impl Fn((f32, f32)) -> (f32, f32)) -> PageRect {
        let ((l, t), (r, b)) = (point((rect.left, rect.top)), point((rect.right, rect.bottom)));
        PageRect::new(l, t, r, b)
    }
}

/// The page's text in reading order, glyphs of each writing direction laid out in their own frame:
/// horizontal text first, then text running down, left and up (rotated pages, axis labels), then
/// slanted text (watermarks).
pub(crate) fn page_text_directed(glyphs: Vec<(Glyph, Direction)>) -> PageText {
    let mut groups: std::collections::BTreeMap<Direction, Vec<Glyph>> = Default::default();
    for (mut glyph, direction) in glyphs {
        glyph.rect = Direction::map(glyph.rect, |p| direction.to_frame(p));
        groups.entry(direction).or_default().push(glyph);
    }
    let mut page = PageText::default();
    for (direction, glyphs) in groups {
        let mut part = page_text(glyphs);
        let back = |rect: PageRect| Direction::map(rect, |p| direction.to_page(p));
        part.glyphs.iter_mut().for_each(|g| g.rect = back(g.rect));
        part.words.iter_mut().for_each(|w| w.rect = back(w.rect));
        part.lines.iter_mut().for_each(|l| l.rect = back(l.rect));
        append(&mut page, part);
    }
    page
}

/// `part` after `page`, its indices and offsets moved past what `page` holds.
fn append(page: &mut PageText, part: PageText) {
    let (text, glyphs, words) = (page.text.len() as u32, page.glyphs.len(), page.words.len());
    page.text.push_str(&part.text);
    page.glyphs.extend(part.glyphs.into_iter().map(|g| GlyphBox { end: g.end + text, ..g }));
    page.words.extend(part.words.into_iter().map(|w| WordBox {
        glyph_start: w.glyph_start + glyphs,
        glyph_end: w.glyph_end + glyphs,
        ..w
    }));
    page.lines.extend(part.lines.into_iter().map(|l| Line {
        word_start: l.word_start + words,
        word_end: l.word_end + words,
        ..l
    }));
}

/// The text of horizontal glyphs (`glyphs`, in any order) in reading order.
pub fn page_text(glyphs: Vec<Glyph>) -> PageText {
    let glyphs = dedupe(glyphs.into_iter().filter(|g| g.rect.width().is_finite()).collect());
    if glyphs.iter().all(is_separator) {
        return PageText::default();
    }
    let em = median(glyphs.iter().filter(|g| !is_separator(g)).map(|g| g.rect.height())).max(1.0);
    let drafts = words(rows(glyphs), em);
    let mut order = Vec::new();
    xy_cut((0..drafts.len()).collect(), &drafts, em, &mut order);
    assemble(drafts, order, em)
}

/// Whether `glyph` is a space or a separator (whitespace).
pub(crate) fn is_separator(glyph: &Glyph) -> bool {
    glyph.text.chars().all(char::is_whitespace)
}

/// Drops a glyph drawn again on top of an identical one (bold faked by overprinting).
fn dedupe(mut glyphs: Vec<Glyph>) -> Vec<Glyph> {
    // Same text first, so a copy lands next to its original whatever the offset's direction.
    glyphs.sort_by(|a, b| {
        a.text.cmp(&b.text).then(a.rect.left.total_cmp(&b.rect.left)).then(a.rect.top.total_cmp(&b.rect.top))
    });
    glyphs.dedup_by(|b, a| {
        a.text == b.text && (a.rect.left - b.rect.left).abs() < 0.5 && (a.rect.top - b.rect.top).abs() < 0.5
    });
    glyphs
}

/// Glyphs grouped into rows (shared baseline), each sorted left to right.
fn rows(mut glyphs: Vec<Glyph>) -> Vec<Vec<Glyph>> {
    let center = |g: &Glyph| (g.rect.top + g.rect.bottom) / 2.0;
    glyphs.sort_by(|a, b| center(a).total_cmp(&center(b)).then(a.rect.left.total_cmp(&b.rect.left)));
    let mut rows: Vec<Vec<Glyph>> = Vec::new();
    let (mut row_center, mut row_height) = (f32::NEG_INFINITY, 0.0f32);
    for glyph in glyphs {
        let (c, h) = (center(&glyph), glyph.rect.height());
        match rows.last_mut() {
            Some(row) if (c - row_center).abs() <= SAME_ROW * h.max(row_height) => {
                row.push(glyph);
                // Running mean, so the row follows its glyphs rather than its first one.
                row_center += (c - row_center) / row.len() as f32;
                row_height = row_height.max(h);
            }
            _ => {
                rows.push(vec![glyph]);
                (row_center, row_height) = (c, h);
            }
        }
    }
    for row in &mut rows {
        row.sort_by(|a, b| a.rect.left.total_cmp(&b.rect.left));
    }
    rows
}

/// Each row split into words at drawn spaces (dropped) and at gaps wider than [`WORD_GAP`] ems.
fn words(rows: Vec<Vec<Glyph>>, em: f32) -> Vec<Draft> {
    let mut drafts: Vec<Draft> = Vec::new();
    for (row, glyphs) in rows.into_iter().enumerate() {
        let mut current: Option<Draft> = None;
        for glyph in glyphs {
            if is_separator(&glyph) {
                drafts.extend(current.take());
                continue;
            }
            if let Some(draft) = current.as_mut()
                && glyph.rect.left - draft.rect.right <= WORD_GAP * em.max(glyph.rect.height())
            {
                draft.rect = draft.rect.union(&glyph.rect);
                draft.glyphs.push(glyph);
                continue;
            }
            drafts.extend(current.take());
            current = Some(Draft { rect: glyph.rect, glyphs: vec![glyph], row });
        }
        drafts.extend(current);
    }
    drafts
}

/// Appends `region`'s words to `order` in reading order, as lines (each a list of word indices).
fn xy_cut(region: Vec<usize>, drafts: &[Draft], em: f32, order: &mut Vec<Vec<usize>>) {
    if region.len() > 1 {
        if let Some(at) = widest_gap(&region, drafts, |r| (r.left, r.right), GUTTER * em) {
            let (left, right): (Vec<usize>, Vec<usize>) = region.iter().partition(|&&i| drafts[i].rect.right <= at);
            if rows_in(&left, drafts) >= GUTTER_LINES && rows_in(&right, drafts) >= GUTTER_LINES {
                xy_cut(left, drafts, em, order);
                xy_cut(right, drafts, em, order);
                return;
            }
        }
        if let Some(at) = widest_gap(&region, drafts, |r| (r.top, r.bottom), 0.0) {
            let (above, below): (Vec<usize>, Vec<usize>) = region.iter().partition(|&&i| drafts[i].rect.bottom <= at);
            xy_cut(above, drafts, em, order);
            xy_cut(below, drafts, em, order);
            return;
        }
    }
    order.extend(lines(region, drafts));
}

/// The middle of the widest gap, at least `min` wide, between the `span`s of `region`'s words.
fn widest_gap(region: &[usize], drafts: &[Draft], span: impl Fn(&PageRect) -> (f32, f32), min: f32) -> Option<f32> {
    let mut spans: Vec<(f32, f32)> = region.iter().map(|&i| span(&drafts[i].rect)).collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut best: Option<(f32, f32)> = None;
    let mut reach = spans[0].1;
    for &(start, end) in &spans[1..] {
        let gap = start - reach;
        if gap > min && best.is_none_or(|(width, _)| gap > width) {
            best = Some((gap, reach + gap / 2.0));
        }
        reach = reach.max(end);
    }
    best.map(|(_, at)| at)
}

fn rows_in(region: &[usize], drafts: &[Draft]) -> usize {
    let mut rows: Vec<usize> = region.iter().map(|&i| drafts[i].row).collect();
    rows.sort_unstable();
    rows.dedup();
    rows.len()
}

/// A leaf region's words as lines: grouped by row overlap, top to bottom, left to right.
fn lines(mut region: Vec<usize>, drafts: &[Draft]) -> Vec<Vec<usize>> {
    region.sort_by(|&a, &b| drafts[a].rect.top.total_cmp(&drafts[b].rect.top).then(a.cmp(&b)));
    let mut lines: Vec<(PageRect, Vec<usize>)> = Vec::new();
    for i in region {
        let rect = drafts[i].rect;
        match lines.last_mut() {
            Some((line, words)) if overlap(line, &rect) >= 0.5 * rect.height().min(line.height()) => {
                *line = line.union(&rect);
                words.push(i);
            }
            _ => lines.push((rect, vec![i])),
        }
    }
    lines
        .into_iter()
        .map(|(_, mut words)| {
            words.sort_by(|&a, &b| drafts[a].rect.left.total_cmp(&drafts[b].rect.left).then(a.cmp(&b)));
            words
        })
        .collect()
}

fn overlap(a: &PageRect, b: &PageRect) -> f32 {
    a.bottom.min(b.bottom) - a.top.max(b.top)
}

/// The words of `order`'s lines with their separators, as one [`PageText`].
fn assemble(drafts: Vec<Draft>, order: Vec<Vec<usize>>, em: f32) -> PageText {
    let mut drafts: Vec<Option<Draft>> = drafts.into_iter().map(Some).collect();
    let mut page = PageText::default();
    for line in order {
        let word_start = page.words.len();
        let mut line_rect: Option<PageRect> = None;
        let mut line_text = String::new();
        for (n, i) in line.into_iter().enumerate() {
            let Some(draft) = drafts[i].take() else { continue };
            if n > 0 {
                let previous = page.words.last().map_or(draft.rect, |w| w.rect);
                separator(
                    &mut page,
                    " ",
                    PageRect::new(previous.right, draft.rect.top, previous.right, draft.rect.bottom),
                );
                line_text.push(' ');
            }
            let glyph_start = page.glyphs.len();
            let text: String = draft.glyphs.iter().map(|g| g.text.as_str()).collect();
            line_text.push_str(&text);
            for glyph in draft.glyphs {
                page.text.push_str(&glyph.text);
                page.glyphs.push(GlyphBox { rect: glyph.rect, end: page.text.len() as u32 });
            }
            page.words.push(WordBox { text, rect: draft.rect, glyph_start, glyph_end: page.glyphs.len() });
            line_rect = Some(line_rect.map_or(draft.rect, |r| r.union(&draft.rect)));
        }
        let Some(rect) = line_rect else { continue };
        separator(&mut page, "\n", PageRect::new(rect.right, rect.top, rect.right, rect.top + em.min(rect.height())));
        page.lines.push(Line { text: line_text, rect, word_start, word_end: page.words.len() });
    }
    page
}

fn separator(page: &mut PageText, text: &str, rect: PageRect) {
    page.text.push_str(text);
    page.glyphs.push(GlyphBox { rect, end: page.text.len() as u32 });
}

fn median(values: impl Iterator<Item = f32>) -> f32 {
    let mut values: Vec<f32> = values.filter(|v| v.is_finite()).collect();
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f32::total_cmp);
    values[values.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `text` as glyphs 6 pt wide and 10 pt tall from (`left`, `top`), with no space glyphs: a gap
    /// of 3 pt between words, as publishers position them.
    fn line(text: &str, left: f32, top: f32) -> Vec<Glyph> {
        let mut x = left;
        let mut glyphs = Vec::new();
        for word in text.split(' ') {
            for c in word.chars() {
                glyphs.push(Glyph { text: c.into(), rect: PageRect::new(x, top, x + 6.0, top + 10.0) });
                x += 6.0;
            }
            x += 3.0;
        }
        glyphs
    }

    #[test]
    fn words_split_on_gaps_without_space_glyphs() {
        let page = page_text(line("Place cells fire", 72.0, 72.0));
        let words: Vec<&str> = page.words.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(words, ["Place", "cells", "fire"]);
        assert_eq!(page.text, "Place cells fire\n");
        assert_eq!(page.lines.len(), 1);
    }

    #[test]
    fn two_columns_are_read_one_after_the_other() {
        // A full-width title, then two columns whose lines share baselines.
        let mut glyphs = line("Title of the paper", 72.0, 40.0);
        for (n, (left, right)) in [("left one", "right one"), ("left two", "right two"), ("left three", "right three")]
            .into_iter()
            .enumerate()
        {
            let top = 80.0 + n as f32 * 12.0;
            glyphs.extend(line(left, 72.0, top));
            glyphs.extend(line(right, 320.0, top));
        }
        let page = page_text(glyphs);
        let lines: Vec<&str> = page.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(
            lines,
            ["Title of the paper", "left one", "left two", "left three", "right one", "right two", "right three"]
        );
    }

    /// Two lines whose wide (20 pt) word gaps line up: too few rows for a gutter.
    #[test]
    fn a_two_line_title_with_coinciding_gaps_is_not_split_into_columns() {
        let mut glyphs = line("Diaphragm", 72.0, 40.0);
        glyphs.extend(line("motor", 146.0, 40.0));
        glyphs.extend(line("Diaphragm", 72.0, 52.0));
        glyphs.extend(line("units", 146.0, 52.0));
        let page = page_text(glyphs);
        let lines: Vec<&str> = page.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(lines, ["Diaphragm motor", "Diaphragm units"]);
    }

    /// Words set closer than the gap rule, with spaces drawn between them.
    #[test]
    fn drawn_spaces_split_tightly_set_words() {
        let mut glyphs = Vec::new();
        for (i, c) in "ab cd".chars().enumerate() {
            let x = 72.0 + i as f32 * 6.0;
            glyphs.push(Glyph { text: c.into(), rect: PageRect::new(x, 72.0, x + 6.0, 82.0) });
        }
        let words: Vec<String> = page_text(glyphs).words.into_iter().map(|w| w.text).collect();
        assert_eq!(words, ["ab", "cd"]);
    }

    /// Glyph texts are slices of the page's text: a ligature is one glyph of two characters.
    #[test]
    fn glyph_texts_and_ranges_read_from_the_page_text() {
        let mut glyphs = vec![Glyph { text: "fi".into(), rect: PageRect::new(72.0, 72.0, 80.0, 82.0) }];
        glyphs.extend(line("ne", 80.0, 72.0));
        glyphs.extend(line("cells", 100.0, 72.0));
        let page = page_text(glyphs);
        assert_eq!(page.text, "fine cells\n");
        assert_eq!((page.glyph_text(0), page.glyph_text(1), page.glyph_text(3)), ("fi", "n", " "));
        assert!(page.is_separator(3) && !page.is_separator(4));
        assert_eq!(page.range_rects(0, 5).0, "fine c");
        assert_eq!(page.range_rects(2, 4).1.len(), 1, "the separator has no box");
        assert_eq!(page.glyph_text(99), "");
    }

    #[test]
    fn overprinted_glyphs_are_read_once() {
        let mut glyphs = line("Bold", 72.0, 72.0);
        glyphs.extend(line("Bold", 72.2, 72.1));
        assert_eq!(page_text(glyphs).text, "Bold\n");
    }

    /// Glyphs 3 pt apart vertically chain under a pairwise tolerance, but a and c (6 pt apart) do
    /// not. The order must not depend on the order the glyphs were drawn in (and `sort_by` may
    /// panic on a comparison that is not a total order).
    #[test]
    fn reading_order_does_not_depend_on_drawing_order() {
        let glyph = |text: &str, left: f32, top: f32| Glyph {
            text: text.into(),
            rect: PageRect::new(left, top, left + 4.0, top + 10.0),
        };
        let (a, b, c) = (glyph("a", 10.0, 0.0), glyph("b", 5.0, 3.0), glyph("c", 0.0, 6.0));
        let permutations = [[&a, &b, &c], [&a, &c, &b], [&b, &a, &c], [&b, &c, &a], [&c, &a, &b], [&c, &b, &a]];
        let texts: Vec<String> =
            permutations.iter().map(|p| page_text(p.iter().map(|g| (*g).clone()).collect()).text).collect();
        assert!(texts.iter().all(|t| *t == texts[0]), "{texts:?}");
    }
}
