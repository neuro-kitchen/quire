//! `saved list | run | rm | rename`: searches kept by name (`quire search … --save NAME`), run
//! again when opened, so new matching references show up by themselves.

use clap::{Args, Subcommand};
use quire_engine::{Engine, SEARCH_LIMIT};

use crate::commands::search::print_hits;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct SavedArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// The saved searches and their queries
    List,
    /// Run a saved search
    Run {
        name: String,
        /// Results shown at most
        #[arg(short = 'n', long, default_value_t = SEARCH_LIMIT)]
        limit: usize,
    },
    /// Forget a saved search
    Rm { name: String },
    /// Rename a saved search
    Rename { name: String, new: String },
}

pub async fn run(engine: &Engine, args: &SavedArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::List => {
            let searches = engine.saved_searches().await?;
            for search in &searches {
                crate::out!("{}  {}", style::paint(style::KEY, &search.name), style::paint(style::DIM, &search.query));
            }
            if searches.is_empty() {
                output::hint("no saved searches: save one with `quire search <query> --save <name>`");
            }
        }
        Action::Run { name, limit } => {
            let (search, hits) = engine.run_saved(name, *limit).await?;
            print_hits(&hits, &search.query);
        }
        Action::Rm { name } => {
            let search = engine.remove_saved(name).await?;
            output::report(Tone::Ok, "removed", search.name);
        }
        Action::Rename { name, new } => {
            let search = engine.rename_saved(name, new).await?;
            output::report(Tone::Ok, "renamed", search.name);
        }
    }
    Ok(())
}
