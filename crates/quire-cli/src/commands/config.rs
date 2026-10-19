//! `config get|set|list`: settings in the config file, git-style keys (`user.email`). Works
//! without opening the library.

use std::path::Path;

use clap::{Args, Subcommand};
use quire_engine::{config, paths, Config};

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Print a value (nothing if unset)
    Get { key: String },
    /// Set a value
    Set { key: String, value: String },
    /// Print every set value
    List,
}

pub fn run(config_flag: Option<&Path>, args: &ConfigArgs) -> anyhow::Result<()> {
    let file = paths::config_file(config_flag)?;
    match &args.action {
        Action::Get { key } => {
            if let Some(value) = Config::load(&file)?.get(key)? {
                println!("{value}");
            }
        }
        Action::Set { key, value } => config::set(&file, key, value)?,
        Action::List => {
            let loaded = Config::load(&file)?;
            for key in config::KEYS {
                if let Some(value) = loaded.get(key)? {
                    println!("{key}={value}");
                }
            }
        }
    }
    Ok(())
}
