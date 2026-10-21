//! `highlight list|comment|rm|export`: passages marked in the TUI reader. `list` prints them as
//! Markdown quotes (ready to paste into notes or a manuscript); `export` writes one Markdown file
//! per paper (highlights and notes) for Obsidian, Logseq or any Markdown folder.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use quire_engine::{Engine, HighlightId, SlugPath, write_if_changed};

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
    /// Write each paper's highlights and notes as Markdown (one file per paper, named by cite key)
    Export {
        /// Only this project
        #[arg(short, long)]
        project: Option<SlugPath>,
        /// With --project: include its subprojects
        #[arg(short, long)]
        recursive: bool,
        /// Folder to write to (e.g. your Obsidian vault's literature folder); without it, print
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
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
                crate::out!("> {}", highlight.quote);
                if let Some(comment) = &highlight.comment {
                    crate::out!("\n{comment}");
                }
                crate::out!("{}\n", style::paint(style::DIM, format!("<!-- {} -->", highlight.id)));
            }
        }
        Action::Comment { id, message } => {
            let highlight = engine.comment_highlight(*id, message).await?;
            output::report(Tone::Ok, "commented", style::paint(style::DIM, highlight.id).to_string());
        }
        Action::Export { project, recursive, output } => {
            let files = engine.annotations_markdown(project.as_ref(), *recursive).await?;
            let Some(dir) = output else {
                for (_, markdown) in &files {
                    crate::data!("{markdown}\n");
                }
                return Ok(());
            };
            let mut written = 0;
            for (reference, markdown) in &files {
                let path = dir.join(format!("{}.md", reference.cite_key));
                if write_if_changed(&path, markdown)? {
                    written += 1;
                    output::report(Tone::Ok, "written", path.display());
                }
            }
            output::hint(format!("papers with highlights or notes: {}; files changed: {written}", files.len()));
        }
        Action::Rm { id } => {
            let highlight = engine.remove_highlight(*id).await?;
            output::report(Tone::Info, "removed", style::paint(style::DIM, highlight.id).to_string());
        }
    }
    Ok(())
}
