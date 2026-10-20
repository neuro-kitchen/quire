//! `ref refetch`: look references up again and update what came from a source. Fields you
//! edited are kept (and shown when the source differs). `--dry-run` only shows the changes.

use std::process::ExitCode;

use clap::Args;
use quire_engine::SlugPath;

use super::exit_code;
use crate::context::Context;
use crate::output::{self, Tone, changes};

#[derive(Args, Debug)]
pub struct RefetchArgs {
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required_unless_present = "project")]
    queries: Vec<String>,
    /// Every reference of this project
    #[arg(short, long, conflicts_with = "queries")]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Show what would change without changing anything
    #[arg(long)]
    dry_run: bool,
}

pub async fn run(context: &Context, args: &RefetchArgs) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    let queries = match &args.project {
        Some(project) => engine
            .list(Some(project), args.recursive, false)
            .await?
            .into_iter()
            .map(|r| r.cite_key.to_string())
            .collect(),
        None => args.queries.clone(),
    };
    let mut failed = 0;
    for query in &queries {
        match engine.refetch(query, args.dry_run).await {
            Ok(updated) => {
                let (tone, label) = match (updated.changed(), args.dry_run) {
                    (false, _) => (Tone::Info, "unchanged"),
                    (true, true) => (Tone::Warn, "would update"),
                    (true, false) => (Tone::Ok, "updated"),
                };
                output::report(tone, label, output::summary(&updated.reference));
                for line in changes::lines(&updated) {
                    anstream::println!("{line}");
                }
            }
            Err(error) => {
                failed += 1;
                output::note(Tone::Error, "failed", format!("{query}: {error}"));
            }
        }
    }
    Ok(exit_code(failed, queries.len().max(1)))
}
