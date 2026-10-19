//! `export`: references of a project (or the library) to stdout or a file.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use quire_engine::{Engine, ExportFormat, SlugPath};

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Only this project
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Write to this file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Biblatex)]
    format: Format,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    /// BibLaTeX (Biber)
    Biblatex,
    /// Classic BibTeX
    Bibtex,
    /// CSL-JSON (Pandoc, Zotero)
    CslJson,
}

pub async fn run(engine: &Engine, args: &ExportArgs) -> anyhow::Result<()> {
    let format = match args.format {
        Format::Biblatex => ExportFormat::Biblatex,
        Format::Bibtex => ExportFormat::Bibtex,
        Format::CslJson => ExportFormat::CslJson,
    };
    let (text, count) = engine.export(args.project.as_ref(), args.recursive, format).await?;
    match &args.output {
        Some(path) => {
            std::fs::write(path, &text).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
            eprintln!("{count} references written to {}", path.display());
        }
        None => print!("{text}"),
    }
    Ok(())
}
