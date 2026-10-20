//! Rendering PDF pages to RGBA bitmaps via hayro.

use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings};
use hayro_interpret::InterpreterSettings;
use hayro_syntax::page::Page;

use super::types::RenderedPage;
use crate::error::DocumentError;

/// Longest side of a rendered page, in pixels.
const MAX_SIDE: u32 = 8192;
/// Most pixels in a rendered page (128 MB of RGBA).
const MAX_PIXELS: u32 = 32 * 1024 * 1024;

/// Renders `page` to an RGBA bitmap at target `width` in pixels (smaller for a page too tall to
/// fit [`MAX_SIDE`] and [`MAX_PIXELS`]; the result says its size).
pub(crate) fn render_page(page: &Page<'_>, width: u32) -> Result<RenderedPage, DocumentError> {
    let (base_w, base_h) = page.render_dimensions();
    if base_w <= 0.0 || base_h <= 0.0 {
        return Err(DocumentError::InvalidDimensions(format!("invalid page dimensions: {base_w}x{base_h}")));
    }

    // The width asked for (clamped), scaled down if the page is so tall that the bitmap would pass
    // MAX_SIDE pixels high or MAX_PIXELS in all.
    let scale = width.clamp(16, MAX_SIDE) as f32 / base_w;
    let scale = scale.min(MAX_SIDE as f32 / base_h).min((MAX_PIXELS as f32 / (base_w * base_h)).sqrt());

    let cache = RenderCache::new();
    let interpreter_settings = InterpreterSettings::default();
    let render_settings = RenderSettings::default();
    let pixmap_settings = PixmapSettings { x_scale: scale, y_scale: scale, bg_color: WHITE };

    let pixmap = hayro::render(page, &cache, &interpreter_settings, &render_settings, &pixmap_settings);
    let out_w = pixmap.width() as u32;
    let out_h = pixmap.height() as u32;

    let raw_slice: &[u8] = bytemuck::cast_slice(pixmap.data());
    let mut pixels = Vec::with_capacity(raw_slice.len());
    pixels.extend_from_slice(raw_slice);

    Ok(RenderedPage { width: out_w, height: out_h, pixels })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A page 10 pt wide and 9999 pt tall asked at 8192 px wide would be 8 million px tall.
    #[test]
    fn a_very_tall_page_is_rendered_within_the_limits() {
        let pdf = String::from_utf8(quire_testkit::tiny_pdf("x")).unwrap().replace("612 792", "10 9999");
        let pdf = hayro_syntax::Pdf::new(pdf.into_bytes()).unwrap();
        let rendered = render_page(&pdf.pages()[0], 8192).unwrap();
        assert!(
            rendered.height <= MAX_SIDE && rendered.width * rendered.height <= MAX_PIXELS,
            "{}x{}",
            rendered.width,
            rendered.height
        );
        assert_eq!(rendered.pixels.len(), (rendered.width * rendered.height * 4) as usize);
    }
}
