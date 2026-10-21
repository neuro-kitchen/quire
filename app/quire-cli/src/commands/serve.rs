//! `serve`: the local endpoint for a browser extension (`quire-connector`). Listens on 127.0.0.1;
//! requests need the token `--token` prints (kept in the system keyring). Runs until Ctrl-C.

use std::sync::Arc;

use clap::Args;
use quire_connector::{DEFAULT_PORT, token};

use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Port on 127.0.0.1
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
    /// Print the access token (to give to the browser extension) and stop
    #[arg(long, conflicts_with = "reset_token")]
    token: bool,
    /// Make a new access token (the old one stops working) and stop
    #[arg(long)]
    reset_token: bool,
}

pub async fn run(context: &Context, args: &ServeArgs) -> anyhow::Result<()> {
    let config = context.config_file()?;
    if args.reset_token {
        token::reset(&config)?;
    }
    let (token, store) = token::load_or_create(&config)?;
    let kept = match &store {
        token::Store::Keyring => "in the system keyring".to_string(),
        token::Store::File(file) => format!("in {} (no keyring available; readable by you only)", file.display()),
    };
    if args.token || args.reset_token {
        println!("{token}");
        output::hint(format!("Token kept {kept}."));
        return Ok(());
    }
    let engine = Arc::new(context.open(context.choice()).await?);
    let library = engine.library().label();
    output::report(
        Tone::Ok,
        "serving",
        format!("http://127.0.0.1:{}  library {}", args.port, style::paint(style::KEY, library)),
    );
    output::hint(format!("Token kept {kept} (`quire serve --token` prints it). Ctrl-C stops."));
    tokio::select! {
        result = quire_connector::serve(engine, args.port, token) => result.map_err(|e| anyhow::anyhow!("port {}: {e}", args.port))?,
        _ = tokio::signal::ctrl_c() => output::report(Tone::Info, "stopped", "Ctrl-C"),
    }
    Ok(())
}
