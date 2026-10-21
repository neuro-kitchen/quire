//! `note add|list|show|edit|rm`: Markdown notes on references, or standalone (as in Zotero). Text
//! comes from `-m` or the editor.

use clap::{Args, Subcommand};
use quire_engine::{Engine, Note, NoteId};

use super::editor;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct NoteArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Write a note on a reference (or a standalone note without one)
    Add {
        /// Cite key, DOI / arXiv ID / PMID, or reference ID
        reference: Option<String>,
        /// The text (Markdown); without it, your editor opens
        #[arg(short, long)]
        message: Option<String>,
    },
    /// List a reference's notes, or the standalone ones
    List { reference: Option<String> },
    /// Print a note
    Show { id: NoteId },
    /// Edit a note in your editor
    Edit { id: NoteId },
    /// Remove a note
    Rm { id: NoteId },
}

pub async fn run(engine: &Engine, args: &NoteArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::Add { reference, message } => {
            let body = match message {
                Some(text) => text.clone(),
                None => editor::edit_text("", ".md")?,
            };
            if body.trim().is_empty() {
                output::hint("Empty note; nothing saved.");
                return Ok(());
            }
            let note = engine.add_note(reference.as_deref(), &body).await?;
            output::report(Tone::Ok, "noted", line(&note));
        }
        Action::List { reference } => {
            let notes = engine.notes(reference.as_deref()).await?;
            if notes.is_empty() {
                output::hint(if reference.is_some() { "No notes on it." } else { "No standalone notes." });
            }
            for note in &notes {
                crate::out!("{}", line(note));
            }
        }
        Action::Show { id } => {
            let note = found(engine, *id).await?;
            crate::data!("{}\n", note.body);
        }
        Action::Edit { id } => {
            let note = found(engine, *id).await?;
            let body = editor::edit_text(&note.body, ".md")?;
            if body.trim_end() == note.body {
                output::report(Tone::Info, "unchanged", line(&note));
                return Ok(());
            }
            let note = engine.edit_note(*id, &body).await?;
            let label = if note.meta.is_deleted() { "removed" } else { "edited" };
            output::report(Tone::Ok, label, line(&note));
        }
        Action::Rm { id } => {
            let note = engine.remove_note(*id).await?;
            output::report(Tone::Info, "removed", line(&note));
        }
    }
    Ok(())
}

async fn found(engine: &Engine, id: NoteId) -> anyhow::Result<Note> {
    engine.note(id).await?.filter(|n| !n.meta.is_deleted()).ok_or_else(|| anyhow::anyhow!("no note `{id}`"))
}

/// `note_01…  Title of the note`.
fn line(note: &Note) -> String {
    format!("{}  {}", style::paint(style::DIM, note.id), note.title())
}
