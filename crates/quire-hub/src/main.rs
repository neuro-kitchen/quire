//! `quire-hub`: run the sync hub, or manage it from the same data folder.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};
use quire_hub::{HubServer, Purpose, Settings, Store, hub_id, hub_key};
use quire_sync::link::HubLink;

#[derive(Parser)]
#[command(name = "quire-hub", version, about = "quire's sync hub: stores your devices' encrypted library and syncs it")]
struct Cli {
    /// The hub's data folder (database, sealed files, its key).
    #[arg(long, env = "QUIRE_HUB_DATA", default_value = "/data", global = true)]
    data: PathBuf,
    /// Your own iroh relay's URL (default: n0's public relays); also put in the links it prints.
    #[arg(long, env = "QUIRE_HUB_RELAY", global = true)]
    relay: Option<String>,
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
        /// Sessions served at once.
        #[arg(long, env = "QUIRE_HUB_MAX_SESSIONS", default_value_t = 64)]
        max_sessions: usize,
    },
    /// Print the hub's ID (what you add in quire).
    Id,
    /// Print the link (and QR code) that adds this hub in quire, with the setup code while unused.
    Link,
    /// Make a one-time code inviting another person (valid 7 days), as a link and QR code.
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
    // An empty value (compose passes unset variables as "") means none.
    let relay = cli.relay.filter(|r| !r.trim().is_empty());
    match cli.command.expect("set above") {
        Command::Serve { quota_mb, max_sessions } => {
            let key = hub_key(&cli.data).context("the hub's key")?;
            let server = HubServer::bind(&key, relay.as_deref()).await.map_err(anyhow::Error::msg)?;
            println!("quire hub {} — ID: {}", env!("CARGO_PKG_VERSION"), server.id());
            if let Some(code) = store.owner_code()? {
                println!("Setup code (first device, once): {code}");
                show_link(&HubLink { id: server.id(), relay: relay.clone(), code: Some(code) });
            }
            let settings = Settings { quota: quota_mb.saturating_mul(1024 * 1024) };
            tokio::select! {
                () = server.run(Arc::new(store), settings, max_sessions) => {}
                () = stopped() => tracing::info!("stopping"),
            }
            server.close().await;
        }
        Command::Id => println!("{}", hub_id(&hub_key(&cli.data)?)),
        Command::Link => {
            let code = store.owner_code()?;
            show_link(&HubLink { id: hub_id(&hub_key(&cli.data)?), relay, code });
        }
        Command::Invite => {
            let code = store.new_code(Purpose::Invite, None, Some(Duration::from_secs(7 * 24 * 3600)))?;
            println!("Invitation code (one person, 7 days): {code}");
            show_link(&HubLink { id: hub_id(&hub_key(&cli.data)?), relay, code: Some(code) });
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

/// Prints `link` for quire (paste it, or scan the QR code with the phone) and the CLI's command.
fn show_link(link: &HubLink) {
    let text = link.to_uri();
    println!("Add this hub in quire: paste this link, or scan the QR code with the phone:\n{text}");
    match qrcode::QrCode::new(text.as_bytes()) {
        Ok(qr) => println!("{}", qr.render::<qrcode::render::unicode::Dense1x2>().quiet_zone(true).build()),
        Err(error) => tracing::warn!(%error, "no QR code for the link"),
    }
    println!("Command line: quire sync hub add '{text}'");
}
