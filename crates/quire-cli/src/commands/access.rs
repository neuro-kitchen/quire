//! `access login|status|logout`: the institutional proxy session. You log in to the proxy in
//! your own browser and paste its session cookie here; quire checks it, keeps it in the system
//! keyring, and `pdf get` then downloads subscribed PDFs through the proxy. Your password never
//! reaches quire, and the cookie goes to the proxy's domain only.

use std::io::IsTerminal;

use clap::{Args, Subcommand};
use quire_engine::{Engine, SecretStore};

use super::pdf::browser;
use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct AccessArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Log in to the proxy in your browser, then paste its session cookie
    Login {
        /// Do not open the browser (log in yourself, then paste)
        #[arg(long)]
        no_browser: bool,
    },
    /// Show the proxy and whether the session still works
    Status,
    /// Forget the session
    Logout,
}

pub async fn run(context: &Context, args: &AccessArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    match &args.action {
        Action::Login { no_browser } => login(engine, !no_browser).await,
        Action::Status => {
            let status = engine.proxy_status(true).await?;
            let Some(prefix) = status.prefix else {
                output::hint("No proxy set: `quire config set access.proxy <prefix>`.");
                return Ok(());
            };
            output::report(Tone::Info, "proxy", &prefix);
            match (status.stored, status.active) {
                (None, _) => output::report(Tone::Warn, "session", "none: `quire access login`"),
                (Some(store), Some(true)) => {
                    output::report(Tone::Ok, "session", format!("logged in ({})", kept(&store)))
                }
                (Some(_), _) => output::report(Tone::Warn, "session", "expired: `quire access login`"),
            }
            Ok(())
        }
        Action::Logout => {
            if engine.proxy_logout()? {
                output::report(Tone::Info, "logged out", "the proxy session is forgotten");
            } else {
                output::hint("There was no proxy session.");
            }
            Ok(())
        }
    }
}

/// Opens the proxy login, explains where the cookie is, reads it without echo, checks and keeps it.
pub async fn login(engine: &Engine, open_browser: bool) -> anyhow::Result<()> {
    anyhow::ensure!(std::io::stdin().is_terminal(), "`quire access login` needs a terminal to paste the cookie into");
    let page = engine.proxy_login_page()?;
    let host = page.host_str().unwrap_or_default().to_string();
    let domain = engine.proxy_domain()?;
    if open_browser {
        output::note(Tone::Info, "opening", page.as_str());
        browser::open(page.as_str())?;
    } else {
        output::note(Tone::Info, "log in at", page.as_str());
    }
    anstream::eprintln!(
        "\n  1. Log in with your university account until the article page opens.\n  \
         2. Open the developer tools (F12):\n     \
            Firefox: Storage → Cookies → https://{host}\n     \
            Chrome/Brave: Application → Cookies → https://{host}\n  \
         3. Copy the value of the cookie named {} (EZproxy's session).\n",
        style::paint(style::KEY, "ezproxy")
    );
    output::hint(format!("It is checked once, kept in your system keyring, and only ever sent to *.{domain}."));
    let pasted = rpassword::prompt_password("  Paste it here (hidden): ")?;
    let store = engine.proxy_login(&pasted).await?;
    output::report(Tone::Ok, "logged in", format!("{} ({})", style::paint(style::KEY, &domain), kept(&store)));
    output::hint(
        "`quire pdf get` now downloads through the proxy (one request every 5 s, to keep your account in good standing).",
    );
    Ok(())
}

fn kept(store: &SecretStore) -> String {
    match store {
        SecretStore::Keyring => "kept in the system keyring".into(),
        SecretStore::File(file) => format!("kept in {} — no keyring available", file.display()),
    }
}
