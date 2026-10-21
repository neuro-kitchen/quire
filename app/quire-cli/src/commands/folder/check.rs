//! `folder check`: the manuscript's cite keys against the library and the folder's project.

use quire_engine::{CiteUse, ManuscriptCheck};

use super::CheckArgs;
use crate::context::Context;
use crate::output::{self, Tone, style};

pub(super) async fn run(context: &Context, args: &CheckArgs) -> anyhow::Result<()> {
    let (project, recursive) = context.project(args.project.as_ref(), args.recursive, false);
    let files = if args.files.is_empty() {
        vec![context.folder.as_ref().map(|f| f.folder.clone()).map_or_else(std::env::current_dir, Ok)?]
    } else {
        args.files.clone()
    };
    let engine = context.engine().await?;
    let check = engine.check_manuscript(&files, project.clone(), recursive).await?;
    let here = std::env::current_dir()?;
    let place = |uses: &[CiteUse]| {
        let first = &uses[0];
        let file = first.file.strip_prefix(&here).unwrap_or(&first.file).display();
        let more = if uses.len() > 1 { format!(" (+{} more)", uses.len() - 1) } else { String::new() };
        style::paint(style::DIM, format!("{file}:{}:{}{more}", first.line, first.column + 1))
    };

    // Keys in one column.
    let width = check
        .unknown
        .iter()
        .map(|u| u.key.chars().count())
        .chain(check.outside.iter().map(|(r, _)| r.cite_key.as_str().chars().count()))
        .max()
        .unwrap_or(0)
        .min(48);
    output::note(
        Tone::Info,
        "checked",
        format!(
            "{} cite key(s) in {} file(s){}",
            check.cited,
            check.files.len(),
            project.as_ref().map(|p| format!(", checked against project {p}")).unwrap_or_default()
        ),
    );
    if !check.unknown.is_empty() {
        heading("not in the library", check.unknown.len());
        for unknown in &check.unknown {
            let hint = if unknown.trashed {
                style::paint(style::WARN, "in the trash: `quire ref restore`")
            } else if unknown.suggestions.is_empty() {
                String::new()
            } else {
                format!("did you mean {}?", unknown.suggestions.join(", "))
            };
            let key = style::paint(style::ERROR, format!("{:<width$}", unknown.key));
            crate::out!("{}", format!("  {key}  {}  {hint}", place(&unknown.uses)).trim_end());
        }
    }
    if let (false, Some(project)) = (check.outside.is_empty(), &project) {
        heading(&format!("not in project {project} (so not in its bibliography)"), check.outside.len());
        for (reference, uses) in &check.outside {
            let key = style::paint(style::WARN, format!("{:<width$}", reference.cite_key.as_str()));
            crate::out!("  {key}  {}", place(uses));
        }
        if args.link {
            for (reference, _) in &check.outside {
                engine.link(&reference.id.to_string(), project).await?;
            }
            output::report(Tone::Ok, "linked", format!("{} reference(s) to {project}", check.outside.len()));
        } else {
            output::hint("`quire folder check --link` puts them into the project.");
        }
    }
    if !check.uncited.is_empty() {
        if args.uncited {
            heading("never cited", check.uncited.len());
            for reference in &check.uncited {
                crate::out!("  {}", output::summary(reference));
            }
        } else {
            output::hint(format!(
                "{} reference(s) of the project are never cited (`--uncited` lists them).",
                check.uncited.len()
            ));
        }
    }
    finish(&check, args.link)
}

fn heading(text: &str, count: usize) {
    crate::out!("\n{} {}", style::paint(style::LABEL, text), style::paint(style::DIM, format!("({count})")));
}

/// Fails (exit status 1, for CI and hooks) while a cited key has no reference in the bibliography.
fn finish(check: &ManuscriptCheck, linked: bool) -> anyhow::Result<()> {
    let outside = if linked { 0 } else { check.outside.len() };
    match check.unknown.len() + outside {
        0 => {
            output::report(Tone::Ok, "ok", "every cited key is in the bibliography");
            Ok(())
        }
        problems => anyhow::bail!("{problems} cited key(s) missing from the bibliography"),
    }
}
