//! `import`: every entry of a `.bib` file, all or nothing.

use std::path::PathBuf;

use clap::Args;
use quire_engine::{Added, Engine, SlugPath};

use crate::output::{self, Tone};

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
        match added {
            Added::New(reference) => {
                new += 1;
                output::report(Tone::Ok, "added", output::summary(reference));
            }
            Added::Existing(reference) => output::report(Tone::Warn, "existing", output::summary(reference)),
        }
    }
    output::hint(format!("{} entries: {new} new, {} already in the library", results.len(), results.len() - new));
    Ok(())
}
