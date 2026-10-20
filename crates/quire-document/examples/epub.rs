//! An EPUB through `epub::to_markdown`, for checking real books. Not shipped.
//!
//! `cargo run --release -p quire-document --example epub -- <book.epub> [--print]`
//!
//! Prints the time, size, words, headings and the first headings; `--print` prints the Markdown.

use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("usage: epub <book.epub> [--print]");
        std::process::exit(2);
    };
    let bytes = std::fs::read(path).expect("readable file");
    let start = Instant::now();
    let markdown = match quire_document::epub::to_markdown(&bytes) {
        Ok(markdown) => markdown,
        Err(e) => {
            println!("{path}\terror: {e}");
            return;
        }
    };
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    if args.iter().any(|a| a == "--print") {
        print!("{markdown}");
        return;
    }
    let headings: Vec<&str> = markdown.lines().filter(|l| l.starts_with('#')).collect();
    println!(
        "{path}\t{ms:.0} ms\t{} KB\t{} words\t{} headings: {:?}",
        markdown.len() / 1024,
        markdown.split_whitespace().count(),
        headings.len(),
        &headings[..headings.len().min(4)]
    );
}
