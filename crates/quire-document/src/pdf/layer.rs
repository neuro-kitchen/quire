//! The text layer: a hayro `Device` that records every glyph with a Unicode value and its box,
//! then [`layout`] puts them in reading order.
//!
//! Boxes (checked against hayro-interpret 0.8.0's source): a glyph's transform maps a 1000-unit em
//! onto the page (`full_transform × scale(1/1000)`, `interpret/text.rs`), and advance widths come
//! from the font's `/Widths` or `/W` in the same units. The box is the advance wide and one em tall,
//! from 800 above the baseline to 200 below: an approximation of the font's ascent and descent. Type 3
//! glyphs have no exposed width and get half an em. A rotated glyph's box bounds its rotated em, and
//! its writing direction ([`Direction`]) comes from its advance on the page.
//!
//! Not read: `/ActualText`. hayro-interpret 0.8.0 gives a device the marked-content tag and MCID
//! only, not the properties holding it (`device.rs`, `begin_marked_content`).

use hayro::kurbo::{BezPath, Point, Rect};
use hayro_interpret::TransformExt;
use hayro_interpret::{ClipPath, Context, Device, DrawMode, DrawProps, Image, ImageDrawProps, SoftMask};
use hayro_syntax::page::Page;

use super::layout::{self, Direction};
use super::types::{Glyph, PageRect, PageText};
use crate::error::DocumentError;

/// Internal Device for extracting text and glyph bounds.
#[derive(Default)]
pub struct TextCollector {
    glyphs: Vec<(Glyph, Direction)>,
}

impl TextCollector {
    pub fn new() -> Self {
        Self { glyphs: Vec::new() }
    }

    pub fn extract(page: &Page<'_>) -> Result<PageText, DocumentError> {
        let mut collector = Self::new();
        let cache = hayro_interpret::InterpreterCache::new();
        let settings = hayro_interpret::InterpreterSettings::default();
        let (width, height) = page.render_dimensions();
        let transform = page.initial_transform(true).to_kurbo();
        let mut context =
            Context::new(transform, Rect::new(0.0, 0.0, width as f64, height as f64), &cache, page.xref(), settings);

        hayro_interpret::interpret_page(page, &mut context, &mut collector);
        Ok(layout::page_text_directed(collector.glyphs))
    }
}

impl Device<'_> for TextCollector {
    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'_>, _: &DrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'_>>, _: hayro_interpret::BlendMode) {}
    fn draw_image(&mut self, _: Image<'_, '_>, _: ImageDrawProps<'_>) {}
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}

    fn draw_glyph_run(
        &mut self,
        glyph_run: &hayro_interpret::font::GlyphRun<'_, '_>,
        props: DrawProps<'_>,
        _: &DrawMode,
    ) {
        for glyph in glyph_run.glyphs() {
            let text_char = match glyph.as_unicode() {
                Some(hayro_cmap::BfString::Char(c)) => expand_ligatures(&c.to_string()),
                Some(hayro_cmap::BfString::String(s)) => expand_ligatures(&s),
                None => continue,
            };

            let total_tf = props.transform * glyph.transform();
            let p0: Point = total_tf * Point::new(0.0, 0.0);

            // Advance width, in the em's 1000 units.
            let adv = match &**glyph {
                hayro_interpret::font::Glyph::Outline(og) => og.advance_width().unwrap_or(500.0) as f64,
                hayro_interpret::font::Glyph::Type3(_) => 500.0,
            };

            // The box around the advance and the em's height, whatever the glyph's rotation.
            let p_adv = total_tf * Point::new(adv, 0.0);
            let corners =
                [(0.0, 800.0), (0.0, -200.0), (adv, 800.0), (adv, -200.0)].map(|(x, y)| total_tf * Point::new(x, y));
            let xs = corners.iter().map(|p| p.x);
            let ys = corners.iter().map(|p| p.y);
            let (left, right) = (xs.clone().fold(f64::MAX, f64::min), xs.fold(f64::MIN, f64::max));
            let (top, bottom) = (ys.clone().fold(f64::MAX, f64::min), ys.fold(f64::MIN, f64::max));
            let rect = PageRect::new(left as f32, top.max(0.0) as f32, right.max(left + 1.0) as f32, bottom as f32);
            let direction = Direction::of(p_adv.x - p0.x, p_adv.y - p0.y);

            self.glyphs.push((Glyph { text: text_char, rect }, direction));
        }
    }
}

/// Unicode's Latin ligatures (U+FB00–FB06) as their letters, so copy and search see "find", not "ﬁnd".
fn expand_ligatures(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{FB00}' => out.push_str("ff"),
            '\u{FB01}' => out.push_str("fi"),
            '\u{FB02}' => out.push_str("fl"),
            '\u{FB03}' => out.push_str("ffi"),
            '\u{FB04}' => out.push_str("ffl"),
            '\u{FB05}' | '\u{FB06}' => out.push_str("st"),
            c => out.push(c),
        }
    }
    out
}
