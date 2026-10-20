//! `ref cites | cited-by | similar`: a reference's neighbours in the citation graph (OpenAlex),
//! marked `●` when already in the library; `--add` adds the others by DOI.

use std::process::ExitCode;

use clap::Args;
use quire_engine::{GraphEntry, GraphKind, SlugPath};

use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct GraphArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    query: String,
    /// Add the works not in the library yet (by DOI)
    #[arg(long)]
    add: bool,
    /// With --add: put them into this project
    #[arg(short, long, requires = "add")]
    project: Option<SlugPath>,
    /// Most works listed (citing works: the most cited first)
    #[arg(long, default_value_t = 25)]
    limit: usize,
    /// Print JSON
    #[arg(long)]
    json: bool,
}

pub async fn run(context: &Context, args: &GraphArgs, kind: GraphKind) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    let (reference, mut entries) = engine.graph(&args.query, kind, args.limit).await?;
    entries.truncate(args.limit);
    if args.json {
        let list: Vec<_> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "openalex": e.work.id, "doi": e.work.doi, "title": e.work.title, "year": e.work.year,
                    "first_author": e.work.first_author, "cited_by": e.work.cited_by,
                    "cite_key": e.in_library.as_ref().map(|r| r.cite_key.to_string()),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(ExitCode::SUCCESS);
    }
    let what = match kind {
        GraphKind::References => "cites",
        GraphKind::Citing => "is cited by",
        GraphKind::Related => "is related to",
    };
    output::hint(format!("{} {what} {} works (OpenAlex)", reference.cite_key, entries.len()));
    for entry in &entries {
        anstream::println!("{}", line(entry));
    }
    if !args.add {
        let missing = entries.iter().filter(|e| e.in_library.is_none() && e.work.doi.is_some()).count();
        if missing > 0 {
            output::hint(format!("{missing} not in the library: add them with --add"));
        }
        return Ok(ExitCode::SUCCESS);
    }
    let (mut failed, mut tried) = (0, 0);
    for entry in entries.iter().filter(|e| e.in_library.is_none()) {
        let Some(doi) = &entry.work.doi else { continue };
        tried += 1;
        match engine.add(doi, args.project.as_ref()).await {
            Ok(added) => output::report(Tone::Ok, "added", output::summary(added.reference())),
            Err(error) => {
                failed += 1;
                output::report(Tone::Error, "failed", format!("{doi}: {error}"));
            }
        }
    }
    Ok(super::exit_code(failed, tried))
}

fn line(entry: &GraphEntry) -> String {
    let work = &entry.work;
    let mark = if entry.in_library.is_some() { style::paint(style::OK, "●") } else { " ".into() };
    let key =
        entry.in_library.as_ref().map_or_else(|| work.doi.clone().unwrap_or_default(), |r| r.cite_key.to_string());
    format!(
        "{mark} {:<4}  {:<20}  {}  {}",
        work.year.map(|y| y.to_string()).unwrap_or_default(),
        work.first_author.as_deref().unwrap_or("").chars().take(20).collect::<String>(),
        work.title.as_deref().unwrap_or("(no title)"),
        style::paint(style::DIM, key),
    )
}
