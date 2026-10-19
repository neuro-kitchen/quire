//! `ref <action>`: everything done to references (add, list, show, link, unlink, rm, restore,
//! copy, check, refetch, edit). `add`, `list` and `show` are also top-level
//! commands.
//!
//! One module per action; `link` also serves `unlink`, `trash` serves `rm` and `restore`.

pub mod abstracts;
pub mod add;
pub mod check;
pub mod copy;
pub mod edit;
pub mod link;
pub mod list;
pub mod refetch;
pub mod relate;
pub mod show;
pub mod trash;

use std::process::ExitCode;

use clap::{Args, Subcommand};

use crate::context::Context;

#[derive(Args, Debug)]
pub struct RefArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Add references by DOI, arXiv ID, PMID, PMCID or URL
    Add(add::AddArgs),
    /// List references as a table
    List(list::ListArgs),
    /// Show one reference
    Show(show::ShowArgs),
    /// Put references into a project
    Link(link::LinkArgs),
    /// Take references out of a project (they stay in the library)
    Unlink(link::LinkArgs),
    /// Move references to the trash
    Rm(trash::TrashArgs),
    /// Take references out of the trash
    Restore(trash::TrashArgs),
    /// Mark two references as related (both ways)
    Relate(relate::RelateArgs),
    /// Remove the relation between two references
    Unrelate(relate::RelateArgs),
    /// Copy references to another library (same ID there)
    Copy(copy::CopyArgs),
    /// Fill in missing abstracts (Europe PMC, else the PDF)
    Abstracts(abstracts::AbstractsArgs),
    /// Report metadata issues and failed lookups
    Check(check::CheckArgs),
    /// Look references up again; fields you edited are kept
    Refetch(refetch::RefetchArgs),
    /// Edit a reference in your editor (as TOML)
    Edit(edit::EditArgs),
}

pub async fn run(context: &Context, args: &RefArgs) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    let done = Ok(ExitCode::SUCCESS);
    match &args.action {
        Action::Add(args) => add::run(context, args).await,
        Action::List(args) => list::run(context, args).await.and(done),
        Action::Show(args) => show::run(engine, args).await.and(done),
        Action::Link(args) => link::run(engine, args, true).await,
        Action::Unlink(args) => link::run(engine, args, false).await,
        Action::Rm(args) => trash::run(engine, args, true).await,
        Action::Restore(args) => trash::run(engine, args, false).await,
        Action::Relate(args) => relate::run(engine, args, true).await.and(done),
        Action::Unrelate(args) => relate::run(engine, args, false).await.and(done),
        Action::Copy(args) => copy::run(context, engine, args).await,
        Action::Abstracts(args) => abstracts::run(context, args).await,
        Action::Check(args) => check::run(context, args).await.and(done),
        Action::Refetch(args) => refetch::run(context, args).await,
        Action::Edit(args) => edit::run(engine, args).await.and(done),
    }
}

/// Exit code for a command over several inputs: success, failure if all failed, 3 if some did.
pub fn exit_code(failed: usize, total: usize) -> ExitCode {
    /// Exit code when some inputs failed and others succeeded.
    const PARTIAL: u8 = 3;
    match failed {
        0 => ExitCode::SUCCESS,
        n if n == total => ExitCode::FAILURE,
        _ => ExitCode::from(PARTIAL),
    }
}
