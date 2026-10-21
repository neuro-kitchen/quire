//! `export`: references of a project (or the library) to stdout or a file, through an export
//! profile. `--keep-updated` also registers the file to be rewritten after every change (Better
//! BibTeX's auto-export); `export auto` lists those files, `export stop <file>` ends one. In a
//! folder with `quire.toml`, its project, profile and output file are the defaults.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use quire_engine::{AutoExportOutcome, ExportFormat, ExportRequest, SlugPath};

use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub struct ExportArgs {
    #[command(subcommand)]
    action: Option<Action>,
    /// Only this project
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// The whole library, even inside a folder with quire.toml
    #[arg(long, conflicts_with = "project")]
    all: bool,
    /// Write to this file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Export profile from the config (`[export.<name>]`)
    #[arg(long)]
    profile: Option<String>,
    /// Format (overrides the profile's)
    #[arg(long, value_enum)]
    format: Option<Format>,
    /// Rewrite the output file after every change from now on
    #[arg(long, requires = "output")]
    keep_updated: bool,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// List the files kept updated
    Auto {
        /// Rewrite them all now (e.g. after changing an export profile)
        #[arg(long)]
        run: bool,
    },
    /// Stop keeping a file updated (the file stays)
    Stop { file: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    /// BibLaTeX (Biber)
    Biblatex,
    /// Classic BibTeX
    Bibtex,
    /// CSL-JSON (Pandoc, Zotero)
    CslJson,
    /// Hayagriva YAML (Typst)
    Hayagriva,
}

impl From<Format> for ExportFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Biblatex => Self::Biblatex,
            Format::Bibtex => Self::Bibtex,
            Format::CslJson => Self::CslJson,
            Format::Hayagriva => Self::Hayagriva,
        }
    }
}

pub async fn run(context: &Context, args: &ExportArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    match &args.action {
        Some(Action::Auto { run: true }) => {
            report_auto(&engine.run_auto_exports(true).await?, true);
            return Ok(());
        }
        Some(Action::Auto { run: false }) => return list_auto(context).await,
        Some(Action::Stop { file }) => {
            if engine.stop_auto_export(file).await? {
                output::report(Tone::Info, "stopped", file.display());
            } else {
                output::report(Tone::Warn, "unchanged", format!("{} was not kept updated", file.display()));
            }
            return Ok(());
        }
        None => {}
    }
    let folder = context.folder.as_ref();
    let (project, recursive) = context.project(args.project.as_ref(), args.recursive, args.all);
    let format = args
        .format
        .map(ExportFormat::from)
        .or_else(|| folder.and_then(|f| f.format.as_deref()).and_then(|f| f.parse().ok()));
    let request = ExportRequest {
        project,
        recursive,
        profile: args.profile.clone().or_else(|| folder.and_then(|f| f.profile.clone())),
        format,
    };
    let output = args
        .output
        .clone()
        .or_else(|| folder.filter(|_| !args.all && args.project.is_none()).and_then(|f| f.output_path()));
    match output {
        Some(path) if args.keep_updated => {
            let (path, count) = engine.keep_updated(&request, &path).await?;
            output::report(
                Tone::Ok,
                "written",
                format!("{} to {}", output::count(count, "reference", "references"), path.display()),
            );
            output::hint("Kept updated after every change; `quire export auto` lists such files.");
        }
        Some(path) => {
            let (text, count) = engine.export(&request).await?;
            let wrote = quire_engine::write_if_changed(&path, &text)?;
            let label = if wrote { "written" } else { "unchanged" };
            output::note(
                Tone::Ok,
                label,
                format!("{} to {}", output::count(count, "reference", "references"), path.display()),
            );
        }
        None => print!("{}", engine.export(&request).await?.0),
    }
    Ok(())
}

async fn list_auto(context: &Context) -> anyhow::Result<()> {
    let exports = context.engine().await?.auto_exports().await?;
    if exports.is_empty() {
        output::hint("No files are kept updated. Use `quire export -o <file> --keep-updated`.");
    }
    for info in exports {
        let export = &info.export;
        let what = match (&info.project, export.project) {
            (Some(path), _) => format!(
                "{}{}",
                style::paint(style::KEY, path),
                if export.recursive { " (with subprojects)" } else { "" }
            ),
            (None, Some(_)) => style::paint(style::ERROR, "(project removed)"),
            (None, None) => "whole library".into(),
        };
        let how = [export.profile.as_ref().map(|p| format!("profile {p}")), export.format.clone()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(", ");
        anstream::println!("{}  {what}  {}", export.path, style::paint(style::DIM, how));
    }
    Ok(())
}

/// One line per automatic export that was rewritten or failed (and, `all`, left unchanged).
pub fn report_auto(results: &[(String, AutoExportOutcome)], all: bool) {
    for (path, outcome) in results {
        match outcome {
            AutoExportOutcome::Written(count) => output::note(
                Tone::Ok,
                "updated",
                format!("{path} ({})", output::count(*count, "reference", "references")),
            ),
            AutoExportOutcome::Unchanged if all => output::note(Tone::Info, "unchanged", path),
            AutoExportOutcome::Unchanged => {}
            AutoExportOutcome::Failed(error) => output::note(Tone::Error, "not updated", format!("{path}: {error}")),
        }
    }
}
