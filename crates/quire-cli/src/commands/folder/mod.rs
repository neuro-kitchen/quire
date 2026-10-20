//! `folder new|show|check`: `quire.toml` links a document's folder to a library and project, so
//! inside it `quire add <doi>` and `quire export` need no flags (`.knowledge/cli-tui.md`);
//! `check` compares the cite keys of the folder's manuscripts with the library and project.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use quire_engine::{ExportRequest, FolderFile, SlugPath};

use crate::context::Context;

mod check;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct FolderArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Write quire.toml in the current folder
    New {
        /// The project this folder's document cites from
        #[arg(short, long)]
        project: SlugPath,
        /// Include its subprojects
        #[arg(short, long)]
        recursive: bool,
        /// Bibliography file, relative to the folder (e.g. refs.bib, refs.yml)
        #[arg(short, long, default_value = "refs.bib")]
        output: PathBuf,
        /// Export profile from the config
        #[arg(long)]
        profile: Option<String>,
        /// Format: biblatex, bibtex, csl-json, hayagriva (default: from the file extension)
        #[arg(long)]
        format: Option<String>,
        /// Do not keep the bibliography file updated
        #[arg(long)]
        no_keep_updated: bool,
    },
    /// Show the quire.toml that applies here
    Show,
    /// Check the manuscript's citations: keys not in the library, not in the project (so not in its
    /// bibliography file), and the project's references never cited. Fails if a key is missing
    Check(CheckArgs),
}

#[derive(Args, Debug)]
struct CheckArgs {
    /// .typ, .tex or .md files, or folders to search (default: the folder of quire.toml, else
    /// the current one)
    files: Vec<PathBuf>,
    /// The project to check against (default: quire.toml's)
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Put cited references that are outside the project into it (the bibliography file follows)
    #[arg(long)]
    link: bool,
    /// Also list the project's references that are never cited
    #[arg(long)]
    uncited: bool,
}

pub async fn run(context: &Context, args: &FolderArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::New { project, recursive, output, profile, format, no_keep_updated } => {
            let here = std::env::current_dir()?;
            let format = format.clone().or_else(|| format_for(output).map(str::to_string));
            let engine = context.engine().await?;
            let file = FolderFile {
                // Only a library named with `--lib` is recorded; otherwise the default applies.
                library: context.lib.clone(),
                project: Some(project.clone()),
                recursive: *recursive,
                output: Some(output.clone()),
                profile: profile.clone(),
                format: format.clone(),
                folder: here.clone(),
            };
            let request = ExportRequest {
                project: Some(project.clone()),
                recursive: *recursive,
                profile: profile.clone(),
                format: format.as_deref().map(str::parse).transpose()?,
            };
            // Check the project and profile exist before writing the file.
            engine.export(&request).await?;
            let path = FolderFile::create(&here, &file)?;
            output::report(Tone::Ok, "created", path.display());
            if !no_keep_updated {
                let (written, count) = engine.keep_updated(&request, &here.join(output)).await?;
                output::report(
                    Tone::Ok,
                    "written",
                    format!("{count} references to {} (kept updated)", written.display()),
                );
            }
        }
        Action::Check(args) => return check::run(context, args).await,
        Action::Show => match &context.folder {
            Some(folder) => {
                anstream::println!(
                    "{}",
                    style::paint(style::DIM, folder.folder.join(quire_engine::folder::FILE_NAME).display())
                );
                let show = |label: &str, value: Option<String>| {
                    if let Some(value) = value {
                        anstream::println!("{}{value}", style::paint(style::LABEL, format!("{label:<12}")));
                    }
                };
                show("library", folder.library.clone());
                show(
                    "project",
                    folder
                        .project
                        .as_ref()
                        .map(|p| format!("{p}{}", if folder.recursive { " (with subprojects)" } else { "" })),
                );
                show("output", folder.output_path().map(|p| p.display().to_string()));
                show("profile", folder.profile.clone());
                show("format", folder.format.clone());
            }
            None => output::hint(
                "No quire.toml here or in a parent folder. Create one with `quire folder new -p <project>`.",
            ),
        },
    }
    Ok(())
}

/// The format a bibliography file's extension implies (`.yml`/`.yaml` → Hayagriva, `.json` → CSL-JSON).
fn format_for(path: &std::path::Path) -> Option<&'static str> {
    match path.extension()?.to_str()? {
        "yml" | "yaml" => Some("hayagriva"),
        "json" => Some("csl-json"),
        _ => None,
    }
}
