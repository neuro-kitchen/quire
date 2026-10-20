//! `pdf import`: attach PDF files to a reference (`--to`), or add each paper by the DOI or arXiv
//! ID printed on its first pages and attach the file to it.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use quire_engine::{Added, SlugPath};

use super::get::report_attached;
use crate::commands::reference::exit_code;
use crate::context::Context;
use crate::output::{self, Tone};

#[derive(Args, Debug)]
pub struct ImportArgs {
    /// PDF files
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Attach to this reference instead of looking the paper up
    #[arg(long, value_name = "REFERENCE")]
    to: Option<String>,
    /// Put new references into this project
    #[arg(short, long)]
    project: Option<SlugPath>,
}

pub async fn run(context: &Context, args: &ImportArgs) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    let (project, _) = context.project(args.project.as_ref(), false, false);
    let mut failed = 0;
    for file in &args.files {
        let from = format!("from {}", file.display());
        let result = match &args.to {
            Some(query) => engine.attach_file(query, file, false).await.map(|attached| (None, attached)),
            None => engine.import_pdf(file, project.as_ref()).await.map(|(added, attached)| (Some(added), attached)),
        };
        match result {
            Ok((added, attached)) => {
                if let Some(Added::New(reference)) = &added {
                    output::report(Tone::Ok, "added", output::summary(reference));
                }
                report_attached(&attached, &from);
            }
            Err(error) => {
                failed += 1;
                output::note(Tone::Error, "failed", format!("{}: {error}", file.display()));
            }
        }
    }
    Ok(exit_code(failed, args.files.len()))
}
