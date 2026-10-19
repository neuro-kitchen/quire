//! `list`: references of a project or of the whole library, one per line (or `--json`).

use clap::Args;
use quire_engine::{Engine, SlugPath};

use crate::output;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Only this project
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Show the trash instead
    #[arg(long)]
    trash: bool,
    /// Print JSON (one array of references)
    #[arg(long)]
    json: bool,
}

pub async fn run(engine: &Engine, args: &ListArgs) -> anyhow::Result<()> {
    let references = engine.list(args.project.as_ref(), args.recursive, args.trash).await?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&references)?);
    } else {
        for reference in &references {
            println!("{}", output::line(reference));
        }
    }
    Ok(())
}
