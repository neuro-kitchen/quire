//! `completions <shell>`: the completion script, e.g. `quire completions fish > ~/.config/fish/completions/quire.fish`.

use clap::{Args, CommandFactory};
use clap_complete::Shell;

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    shell: Shell,
}

pub fn run(args: &CompletionsArgs) -> anyhow::Result<()> {
    clap_complete::generate(args.shell, &mut crate::Cli::command(), "quire", &mut std::io::stdout());
    Ok(())
}
