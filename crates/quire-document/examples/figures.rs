//! Finds the figures of every PDF in a folder and saves each as a PNG, to check the detection by
//! eye: `cargo run --release -p quire-document --example figures -- <pdf folder> <out folder> [max]`.
use std::path::PathBuf;

use quire_document::pdf::{figures, page_sizes, render_page_at};

fn main() {
    let mut args = std::env::args().skip(1);
    let (input, output) = (PathBuf::from(args.next().unwrap()), PathBuf::from(args.next().unwrap()));
    let max_png: usize = args.next().map_or(40, |n| n.parse().unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut pdfs: Vec<PathBuf> =
        walk(&input).into_iter().filter(|p| p.extension().is_some_and(|e| e == "pdf")).collect();
    pdfs.sort();
    let (mut papers, mut with_figures, mut total, mut saved) = (0, 0, 0, 0);
    for pdf in &pdfs {
        papers += 1;
        let Ok(found) = figures(pdf) else { continue };
        if !found.is_empty() {
            with_figures += 1;
        }
        total += found.len();
        let sizes = page_sizes(pdf).unwrap();
        for f in &found {
            if saved >= max_png {
                break;
            }
            let size = sizes[f.page as usize];
            let page = render_page_at(pdf, f.page, 1000).unwrap();
            let scale = page.width as f32 / size.width;
            let (x0, y0) = ((f.rect.left * scale) as u32, (f.rect.top * scale) as u32);
            let (x1, y1) =
                (((f.rect.right * scale) as u32).min(page.width), ((f.rect.bottom * scale) as u32).min(page.height));
            let image = image::RgbaImage::from_raw(page.width, page.height, page.pixels).unwrap();
            let crop =
                image::imageops::crop_imm(&image, x0, y0, x1.saturating_sub(x0).max(1), y1.saturating_sub(y0).max(1))
                    .to_image();
            let name = format!(
                "{saved:03}-{}-p{}-{}.png",
                pdf.file_stem().unwrap().to_string_lossy().chars().take(12).collect::<String>(),
                f.page + 1,
                f.label.replace(' ', "_")
            );
            crop.save(output.join(name)).unwrap();
            saved += 1;
        }
    }
    println!("{papers} PDFs, {with_figures} with figures found, {total} figures, {saved} saved");
}

fn walk(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() { out.extend(walk(&path)) } else { out.push(path) }
    }
    out
}
