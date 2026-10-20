//! `show`: one reference by cite key, identifier or ID, with its projects, PDFs, tags, notes and
//! related references.

use clap::Args;
use quire_engine::Engine;

use crate::commands::tag::paint_tag;
use crate::output::reference;

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID (ref_…)
    query: String,
    /// Print JSON
    #[arg(long)]
    json: bool,
}

pub async fn run(engine: &Engine, args: &ShowArgs) -> anyhow::Result<()> {
    let (found, projects) = engine.show(&args.query).await?;
    let projects: Vec<String> = projects.iter().map(ToString::to_string).collect();
    let id = found.id.to_string();
    let (_, pdfs) = engine.pdfs_of(&id).await?;
    let tags = engine.tags_of(vec![found.id]).await?.remove(&found.id).unwrap_or_default();
    let notes = engine.notes(Some(&id)).await?;
    let related = engine.related(&id).await?;
    if args.json {
        let attachments: Vec<_> =
            pdfs.iter().map(|(a, path)| serde_json::json!({ "attachment": a, "path": path })).collect();
        let tag_list: Vec<_> = tags
            .iter()
            .map(|(t, kind)| serde_json::json!({ "name": t.name, "kind": kind, "color": t.color }))
            .collect();
        let related: Vec<String> = related.iter().map(|r| r.cite_key.to_string()).collect();
        let value = serde_json::json!({
            "reference": found, "projects": projects, "pdfs": attachments, "tags": tag_list, "notes": notes, "related": related,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        let context = reference::Context {
            projects,
            pdfs,
            tags: tags.iter().map(|(t, _)| paint_tag(t, &t.name)).collect(),
            notes: notes.iter().map(|n| (n.id.to_string(), n.title())).collect(),
            related: related.iter().map(|r| r.cite_key.to_string()).collect(),
        };
        anstream::println!("{}", reference::block(&found, &context));
    }
    Ok(())
}
