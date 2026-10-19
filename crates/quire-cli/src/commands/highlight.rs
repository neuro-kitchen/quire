//! `highlight list|comment|rm`: passages marked in the TUI reader. `list` prints them as Markdown
//! quotes (ready to paste into notes or a manuscript).

use clap::{Args, Subcommand};
use quire_engine::{Engine, HighlightId};

use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct HighlightArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Print a paper's highlights as Markdown quotes, with their comments
    List {
        /// Cite key, DOI / arXiv ID / PMID, or reference ID
        reference: String,
    },
    /// Set a highlight's comment (empty text removes it)
    Comment {
        id: HighlightId,
        /// The comment (Markdown)
        #[arg(short, long)]
        message: String,
    },
    /// Remove a highlight
    Rm { id: HighlightId },
}

pub async fn run(engine: &Engine, args: &HighlightArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::List { reference } => {
            let (reference, highlights) = engine.highlights(reference).await?;
            if highlights.is_empty() {
                output::hint(format!(
                    "No highlights in {} (mark passages with `v` in the reader).",
                    reference.cite_key
                ));
            }
            for highlight in &highlights {
                println!("> {}", highlight.quote);
                if let Some(comment) = &highlight.comment {
                    println!("\n{comment}");
                }
                anstream::println!("{}\n", style::paint(style::DIM, format!("<!-- {} -->", highlight.id)));
            }
        }
        Action::Comment { id, message } => {
            let highlight = engine.comment_highlight(*id, message).await?;
            output::report(Tone::Ok, "commented", style::paint(style::DIM, highlight.id).to_string());
        }
        Action::Rm { id } => {
            let highlight = engine.remove_highlight(*id).await?;
            output::report(Tone::Info, "removed", style::paint(style::DIM, highlight.id).to_string());
        }
    }
    Ok(())
}
