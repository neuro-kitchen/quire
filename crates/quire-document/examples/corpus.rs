//! Rendering and text-layer check for one PDF against poppler (`pdftoppm`, `pdftotext`), for the
//! corpus runs recorded in `.knowledge/pdf-engine.md`. Not shipped.
//!
//! `cargo run --release -p quire-document --example corpus -- <pdf> <out-dir> [--pages N] [--images]`
//! (`--text <page>`: print that page's text layer instead; `--boxes <page>`: write that page at 2×
//! with its word and line boxes)
//!
//! Prints one tab-separated line: file, pages, open ms, render ms per page at 1×, 2× and 3× (1× is
//! 72 dpi), text ms per page, word F1 and word-pair F1 against `pdftotext` (pairs measure reading
//! order), mean pixel difference against `pdftoppm` at 1× (0–255) and the share of pixels that
//! differ by more than 64, then a status. `--images` writes, for the first page, hayro's and
//! poppler's renders and hayro's render with the word boxes drawn on it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use image::{Rgba, RgbaImage};

mod common;
use common::{f1, pairs, words};

const HEADER: &str = "file\tpages\topen_ms\trender1_ms\trender2_ms\trender3_ms\ttext_ms\tword_f1\tpair_f1\tdiff_mean\tdiff_share\tstatus";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--header") {
        println!("{HEADER}");
        return;
    }
    let (Some(pdf), Some(out)) = (args.first(), args.get(1)) else {
        eprintln!("usage: corpus <pdf> <out-dir> [--pages N] [--images] | corpus --header");
        std::process::exit(2);
    };
    let pages = args.iter().position(|a| a == "--pages").and_then(|i| args.get(i + 1)?.parse().ok()).unwrap_or(3);
    let images = args.iter().any(|a| a == "--images");
    if let Some(page) = args.iter().position(|a| a == "--text").and_then(|i| args.get(i + 1)?.parse::<u32>().ok()) {
        // The text layer of one page (from 1), for looking at what the scores measure.
        let text = quire_document::pdf::page_text(&args[0], page - 1).expect("page text");
        print!("{}", text.text);
        return;
    }
    let (pdf, out) = (PathBuf::from(pdf), PathBuf::from(out));
    if let Some(page) = args.iter().position(|a| a == "--boxes").and_then(|i| args.get(i + 1)?.parse::<u32>().ok()) {
        // One page (from 1) rendered at 2× with its word and line boxes.
        let size = quire_document::pdf::page_sizes(&pdf).expect("sizes")[page as usize - 1];
        let rendered = quire_document::pdf::render_page_at(&pdf, page - 1, (size.width * 2.0) as u32).expect("render");
        let mut image = RgbaImage::from_raw(rendered.width, rendered.height, rendered.pixels).expect("bitmap");
        let text = quire_document::pdf::page_text(&pdf, page - 1).expect("page text");
        draw_boxes(&mut image, &text, rendered.width as f32 / size.width);
        std::fs::create_dir_all(&out).expect("out dir");
        image.save(out.join(format!("page-{page}-words.png"))).expect("saved");
        return;
    }
    std::fs::create_dir_all(&out).expect("out dir");
    let name = pdf.file_stem().unwrap_or_default().to_string_lossy().to_string();

    let row = std::panic::catch_unwind(|| check(&pdf, &out, &name, pages, images))
        .unwrap_or_else(|_| Err("panic".into()))
        .unwrap_or_else(|e| format!("\t\t\t\t\t\t\t\t\t\t{}", e.replace(['\t', '\n'], " ")));
    println!("{name}\t{row}");
}

fn check(pdf: &Path, out: &Path, name: &str, pages: u32, images: bool) -> Result<String, String> {
    let start = Instant::now();
    let sizes = quire_document::pdf::page_sizes(pdf).map_err(|e| format!("open: {e}"))?;
    let open_ms = ms(start);
    let n = (sizes.len() as u32).min(pages);

    let mut render = [0.0; 3];
    let mut first = None;
    for page in 0..n {
        let width = sizes[page as usize].width;
        for (scale, total) in render.iter_mut().enumerate() {
            let start = Instant::now();
            let rendered = quire_document::pdf::render_page_at(pdf, page, (width * (scale + 1) as f32).round() as u32)
                .map_err(|e| format!("render p{page}: {e}"))?;
            *total += ms(start);
            if page == 0 && scale == 0 {
                first = Some(rendered);
            }
        }
    }

    // Uncached: the extractor itself, not the cache.
    let document = quire_document::pdf::global_cache().get_pdf(pdf).map_err(|e| e.to_string())?;
    let mut text_ms = 0.0;
    let mut ours = String::new();
    for (page, p) in document.pages().iter().take(n as usize).enumerate() {
        let start = Instant::now();
        let text = quire_document::pdf::layer::TextCollector::extract(p).map_err(|e| format!("text p{page}: {e}"))?;
        text_ms += ms(start);
        ours.push_str(&text.text);
        ours.push('\n');
    }

    let theirs = poppler_text(pdf, n)?;
    let (word_f1, pair_f1) = (f1(&words(&ours), &words(&theirs)), f1(&pairs(&ours), &pairs(&theirs)));

    let first = first.ok_or("no pages")?;
    let ours_img = RgbaImage::from_raw(first.width, first.height, first.pixels).ok_or("bad bitmap")?;
    let theirs_img = poppler_page(pdf, out, name)?;
    let (diff_mean, diff_share) = diff(&ours_img, &theirs_img);

    if images {
        ours_img.save(out.join(format!("{name}-hayro.png"))).map_err(|e| e.to_string())?;
        theirs_img.save(out.join(format!("{name}-poppler.png"))).map_err(|e| e.to_string())?;
        let mut boxes = ours_img.clone();
        let text = quire_document::pdf::page_text(pdf, 0).map_err(|e| e.to_string())?;
        draw_boxes(&mut boxes, &text, first.width as f32 / sizes[0].width);
        boxes.save(out.join(format!("{name}-words.png"))).map_err(|e| e.to_string())?;
    }

    let per = |total: f64| total / n.max(1) as f64;
    Ok(format!(
        "{}\t{open_ms:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{word_f1:.3}\t{pair_f1:.3}\t{diff_mean:.1}\t{diff_share:.3}\tok",
        sizes.len(),
        per(render[0]),
        per(render[1]),
        per(render[2]),
        per(text_ms),
    ))
}

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn poppler_text(pdf: &Path, pages: u32) -> Result<String, String> {
    let output = Command::new("pdftotext")
        .args(["-f", "1", "-l", &pages.to_string(), "-enc", "UTF-8"])
        .arg(pdf)
        .arg("-")
        .output()
        .map_err(|e| format!("pdftotext: {e}"))?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Page 1 at 72 dpi from `pdftoppm`, cropped like hayro (CropBox, not MediaBox).
fn poppler_page(pdf: &Path, out: &Path, name: &str) -> Result<RgbaImage, String> {
    let prefix = out.join(format!("{name}-poppler-tmp"));
    let status = Command::new("pdftoppm")
        .args(["-r", "72", "-f", "1", "-l", "1", "-cropbox", "-png", "-singlefile"])
        .arg(pdf)
        .arg(&prefix)
        .status()
        .map_err(|e| format!("pdftoppm: {e}"))?;
    if !status.success() {
        return Err("pdftoppm failed".into());
    }
    let path = prefix.with_extension("png");
    let img = image::open(&path).map_err(|e| format!("pdftoppm output: {e}"))?.to_rgba8();
    let _ = std::fs::remove_file(path);
    Ok(img)
}

/// Mean absolute difference (0–255) over the shared area, and the share of pixels whose largest
/// channel difference exceeds 64.
fn diff(a: &RgbaImage, b: &RgbaImage) -> (f64, f64) {
    let (w, h) = (a.width().min(b.width()), a.height().min(b.height()));
    let (mut sum, mut far) = (0u64, 0u64);
    for y in 0..h {
        for x in 0..w {
            let (p, q) = (a.get_pixel(x, y), b.get_pixel(x, y));
            let d: Vec<u64> = (0..3).map(|i| (p[i] as i32 - q[i] as i32).unsigned_abs() as u64).collect();
            sum += d.iter().sum::<u64>();
            if d.iter().any(|&v| v > 64) {
                far += 1;
            }
        }
    }
    let area = (w as u64 * h as u64).max(1);
    (sum as f64 / (area * 3) as f64, far as f64 / area as f64)
}

/// Words in red, lines in blue.
fn draw_boxes(image: &mut RgbaImage, text: &quire_document::pdf::PageText, scale: f32) {
    for word in &text.words {
        outline(image, word.rect, scale, Rgba([220, 30, 30, 255]));
    }
    for line in &text.lines {
        outline(image, line.rect, scale, Rgba([30, 90, 220, 255]));
    }
}

fn outline(img: &mut RgbaImage, rect: quire_document::pdf::PageRect, scale: f32, color: Rgba<u8>) {
    let clamp = |v: f32, max: u32| ((v * scale) as i64).clamp(0, max as i64 - 1) as u32;
    let (l, r) = (clamp(rect.left, img.width()), clamp(rect.right, img.width()));
    let (t, b) = (clamp(rect.top, img.height()), clamp(rect.bottom, img.height()));
    for x in l..=r {
        img.put_pixel(x, t, color);
        img.put_pixel(x, b, color);
    }
    for y in t..=b {
        img.put_pixel(l, y, color);
        img.put_pixel(r, y, color);
    }
}
