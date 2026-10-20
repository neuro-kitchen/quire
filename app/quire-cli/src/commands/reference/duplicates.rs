//! `ref duplicates` / `ref merge` / `ref distinct`: references that are the same work (the arXiv
//! preprint and the journal version, a `.bib` import of a paper already added).

use clap::Args;
use quire_engine::{Engine, Field, Reference};

use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct DuplicatesArgs {}

#[derive(Args, Debug)]
pub struct MergeArgs {
    /// The reference to keep (cite key, DOI / arXiv ID / PMID, or reference ID)
    keep: String,
    /// References merged into it, then moved to the trash
    #[arg(required = true)]
    others: Vec<String>,
}

#[derive(Args, Debug)]
pub struct DistinctArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    first: String,
    /// The reference it is not a duplicate of
    second: String,
}

fn line(reference: &Reference) -> String {
    let year = reference.data.issued.map(|d| d.get_year().to_string()).unwrap_or_default();
    let ids: Vec<String> = reference.data.identifiers.iter().map(ToString::to_string).collect();
    format!("{}  {}  {}", style::paint(style::KEY, &reference.cite_key), year, style::paint(style::DIM, ids.join(" ")))
}

pub async fn list(engine: &Engine, _args: &DuplicatesArgs) -> anyhow::Result<()> {
    let groups = engine.duplicates().await?;
    for group in &groups {
        anstream::println!("{}", line(&group.reference));
        for (other, reason) in &group.duplicates {
            anstream::println!("  ≈ {}  {}", line(other), style::paint(style::DIM, reason.describe()));
        }
    }
    match groups.len() {
        0 => output::hint("no duplicates found"),
        n => output::hint(format!(
            "{n} group(s); `quire ref merge <keep> <other>` merges, `quire ref distinct <a> <b>` keeps both"
        )),
    }
    Ok(())
}

pub async fn merge(engine: &Engine, args: &MergeArgs) -> anyhow::Result<()> {
    for other in &args.others {
        let merged = engine.merge(&args.keep, other).await?;
        let fields: Vec<String> = merged.fields.iter().map(Field::to_string).collect();
        let mut moved = Vec::new();
        for (count, what) in [
            (merged.projects, "project link(s)"),
            (merged.tags, "tag(s)"),
            (merged.notes, "note(s)"),
            (merged.attachments, "PDF(s)"),
            (merged.relations, "relation(s)"),
        ] {
            if count > 0 {
                moved.push(format!("{count} {what}"));
            }
        }
        if !fields.is_empty() {
            moved.push(format!("fields: {}", fields.join(", ")));
        }
        output::report(
            Tone::Ok,
            "merged",
            format!(
                "{} into {}{}",
                style::paint(style::KEY, &merged.removed.cite_key),
                style::paint(style::KEY, &merged.kept.cite_key),
                if moved.is_empty() { String::new() } else { format!(" ({})", moved.join("; ")) }
            ),
        );
    }
    output::hint("merged references are in the trash (`quire ref restore` brings them back, without what moved)");
    Ok(())
}

pub async fn distinct(engine: &Engine, args: &DistinctArgs) -> anyhow::Result<()> {
    let recorded = engine.dismiss_duplicate(&args.first, &args.second).await?;
    let pair = format!("{} ≠ {}", style::paint(style::KEY, &args.first), style::paint(style::KEY, &args.second));
    if recorded {
        output::report(Tone::Ok, "distinct", pair);
    } else {
        output::report(Tone::Warn, "unchanged", format!("{pair} already recorded"));
    }
    Ok(())
}
