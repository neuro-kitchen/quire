//! The reader's PDF load as the app makes it, through the Kotlin API's own functions (their
//! runtime and blocking pool), for `.knowledge/pdf-engine.md`. Not shipped.
//!
//! `cargo run --release -p quire-ffi --example scroll -- <pdf>...` (8 papers: the cache's size)
//!
//! For each paper: reading (one page at a time), then fast scrolling (the view moves a page every
//! 50 ms, two pages in view at 2×, each with its text layer for the selection overlay). Then
//! switching: a page from each paper in turn, twice round. Prints the latency of finished renders
//! (median, p90, max) and how many were cancelled by a newer request for the same page.

use std::time::{Duration, Instant};

use quire_ffi::{pdf_page_sizes, pdf_page_text, pdf_render_page};

/// Fit width on a 1200 px window at 2× (a 612 pt page).
const WIDTH: u32 = 1224;
/// How often the view moves while scrolling fast.
const SCROLL_STEP: Duration = Duration::from_millis(50);

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let papers: Vec<String> = std::env::args().skip(1).collect();
    if papers.is_empty() {
        eprintln!("usage: scroll <pdf>...");
        std::process::exit(2);
    }
    let (mut reading, mut scrolling, mut switching, mut cancelled) = (Vec::new(), Vec::new(), Vec::new(), 0);
    for paper in &papers {
        let pages = pdf_page_sizes(paper.clone()).await.expect("page sizes").len() as u32;
        for page in 0..pages.min(10) {
            reading.push(render(paper, page).await.expect("render"));
        }
        let mut requests = Vec::new();
        for top in 0..pages {
            for page in top..(top + 2).min(pages) {
                let paper = paper.clone();
                requests.push(tokio::spawn(async move {
                    let text = tokio::spawn(pdf_page_text(paper.clone(), page));
                    let rendered = render(&paper, page).await;
                    text.await.expect("task").expect("text");
                    rendered
                }));
            }
            tokio::time::sleep(SCROLL_STEP).await;
        }
        for request in requests {
            match request.await.expect("task") {
                Ok(latency) => scrolling.push(latency),
                Err(_) => cancelled += 1,
            }
        }
    }
    for round in 0..2u32 {
        for paper in &papers {
            switching.push(render(paper, round).await.expect("render"));
        }
    }
    report("reading", reading);
    report("scrolling", scrolling);
    report("switching", switching);
    println!("cancelled\t{cancelled}");
}

/// Milliseconds from request to bitmap.
async fn render(paper: &str, page: u32) -> Result<f64, String> {
    let start = Instant::now();
    pdf_render_page(paper.to_string(), page, WIDTH).await.map_err(|e| e.to_string())?;
    Ok(start.elapsed().as_secs_f64() * 1000.0)
}

fn report(name: &str, mut latencies: Vec<f64>) {
    latencies.sort_by(f64::total_cmp);
    let at = |q: f64| latencies.get(((latencies.len() as f64 - 1.0) * q).round() as usize).copied().unwrap_or(0.0);
    println!("{name}\tn {}\tp50 {:.1} ms\tp90 {:.1} ms\tmax {:.1} ms", latencies.len(), at(0.5), at(0.9), at(1.0));
}
