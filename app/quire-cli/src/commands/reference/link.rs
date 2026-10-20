//! `ref link` / `ref unlink`: put references into a project or take them out (they stay in the
//! library).

use std::process::ExitCode;

use clap::Args;
use quire_engine::{Engine, SlugPath};

use super::exit_code;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct LinkArgs {
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required = true)]
    queries: Vec<String>,
    /// The project, e.g. `thesis/chapter-2`
    #[arg(short, long)]
    project: SlugPath,
}

pub async fn run(engine: &Engine, args: &LinkArgs, link: bool) -> anyhow::Result<ExitCode> {
    let project = style::paint(style::KEY, &args.project);
    let mut failed = 0;
    for query in &args.queries {
        let result =
            if link { engine.link(query, &args.project).await } else { engine.unlink(query, &args.project).await };
        match result {
            Ok((reference, changed)) => {
                let key = style::paint(style::KEY, &reference.cite_key);
                match (link, changed) {
                    (true, true) => output::report(Tone::Ok, "linked", format!("{key} → {project}")),
                    (true, false) => output::report(Tone::Warn, "unchanged", format!("{key} is already in {project}")),
                    (false, true) => output::report(Tone::Ok, "unlinked", format!("{key} from {project}")),
                    (false, false) => output::report(Tone::Warn, "unchanged", format!("{key} was not in {project}")),
                }
            }
            Err(error) => {
                failed += 1;
                output::note(Tone::Error, "failed", format!("{query}: {error}"));
            }
        }
    }
    Ok(exit_code(failed, args.queries.len()))
}
