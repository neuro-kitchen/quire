//! `project new|list`.

use clap::{Args, Subcommand};
use quire_engine::{Engine, SlugPath};

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
    /// List projects as a tree
    List,
}

pub async fn run(engine: &Engine, args: &ProjectArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::New { name, parent } => {
            let (path, _) = engine.create_project(name, parent.as_ref()).await?;
            println!("{path}");
        }
        Action::List => {
            for (path, project) in engine.projects().await? {
                let depth = path.segments().len() - 1;
                println!("{}{}  ({})", "  ".repeat(depth), path, project.name);
            }
        }
    }
    Ok(())
}
