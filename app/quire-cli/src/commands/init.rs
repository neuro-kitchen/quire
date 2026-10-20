//! `init`: creates the chosen library (opening the engine does) and sets `user.email`. Without
//! `--email`, offers the address from `git config user.email` and asks before saving it, since
//! it will be sent to Crossref and NCBI (`.knowledge/config.md`).

use std::io::{BufRead, IsTerminal, Write};
use std::process::Command;

use clap::Args;
use quire_engine::{Engine, config};

use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Contact email to send to metadata services (skips the question)
    #[arg(long)]
    email: Option<String>,
}

pub async fn run(context: &Context, args: &InitArgs) -> anyhow::Result<()> {
    let engine = context.engine().await?;
    let library = engine.library();
    output::report(
        Tone::Ok,
        "library",
        format!("{}  {}", style::paint(style::KEY, library.label()), style::paint(style::DIM, library.path.display())),
    );
    output::report(Tone::Info, "config", style::paint(style::DIM, engine.config_file().display()));
    if library.name.is_none() {
        output::hint(
            "This folder is not a registered library; to keep using it by name: `quire library new <name> --path <folder>`.",
        );
    }
    if let Some(email) = &args.email {
        return save(engine, email);
    }
    if let Some(email) = &engine.config().user.email {
        output::report(Tone::Info, "email", email);
        return Ok(());
    }
    let suggestion = git_email();
    if !std::io::stdin().is_terminal() {
        output::note(
            Tone::Warn,
            "no email",
            "Crossref and NCBI ask for one; set it with `quire config set user.email <address>`.",
        );
        return Ok(());
    }
    let prompt = match &suggestion {
        Some(email) => format!("Contact email for Crossref/NCBI requests [{email}] (empty = use it, `-` = none): "),
        None => "Contact email for Crossref/NCBI requests (empty = none): ".into(),
    };
    eprint!("{prompt}");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    match (answer.trim(), suggestion) {
        ("-", _) | ("", None) => {
            output::hint("No email saved; set one later with `quire config set user.email <address>`.");
            Ok(())
        }
        ("", Some(email)) => save(engine, &email),
        (typed, _) => save(engine, typed),
    }
}

fn save(engine: &Engine, email: &str) -> anyhow::Result<()> {
    config::set(engine.config_file(), "user.email", email)?;
    output::report(Tone::Ok, "email", email);
    Ok(())
}

/// `git config --get user.email`, if git is installed and the value is set.
fn git_email() -> Option<String> {
    let output = Command::new("git").args(["config", "--get", "user.email"]).output().ok()?;
    let email = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (output.status.success() && !email.is_empty()).then_some(email)
}
