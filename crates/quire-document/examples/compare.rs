//! The two PDF text extractors on one whole PDF, against `pdftotext`, for the decision recorded
//! in `.knowledge/pdf-engine.md` (one extractor or two). Not shipped.
//!
//! `cargo run --release -p quire-document --example compare -- <pdf>` (or `--header`)
//!
//! Prints one tab-separated line: file, pages; for the Text view's Markdown (`pdf::to_markdown`:
//! pdf-inspector plus quire's cleanup) its time, word F1, word-pair F1, headings and whether it
//! found a references heading; then the same scores and time for the text layer (`page_text` on
//! every page) and for the layer's own reader Markdown (`pdf::layer_to_markdown`: words, pairs,
//! headings, references); how much the two Markdowns agree (words, pairs); then a status.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use quire_document::paper_markdown::is_references_title;
use quire_document::pdf;

mod common;
use common::{f1, pairs, words};

const HEADER: &str = "file\tpages\tmd_ms\tmd_word_f1\tmd_pair_f1\tmd_headings\tmd_references\tlayer_ms\tlayer_word_f1\tlayer_pair_f1\tlmd_word_f1\tlmd_pair_f1\tlmd_headings\tlmd_references\tagree_word\tagree_pair\tstatus";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--header") {
        println!("{HEADER}");
        return;
    }
    let Some(path) = args.first().map(Path::new) else {
        eprintln!("usage: compare <pdf> | compare --header");
        std::process::exit(2);
    };
    let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let row = std::panic::catch_unwind(|| compare(path))
        .unwrap_or_else(|_| Err("panic".into()))
        .unwrap_or_else(|e| format!("\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t{}", e.replace(['\t', '\n'], " ")));
    println!("{name}\t{row}");
}

fn compare(path: &Path) -> Result<String, String> {
    let reference =
        Command::new("pdftotext").args(["-enc", "UTF-8"]).arg(path).arg("-").output().map_err(|e| e.to_string())?;
    let reference = String::from_utf8_lossy(&reference.stdout).into_owned();
    let (ref_words, ref_pairs) = (words(&reference), pairs(&reference));

    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let start = Instant::now();
    let markdown = pdf::to_markdown(&bytes).unwrap_or_default();
    let md_ms = start.elapsed().as_secs_f64() * 1000.0;
    let (headings, references) = structure(&markdown);
    let layer_markdown = pdf::layer_to_markdown(&bytes).unwrap_or_default();
    let (layer_headings, layer_references) = structure(&layer_markdown);

    let pages = pdf::page_sizes(path).map_err(|e| e.to_string())?.len() as u32;
    let start = Instant::now();
    let mut layer = String::new();
    for page in 0..pages {
        layer.push_str(&pdf::page_text(path, page).map_err(|e| e.to_string())?.text);
    }
    let layer_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(format!(
        "{pages}\t{md_ms:.0}\t{:.3}\t{:.3}\t{headings}\t{references}\t{layer_ms:.0}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{layer_headings}\t{layer_references}\t{:.3}\t{:.3}\tok",
        f1(&words(&markdown), &ref_words),
        f1(&pairs(&markdown), &ref_pairs),
        f1(&words(&layer), &ref_words),
        f1(&pairs(&layer), &ref_pairs),
        f1(&words(&layer_markdown), &ref_words),
        f1(&pairs(&layer_markdown), &ref_pairs),
        f1(&words(&markdown), &words(&layer_markdown)),
        f1(&pairs(&markdown), &pairs(&layer_markdown)),
    ))
}

/// How many headings Markdown has, and whether one is a References heading.
fn structure(markdown: &str) -> (usize, bool) {
    let headings: Vec<&str> =
        markdown.lines().filter_map(|l| l.strip_prefix('#')).map(|h| h.trim_start_matches('#').trim()).collect();
    (headings.len(), headings.iter().any(|h| is_references_title(h)))
}
