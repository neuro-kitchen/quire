//! `ref mark <status> <refs…>`: reading status (unread, reading, read). Opening a paper in the
//! reader marks it reading.

use clap::Args;
use quire_engine::{Engine, ReadingStatus};

use crate::output::{self, Tone};

#[derive(Args, Debug)]
pub struct MarkArgs {
    /// unread, reading or read
    status: ReadingStatus,
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required = true)]
    queries: Vec<String>,
}

pub async fn run(engine: &Engine, args: &MarkArgs) -> anyhow::Result<std::process::ExitCode> {
    let mut failed = 0;
    for query in &args.queries {
        match engine.set_reading(query, args.status).await {
            Ok((reference, _, true)) => output::report(Tone::Ok, args.status.as_str(), &reference.cite_key),
            Ok((reference, _, false)) => {
                output::report(Tone::Warn, "already", format!("{} {}", reference.cite_key, args.status))
            }
            Err(error) => {
                failed += 1;
                output::report(Tone::Error, "failed", format!("{query}: {error}"));
            }
        }
    }
    Ok(super::exit_code(failed, args.queries.len()))
}
