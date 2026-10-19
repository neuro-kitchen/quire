//! `import`: every entry of a `.bib` file, all or nothing.

use std::path::PathBuf;

use clap::Args;
use quire_engine::{Added, Engine, SlugPath};

use crate::output;

#[derive(Args, Debug)]
pub struct ImportArgs {
    /// BibTeX or BibLaTeX file
    file: PathBuf,
    /// Put the references into this project
    #[arg(short, long)]
    project: Option<SlugPath>,
}

pub async fn run(engine: &Engine, args: &ImportArgs) -> anyhow::Result<()> {
    let results = engine.import_bib(&args.file, args.project.as_ref()).await?;
    let mut new = 0;
    for added in &results {
        let label = match added {
            Added::New(_) => {
                new += 1;
                "added   "
            }
            Added::Existing(_) => "existing",
        };
        println!("{label}  {}", output::line(added.reference()));
    }
    eprintln!("{} entries: {new} new, {} already in the library", results.len(), results.len() - new);
    Ok(())
}
