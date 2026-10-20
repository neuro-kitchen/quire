//! `pdf ocr <refs…>`: OCR of scanned PDFs (pages that are images only). OCRmyPDF, when
//! installed, gives the PDF a text layer; else Tesseract (with poppler's pdftoppm) reads the pages
//! into the paper's text. Language: `reader.ocr_language` (Tesseract codes, default eng).

use std::process::ExitCode;

use clap::Args;
use quire_engine::{Engine, OcrOutcome};

use crate::output::{self, Tone};

#[derive(Args, Debug)]
pub struct OcrArgs {
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required = true)]
    queries: Vec<String>,
}

pub async fn run(engine: &Engine, args: &OcrArgs) -> anyhow::Result<ExitCode> {
    let mut failed = 0;
    for query in &args.queries {
        match engine.ocr(query).await {
            Ok((reference, OcrOutcome::TextLayer)) => {
                output::report(Tone::Ok, "ocr", format!("{}: the PDF has a text layer now", reference.cite_key));
            }
            Ok((reference, OcrOutcome::Text { words })) => {
                output::report(Tone::Ok, "ocr", format!("{}: {words} words read", reference.cite_key));
            }
            Err(error) => {
                failed += 1;
                output::report(Tone::Error, "failed", format!("{query}: {error}"));
            }
        }
    }
    Ok(crate::commands::reference::exit_code(failed, args.queries.len()))
}
