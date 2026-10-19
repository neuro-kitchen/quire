//! `quire`: a thin command line over `quire-engine`. Each command lives in its own module with
//! its own arguments; this file only parses and dispatches.
//!
//! Results go to stdout, messages and logs to stderr. Exit codes: 0 success, 1 error, 2 usage
//! error (clap), 3 partial success (some inputs failed).

mod commands;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use quire_engine::{Engine, OpenOptions};

#[derive(Parser, Debug)]
#[command(name = "quire", version, about = "A reference manager: projects, DOI/arXiv/PubMed/URL lookup, .bib export")]
struct Cli {
    /// Library directory (default: ~/.local/share/quire/library)
    #[arg(long, global = true, env = quire_engine::paths::LIBRARY_ENV)]
    library: Option<PathBuf>,

    /// Config file (default: ~/.config/quire/config.toml)
    #[arg(long, global = true, env = quire_engine::paths::CONFIG_ENV)]
    config: Option<PathBuf>,

    /// Show debug logs (requests, database)
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create the library and set your contact email (asked for by Crossref and NCBI)
    Init(commands::init::InitArgs),
    /// Read or change settings (user.email, user.name, library.path)
    Config(commands::config::ConfigArgs),
    /// Create and list projects (groups of references; can be nested)
    Project(commands::project::ProjectArgs),
    /// Add references by DOI, arXiv ID, PMID, PMCID or URL
    Add(commands::add::AddArgs),
    /// Import a .bib file
    Import(commands::import::ImportArgs),
    /// List references (of a project, or the whole library)
    List(commands::list::ListArgs),
    /// Show one reference
    Show(commands::show::ShowArgs),
    /// Put a reference into a project
    Link(commands::link::LinkArgs),
    /// Take a reference out of a project (it stays in the library)
    Unlink(commands::link::LinkArgs),
    /// Move a reference to the trash
    Rm(commands::trash::TrashArgs),
    /// Take a reference out of the trash
    Restore(commands::trash::TrashArgs),
    /// Write references as BibLaTeX, BibTeX or CSL-JSON
    Export(commands::export::ExportArgs),
}

impl Cli {
    fn open_options(&self) -> OpenOptions {
        OpenOptions { library: self.library.clone(), config_file: self.config.clone(), endpoints: None }
    }

    async fn engine(&self) -> anyhow::Result<Engine> {
        Ok(Engine::open(self.open_options()).await?)
    }
}

/// Log level without `--verbose`: warnings only, so the output stays the command's.
const DEFAULT_LOG_LEVEL: tracing::Level = tracing::Level::WARN;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let level = if cli.verbose { tracing::Level::DEBUG } else { DEFAULT_LOG_LEVEL };
    tracing_subscriber::fmt().with_max_level(level).with_writer(std::io::stderr).init();
    match run(&cli).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: &Cli) -> anyhow::Result<ExitCode> {
    let done = Ok(ExitCode::SUCCESS);
    match &cli.command {
        Command::Init(args) => commands::init::run(cli.engine().await?, args).await.and(done),
        Command::Config(args) => commands::config::run(cli.config.as_deref(), args).and(done),
        Command::Project(args) => commands::project::run(&cli.engine().await?, args).await.and(done),
        Command::Add(args) => commands::add::run(&cli.engine().await?, args).await,
        Command::Import(args) => commands::import::run(&cli.engine().await?, args).await.and(done),
        Command::List(args) => commands::list::run(&cli.engine().await?, args).await.and(done),
        Command::Show(args) => commands::show::run(&cli.engine().await?, args).await.and(done),
        Command::Link(args) => commands::link::run(&cli.engine().await?, args, true).await.and(done),
        Command::Unlink(args) => commands::link::run(&cli.engine().await?, args, false).await.and(done),
        Command::Rm(args) => commands::trash::run(&cli.engine().await?, args, true).await.and(done),
        Command::Restore(args) => commands::trash::run(&cli.engine().await?, args, false).await.and(done),
        Command::Export(args) => commands::export::run(&cli.engine().await?, args).await.and(done),
    }
}
