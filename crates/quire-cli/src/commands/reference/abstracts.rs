//! `ref abstracts`: fill in missing abstracts from Europe PMC, else from the paper's PDF text.
//! References that already have one are left alone.

use std::process::ExitCode;

use clap::Args;
use quire_engine::{AbstractOutcome, SlugPath};

use super::exit_code;
use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct AbstractsArgs {
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required_unless_present_any = ["project", "all"])]
    queries: Vec<String>,
    /// Every reference of this project
    #[arg(short, long, conflicts_with = "queries")]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Every reference in the library
    #[arg(long, conflicts_with_all = ["queries", "project"])]
    all: bool,
    /// Do not read abstracts from PDFs (Europe PMC only)
    #[arg(long)]
    no_pdf: bool,
    /// Show what would be filled in without storing it
    #[arg(long)]
    dry_run: bool,
}

pub async fn run(context: &Context, args: &AbstractsArgs) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    let queries: Vec<String> = if args.queries.is_empty() {
        let (project, recursive) = context.project(args.project.as_ref(), args.recursive, args.all);
        let references = engine.list(project.as_ref(), recursive, false).await?;
        let missing = references.into_iter().filter(|r| r.data.get(quire_model::TextField::Abstract).is_none());
        missing.map(|r| r.cite_key.to_string()).collect()
    } else {
        args.queries.clone()
    };
    if queries.is_empty() {
        output::hint("Every reference already has an abstract.");
        return Ok(ExitCode::SUCCESS);
    }
    let (mut filled, mut failed) = (0, 0);
    for (index, query) in queries.iter().enumerate() {
        output::hint(format!("[{}/{}] {query}", index + 1, queries.len()));
        match engine.fill_abstract(query, !args.no_pdf, args.dry_run).await {
            Ok((reference, AbstractOutcome::Found { source, text })) => {
                filled += 1;
                let label = if args.dry_run { "would fill" } else { "filled" };
                let preview = output::truncate(&text, 70);
                output::report(
                    Tone::Ok,
                    label,
                    format!(
                        "{}  {}  {}",
                        style::paint(style::KEY, &reference.cite_key),
                        style::paint(style::DIM, format!("from {source}")),
                        preview
                    ),
                );
            }
            Ok((reference, AbstractOutcome::Present)) => {
                output::report(Tone::Info, "has one", style::paint(style::KEY, &reference.cite_key));
            }
            Ok((reference, AbstractOutcome::NotFound)) => {
                output::report(Tone::Warn, "not found", style::paint(style::KEY, &reference.cite_key));
            }
            Err(error) => {
                failed += 1;
                output::note(Tone::Error, "failed", format!("{query}: {error}"));
            }
        }
    }
    output::hint(format!(
        "{filled} of {} abstracts {}",
        queries.len(),
        if args.dry_run { "found (dry run)" } else { "filled" }
    ));
    Ok(exit_code(failed, queries.len()))
}
