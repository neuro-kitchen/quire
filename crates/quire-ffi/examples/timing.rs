//! Times the bridge calls the desktop app makes: opening, the first
//! queries, opening a paper, PDF pages. `cargo run --release -p quire-ffi --example timing -- <library> <config>`
use std::time::Instant;

use quire_ffi::{Query, SortColumn, Source, open_library_at, pdf_find, pdf_page_sizes, pdf_render_page};

fn ms(start: Instant) -> String {
    format!("{:>8.1} ms", start.elapsed().as_secs_f64() * 1000.0)
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let (library, config) = (args.next().unwrap(), args.next().unwrap());
    let t = Instant::now();
    let lib = open_library_at(library, Some(config)).await.unwrap();
    println!("open library            {}", ms(t));
    let query =
        Query { source: Source::All, text: String::new(), tags: vec![], sort: SortColumn::DateAdded, ascending: false };
    for round in 1..=3 {
        let t = Instant::now();
        let refs = lib.references(query.clone()).await.unwrap();
        println!("references #{round} ({:>3})    {}", refs.len(), ms(t));
    }
    let t = Instant::now();
    lib.projects().await.unwrap();
    println!("projects                {}", ms(t));
    let t = Instant::now();
    lib.tags().await.unwrap();
    println!("tags                    {}", ms(t));
    let refs = lib.references(query).await.unwrap();
    let with_pdf: Vec<_> = refs.iter().filter(|r| r.has_pdf).take(3).collect();
    for r in &with_pdf {
        let t = Instant::now();
        let paper = lib.paper(r.id.clone()).await;
        println!("paper (reader tabs)     {}  {}", ms(t), if paper.is_ok() { "ok" } else { "error" });
        let files = lib.pdf_files(r.id.clone()).await.unwrap();
        let Some(path) = files.first().cloned() else { continue };
        let t = Instant::now();
        pdf_find(path.clone(), vec!["the results".into(), "we show".into()]).await.unwrap();
        println!("  locate 2 highlights     {}", ms(t));
        let t = Instant::now();
        let sizes = pdf_page_sizes(path.clone()).await.unwrap();
        println!("  page sizes ({:>3} pages) {}", sizes.len(), ms(t));
        for width in [900u32, 1400, 2800] {
            let t = Instant::now();
            let page = pdf_render_page(path.clone(), 0, width).await.unwrap();
            println!("  render p1 @{width:>4}px     {}  {:>5.1} MB of pixels", ms(t), page.pixels.len() as f64 / 1e6);
        }
        let t = Instant::now();
        pdf_render_page(path.clone(), 0, 1400).await.unwrap();
        println!("  render p1 @1400 again   {}", ms(t));
        if sizes.len() > 1 {
            let t = Instant::now();
            pdf_render_page(path.clone(), 1, 1400).await.unwrap();
            println!("  render p2 @1400         {}", ms(t));
        }
    }
}
