//! `sync`: keep this library the same on your devices, directly between them over QUIC (no
//! server in between; `.knowledge/sync.md`). On each device: `quire sync id` gives its ID; add
//! the other one with `quire sync peer add <name> <id>`. Then one runs `quire sync serve` and the
//! other `quire sync with <name>`. A new, empty library joins the other's on its first sync.

use clap::{Args, Subcommand};
use quire_engine::SyncReport;
use quire_engine::config;

use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct SyncArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Print this device's ID (to add it as a peer on your other devices)
    Id,
    /// Your other devices
    Peer {
        #[command(subcommand)]
        action: PeerAction,
    },
    /// Sync with a device now (it must be running `quire sync serve`)
    With {
        /// A peer's name, or a device ID
        peer: String,
    },
    /// Wait for your devices to sync with this one (Ctrl-C stops)
    Serve,
}

#[derive(Subcommand, Debug)]
enum PeerAction {
    /// Add a device: a name you choose and its ID (`quire sync id` on it)
    Add { name: String, id: String },
    /// List your devices
    List,
    /// Forget a device
    Rm { name: String },
}

pub async fn run(context: &Context, args: &SyncArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::Id => {
            let engine = context.engine().await?;
            let (id, store) = engine.sync_id()?;
            println!("{id}");
            let kept = match store {
                quire_engine::SecretStore::Keyring => "in the system keyring".to_string(),
                quire_engine::SecretStore::File(file) => format!("in {} (readable by you only)", file.display()),
            };
            output::hint(format!(
                "This device's key is kept {kept}. On your other devices: `quire sync peer add <name> {id}`"
            ));
        }
        Action::Peer { action } => peer(context, action)?,
        Action::With { peer } => {
            let engine = context.engine().await?;
            output::hint(format!("connecting to {peer}…"));
            let report = engine.sync_with(peer).await?;
            show(&report);
        }
        Action::Serve => {
            let engine = context.engine().await?;
            let library = engine.library().label();
            let serving = engine.sync_serve(
                |id| {
                    output::report(
                        Tone::Ok,
                        "serving",
                        format!("library {}  device {id}", style::paint(style::KEY, &library)),
                    );
                    output::hint("Your peers can sync now (`quire sync with <name>`). Ctrl-C stops.");
                },
                |outcome| match outcome {
                    Ok(report) => show(&report),
                    Err(error) => output::note(Tone::Warn, "refused", error),
                },
            );
            tokio::select! {
                result = serving => result?,
                _ = tokio::signal::ctrl_c() => output::report(Tone::Info, "stopped", "Ctrl-C"),
            }
        }
    }
    Ok(())
}

fn peer(context: &Context, action: &PeerAction) -> anyhow::Result<()> {
    let file = context.config_file()?;
    match action {
        PeerAction::Add { name, id } => {
            anyhow::ensure!(quire_engine::is_device_id(id), "`{id}` is not a device ID (`quire sync id` prints one)");
            config::set(&file, &format!("peers.{name}.id"), id.trim())?;
            output::report(
                Tone::Ok,
                "added",
                format!("{} {}", style::paint(style::KEY, name), style::paint(style::DIM, id)),
            );
        }
        PeerAction::List => {
            let peers = config::Config::load(&file)?.peers;
            if peers.is_empty() {
                output::hint("No peers yet: `quire sync peer add <name> <device-id>`.");
            }
            for (name, peer) in peers {
                anstream::println!("{}  {}", style::paint(style::KEY, name), style::paint(style::DIM, peer.id));
            }
        }
        PeerAction::Rm { name } => {
            if config::unset(&file, &format!("peers.{name}.id"))? {
                output::report(Tone::Ok, "removed", style::paint(style::KEY, name));
            } else {
                output::report(Tone::Warn, "unchanged", format!("no peer {name}"));
            }
        }
    }
    Ok(())
}

/// What a session did.
fn show(report: &SyncReport) {
    let r = &report.received;
    let peer: String = report.peer.chars().take(12).collect();
    let mut parts = vec![format!("{} change(s) sent, {} received", report.sent, r.applied)];
    if report.files_sent + report.files_received > 0 {
        parts.push(format!("{} PDF(s) sent, {} received", report.files_sent, report.files_received));
    }
    if report.joined {
        parts.push("joined its library".into());
    }
    if let Some(user) = &report.collaborator {
        parts.push(format!("a collaborator ({}…): shared projects only", &user[..12.min(user.len())]));
    }
    output::report(Tone::Ok, "synced", format!("with {peer}…  {}", parts.join("; ")));
    if r.refused > 0 {
        output::note(
            Tone::Warn,
            "refused",
            format!("{} change(s): beyond the sender's role, or stamped by a clock more than a day ahead", r.refused),
        );
    }
    for (old, new) in &r.renamed {
        output::note(Tone::Warn, "renamed", format!("{old} → {new} (another paper had that key)"));
    }
    for (key, id) in &r.dropped_identifiers {
        output::note(
            Tone::Warn,
            "duplicate?",
            format!("{key}: {id} belongs to another reference here (`quire ref duplicates`)"),
        );
    }
}
