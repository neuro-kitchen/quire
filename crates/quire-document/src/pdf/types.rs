//! Data types for PDF dimensions, coordinates, rendering results, and text layer structures.

/// Page dimensions in PDF points (1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

/// A bounding rectangle on a PDF page in points, measured from the top-left corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl PageRect {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self { left: left.min(right), top: top.min(bottom), right: left.max(right), bottom: top.max(bottom) }
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }

    pub fn union(&self, other: &PageRect) -> PageRect {
        PageRect {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

/// A rendered page bitmap: `width` x `height` pixels, 32-bit RGBA row by row.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedPage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// A positioned glyph as the text layer collects it (the layout's input).
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub text: String,
    pub rect: PageRect,
}

/// A glyph of a laid-out page, or a separator the layout added between words (a space) or after a
/// line (a newline): whitespace with an empty box. Its text is [`PageText::glyph_text`], a slice of
/// the page's text ending at byte `end` (a page holds thousands: no string of its own each).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphBox {
    pub rect: PageRect,
    pub end: u32,
}

/// A word formed by consecutive glyphs.
#[derive(Debug, Clone, PartialEq)]
pub struct WordBox {
    pub text: String,
    pub rect: PageRect,
    pub glyph_start: usize,
    pub glyph_end: usize,
}

/// A line of words on a common baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    pub rect: PageRect,
    pub word_start: usize,
    pub word_end: usize,
}

/// Full extracted text layer for one page.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageText {
    pub text: String,
    pub glyphs: Vec<GlyphBox>,
    pub words: Vec<WordBox>,
    pub lines: Vec<Line>,
}

impl PageText {
    /// The text of glyph `index` (empty past the end).
    pub fn glyph_text(&self, index: usize) -> &str {
        let Some(glyph) = self.glyphs.get(index) else { return "" };
        let start = index.checked_sub(1).map_or(0, |i| self.glyphs[i].end as usize);
        &self.text[start..glyph.end as usize]
    }

    /// Whether glyph `index` is a separator the layout added.
    pub fn is_separator(&self, index: usize) -> bool {
        self.glyph_text(index).chars().all(char::is_whitespace)
    }

    /// Finds the word at point `(x, y)`.
    pub fn word_at(&self, x: f32, y: f32) -> Option<&WordBox> {
        self.words.iter().find(|w| w.rect.contains(x, y))
    }

    /// Finds the index of the nearest glyph to point `(x, y)`, separators excluded (from anywhere on
    /// the page, so a drag into the margin still selects up to the line's end).
    pub fn nearest_glyph(&self, x: f32, y: f32) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_dist = f32::MAX;
        for (i, g) in self.glyphs.iter().enumerate().filter(|(i, _)| !self.is_separator(*i)) {
            let cx = (g.rect.left + g.rect.right) / 2.0;
            let cy = (g.rect.top + g.rect.bottom) / 2.0;
            let dx = x - cx;
            let dy = y - cy;
            let dist = dx * dx + dy * dy;
            if dist < best_dist {
                best_dist = dist;
                best = Some(i);
            }
        }
        best
    }

    /// Returns the text (separators included) and the selection boxes (separators excluded) for a
    /// glyph range `[start, end)`.
    pub fn range_rects(&self, start: usize, end: usize) -> (String, Vec<PageRect>) {
        if start >= end || start >= self.glyphs.len() {
            return (String::new(), Vec::new());
        }
        let end = end.min(self.glyphs.len());
        let from = start.checked_sub(1).map_or(0, |i| self.glyphs[i].end as usize);
        let text = self.text[from..self.glyphs[end - 1].end as usize].to_string();

        // Group glyph bounding rects by line / proximity into compact selection boxes
        let mut rects = Vec::new();
        let mut current_box: Option<PageRect> = None;

        for g in (start..end).filter(|&i| !self.is_separator(i)).map(|i| &self.glyphs[i]) {
            if let Some(cur) = current_box {
                // If on same line (vertical overlap and reasonable horizontal distance)
                let vert_overlap = (g.rect.top < cur.bottom) && (g.rect.bottom > cur.top);
                let horiz_near = (g.rect.left - cur.right).abs() < (g.rect.height() * 1.5);
                if vert_overlap && horiz_near {
                    current_box = Some(cur.union(&g.rect));
                } else {
                    rects.push(cur);
                    current_box = Some(g.rect);
                }
            } else {
                current_box = Some(g.rect);
            }
        }
        if let Some(cur) = current_box {
            rects.push(cur);
        }

        (text, rects)
    }
}

/// Where one searched quote appears on a page.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMatch {
    pub quote: u32,
    pub page: u32,
    pub rects: Vec<PageRect>,
}
