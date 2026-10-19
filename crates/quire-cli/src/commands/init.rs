//! `init`: creates the library (opening the engine does) and sets `user.email`. Without
//! `--email`, offers the address from `git config user.email` and asks before saving it, since
//! it will be sent to Crossref and NCBI (`.knowledge/config.md`).

use std::io::{BufRead, IsTerminal, Write};
use std::process::Command;

use clap::Args;
use quire_engine::{config, Engine};

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Contact email to send to metadata services (skips the question)
    #[arg(long)]
    email: Option<String>,
}

pub async fn run(engine: Engine, args: &InitArgs) -> anyhow::Result<()> {
    println!("library  {}", engine.library_dir().display());
    println!("config   {}", engine.config_file().display());
    if let Some(email) = &args.email {
        return save(&engine, email);
    }
    if let Some(email) = &engine.config().user.email {
        println!("email    {email}");
        return Ok(());
    }
    let suggestion = git_email();
    if !std::io::stdin().is_terminal() {
        eprintln!("No contact email set. Crossref and NCBI ask for one; set it with `quire config set user.email <address>`.");
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
            eprintln!("No email saved; set one later with `quire config set user.email <address>`.");
            Ok(())
        }
        ("", Some(email)) => save(&engine, &email),
        (typed, _) => save(&engine, typed),
    }
}

fn save(engine: &Engine, email: &str) -> anyhow::Result<()> {
    config::set(engine.config_file(), "user.email", email)?;
    println!("email    {email}");
    Ok(())
}

/// `git config --get user.email`, if git is installed and the value is set.
fn git_email() -> Option<String> {
    let output = Command::new("git").args(["config", "--get", "user.email"]).output().ok()?;
    let email = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (output.status.success() && !email.is_empty()).then_some(email)
}
