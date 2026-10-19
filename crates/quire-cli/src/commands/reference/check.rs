//! `ref check`: references with metadata issues (fields missing or doubtful, a fallback source
//! used, blocked by a publisher) and inputs whose lookup failed.

use clap::Args;
use quire_engine::{Issue, Severity, SlugPath};

use crate::context::Context;

use crate::output::{self, style};

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Only this project
    #[arg(short, long)]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// The whole library, even inside a folder with quire.toml
    #[arg(long, conflicts_with = "project")]
    all: bool,
    /// Only errors, not warnings
    #[arg(long)]
    errors: bool,
    /// Print JSON
    #[arg(long)]
    json: bool,
}

pub async fn run(context: &Context, args: &CheckArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let (project, recursive) = context.project(args.project.as_ref(), args.recursive, args.all);
    let mut checked = engine.check(project.as_ref(), recursive).await?;
    if args.errors {
        checked.retain_mut(|c| {
            c.issues.retain(|i| i.severity() == Severity::Error);
            !c.issues.is_empty()
        });
    }
    let failures = engine.failures().await?;
    if args.json {
        let references: Vec<_> = checked
            .iter()
            .map(|c| serde_json::json!({ "cite_key": c.reference.cite_key, "id": c.reference.id, "issues": c.issues }))
            .collect();
        let failed: Vec<_> = failures
            .iter()
            .map(|(f, project)| serde_json::json!({ "input": f.input, "project": project.as_ref().map(ToString::to_string), "error": f.error, "attempts": f.attempts }))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "references": references, "failed_inputs": failed }))?
        );
        return Ok(());
    }
    for item in &checked {
        anstream::println!("{}", output::summary(&item.reference));
        for issue in &item.issues {
            anstream::println!("    {}", issue_line(issue));
        }
    }
    if !failures.is_empty() {
        if !checked.is_empty() {
            println!();
        }
        anstream::println!("{}", style::paint(style::LABEL, "Failed lookups"));
        for (failure, project) in &failures {
            let project = project.as_ref().map(|p| format!("  → {}", style::paint(style::KEY, p))).unwrap_or_default();
            let attempts =
                if failure.attempts > 1 { format!("  ({} attempts)", failure.attempts) } else { String::new() };
            anstream::println!("  {}{project}{}", failure.input, style::paint(style::DIM, attempts));
            anstream::println!("    {}", style::paint(style::DIM, &failure.error));
        }
        output::hint("Retry them with `quire add --retry`, or forget them with `quire add --forget-failed`.");
    }
    if checked.is_empty() && failures.is_empty() {
        output::hint("No issues found.");
    }
    Ok(())
}

/// `✗ no date` in red, `⚠ title in capitals` in yellow.
pub fn issue_line(issue: &Issue) -> String {
    match issue.severity() {
        Severity::Error => format!("{} {}", style::paint(style::ERROR, "✗"), issue.message()),
        Severity::Warning => format!("{} {}", style::paint(style::WARN, "⚠"), issue.message()),
    }
}
