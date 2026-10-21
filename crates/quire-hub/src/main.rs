//! `quire-hub`: run the sync hub, or manage it from the same data folder.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};
use quire_hub::{HubServer, Purpose, Settings, Store, hub_id, hub_key};

#[derive(Parser)]
#[command(name = "quire-hub", version, about = "quire's sync hub: stores your devices' encrypted library and syncs it")]
struct Cli {
    /// The hub's data folder (database, sealed files, its key).
    #[arg(long, env = "QUIRE_HUB_DATA", default_value = "/data", global = true)]
    data: PathBuf,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the hub (the default).
    Serve {
        /// Storage limit of each new account, in megabytes (0: none).
        #[arg(long, env = "QUIRE_HUB_QUOTA_MB", default_value_t = 0)]
        quota_mb: u64,
        /// Your own iroh relay's URL (default: n0's public relays).
        #[arg(long, env = "QUIRE_HUB_RELAY")]
        relay: Option<String>,
        /// Sessions served at once.
        #[arg(long, env = "QUIRE_HUB_MAX_SESSIONS", default_value_t = 64)]
        max_sessions: usize,
    },
    /// Print the hub's ID (what you add in quire).
    Id,
    /// Make a one-time code inviting another person (valid 7 days).
    Invite,
    /// Exit 0 if the hub's data folder and database work (the container's health check).
    Health,
    /// Accounts made before their person registers (the cloud's account page), and their removal
    Account {
        #[command(subcommand)]
        action: AccountAction,
    },
}

#[derive(Subcommand)]
enum AccountAction {
    /// Make an account and print its ID and a link code (valid 24 hours) for its person's app
    New {
        /// Its storage limit in megabytes (0: none)
        #[arg(long, default_value_t = 0)]
        quota_mb: u64,
    },
    /// Delete an account and everything it stored (its person's request)
    Delete { id: i64 },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli = Cli::parse();
    if cli.command.is_none() {
        // No subcommand: `serve`, with its options read from the environment as usual.
        cli = Cli::parse_from(std::env::args().chain(["serve".to_string()]));
    }
    // Colours only on a terminal (a container's log is a file).
    tracing_subscriber::fmt()
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "quire_hub=info,warn".into()),
        )
        .init();
    let store = Store::open(&cli.data).with_context(|| format!("data folder {}", cli.data.display()))?;
    match cli.command.expect("set above") {
        Command::Serve { quota_mb, relay, max_sessions } => {
            // An empty value (compose passes unset variables as "") means none.
            let relay = relay.filter(|r| !r.trim().is_empty());
            let key = hub_key(&cli.data).context("the hub's key")?;
            let server = HubServer::bind(&key, relay.as_deref()).await.map_err(anyhow::Error::msg)?;
            println!("quire hub {} — ID: {}", env!("CARGO_PKG_VERSION"), server.id());
            if let Some(code) = store.owner_code()? {
                println!("Setup code (first device, once): {code}");
                println!("In quire: `quire sync hub add {} {code}`", server.id());
            }
            let settings = Settings { quota: quota_mb.saturating_mul(1024 * 1024) };
            tokio::select! {
                () = server.run(Arc::new(store), settings, max_sessions) => {}
                () = stopped() => tracing::info!("stopping"),
            }
            server.close().await;
        }
        Command::Id => println!("{}", hub_id(&hub_key(&cli.data)?)),
        Command::Invite => {
            println!("{}", store.new_code(Purpose::Invite, None, Some(Duration::from_secs(7 * 24 * 3600)))?);
        }
        Command::Account { action: AccountAction::New { quota_mb } } => {
            let (id, code) = store.new_account(quota_mb.saturating_mul(1024 * 1024))?;
            println!("account {id}\nlink code {code}");
        }
        Command::Account { action: AccountAction::Delete { id } } => {
            store.delete_account(id)?;
            println!("deleted account {id}");
        }
        Command::Health => {
            anyhow::ensure!(store.healthy(), "the hub's database does not answer");
            hub_key(&cli.data)?;
        }
    }
    Ok(())
}

/// Ctrl-C, or SIGTERM (how a container is stopped).
async fn stopped() {
    #[cfg(unix)]
    {
        let mut term =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
