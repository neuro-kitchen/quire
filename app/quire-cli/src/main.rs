//! `quire`: a thin command line over `quire-engine`. Commands are grouped by what they act on
//! (`ref`, `project`, `library`, `config`); the everyday ones also exist at the top level
//! (`add`, `list`, `show`). Each command lives in its own module with its own arguments; this
//! file only parses and dispatches.
//!
//! Results go to stdout, messages and logs to stderr. Exit codes: 0 success, 1 error, 2 usage
//! error (clap), 3 partial success (some inputs failed).

mod commands;
mod context;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use commands::reference;
use context::Context;
use output::style;

#[derive(Parser, Debug)]
#[command(name = "quire", version, styles = style::CLAP)]
#[command(about = "A reference manager: projects, DOI/arXiv/PubMed/URL lookup, .bib export")]
pub struct Cli {
    /// Use this registered library [env: QUIRE_LIB]
    #[arg(long, global = true, value_name = "NAME", conflicts_with = "library")]
    lib: Option<String>,

    /// Use the library in this folder, registered or not [env: QUIRE_LIBRARY]
    #[arg(long, global = true, value_name = "PATH")]
    library: Option<PathBuf>,

    /// Config file (default: ~/.config/quire/config.toml)
    #[arg(long, global = true, value_name = "FILE", env = quire_engine::paths::CONFIG_ENV)]
    config: Option<PathBuf>,

    /// Show debug logs (requests, database)
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Browse the library in the terminal
    #[cfg(feature = "tui")]
    Tui,
    /// Add references by DOI, arXiv ID, PMID, PMCID or URL (= `ref add`)
    Add(reference::add::AddArgs),
    /// List references as a table (= `ref list`)
    List(reference::list::ListArgs),
    /// Show one reference (= `ref show`)
    Show(reference::show::ShowArgs),
    /// Work with references: add, list, show, edit, refetch, check, link, unlink, rm, restore, copy
    #[command(name = "ref")]
    Reference(reference::RefArgs),
    /// Tags on references: add, rm, list, rename, delete, color
    Tag(commands::tag::TagArgs),
    /// Notes on references or standalone: add, list, show, edit, rm
    Note(commands::note::NoteArgs),
    /// Search the library: metadata, tags, notes, highlights and papers' text
    Search(commands::search::SearchArgs),
    /// Saved searches: list, run, rm, rename
    Saved(commands::saved::SavedArgs),
    /// New papers from arXiv, PubMed or Crossref searches: add, list, check, rm
    Alert(commands::alert::AlertArgs),
    /// Who you are to collaborators: your user ID, your key for your other devices
    User(commands::user::UserArgs),
    /// Keep this library the same on your devices (directly, over QUIC)
    Sync(commands::sync::SyncArgs),
    /// Passages highlighted in the reader: list (as Markdown quotes), comment, remove
    Highlight(commands::highlight::HighlightArgs),
    /// PDFs: get (download), import, list, open, path, rm
    Pdf(commands::pdf::PdfArgs),
    /// Work with projects: new, list, tree, rename, rm
    Project(commands::project::ProjectArgs),
    /// Import a .bib file or a Zotero library (its data folder)
    Import(commands::import::ImportArgs),
    /// Write references as BibLaTeX, BibTeX, CSL-JSON or Hayagriva; keep files updated
    Export(commands::export::ExportArgs),
    /// Link this folder to a project: quire.toml (new, show)
    Folder(commands::folder::FolderArgs),
    /// Work with libraries: new, list, use, rename, rm
    Library(commands::library::LibraryArgs),
    /// Create the library and set your contact email (asked for by Crossref and NCBI)
    Init(commands::init::InitArgs),
    /// Read or change settings
    Config(commands::config::ConfigArgs),
    /// Local endpoint for a browser extension (save the page or PDF you are viewing)
    Serve(commands::serve::ServeArgs),
    /// Print a shell completion script
    Completions(commands::completions::CompletionsArgs),
}

/// Log level of quire's crates without `--verbose`: warnings only, so the output stays the command's.
const DEFAULT_LOG_LEVEL: tracing::Level = tracing::Level::WARN;

/// Our crates log at `DEFAULT_LOG_LEVEL` (debug with `--verbose`); dependencies only errors
/// (lopdf, used by pdf-inspector, warns about every odd font in a PDF).
fn init_logging(verbose: bool) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let ours = if verbose { tracing::Level::DEBUG } else { DEFAULT_LOG_LEVEL };
    let crates =
        ["quire", "quire_cli", "quire_engine", "quire_fetch", "quire_store", "quire_core", "quire_io", "quire_model"];
    let filter = crates.iter().fold(tracing_subscriber::filter::Targets::new(), |f, name| f.with_target(*name, ours));
    let filter = filter.with_default(tracing::Level::ERROR);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() -> ExitCode {
    restore_sigpipe();
    let cli = Cli::parse();
    init_logging(cli.verbose);
    let result = match Context::new(cli.lib.clone(), cli.library.clone(), cli.config.clone()) {
        Ok(context) => match run(&context, &cli.command).await {
            Ok(code) => auto_exports(&context).await.map(|()| code),
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            crate::err!("{} {error:#}", style::paint(style::ERROR, "error:"));
            ExitCode::FAILURE
        }
    }
}

async fn run(context: &Context, command: &Command) -> anyhow::Result<ExitCode> {
    migrate_config(context)?;
    let done = Ok(ExitCode::SUCCESS);
    match command {
        #[cfg(feature = "tui")]
        Command::Tui => {
            let engine = std::sync::Arc::new(context.open(context.choice()).await?);
            quire_tui::run(engine).await.map(|()| ExitCode::SUCCESS)
        }
        Command::Add(args) => reference::add::run(context, args).await,
        Command::List(args) => reference::list::run(context, args).await.and(done),
        Command::Show(args) => reference::show::run(context.engine().await?, args).await.and(done),
        Command::Reference(args) => reference::run(context, args).await,
        Command::Pdf(args) => commands::pdf::run(context, args).await,
        Command::Tag(args) => commands::tag::run(context.engine().await?, args).await.and(done),
        Command::Note(args) => commands::note::run(context.engine().await?, args).await.and(done),
        Command::Search(args) => commands::search::run(context.engine().await?, args).await.and(done),
        Command::Saved(args) => commands::saved::run(context.engine().await?, args).await.and(done),
        Command::Alert(args) => commands::alert::run(context.engine().await?, args).await,
        Command::User(args) => commands::user::run(context.engine().await?, args).and(done),
        Command::Sync(args) => commands::sync::run(context, args).await.and(done),
        Command::Highlight(args) => commands::highlight::run(context.engine().await?, args).await.and(done),
        Command::Project(args) => commands::project::run(context.engine().await?, args).await.and(done),
        Command::Import(args) => commands::import::run(context.engine().await?, args).await.and(done),
        Command::Export(args) => commands::export::run(context, args).await.and(done),
        Command::Folder(args) => commands::folder::run(context, args).await.and(done),
        Command::Library(args) => commands::library::run(context, args).await.and(done),
        Command::Init(args) => commands::init::run(context, args).await.and(done),
        Command::Config(args) => commands::config::run(context, args).await.and(done),
        Command::Serve(args) => commands::serve::run(context, args).await.and(done),
        Command::Completions(args) => commands::completions::run(args).and(done),
    }
}

/// After a command that changed the library: rewrites its automatic exports and says which.
async fn auto_exports(context: &Context) -> anyhow::Result<()> {
    let Some(engine) = context.opened() else { return Ok(()) };
    let results = engine.run_auto_exports(false).await?;
    commands::export::report_auto(&results, false);
    Ok(())
}

/// Rust ignores SIGPIPE, so printing into a closed pipe (`quire list | head`) panics. Restore
/// the default: the process ends quietly, like other Unix tools.
fn restore_sigpipe() {
    #[cfg(unix)]
    // SAFETY: called first in `main`, before any other thread exists; SIG_DFL is a valid handler.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

/// Moves a pre-0.2 `library.path` to a named library, once, and says so.
fn migrate_config(context: &Context) -> anyhow::Result<()> {
    if let Some(message) = quire_engine::config::migrate(&context.config_file()?)? {
        output::note(output::Tone::Info, "migrated", message);
    }
    Ok(())
}
