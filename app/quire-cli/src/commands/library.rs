//! `library new|list|use|rename|rm|backup|restore`: named libraries (`.knowledge/library-model.md`).
//! Each is its own folder; the config file only records where. `rm` forgets a library, it does not
//! delete its folder. `backup` writes the open library to one `.tar.gz`; `restore` makes a new
//! library from one.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use quire_engine::config::Config;
use quire_engine::{LibraryChoice, LibraryInfo, libraries};

use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct LibraryArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Create and register a library
    New {
        /// Short name: lowercase letters, digits and `-`
        name: String,
        /// Folder (default: ~/.local/share/quire/libraries/<name>); an existing library folder is registered as is
        #[arg(long)]
        path: Option<PathBuf>,
        /// Also make it the default library
        #[arg(long)]
        default: bool,
    },
    /// List libraries (* = default)
    List {
        /// Print JSON
        #[arg(long)]
        json: bool,
    },
    /// Make a library the default
    Use { name: String },
    /// Rename a library (its folder stays)
    Rename { old: String, new: String },
    /// Forget a library; its folder and files are kept
    Rm { name: String },
    /// Write the library (database, PDFs, texts) to one .tar.gz
    Backup {
        /// The archive (default: quire-<library>-<date>.tar.gz here)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Make a new library from a backup and register it as `name`
    Restore {
        archive: PathBuf,
        name: String,
        /// Folder (default: ~/.local/share/quire/libraries/<name>)
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

pub async fn run(context: &Context, args: &LibraryArgs) -> anyhow::Result<()> {
    let file = context.config_file()?;
    match &args.action {
        Action::New { name, path, default } => {
            let info = libraries::register(&file, name, path.as_deref())?;
            context.open(LibraryChoice::Named(name.clone())).await?;
            output::report(Tone::Ok, "created", describe(&info));
            if *default {
                libraries::set_default(&file, name)?;
                output::report(Tone::Ok, "default", style::paint(style::KEY, name));
            } else {
                output::hint(format!(
                    "Use it with `--lib {name}`, or make it the default with `quire library use {name}`."
                ));
            }
        }
        Action::List { json } => {
            let all = libraries::list(&Config::load(&file)?)?;
            if *json {
                let list: Vec<_> = all
                    .iter()
                    .map(|l| serde_json::json!({ "name": l.name, "path": l.path, "default": l.is_default, "registered": l.registered, "exists": l.exists() }))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else {
                let width = all.iter().map(|l| l.label().chars().count()).max().unwrap_or(0);
                for library in &all {
                    let mark = if library.is_default { style::paint(style::OK, "*") } else { " ".into() };
                    let name = style::paint(style::KEY, format!("{:<width$}", library.label()));
                    let state = match (library.registered, library.exists()) {
                        (_, false) => "  (not created yet)",
                        (false, true) => "  (built-in default)",
                        (true, true) => "",
                    };
                    anstream::println!(
                        "{mark} {name}  {}{}",
                        style::paint(style::DIM, library.path.display()),
                        style::paint(style::DIM, state)
                    );
                }
            }
        }
        Action::Use { name } => {
            libraries::set_default(&file, name)?;
            output::report(Tone::Ok, "default", style::paint(style::KEY, name));
        }
        Action::Rename { old, new } => {
            let info = libraries::rename(&file, old, new)?;
            output::report(Tone::Ok, "renamed", format!("{} → {}", style::paint(style::KEY, old), describe(&info)));
        }
        Action::Backup { output } => {
            let engine = context.engine().await?;
            let label = engine.library().label().replace(['/', ' '], "-");
            let date = quire_model::Timestamp::now().date();
            let output = output.clone().unwrap_or_else(|| PathBuf::from(format!("quire-{label}-{date}.tar.gz")));
            let report = engine.backup(&output).await?;
            output::report(
                Tone::Ok,
                "backed up",
                format!(
                    "{} ({}, {:.1} MB)",
                    report.path.display(),
                    output::count(report.files, "file", "files"),
                    report.bytes as f64 / 1e6
                ),
            );
            output::hint("It holds your library unencrypted (notes, PDFs): keep it somewhere private.");
        }
        Action::Restore { archive, name, path } => {
            let info = libraries::register(&file, name, path.as_deref())?;
            if let Err(error) = quire_engine::engine::backup::restore(archive, &info.path) {
                libraries::unregister(&file, name)?;
                return Err(error.into());
            }
            context.open(LibraryChoice::Named(name.clone())).await?;
            output::report(Tone::Ok, "restored", describe(&info));
            output::hint(format!("Use it with `--lib {name}`; it syncs as a new device of its library."));
        }
        Action::Rm { name } => {
            let info = libraries::unregister(&file, name)?;
            output::report(Tone::Info, "forgotten", describe(&info));
            output::hint(format!("Its folder is kept: {}", info.path.display()));
        }
    }
    Ok(())
}

/// `name  /path/to/folder`.
fn describe(info: &LibraryInfo) -> String {
    format!("{}  {}", style::paint(style::KEY, info.label()), style::paint(style::DIM, info.path.display()))
}
