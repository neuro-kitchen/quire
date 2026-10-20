//! `alert add | list | check | rm`: searches of arXiv, PubMed or Crossref checked for new papers.
//! A check lists what appeared since the last one and is not in the library; `--add` adds it.

use std::process::ExitCode;

use clap::{Args, Subcommand};
use quire_engine::{AlertSource, Engine, SlugPath};

use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct AlertArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Add an alert (or change the one with this name)
    Add {
        name: String,
        /// arxiv, pubmed or crossref
        source: AlertSource,
        /// The search: plain words, or the source's syntax (arXiv au: ti: cat:, PubMed [au] [ti],
        /// Crossref author:)
        #[arg(required = true)]
        query: Vec<String>,
    },
    /// The alerts and when each was last checked
    List,
    /// Look for new papers (every alert, or the one named)
    Check {
        name: Option<String>,
        /// Add the new papers to the library
        #[arg(long)]
        add: bool,
        /// With --add: put them into this project
        #[arg(short, long, requires = "add")]
        project: Option<SlugPath>,
        /// Look this many days back instead of since the last check
        #[arg(long)]
        days: Option<i64>,
    },
    /// Remove an alert
    Rm { name: String },
}

pub async fn run(engine: &Engine, args: &AlertArgs) -> anyhow::Result<ExitCode> {
    match &args.action {
        Action::Add { name, source, query } => {
            let alert = engine.add_alert(name, *source, &query.join(" ")).await?;
            output::report(Tone::Ok, "alert", format!("{} ({}: {})", alert.name, alert.source, alert.query));
        }
        Action::List => {
            let alerts = engine.alerts().await?;
            for alert in &alerts {
                let checked =
                    alert.checked_at.map_or_else(|| "never checked".into(), |t| format!("checked {}", t.date()));
                anstream::println!(
                    "{}  {}  {}  {}",
                    style::paint(style::KEY, &alert.name),
                    alert.source,
                    alert.query,
                    style::paint(style::DIM, checked)
                );
            }
            if alerts.is_empty() {
                output::hint("no alerts: add one with `quire alert add <name> arxiv|pubmed|crossref <query>`");
            }
        }
        Action::Check { name, add, project, days } => {
            let checks = engine.check_alerts(name.as_deref(), *days).await?;
            let mut failed = 0;
            for check in checks {
                if let Some(error) = &check.error {
                    failed += 1;
                    output::report(Tone::Error, "failed", format!("{}: {error}", check.alert.name));
                    continue;
                }
                output::hint(format!("{} ({}): {} new", check.alert.name, check.alert.source, check.new.len()));
                for data in check.new {
                    let line = found_line(&data);
                    if *add {
                        let added = engine.add_found(data, check.alert.source, project.as_ref()).await?;
                        output::report(Tone::Ok, "added", output::summary(added.reference()));
                    } else {
                        anstream::println!("  {line}");
                    }
                }
            }
            if failed > 0 {
                return Ok(ExitCode::FAILURE);
            }
        }
        Action::Rm { name } => {
            let alert = engine.remove_alert(name).await?;
            output::report(Tone::Ok, "removed", alert.name);
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// `2026  Camarneiro  Title  doi:…` for a found work.
fn found_line(data: &quire_model::ReferenceData) -> String {
    let year = data.issued.map(|d| d.get_year().to_string()).unwrap_or_default();
    let author = data.creators.first().map(quire_model::Creator::sort_name).unwrap_or_default();
    let id = data.identifiers.first().map(ToString::to_string).unwrap_or_default();
    format!(
        "{year:<4}  {:<20}  {}  {}",
        author.chars().take(20).collect::<String>(),
        data.title().unwrap_or("(no title)"),
        style::paint(style::DIM, id)
    )
}
