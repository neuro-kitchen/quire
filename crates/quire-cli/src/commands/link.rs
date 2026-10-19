//! `link` / `unlink`: put a reference into a project or take it out (it stays in the library).

use clap::Args;
use quire_engine::{Engine, SlugPath};

#[derive(Args, Debug)]
pub struct LinkArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    query: String,
    /// The project, e.g. `thesis/chapter-2`
    #[arg(short, long)]
    project: SlugPath,
}

pub async fn run(engine: &Engine, args: &LinkArgs, link: bool) -> anyhow::Result<()> {
    let (reference, changed) = if link {
        engine.link(&args.query, &args.project).await?
    } else {
        engine.unlink(&args.query, &args.project).await?
    };
    let message = match (link, changed) {
        (true, true) => "linked to",
        (true, false) => "already in",
        (false, true) => "unlinked from",
        (false, false) => "was not in",
    };
    println!("{} {message} {}", reference.cite_key, args.project);
    Ok(())
}
