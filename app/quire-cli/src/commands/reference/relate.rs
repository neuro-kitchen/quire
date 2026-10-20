//! `ref relate` / `ref unrelate`: mark two references as related (both ways), as Zotero's related
//! items.

use clap::Args;
use quire_engine::Engine;

use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct RelateArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    first: String,
    /// The other reference
    second: String,
}

pub async fn run(engine: &Engine, args: &RelateArgs, relate: bool) -> anyhow::Result<()> {
    let (a, b, changed) = if relate {
        engine.relate(&args.first, &args.second).await?
    } else {
        engine.unrelate(&args.first, &args.second).await?
    };
    let pair = format!("{} ↔ {}", style::paint(style::KEY, &a.cite_key), style::paint(style::KEY, &b.cite_key));
    match (relate, changed) {
        (true, true) => output::report(Tone::Ok, "related", pair),
        (false, true) => output::report(Tone::Ok, "unrelated", pair),
        (true, false) => output::report(Tone::Warn, "unchanged", format!("{pair} already related")),
        (false, false) => output::report(Tone::Warn, "unchanged", format!("{pair} were not related")),
    }
    Ok(())
}
