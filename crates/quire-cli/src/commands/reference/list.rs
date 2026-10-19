//! `list`: references of a project or of the whole library, as a table (`--json` for scripts).

use clap::Args;
use quire_engine::{PdfAttempt, PdfStatus, SlugPath};

use crate::context::Context;

use crate::output::table::{self, Column, DEFAULT_COLUMNS};
use crate::output::{self, terminal};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Only this project
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Only references carrying this tag (repeat for several: all must match)
    #[arg(short, long = "tag", value_name = "TAG")]
    tags: Vec<String>,
    /// The whole library, even inside a folder with quire.toml
    #[arg(long, conflicts_with = "project")]
    all: bool,
    /// Show the trash instead
    #[arg(long)]
    trash: bool,
    /// Columns to show, comma-separated
    #[arg(short, long, value_enum, value_delimiter = ',')]
    columns: Vec<Column>,
    /// Print JSON (one array of references)
    #[arg(long, conflicts_with = "columns")]
    json: bool,
}

pub async fn run(context: &Context, args: &ListArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let (project, recursive) = context.project(args.project.as_ref(), args.recursive, args.all);
    if args.json {
        let references = engine.list_tagged(project.as_ref(), recursive, args.trash, &args.tags).await?;
        println!("{}", serde_json::to_string_pretty(&references)?);
        return Ok(());
    }
    let references = engine.list_with_issues(project.as_ref(), recursive, args.trash, &args.tags).await?;
    if references.is_empty() {
        output::hint(if args.trash { "The trash is empty." } else { "No references." });
        return Ok(());
    }
    let columns = if args.columns.is_empty() { DEFAULT_COLUMNS } else { &args.columns };
    let pdfs = engine.pdf_statuses().await?;
    let tags = if columns.contains(&Column::Tags) {
        engine.tags_of(references.iter().map(|c| c.reference.id).collect()).await?
    } else {
        Default::default()
    };
    let extras = table::Extras { pdfs, tags };
    anstream::println!("{}", table::references(&references, columns, &extras));
    let pdfs = extras.pdfs;
    if terminal::stdout_is_terminal() {
        let count = references.len();
        let with_issues = references.iter().filter(|c| !c.issues.is_empty()).count();
        let issues =
            if with_issues > 0 { format!(", {with_issues} with issues (`quire ref check`)") } else { String::new() };
        output::hint(format!("{count} reference{}{issues}", if count == 1 { "" } else { "s" }));
        output::hint(pdf_summary(&references, &pdfs));
    }
    Ok(())
}

/// `PDFs: 4 available, 1 blocked, 2 not found, 1 never tried (`quire pdf get`)`.
fn pdf_summary(
    references: &[quire_engine::Checked],
    pdfs: &std::collections::HashMap<quire_model::ReferenceId, PdfStatus>,
) -> String {
    let mut counts = [0usize; 6];
    for checked in references {
        let slot = match pdfs.get(&checked.reference.id) {
            Some(PdfStatus::Available { .. }) => 0,
            Some(PdfStatus::Missing(PdfAttempt::Blocked)) => 1,
            Some(PdfStatus::Missing(PdfAttempt::NotFound)) => 2,
            Some(PdfStatus::Missing(PdfAttempt::NoIdentifier)) => 3,
            Some(PdfStatus::Missing(PdfAttempt::Error)) => 4,
            None => 5,
        };
        counts[slot] += 1;
    }
    let names = ["available", "blocked", "not found", "no DOI", "error", "never tried"];
    let parts: Vec<String> =
        counts.iter().zip(names).filter(|(n, _)| **n > 0).map(|(n, name)| format!("{n} {name}")).collect();
    let next = if counts[5] > 0 {
        " (`quire pdf get --all`)"
    } else if counts[1] > 0 {
        " (`quire pdf get --blocked`)"
    } else {
        ""
    };
    format!("PDFs: {}{next}", parts.join(", "))
}
