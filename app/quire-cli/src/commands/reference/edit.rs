//! `ref edit`: open a reference in `$VISUAL` / `$EDITOR` as TOML; the fields you change become
//! yours (refetch keeps them). An unreadable edit can be fixed in the editor again. `--file`
//! applies a prepared document instead, `--print` only prints it.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use clap::Args;
use quire_engine::Engine;

use crate::commands::editor;
use crate::output::{self, Tone, changes};

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Cite key, DOI / arXiv ID / PMID, or reference ID
    query: String,
    /// Apply this edited TOML file instead of opening an editor
    #[arg(long, value_name = "FILE", conflicts_with = "print")]
    file: Option<PathBuf>,
    /// Print the editable TOML and stop
    #[arg(long)]
    print: bool,
    /// Show what would change without changing anything
    #[arg(long)]
    dry_run: bool,
}

pub async fn run(engine: &Engine, args: &EditArgs) -> anyhow::Result<()> {
    let (id, original) = engine.edit_document(&args.query).await?;
    if args.print {
        crate::data!("{original}");
        return Ok(());
    }
    if let Some(file) = &args.file {
        let text = std::fs::read_to_string(file).map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
        return report(&engine.apply_edit(id, &text, args.dry_run).await?, args.dry_run);
    }
    // A new private file (0600, random name): no other user can read it or plant one first.
    let file = tempfile::Builder::new().prefix("quire-edit-").suffix(".toml").tempfile()?;
    std::fs::write(file.path(), &original)?;
    edit_loop(engine, id, file.path(), &original, args.dry_run).await
}

/// Opens the editor until the document is applied, unchanged, or the user gives up.
async fn edit_loop(
    engine: &Engine,
    id: quire_model::ReferenceId,
    path: &Path,
    original: &str,
    dry_run: bool,
) -> anyhow::Result<()> {
    loop {
        editor::open(path)?;
        let text = std::fs::read_to_string(path)?;
        if text == original {
            output::report(Tone::Info, "unchanged", "nothing edited");
            return Ok(());
        }
        match engine.apply_edit(id, &text, dry_run).await {
            Ok(updated) => return report(&updated, dry_run),
            Err(error) if std::io::stdin().is_terminal() => {
                output::note(Tone::Error, "invalid", &error);
                if !editor::ask("Edit again? [Y/n] ")? {
                    anyhow::bail!("edit discarded");
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn report(updated: &quire_engine::Updated, dry_run: bool) -> anyhow::Result<()> {
    let label = match (updated.changed(), dry_run) {
        (false, _) => "unchanged",
        (true, true) => "would edit",
        (true, false) => "edited",
    };
    output::report(if updated.changed() { Tone::Ok } else { Tone::Info }, label, output::summary(&updated.reference));
    for line in changes::lines(updated) {
        crate::out!("{line}");
    }
    Ok(())
}
