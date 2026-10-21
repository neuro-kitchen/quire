//! `pdf list|open|path|text|extract|rm`: the PDFs references have, and their text.

use clap::Args;
use quire_engine::paper_markdown::{FRONT_START, caption_label};
use quire_engine::{Engine, PdfKind, PdfMatch, SlugPath};

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
                let facts = format!("{}{kind}", output::count(attachment.pages.unwrap_or(0), "page", "pages"));
                crate::out!(
                    "{}  {}  {}",
                    style::paint(style::KEY, &reference.cite_key),
                    style::paint(style::DIM, facts),
                    path.display()
                );
            }
        }
    }
    for reference in &missing {
        crate::out!("{}  {}", style::paint(style::KEY, &reference.cite_key), style::paint(style::WARN, "no PDF"));
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

/// Opens the PDF, fetching it first if it is not on this device (the hub, another of your
/// devices, then the source: lazy PDFs, `.knowledge/sync-hub.md`).
pub async fn open(engine: &Engine, args: &QueryArgs) -> anyhow::Result<()> {
    let opened = engine.open_pdf(&args.query).await?;
    match &opened.source {
        quire_engine::PdfSource::Here => {}
        quire_engine::PdfSource::Hub => output::hint("downloaded from your sync hub"),
        quire_engine::PdfSource::Peer(name) => output::hint(format!("fetched from your device {name}")),
        quire_engine::PdfSource::Found(source) => output::hint(format!("found again ({source}) and attached")),
    }
    Ok(quire_engine::tools::open(&opened.path.display().to_string())?)
}

#[derive(Args, Debug)]
pub struct KeepArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID; with --project, a project path
    target: String,
    /// TARGET is a project (its subprojects too)
    #[arg(long)]
    project: bool,
    /// Stop keeping it downloaded
    #[arg(long)]
    off: bool,
}

/// Keeps a paper's (or a project's) PDFs downloaded on this device, whatever `files.download`.
pub async fn keep(engine: &Engine, args: &KeepArgs) -> anyhow::Result<()> {
    let changed = if args.project {
        engine.keep_project_offline(&args.target.parse()?, !args.off).await?
    } else {
        engine.keep_reference_offline(&args.target, !args.off).await?
    };
    let what = if args.off { "no longer kept offline" } else { "kept offline (downloaded at the next sync)" };
    output::report(
        if changed { Tone::Ok } else { Tone::Info },
        if changed { "done" } else { "unchanged" },
        format!("{}: {what}", args.target),
    );
    Ok(())
}

/// Removes PDFs past `files.cache_mb` (only those your sync hub holds; never one kept offline).
pub async fn evict(engine: &Engine) -> anyhow::Result<()> {
    let eviction = engine.evict_pdfs().await?;
    output::report(
        Tone::Ok,
        "evicted",
        format!(
            "{} PDF(s), {} MB freed; {} MB of PDFs here",
            eviction.removed,
            eviction.freed / (1024 * 1024),
            eviction.kept / (1024 * 1024)
        ),
    );
    Ok(())
}

pub async fn path(engine: &Engine, args: &QueryArgs) -> anyhow::Result<()> {
    let (reference, pdfs) = engine.pdfs_of(&args.query).await?;
    anyhow::ensure!(!pdfs.is_empty(), "`{}` has no PDF", reference.cite_key);
    for (_, path) in pdfs {
        crate::out!("{}", path.display());
    }
    Ok(())
}

#[derive(Args, Debug)]
pub struct TextArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    query: String,
    /// Extract again instead of reading the stored text (after a quire update, or to retry online)
    #[arg(long)]
    refresh: bool,
}

/// The text reader mode shows: arXiv's or Europe PMC's full text, else the PDF's.
pub async fn text(engine: &Engine, args: &TextArgs) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    if args.refresh {
        engine.forget_text(&args.query).await?;
    }
    let (reference, text) = engine.reader_text(&args.query).await?;
    crate::data!("{}", text.markdown);
    output::hint(format!("{}: {} text in {} ms", reference.cite_key, text.source.label(), start.elapsed().as_millis()));
    Ok(())
}

#[derive(Args, Debug)]
pub struct ExtractArgs {
    /// References (cite keys, DOIs, IDs); none: all with a PDF or a stored text
    refs: Vec<String>,
    /// Only this project
    #[arg(short, long, conflicts_with = "refs")]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
}

/// Extracts the reader text again (online full text or PDF conversion) and reports what came
/// out: source, words, sections, captions, front matter.
pub async fn extract(context: &Context, args: &ExtractArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let queries: Vec<String> = if args.refs.is_empty() {
        let (project, recursive) = context.project(args.project.as_ref(), args.recursive, false);
        let with_pdf = engine.references_with_pdfs().await?;
        let mut queries = Vec::new();
        for reference in engine.list(project.as_ref(), recursive, false).await? {
            // A stored text without a PDF (an online full text) is fetched again too.
            let stored = engine.forget_text(&reference.id.to_string()).await? > 0;
            if stored || with_pdf.contains(&reference.id) {
                queries.push(reference.cite_key.to_string());
            }
        }
        queries
    } else {
        args.refs.clone()
    };
    let (mut done, mut failed) = (0, 0);
    for query in &queries {
        engine.forget_text(query).await?;
        match engine.reader_text(query).await {
            Ok((reference, text)) => {
                done += 1;
                let stats = TextStats::of(&text.markdown);
                crate::out!(
                    "{}  {}",
                    style::paint(style::KEY, &reference.cite_key),
                    style::paint(style::DIM, format!("{}  {stats}", text.source.label()))
                );
            }
            Err(error) => {
                failed += 1;
                output::report(Tone::Warn, "failed", format!("{}  {error:#}", style::paint(style::KEY, query)));
            }
        }
    }
    output::hint(format!("{} extracted, {failed} failed", output::count(done, "text", "texts")));
    Ok(())
}

/// What an extracted text contains, for judging the extraction.
struct TextStats {
    words: usize,
    sections: usize,
    captions: usize,
    front: bool,
}

impl TextStats {
    fn of(markdown: &str) -> Self {
        let blocks: Vec<&str> = markdown.split("\n\n").map(str::trim).collect();
        Self {
            words: markdown.split_whitespace().count(),
            sections: blocks.iter().filter(|b| b.starts_with('#')).count(),
            captions: blocks.iter().filter(|b| caption_label(b).is_some()).count(),
            front: markdown.contains(FRONT_START),
        }
    }
}

impl std::fmt::Display for TextStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} words, {} sections, {} captions", self.words, self.sections, self.captions)?;
        if self.front {
            f.write_str(", front matter")?;
        }
        Ok(())
    }
}

pub async fn remove(engine: &Engine, args: &QueryArgs) -> anyhow::Result<()> {
    let count = engine.remove_pdfs(&args.query).await?;
    output::report(Tone::Info, "removed", format!("{count} PDF(s) from {}", args.query));
    Ok(())
}
