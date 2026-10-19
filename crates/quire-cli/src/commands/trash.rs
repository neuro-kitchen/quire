//! `rm` / `restore`: move a reference to the trash or back. Project links are kept, so a
//! restored reference is back where it was.

use clap::Args;
use quire_engine::Engine;

#[derive(Args, Debug)]
pub struct TrashArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    query: String,
}

pub async fn run(engine: &Engine, args: &TrashArgs, to_trash: bool) -> anyhow::Result<()> {
    let reference = if to_trash { engine.trash(&args.query).await? } else { engine.restore(&args.query).await? };
    let state = if to_trash { "moved to the trash" } else { "restored" };
    println!("{} {state}", reference.cite_key);
    Ok(())
}
