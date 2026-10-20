//! `ref copy`: copy references to another library. The copy keeps the reference's ID, so the
//! same work is recognisable across libraries; one already there is only linked.

use std::process::ExitCode;

use clap::Args;
use quire_engine::{Added, Engine, LibraryChoice, SlugPath};

use super::exit_code;
use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct CopyArgs {
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required = true)]
    queries: Vec<String>,
    /// The library to copy to
    #[arg(long, value_name = "NAME")]
    to: String,
    /// Put the copies into this project of the target library
    #[arg(short, long)]
    project: Option<SlugPath>,
}

pub async fn run(context: &Context, engine: &Engine, args: &CopyArgs) -> anyhow::Result<ExitCode> {
    let target = context.open(LibraryChoice::Named(args.to.clone())).await?;
    let to = style::paint(style::KEY, &args.to);
    let mut failed = 0;
    for query in &args.queries {
        match engine.copy_to(query, &target, args.project.as_ref()).await {
            Ok(Added::New(reference)) => {
                output::report(Tone::Ok, "copied", format!("{} → {to}", output::summary(&reference)))
            }
            Ok(Added::Existing(reference)) => {
                output::report(Tone::Warn, "existing", format!("{} in {to}", output::summary(&reference)))
            }
            Err(error) => {
                failed += 1;
                output::note(Tone::Error, "failed", format!("{query}: {error}"));
            }
        }
    }
    Ok(exit_code(failed, args.queries.len()))
}
