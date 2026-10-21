//! `import`: every entry of a `.bib` file (all or nothing), or a whole Zotero library (its data
//! folder, `~/Zotero` by default in Zotero).

use std::path::{Path, PathBuf};

use clap::Args;
use quire_engine::{Added, Engine, SlugPath};

use crate::output::{self, Tone};

#[derive(Args, Debug)]
pub struct ImportArgs {
    /// BibTeX or BibLaTeX file, or a Zotero data folder (with zotero.sqlite)
    file: PathBuf,
    /// Put the references into this project (Zotero: its collections go under it)
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// Zotero: the folder linked attachments are relative to ("Linked Attachment Base Directory")
    #[arg(long)]
    base: Option<PathBuf>,
}

pub async fn run(engine: &Engine, args: &ImportArgs) -> anyhow::Result<()> {
    if let Some(folder) = zotero_folder(&args.file) {
        return zotero(engine, &folder, args).await;
    }
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
    output::hint(format!(
        "{}: {new} new, {} already in the library",
        output::count(results.len(), "entry", "entries"),
        results.len() - new
    ));
    Ok(())
}

/// The Zotero data folder `path` names: the folder holding `zotero.sqlite`, or that file.
fn zotero_folder(path: &Path) -> Option<PathBuf> {
    let database = quire_engine::zotero::DATABASE;
    if path.is_dir() && path.join(database).is_file() {
        return Some(path.to_path_buf());
    }
    (path.file_name()? == database).then(|| path.parent().map(Path::to_path_buf)).flatten()
}

async fn zotero(engine: &Engine, folder: &Path, args: &ImportArgs) -> anyhow::Result<()> {
    output::hint(format!("reading {} (a copy: Zotero may stay open)", folder.display()));
    let report = engine.import_zotero(folder, args.project.as_ref(), args.base.as_deref()).await?;
    for reference in &report.added {
        output::report(Tone::Ok, "added", output::summary(reference));
    }
    for line in &report.skipped {
        output::report(Tone::Warn, "skipped", line);
    }
    output::hint(format!(
        "{} new, {} already here; {}, {}, {}, {}",
        report.added.len(),
        report.existing,
        output::count(report.projects, "project", "projects"),
        output::count(report.notes, "note", "notes"),
        output::count(report.pdfs, "PDF", "PDFs"),
        output::count(report.highlights, "highlight", "highlights")
    ));
    Ok(())
}
