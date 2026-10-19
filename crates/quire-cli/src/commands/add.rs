//! `add`: one or more DOIs, arXiv IDs, PMIDs, PMCIDs or URLs, from the arguments and/or a file
//! (one per line, `#` comments). Each input is reported; failures do not stop the rest, and the
//! exit code is 3 if some failed.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use quire_engine::{Added, Engine, SlugPath};

use crate::output;

#[derive(Args, Debug)]
pub struct AddArgs {
    /// DOI, arXiv ID, PMID, PMCID or URL (several allowed)
    inputs: Vec<String>,
    /// Put the references into this project, e.g. `thesis/chapter-2`
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// Also read inputs from this file, one per line
    #[arg(long)]
    from: Option<PathBuf>,
}

/// Exit code when some inputs failed and others were added.
const PARTIAL: u8 = 3;

pub async fn run(engine: &Engine, args: &AddArgs) -> anyhow::Result<ExitCode> {
    let mut inputs = args.inputs.clone();
    if let Some(file) = &args.from {
        let text = std::fs::read_to_string(file).map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
        inputs.extend(text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(String::from));
    }
    if inputs.is_empty() {
        anyhow::bail!("nothing to add: give identifiers or URLs, or --from <file>");
    }

    let mut failed = 0;
    for input in &inputs {
        match engine.add(input, args.project.as_ref()).await {
            Ok(Added::New(reference)) => println!("added     {}", output::line(&reference)),
            Ok(Added::Existing(reference)) => println!("existing  {}", output::line(&reference)),
            Err(error) => {
                failed += 1;
                eprintln!("failed    {input}: {error}");
            }
        }
    }
    Ok(match failed {
        0 => ExitCode::SUCCESS,
        n if n == inputs.len() => ExitCode::FAILURE,
        _ => ExitCode::from(PARTIAL),
    })
}
