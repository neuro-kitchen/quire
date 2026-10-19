//! `project new|list|tree|rename|rm`.

use clap::{Args, Subcommand};
use quire_engine::{Engine, SlugPath};

use crate::output::{self, Tone, style, tree};

#[derive(Args, Debug)]
pub struct ProjectArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Create a project; its slug is made from the name ("Chapter 2" → chapter-2)
    New {
        name: String,
        /// Create it inside this project, e.g. `thesis`
        #[arg(long)]
        parent: Option<SlugPath>,
    },
    /// List project paths, one per line, with their number of references
    List {
        /// Print JSON
        #[arg(long)]
        json: bool,
    },
    /// Draw projects as a tree
    Tree,
    /// Rename a project (its slug follows the new name)
    Rename { path: SlugPath, name: String },
    /// Move a project to the trash; its references stay in the library
    Rm {
        path: SlugPath,
        /// Also its subprojects
        #[arg(short, long)]
        recursive: bool,
    },
}

pub async fn run(engine: &Engine, args: &ProjectArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::New { name, parent } => {
            let (path, _) = engine.create_project(name, parent.as_ref()).await?;
            output::report(Tone::Ok, "created", style::paint(style::KEY, path));
        }
        Action::List { json } => {
            let projects = engine.project_summaries().await?;
            if *json {
                let list: Vec<_> = projects
                    .iter()
                    .map(|p| serde_json::json!({ "path": p.path.to_string(), "name": p.project.name, "id": p.project.id, "references": p.references }))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else if projects.is_empty() {
                output::hint("No projects. Create one with `quire project new <name>`.");
            } else {
                let width = projects.iter().map(|p| p.path.to_string().chars().count()).max().unwrap_or(0);
                for p in &projects {
                    let path = format!("{:<width$}", p.path.to_string());
                    anstream::println!(
                        "{}  {}",
                        style::paint(style::KEY, path),
                        style::paint(style::DIM, p.references)
                    );
                }
            }
        }
        Action::Tree => {
            let projects = engine.project_summaries().await?;
            if projects.is_empty() {
                output::hint("No projects. Create one with `quire project new <name>`.");
            } else {
                anstream::println!("{}", tree::projects(&projects));
            }
        }
        Action::Rename { path, name } => {
            let (new_path, _) = engine.rename_project(path, name).await?;
            output::report(
                Tone::Ok,
                "renamed",
                format!("{} → {}", style::paint(style::KEY, path), style::paint(style::KEY, new_path)),
            );
        }
        Action::Rm { path, recursive } => {
            let count = engine.remove_project(path, *recursive).await?;
            let extra = if count > 1 { format!(" and {} subprojects", count - 1) } else { String::new() };
            output::report(Tone::Info, "trashed", format!("{}{extra}", style::paint(style::KEY, path)));
        }
    }
    Ok(())
}
