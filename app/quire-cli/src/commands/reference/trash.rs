//! `ref rm` / `ref restore`: move references to the trash or back. Project links are kept, so a
//! restored reference is back where it was.

use std::process::ExitCode;

use clap::Args;
use quire_engine::Engine;

use super::exit_code;
use crate::output::{self, Tone};

#[derive(Args, Debug)]
pub struct TrashArgs {
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required = true)]
    queries: Vec<String>,
}

pub async fn run(engine: &Engine, args: &TrashArgs, to_trash: bool) -> anyhow::Result<ExitCode> {
    let mut failed = 0;
    for query in &args.queries {
        let result = if to_trash { engine.trash(query).await } else { engine.restore(query).await };
        match result {
            Ok(reference) if to_trash => output::report(Tone::Info, "trashed", output::summary(&reference)),
            Ok(reference) => output::report(Tone::Ok, "restored", output::summary(&reference)),
            Err(error) => {
                failed += 1;
                output::note(Tone::Error, "failed", format!("{query}: {error}"));
            }
        }
    }
    Ok(exit_code(failed, args.queries.len()))
}
