//! `add`: one or more DOIs, arXiv IDs, PMIDs, PMCIDs or URLs, from the arguments and/or a file
//! (one per line, `#` comments). Each input is reported; failures do not stop the rest, and the
//! exit code is 3 if some failed. With several inputs on a terminal, a progress bar shows on
//! stderr. Failed lookups are remembered: `--retry` tries them again (into the project they were
//! meant for), `--forget-failed` drops them. A new reference's metadata issues are listed under it.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use quire_engine::{Added, SlugPath};

use super::{check, exit_code};
use crate::context::Context;
use crate::output::{self, Tone, terminal};

#[derive(Args, Debug)]
pub struct AddArgs {
    /// DOI, arXiv ID, PMID, PMCID or URL (several allowed)
    inputs: Vec<String>,
    /// Put the references into this project, e.g. `thesis/chapter-2`
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// Also read inputs from this file, one per line
    #[arg(long, value_name = "FILE")]
    from: Option<PathBuf>,
    /// Also retry every earlier failed lookup (see `quire ref check`)
    #[arg(long)]
    retry: bool,
    /// Forget every earlier failed lookup instead of adding anything
    #[arg(long, conflicts_with_all = ["inputs", "from", "retry"])]
    forget_failed: bool,
}

pub async fn run(context: &Context, args: &AddArgs) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    // Inside a folder with quire.toml, its project is the default.
    let (project, _) = context.project(args.project.as_ref(), false, false);
    if args.forget_failed {
        for (failure, _) in engine.failures().await? {
            engine.forget_failure(&failure.input).await?;
            output::report(Tone::Info, "forgotten", &failure.input);
        }
        return Ok(ExitCode::SUCCESS);
    }
    let mut inputs: Vec<(String, Option<SlugPath>)> =
        args.inputs.iter().map(|i| (i.clone(), project.clone())).collect();
    if let Some(file) = &args.from {
        let text = std::fs::read_to_string(file).map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
        let lines = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#'));
        inputs.extend(lines.map(|line| (line.to_string(), project.clone())));
    }
    if args.retry {
        // Into the project given now, else the one it was meant for.
        let failures = engine.failures().await?;
        inputs.extend(failures.into_iter().map(|(failure, project)| (failure.input, args.project.clone().or(project))));
    }
    if inputs.is_empty() {
        anyhow::bail!("nothing to add: give identifiers or URLs, --from <file>, or --retry");
    }

    let progress = progress_bar(inputs.len());
    let mut failed = 0;
    for (input, project) in &inputs {
        progress.set_message(input.clone());
        let result = engine.add(input, project.as_ref()).await;
        let issues = match &result {
            Ok(Added::New(reference)) => engine.issues(reference).await.unwrap_or_default(),
            _ => Vec::new(),
        };
        progress.suspend(|| match result {
            Ok(Added::New(reference)) => {
                output::report(Tone::Ok, "added", output::summary(&reference));
                for issue in &issues {
                    anstream::println!("{:>12}{}", "", check::issue_line(issue));
                }
            }
            Ok(Added::Existing(reference)) => output::report(Tone::Warn, "existing", output::summary(&reference)),
            Err(error) => {
                failed += 1;
                output::note(
                    Tone::Error,
                    if error.is_blocked() { "blocked" } else { "failed" },
                    format!("{input}: {error}"),
                );
            }
        });
        progress.inc(1);
    }
    if failed > 0 && !engine.failures().await?.is_empty() {
        output::hint("Failed lookups are kept: `quire ref check` lists them, `quire add --retry` tries again.");
    }
    progress.finish_and_clear();
    Ok(exit_code(failed, inputs.len()))
}

/// A bar on stderr for several inputs on a terminal; hidden otherwise.
fn progress_bar(len: usize) -> ProgressBar {
    if len < 2 || !terminal::stderr_is_terminal() {
        return ProgressBar::hidden();
    }
    let style = ProgressStyle::with_template("{spinner:.cyan} [{bar:30.cyan/blue}] {pos}/{len} {wide_msg:.dim}")
        .expect("valid template")
        .progress_chars("━╸─");
    let bar = ProgressBar::new(len as u64).with_style(style);
    bar.enable_steady_tick(std::time::Duration::from_millis(100));
    bar
}
