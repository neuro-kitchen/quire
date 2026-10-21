//! Where a PDF's figures are, for showing them in the reader: a text reading of a PDF keeps the
//! captions but no images, so a figure is found from its caption in the text layer instead.
//!
//! A caption starts a line with its label and a separator right after the number (`Figure 2.`,
//! `Fig. 3:`, `Figure 1 |`, `Extended Data Fig. 4`), which a sentence mentioning it ("Figure 2
//! shows") does not have. The figure is the area above the caption, up to the nearest line of body
//! text (a line spanning most of the column), across the caption's column, or the page when the
//! caption is wider than a column. Short lines in that area are the figure's own labels (axes,
//! legends, panel letters), so it grows over them. This holds for drawn charts as well as images.

use std::path::Path;

use super::pages::{page_sizes, page_text, render_page_at};
use super::types::{Line, PageRect, PageSize, PageText};
use crate::error::DocumentError;

/// A figure found on a page.
#[derive(Debug, Clone, PartialEq)]
pub struct FigureRegion {
    /// 0-based page.
    pub page: u32,
    /// `Figure 2`, `Extended Data Figure 4`, as printed (with `Fig.` written out).
    pub label: String,
    /// The caption's text, label included.
    pub caption: String,
    /// The figure, without its caption, in points from the page's top-left.
    pub rect: PageRect,
}

/// Shortest figure kept, in points (a caption with nothing drawn above it is not a figure here).
const MIN_HEIGHT: f32 = 40.0;
/// A body text line spans at least this share of its column.
const BODY_SHARE: f32 = 0.7;
/// A body text line has at least this many characters (labels are short).
const BODY_CHARS: usize = 40;
/// Lines of a caption read at most (the rest of a long caption is not needed).
const CAPTION_LINES: usize = 12;
/// The top band of a page holding its running header, as a share of its height.
const HEADER_BAND: f32 = 0.07;

/// Every figure of the PDF at `path`, page by page. An area with almost nothing drawn in it besides
/// text (a caption whose figure is on another page, as manuscripts place them) is left out.
pub fn figures(path: impl AsRef<Path>) -> Result<Vec<FigureRegion>, DocumentError> {
    let path = path.as_ref();
    let mut out = Vec::new();
    for (index, size) in page_sizes(path)?.into_iter().enumerate() {
        let text = page_text(path, index as u32)?;
        let found = page_figures(&text, size, index as u32);
        if found.is_empty() {
            continue;
        }
        let page = render_page_at(path, index as u32, INK_WIDTH)?;
        let scale = page.width as f32 / size.width;
        for mut figure in found {
            if ink(&page.pixels, page.width, page.height, scale, figure.rect, &text) < MIN_INK {
                continue;
            }
            // Down to what is drawn (an area that ran to the page's top keeps no blank margin).
            if let Some(bounds) = drawn_bounds(&page.pixels, page.width, page.height, scale, figure.rect) {
                figure.rect = bounds;
            }
            out.push(figure);
        }
    }
    Ok(out)
}

/// Width the page is rendered at to see whether a figure's area has anything drawn in it.
const INK_WIDTH: u32 = 400;
/// Share of an area's pixels (outside text) that must be drawn for it to be a figure.
const MIN_INK: f32 = 0.005;

/// The share of the pixels of `rect` (in points) that are drawn (not near white), outside the
/// text lines: what is left is the figure's drawing or image.
fn ink(pixels: &[u8], width: u32, height: u32, scale: f32, rect: PageRect, text: &PageText) -> f32 {
    let to_px = |v: f32, max: u32| ((v * scale).max(0.0) as u32).min(max);
    let (x0, x1) = (to_px(rect.left, width), to_px(rect.right, width));
    let (y0, y1) = (to_px(rect.top, height), to_px(rect.bottom, height));
    let text_boxes: Vec<PageRect> =
        text.lines.iter().map(|l| l.rect).filter(|r| overlap(r.top, r.bottom, rect.top, rect.bottom) > 0.0).collect();
    let (mut drawn, mut counted) = (0u32, 0u32);
    for y in y0..y1 {
        for x in x0..x1 {
            let (px, py) = (x as f32 / scale, y as f32 / scale);
            if text_boxes.iter().any(|r| r.contains(px, py)) {
                continue;
            }
            counted += 1;
            let i = ((y * width + x) * 4) as usize;
            if pixels[i..i + 3].iter().any(|&c| c < 200) {
                drawn += 1;
            }
        }
    }
    if counted == 0 { 0.0 } else { drawn as f32 / counted as f32 }
}

/// The box of everything drawn in `rect` (labels included), with a small margin, inside `rect`.
fn drawn_bounds(pixels: &[u8], width: u32, height: u32, scale: f32, rect: PageRect) -> Option<PageRect> {
    let to_px = |v: f32, max: u32| ((v * scale).max(0.0) as u32).min(max);
    let (x0, x1) = (to_px(rect.left, width), to_px(rect.right, width));
    let (y0, y1) = (to_px(rect.top, height), to_px(rect.bottom, height));
    let (mut left, mut top, mut right, mut bottom) = (u32::MAX, u32::MAX, 0, 0);
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * width + x) * 4) as usize;
            if pixels[i..i + 3].iter().any(|&c| c < 200) {
                (left, top, right, bottom) = (left.min(x), top.min(y), right.max(x), bottom.max(y));
            }
        }
    }
    if left > right {
        return None;
    }
    let margin = 4.0;
    Some(PageRect::new(
        (left as f32 / scale - margin).max(rect.left),
        (top as f32 / scale - margin).max(rect.top),
        ((right + 1) as f32 / scale + margin).min(rect.right),
        ((bottom + 1) as f32 / scale + margin).min(rect.bottom),
    ))
}

/// The figures on one page, from its text layer.
pub fn page_figures(text: &PageText, size: PageSize, page: u32) -> Vec<FigureRegion> {
    let lines: Vec<&Line> = text.lines.iter().filter(|l| !l.text.trim().is_empty()).collect();
    if lines.is_empty() {
        return Vec::new();
    }
    // Where the text is: its margins, and the middle between two columns.
    let left = lines.iter().map(|l| l.rect.left).fold(f32::MAX, f32::min);
    let right = lines.iter().map(|l| l.rect.right).fold(f32::MIN, f32::max);
    // A running header (journal, page number) sits in the top band; a figure starts below it.
    let header_band = size.height * HEADER_BAND;
    let top_margin = lines
        .iter()
        .filter(|l| l.rect.top < header_band)
        .map(|l| l.rect.bottom + 2.0)
        .fold(lines.iter().map(|l| l.rect.top).fold(f32::MAX, f32::min), f32::max)
        .max(0.0);
    let middle = (left + right) / 2.0;
    // Two columns when several lines of prose sit wholly in each half; else one column.
    let prose_in = |half: fn(&Line, f32) -> bool| {
        lines.iter().filter(|l| half(l, middle) && l.text.trim().chars().count() >= BODY_CHARS).count() >= 3
    };
    let two_columns = prose_in(|l, m| l.rect.right <= m + 6.0) && prose_in(|l, m| l.rect.left >= m - 6.0);
    // Body text is judged against a text column, also above a figure as wide as the page.
    let text_column = if two_columns { (right - left) / 2.0 } else { right - left };
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(label) = caption_label(&line.text) else { continue };
        let caption_rect = caption_extent(&lines, i);
        // The caption's column, or the page's text width for a caption across both columns.
        let wide = !two_columns || caption_rect.width() > (right - left) * 0.6;
        let (col_left, col_right) = if wide {
            (left, right)
        } else if caption_rect.right <= middle + 6.0 {
            (left, middle)
        } else if caption_rect.left >= middle - 6.0 {
            (middle, right)
        } else {
            (left, right)
        };
        // Up from the caption to the first line of body text in the column.
        let above: Vec<&Line> = lines
            .iter()
            .filter(|l| {
                l.rect.bottom <= line.rect.top + 1.0
                    && l.rect.top >= header_band
                    && overlap(l.rect.left, l.rect.right, col_left, col_right)
                        > l.rect.width().min(col_right - col_left) * 0.5
            })
            .copied()
            .collect();
        let body = body_lines(&above, text_column).into_iter().map(|l| l.rect.bottom).reduce(f32::max);
        let top = body.map_or(top_margin, |b| b + 2.0);
        let bottom = line.rect.top - 2.0;
        if bottom - top < MIN_HEIGHT {
            continue;
        }
        let rect = PageRect::new(col_left.max(0.0), top.max(0.0), col_right.min(size.width), bottom.min(size.height));
        let caption = lines[i..(i + CAPTION_LINES).min(lines.len())]
            .iter()
            .take_while(|l| l.rect.top <= caption_rect.bottom + 0.5)
            .filter(|l| overlap(l.rect.left, l.rect.right, caption_rect.left, caption_rect.right) > 0.0)
            .map(|l| l.text.trim())
            .collect::<Vec<_>>()
            .join(" ");
        out.push(FigureRegion { page, label, caption, rect });
    }
    out
}

/// The body text among `lines` (one column, top to bottom): long lines, and the short lines that
/// end their paragraphs (just below a body line, at its left edge).
fn body_lines<'a>(lines: &[&'a Line], column_width: f32) -> Vec<&'a Line> {
    let mut sorted: Vec<&Line> = lines.to_vec();
    sorted.sort_by(|a, b| a.rect.top.total_cmp(&b.rect.top));
    let mut body: Vec<&Line> = Vec::new();
    for line in sorted {
        let long = line.rect.width() >= column_width * BODY_SHARE && line.text.trim().chars().count() >= BODY_CHARS;
        let ends_paragraph = body.last().is_some_and(|prev| {
            let gap = line.rect.top - prev.rect.bottom;
            gap < prev.rect.height() * 1.2 && (line.rect.left - prev.rect.left).abs() < 6.0
        });
        if long || ends_paragraph {
            body.push(line);
        }
    }
    body
}

/// The caption's label when `text` starts a caption: `Figure 2`, `Extended Data Figure 4`,
/// `Supplementary Figure S3`. `None` for a sentence that mentions a figure.
pub(crate) fn caption_label(text: &str) -> Option<String> {
    let text = text.trim_start();
    let (prefix, rest) = [
        "Extended Data Figure",
        "Extended Data Fig.",
        "Supplementary Figure",
        "Supplementary Fig.",
        "Figure",
        "FIGURE",
        "Fig.",
        "FIG.",
        "Fig",
    ]
    .iter()
    .find_map(|p| text.strip_prefix(p).map(|rest| (*p, rest)))?;
    let rest = rest.trim_start();
    let number: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    if number.is_empty() || !number.chars().any(|c| c.is_ascii_digit()) || number.len() > 4 {
        return None;
    }
    // A separator right after the number (or nothing more on the line): a caption, not a mention.
    let after = rest[number.len()..].trim_start();
    let separated = after.is_empty() || after.starts_with(['.', ':', '|', '—', '–', '-', '(']);
    if !separated {
        return None;
    }
    let name = match prefix {
        "Extended Data Fig." | "Extended Data Figure" => "Extended Data Figure",
        "Supplementary Fig." | "Supplementary Figure" => "Supplementary Figure",
        _ => "Figure",
    };
    Some(format!("{name} {number}"))
}

/// The caption starting at line `start`: it and the lines just below it in the same column.
fn caption_extent(lines: &[&Line], start: usize) -> PageRect {
    let first = lines[start];
    let height = first.rect.height().max(1.0);
    let mut rect = first.rect;
    for line in lines.iter().skip(start + 1).take(CAPTION_LINES - 1) {
        let gap = line.rect.top - rect.bottom;
        let same_column =
            overlap(line.rect.left, line.rect.right, first.rect.left, first.rect.right.max(rect.right)) > 0.0;
        if !(-1.0..=height * 1.2).contains(&gap) || !same_column {
            break;
        }
        rect = rect.union(&line.rect);
    }
    rect
}

fn overlap(a_left: f32, a_right: f32, b_left: f32, b_right: f32) -> f32 {
    (a_right.min(b_right) - a_left.max(b_left)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str, left: f32, top: f32, right: f32) -> Line {
        Line { text: text.into(), rect: PageRect::new(left, top, right, top + 10.0), word_start: 0, word_end: 0 }
    }

    const PAGE: PageSize = PageSize { width: 612.0, height: 792.0 };
    const BODY: &str = "a line of body text long enough to count as running prose here";

    #[test]
    fn captions_are_told_from_mentions() {
        assert_eq!(caption_label("Figure 2. Place cells fire").as_deref(), Some("Figure 2"));
        assert_eq!(caption_label("Fig. 3: Grid cells").as_deref(), Some("Figure 3"));
        assert_eq!(caption_label("Figure 1 | Thermometry").as_deref(), Some("Figure 1"));
        assert_eq!(caption_label("Extended Data Fig. 4 Controls").map(|_| ()), None, "no separator");
        assert_eq!(caption_label("Extended Data Fig. 4. Controls").as_deref(), Some("Extended Data Figure 4"));
        assert_eq!(caption_label("FIG. 2b. Spectra").as_deref(), Some("Figure 2b"));
        assert_eq!(caption_label("Figure 2 shows that the cells"), None, "a sentence");
        assert_eq!(caption_label("Figures 2 and 3. Both"), None);
        assert_eq!(caption_label("Figure"), None);
    }

    /// A one-column page: body text, a chart with axis labels, its caption, more text.
    #[test]
    fn the_area_above_a_caption_up_to_body_text() {
        let text = PageText {
            lines: vec![
                line(BODY, 72.0, 72.0, 540.0),
                line(BODY, 72.0, 86.0, 540.0),
                line("1.0", 80.0, 140.0, 95.0),
                line("Time (s)", 250.0, 300.0, 300.0),
                line("Figure 1. Temperature over time in a living cell.", 72.0, 330.0, 400.0),
                line(BODY, 72.0, 360.0, 540.0),
            ],
            ..PageText::default()
        };
        let found = page_figures(&text, PAGE, 0);
        assert_eq!(found.len(), 1);
        let f = &found[0];
        assert_eq!(f.label, "Figure 1");
        assert!(f.caption.starts_with("Figure 1. Temperature"));
        assert!((f.rect.top - 98.0).abs() < 0.5, "below the last body line: {:?}", f.rect);
        assert!((f.rect.bottom - 328.0).abs() < 0.5, "above the caption: {:?}", f.rect);
        assert!(f.rect.left <= 72.0 && f.rect.right >= 540.0, "the text width: {:?}", f.rect);
    }

    /// Two columns: a figure in the right column takes that column only.
    #[test]
    fn a_figure_in_one_column() {
        let mut lines: Vec<Line> = (0..3)
            .flat_map(|i| {
                let top = 400.0 + 14.0 * i as f32;
                [line(BODY, 72.0, top, 296.0), line(BODY, 316.0, top, 540.0)]
            })
            .collect();
        lines.extend([
            line(BODY, 72.0, 72.0, 296.0),
            line(BODY, 316.0, 72.0, 540.0),
            line(BODY, 72.0, 300.0, 296.0),
            line("Fig. 2: Spikes.", 316.0, 300.0, 420.0),
        ]);
        let text = PageText { lines, ..PageText::default() };
        let found = page_figures(&text, PAGE, 3);
        assert_eq!(found.len(), 1);
        let f = &found[0];
        assert_eq!((f.page, f.label.as_str()), (3, "Figure 2"));
        assert!(f.rect.left >= 300.0 && f.rect.right >= 530.0, "the right column: {:?}", f.rect);
        assert!((f.rect.top - 84.0).abs() < 0.5);
    }

    /// Seen in real papers: a figure as wide as a two-column page took in all the text above it
    /// (single-column lines were not "body" against the page's width).
    #[test]
    fn a_wide_figure_on_a_two_column_page() {
        let mut lines: Vec<Line> = (0..6)
            .flat_map(|i| {
                let top = 72.0 + 14.0 * i as f32;
                [line(BODY, 72.0, top, 296.0), line(BODY, 316.0, top, 540.0)]
            })
            .collect();
        lines.push(line(
            "Fig. 3. Adhesive events at the implanted surface, from A to D, over weeks.",
            72.0,
            600.0,
            540.0,
        ));
        let found = page_figures(&PageText { lines, ..PageText::default() }, PAGE, 0);
        assert_eq!(found.len(), 1);
        assert!(found[0].rect.top >= 72.0 + 14.0 * 5.0 + 10.0, "below the last row of text: {:?}", found[0].rect);
        assert!(found[0].rect.left <= 72.0 && found[0].rect.right >= 540.0, "across both columns");
    }

    /// Seen in real papers: a running header above a figure stayed in it; the short last line of a
    /// paragraph above a caption was taken for figure labels.
    #[test]
    fn headers_and_paragraph_ends_stay_out() {
        let text = PageText {
            lines: vec![
                line("Sensors 2023, 23, 2927", 72.0, 30.0, 180.0),
                line("4 of 29", 500.0, 30.0, 540.0),
                line("Flicker noise", 120.0, 120.0, 180.0),
                line("Figure 2. Noise.", 72.0, 300.0, 200.0),
                line(BODY, 72.0, 400.0, 540.0),
                line("performed.", 72.0, 414.0, 130.0),
                line("Figure 3. Pacing.", 72.0, 600.0, 200.0),
            ],
            ..PageText::default()
        };
        let found = page_figures(&text, PAGE, 0);
        assert_eq!(found.len(), 2);
        assert!(found[0].rect.top >= 42.0, "below the header: {:?}", found[0].rect);
        assert!(found[1].rect.top >= 426.0, "below the paragraph's last line: {:?}", found[1].rect);
    }

    /// A caption right under body text has no figure above it here (it may sit above its figure).
    #[test]
    fn no_room_no_figure() {
        let text = PageText {
            lines: vec![line(BODY, 72.0, 72.0, 540.0), line("Figure 3. Something.", 72.0, 90.0, 300.0)],
            ..PageText::default()
        };
        assert!(page_figures(&text, PAGE, 0).is_empty());
    }
}
