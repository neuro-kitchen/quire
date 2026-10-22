//! `user id | key export | key import`: who you are to collaborators. Your user key is the same
//! on all your devices (copy it with `key export` / `key import`); projects are shared with your
//! user ID.

use clap::{Args, Subcommand};
use quire_engine::Engine;

use crate::output::{self, Tone};

#[derive(Args, Debug)]
pub struct UserArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Your user ID: what others share projects with
    Id,
    /// Your user key, to give your other devices (keep it secret)
    Key {
        #[command(subcommand)]
        action: KeyAction,
    },
    /// What to keep safe before syncing through a hub (the hub cannot recover your library)
    RecoveryKit,
    /// A library's later keys (after a device was unlinked), from a recovery kit
    LibraryKeys {
        #[command(subcommand)]
        action: LibraryKeysAction,
    },
}

#[derive(Subcommand, Debug)]
enum LibraryKeysAction {
    /// Use the keys of a recovery kit's `Library keys:` line (`<library ID>=<epoch>:<key>,…`)
    Import { keys: String },
}

#[derive(Subcommand, Debug)]
enum KeyAction {
    /// Print the key (run `quire user key import <key>` on your other device)
    Export,
    /// Use this key here (from `quire user key export` on your other device)
    Import { key: String },
}

pub async fn run(engine: &Engine, args: &UserArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::Id => {
            let (id, store) = engine.user_id()?;
            println!("{id}");
            output::hint(format!("key kept in {}", store_label(&store)));
        }
        Action::Key { action: KeyAction::Export } => {
            output::report(
                Tone::Warn,
                "secret",
                "anyone with this key can act as you; copy it to your own devices only",
            );
            println!("{}", engine.export_user_key()?);
        }
        Action::RecoveryKit => {
            output::report(Tone::Warn, "secret", "print it or keep it in a password manager; never share it");
            print!("{}", engine.recovery_kit().await?);
        }
        Action::LibraryKeys { action: LibraryKeysAction::Import { keys } } => {
            engine.import_library_keys(keys)?;
            output::report(Tone::Ok, "imported", "the library's later keys");
        }
        Action::Key { action: KeyAction::Import { key } } => {
            let (id, store) = engine.import_user_key(key)?;
            output::report(Tone::Ok, "imported", format!("user {} (kept in {})", id.short(), store_label(&store)));
        }
    }
    Ok(())
}

fn store_label(store: &quire_engine::secrets::Store) -> String {
    match store {
        quire_engine::secrets::Store::Keyring => "the system keyring".into(),
        quire_engine::secrets::Store::File(path) => path.display().to_string(),
    }
}
