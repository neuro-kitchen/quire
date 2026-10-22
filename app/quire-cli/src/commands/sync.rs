//! `sync`: keep this library the same on your devices, directly between them over QUIC (no
//! server in between; `.knowledge/sync.md`). On each device: `quire sync id` gives its ID; add
//! the other one with `quire sync peer add <name> <id>`. Then one runs `quire sync serve` and the
//! other `quire sync with <name>`. A new, empty library joins the other's on its first sync.
//!
//! Or through a sync hub (`.knowledge/sync-hub.md`): an always-on store your devices push their
//! encrypted changes to and pull from, which cannot read them: `quire sync hub add <hub-id> <code>`
//! once, then `quire sync hub now` on any device.
//!
//! A new device gets this person's identity, hub and relay by linking (no key shown): `quire sync
//! link` on a device that has them shows a code (and QR code); `quire sync join <code>` on the new one.

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
    /// Link a new device: show a one-time code (and QR code) for it, then allow it (two minutes)
    Link,
    /// Join your other device's code (`quire sync link` there, or the app's "Link a device")
    Join {
        /// The code (`quire://device?…`)
        code: String,
        /// This device's name, shown on the other one (default: the host name)
        #[arg(long)]
        name: Option<String>,
    },
    /// Sync through a hub (self-hosted container or the cloud) that stores your encrypted library
    Hub {
        #[command(subcommand)]
        action: HubAction,
    },
}

#[derive(Subcommand, Debug)]
enum HubAction {
    /// Use a hub: the link it prints (`quire://hub?…`), or its ID (`quire-hub id`) and, the first
    /// time, its setup or link code
    Add { id: String, code: Option<String> },
    /// Sync through the hub now
    Now,
    /// This person's devices the hub knows
    Devices,
    /// Unlink a device (a lost phone): the hub refuses it and the library key rotates, so it
    /// cannot read anything new. Its device ID (`quire sync hub devices`)
    #[command(alias = "revoke")]
    Unlink { device: String },
    /// Stop using the hub (it keeps your encrypted data until you delete your account there)
    Rm,
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
        Action::Hub { action } => hub(context, action).await?,
        Action::Link => link(context).await?,
        Action::Join { code, name } => join(context, code, name.as_deref()).await?,
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
                crate::out!("{}  {}", style::paint(style::KEY, name), style::paint(style::DIM, peer.id));
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

async fn hub(context: &Context, action: &HubAction) -> anyhow::Result<()> {
    let file = context.config_file()?;
    match action {
        HubAction::Add { id, code } => {
            // A link carries the ID, our own relay and the code; a code given beside it wins.
            let (id, code) = if id.trim().starts_with("quire:") {
                let link = quire_engine::HubLink::parse(id)?;
                match &link.relay {
                    Some(relay) => config::set(&file, "sync.relay", relay)?,
                    None => _ = config::unset(&file, "sync.relay")?,
                }
                (link.id, code.clone().or(link.code))
            } else {
                (id.trim().to_string(), code.clone())
            };
            anyhow::ensure!(quire_engine::is_device_id(&id), "`{id}` is not a hub ID (`quire-hub id` prints it)");
            config::set(&file, "hub.id", &id)?;
            let engine = context.engine().await?;
            output::hint("connecting to the hub…");
            let report = engine.sync_with_hub(code.as_deref()).await?;
            show_hub(&report);
            if report.registered {
                output::hint(
                    "The hub cannot read your library, so it cannot recover it either: keep your recovery kit \
                     safe (`quire user recovery-kit`).",
                );
            }
        }
        HubAction::Now => {
            let engine = context.engine().await?;
            output::hint("connecting to the hub…");
            show_hub(&engine.sync_with_hub(None).await?);
        }
        HubAction::Devices => {
            let engine = context.engine().await?;
            let (me, _) = engine.sync_id()?;
            for device in engine.hub_devices().await? {
                let name = device.name.as_deref().unwrap_or("unnamed device");
                let mark = if device.device == me { "  (this one)" } else { "" };
                let state = if device.revoked { style::paint(style::DIM, "  unlinked") } else { "".into() };
                // The ID only shortened: enough to tell devices apart, and to unlink one by its start.
                let short = device.device.get(..12).unwrap_or(&device.device);
                crate::out!(
                    "{}{mark}{state}  {}",
                    style::paint(style::KEY, name),
                    style::paint(style::DIM, format!("{short}…"))
                );
                if device.last_session > 0 {
                    crate::out!(
                        "    last sync {}: sent {} batch(es), received {}; {} PDF(s) up, {} down",
                        when(device.last_session),
                        device.batches_in,
                        device.batches_out,
                        device.files_in,
                        device.files_out
                    );
                } else {
                    crate::out!("    last seen {}", when(device.last_seen));
                }
            }
        }
        HubAction::Unlink { device } => {
            let engine = context.engine().await?;
            let rotation = engine.hub_unlink(device).await?;
            output::report(
                Tone::Ok,
                "unlinked",
                format!(
                    "{}: the key rotated (epoch {}), sealed for {} device(s)",
                    style::paint(style::KEY, device),
                    rotation.epoch,
                    rotation.sealed_for
                ),
            );
            if !rotation.left_out.is_empty() {
                output::note(
                    Tone::Warn,
                    "left out",
                    format!(
                        "{} device(s) run an older quire and cannot read new data: update and link them again",
                        rotation.left_out.len()
                    ),
                );
            }
            output::hint("keep your recovery kit again: `quire user recovery-kit` (it now holds the new key)");
        }
        HubAction::Rm => {
            if config::unset(&file, "hub.id")? {
                output::report(Tone::Ok, "removed", "this device no longer syncs through the hub");
            } else {
                output::report(Tone::Warn, "unchanged", "no hub was set");
            }
        }
    }
    Ok(())
}

/// What a hub session did.
fn show_hub(report: &quire_engine::HubReport) {
    let mut parts = vec![format!("{} change(s) sent, {} received", report.pushed, report.pulled.applied)];
    if report.files_uploaded + report.files_downloaded > 0 {
        parts.push(format!("{} PDF(s) uploaded, {} downloaded", report.files_uploaded, report.files_downloaded));
    }
    if report.joined {
        parts.push("joined your library".into());
    }
    if report.registered {
        parts.push("registered".into());
    }
    if report.quota > 0 {
        parts.push(format!("{} of {} MB used", report.used / (1024 * 1024), report.quota / (1024 * 1024)));
    }
    output::report(Tone::Ok, "synced", format!("through the hub  {}", parts.join("; ")));
    if report.over_quota > 0 {
        output::note(
            Tone::Warn,
            "full",
            format!("{} PDF(s) not uploaded: the hub's storage is full", report.over_quota),
        );
    }
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

/// Offers to link a device: shows the code, then asks before giving it anything.
async fn link(context: &Context) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let (offer, grant) = engine.link_offer().await?;
    let code = offer.link().to_uri();
    println!("On the new device, scan this QR code, or run `quire sync join '{code}'`:");
    match qrcode::QrCode::new(code.as_bytes()) {
        Ok(qr) => println!("{}", qr.render::<qrcode::render::unicode::Dense1x2>().quiet_zone(true).build()),
        Err(error) => output::hint(format!("no QR code: {error}")),
    }
    output::hint("valid for two minutes, once");
    let request = offer.request().await;
    let request = match request {
        Ok(request) => request,
        Err(error) => {
            offer.close().await;
            return Err(error.into());
        }
    };
    println!("\n{} asks to join. Check words: {}", style::paint(style::KEY, &request.name), request.words.join(" "));
    let request_name = request.name.clone();
    // A security decision: yes only when typed.
    eprint!("Do the same words show on that device? Allow it to have your identity and hub? [y/N] ");
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    let allowed = answer.trim().eq_ignore_ascii_case("y") || answer.trim().eq_ignore_ascii_case("yes");
    let outcome = if allowed {
        // The hub lets no unknown device in: introduce it first, then hand over the keys.
        match engine.hub_introduce(&request.device).await {
            Ok(()) => request.allow(&grant).await,
            Err(error) => {
                let _ = request.refuse().await;
                offer.close().await;
                return Err(error.into());
            }
        }
    } else {
        request.refuse().await
    };
    offer.close().await;
    if offer.wrong_codes() > 0 {
        engine
            .record_activity(quire_engine::activity::kind::WRONG_CODE, None, &offer.wrong_codes().to_string())
            .await?;
    }
    outcome?;
    let event =
        if allowed { quire_engine::activity::kind::LINK_ALLOWED } else { quire_engine::activity::kind::LINK_REFUSED };
    engine.record_activity(event, Some(&request_name), "").await?;
    if allowed {
        output::report(Tone::Ok, "linked", "the new device now syncs as you");
    } else {
        output::report(Tone::Warn, "refused", "nothing was given");
    }
    Ok(())
}

/// Joins another device's code: shows the check words, waits for the person to allow it there.
async fn join(context: &Context, code: &str, name: Option<&str>) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let name =
        name.map(str::to_string).or_else(|| std::env::var("HOSTNAME").ok()).unwrap_or_else(|| "a computer".into());
    output::hint("connecting to your other device…");
    let joining = engine.link_join(code, &name).await?;
    println!("Check words: {}", style::paint(style::KEY, joining.words.join(" ")));
    output::hint("allow this device on the other one if it shows the same words");
    let grant = joining.finish().await?;
    let (user, _) = engine.apply_grant(&grant).await?;
    output::report(Tone::Ok, "linked", format!("this device is now {user}"));
    if grant.hub.is_some() {
        output::hint("run `quire sync hub now` to get your library");
    }
    Ok(())
}

/// A time from the hub (milliseconds since 1970) as how long ago it was.
fn when(millis: i64) -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let seconds = (i128::try_from(now).unwrap_or(0) - i128::from(millis)).max(0) / 1000;
    match seconds {
        0..60 => "just now".into(),
        60..3600 => format!("{} min ago", seconds / 60),
        3600..86_400 => format!("{} h ago", seconds / 3600),
        _ => format!("{} days ago", seconds / 86_400),
    }
}
