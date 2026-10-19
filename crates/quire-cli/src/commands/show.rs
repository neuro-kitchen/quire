//! `show`: one reference by cite key, identifier or ID.

use clap::Args;
use quire_engine::Engine;

use crate::output;

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID (ref_…)
    query: String,
    /// Print JSON
    #[arg(long)]
    json: bool,
}

pub async fn run(engine: &Engine, args: &ShowArgs) -> anyhow::Result<()> {
    let (reference, projects) = engine.show(&args.query).await?;
    let projects: Vec<String> = projects.iter().map(ToString::to_string).collect();
    if args.json {
        let value = serde_json::json!({ "reference": reference, "projects": projects });
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        println!("{}", output::block(&reference, &projects));
    }
    Ok(())
}
