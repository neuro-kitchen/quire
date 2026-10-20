//! `config get|set|unset|list|keys`: settings in the config file, git-style keys (`user.email`,
//! `libraries.lab.path`). Works without opening a library.

use clap::{Args, Subcommand};
use quire_engine::config::{self, Config, KEYS};

use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Print a value (nothing if unset)
    Get { key: String },
    /// Set a value (lists: comma-separated)
    Set { key: String, value: String },
    /// Remove a value
    Unset { key: String },
    /// Print every set value
    List,
    /// Print every key quire knows, with what it does
    Keys,
}

pub async fn run(context: &Context, args: &ConfigArgs) -> anyhow::Result<()> {
    let file = context.config_file()?;
    match &args.action {
        Action::Get { key } => {
            if let Some(value) = Config::load(&file)?.get(key)? {
                println!("{value}");
            }
        }
        Action::Set { key, value } => {
            config::set(&file, key, value)?;
            output::report(Tone::Ok, "set", format!("{} = {value}", style::paint(style::KEY, key)));
        }
        Action::Unset { key } => {
            if config::unset(&file, key)? {
                output::report(Tone::Ok, "unset", style::paint(style::KEY, key));
            } else {
                output::report(Tone::Warn, "unchanged", format!("{} was not set", style::paint(style::KEY, key)));
            }
        }
        Action::List => {
            for (key, value) in Config::load(&file)?.entries() {
                anstream::println!("{}={value}", style::paint(style::KEY, key));
            }
            output::hint(format!("from {}", file.display()));
        }
        Action::Keys => {
            let width = KEYS.iter().map(|k| k.pattern.len()).max().unwrap_or(0);
            for spec in KEYS {
                anstream::println!("{}  {}", style::paint(style::KEY, format!("{:<width$}", spec.pattern)), spec.help);
            }
        }
    }
    Ok(())
}
