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
}

#[derive(Subcommand, Debug)]
enum KeyAction {
    /// Print the key (run `quire user key import <key>` on your other device)
    Export,
    /// Use this key here (from `quire user key export` on your other device)
    Import { key: String },
}

pub fn run(engine: &Engine, args: &UserArgs) -> anyhow::Result<()> {
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
