//! `pdf <action>`: PDFs of references (`.knowledge/roadmap.md`, M7).
//!
//! - [`get`]: find and download; `--browser` for publishers that block automated downloads.
//! - [`import`]: attach a PDF file, or add the paper it is.
//! - [`files`]: `list`, `open`, `path`, `text`, `extract`, `rm`.
//! - [`browser`]: opening pages and files, watching the downloads folder.

pub mod browser;
pub mod files;
pub mod get;
pub mod import;
pub mod ocr;

use std::process::ExitCode;

use clap::{Args, Subcommand};

use crate::context::Context;

#[derive(Args, Debug)]
pub struct PdfArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Find and download PDFs (open access, arXiv, publisher pages; --browser for blocked ones)
    Get(get::GetArgs),
    /// Attach PDF files, or add the papers they are (by the DOI or arXiv ID they print)
    Import(import::ImportArgs),
    /// List references with (or without) a PDF
    List(files::ListArgs),
    /// Open a reference's PDF
    Open(files::QueryArgs),
    /// Print the path of a reference's PDF
    Path(files::QueryArgs),
    /// Print the paper's text as reader mode shows it (arXiv / Europe PMC full text, else the PDF's)
    Text(files::TextArgs),
    /// Extract papers' text again (after a quire update, or to retry online sources)
    Extract(files::ExtractArgs),
    /// Re-check attached PDFs against their references (DOI or title); changes nothing
    Verify(files::ListArgs),
    /// OCR of scanned PDFs (OCRmyPDF, or Tesseract with poppler, when installed)
    Ocr(ocr::OcrArgs),
    /// Remove a reference's PDFs (the files stay in the store)
    Rm(files::QueryArgs),
}

pub async fn run(context: &Context, args: &PdfArgs) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    let done = Ok(ExitCode::SUCCESS);
    match &args.action {
        Action::Get(args) => get::run(context, args).await,
        Action::Import(args) => import::run(context, args).await,
        Action::List(args) => files::list(context, args).await.and(done),
        Action::Open(args) => files::open(engine, args).await.and(done),
        Action::Path(args) => files::path(engine, args).await.and(done),
        Action::Text(args) => files::text(engine, args).await.and(done),
        Action::Extract(args) => files::extract(context, args).await.and(done),
        Action::Verify(args) => files::verify(context, args).await.and(done),
        Action::Ocr(args) => ocr::run(engine, args).await,
        Action::Rm(args) => files::remove(engine, args).await.and(done),
    }
}
