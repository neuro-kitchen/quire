//! `pdf list|open|path|text|rm`: the PDFs references have, and their text.

use clap::Args;
use quire_engine::{Engine, PdfKind, PdfMatch, SlugPath};

use super::browser;
use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    query: String,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Only this project
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Only references without a PDF
    #[arg(long)]
    missing: bool,
}

pub async fn list(context: &Context, args: &ListArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let (project, recursive) = context.project(args.project.as_ref(), args.recursive, false);
    let with_pdf = engine.references_with_pdfs().await?;
    let references = engine.list(project.as_ref(), recursive, false).await?;
    let (have, missing): (Vec<_>, Vec<_>) = references.into_iter().partition(|r| with_pdf.contains(&r.id));
    if !args.missing {
        for reference in &have {
            for (attachment, path) in engine.pdfs_of(reference.cite_key.as_str()).await?.1 {
                let kind = match attachment.pdf_kind {
                    Some(PdfKind::Scanned) => ", scanned",
                    Some(PdfKind::Mixed) => ", partly scanned",
                    _ => "",
                };
                let facts = format!("{} pages{kind}", attachment.pages.unwrap_or(0));
                anstream::println!(
                    "{}  {}  {}",
                    style::paint(style::KEY, &reference.cite_key),
                    style::paint(style::DIM, facts),
                    path.display()
                );
            }
        }
    }
    for reference in &missing {
        anstream::println!(
            "{}  {}",
            style::paint(style::KEY, &reference.cite_key),
            style::paint(style::WARN, "no PDF")
        );
    }
    output::hint(format!("{} with a PDF, {} without (`quire pdf get` finds them)", have.len(), missing.len()));
    Ok(())
}

pub async fn verify(context: &Context, args: &ListArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let (project, recursive) = context.project(args.project.as_ref(), args.recursive, false);
    let references = engine.list(project.as_ref(), recursive, false).await?;
    let checked = engine.verify_pdfs(references).await?;
    let (mut by_id, mut by_title, mut unknown, mut doubtful) = (0, 0, 0, 0);
    for (reference, _, verdict) in &checked {
        match verdict {
            PdfMatch::Identifier => by_id += 1,
            PdfMatch::Title => by_title += 1,
            PdfMatch::Unknown => unknown += 1,
            _ => {
                doubtful += 1;
                let reason = verdict.reason().unwrap_or_default();
                output::report(
                    Tone::Warn,
                    "check",
                    format!("{}  {reason}", style::paint(style::KEY, &reference.cite_key)),
                );
            }
        }
    }
    output::hint(format!(
        "{} PDFs: {by_id} confirmed by DOI/arXiv ID, {by_title} by title, {unknown} without text to compare, {doubtful} doubtful",
        checked.len()
    ));
    Ok(())
}

pub async fn open(engine: &Engine, args: &QueryArgs) -> anyhow::Result<()> {
    let (reference, pdfs) = engine.pdfs_of(&args.query).await?;
    let (_, path) = pdfs.first().ok_or_else(|| {
        anyhow::anyhow!("`{}` has no PDF; `quire pdf get {}` looks for one", reference.cite_key, reference.cite_key)
    })?;
    browser::open(&path.display().to_string())
}

pub async fn path(engine: &Engine, args: &QueryArgs) -> anyhow::Result<()> {
    let (reference, pdfs) = engine.pdfs_of(&args.query).await?;
    anyhow::ensure!(!pdfs.is_empty(), "`{}` has no PDF", reference.cite_key);
    for (_, path) in pdfs {
        println!("{}", path.display());
    }
    Ok(())
}

pub async fn text(engine: &Engine, args: &QueryArgs) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let (reference, markdown, stored) = engine.paper_text(&args.query).await?;
    print!("{markdown}");
    let how = if stored { "read from the store" } else { "converted and stored" };
    output::hint(format!("{}: {how} in {} ms", reference.cite_key, start.elapsed().as_millis()));
    Ok(())
}

pub async fn remove(engine: &Engine, args: &QueryArgs) -> anyhow::Result<()> {
    let count = engine.remove_pdfs(&args.query).await?;
    output::report(Tone::Info, "removed", format!("{count} PDF(s) from {}", args.query));
    Ok(())
}
