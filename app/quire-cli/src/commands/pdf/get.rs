//! `pdf get`: find and download PDFs (page link, arXiv, Unpaywall, publisher page). Every
//! reference is first downloaded directly, also with `--browser` and also when an earlier run
//! found it blocked; only if that fails does `--browser` open its page in your browser (where you
//! log in yourself: quire never handles your login), and the PDF you download there is attached.

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use clap::Args;
use quire_engine::{Attached, Engine, PdfKind, PdfOutcome, Reference, SlugPath};

use super::browser::{self, Waited};
use crate::commands::reference::exit_code;
use crate::context::Context;
use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct GetArgs {
    /// Cite keys, DOIs / arXiv IDs / PMIDs, or reference IDs
    #[arg(required_unless_present_any = ["project", "blocked", "all"])]
    queries: Vec<String>,
    /// Every reference of this project that has no PDF
    #[arg(short, long, conflicts_with = "queries")]
    project: Option<SlugPath>,
    /// With --project: include its subprojects
    #[arg(short, long)]
    recursive: bool,
    /// Every reference marked blocked by an earlier `pdf get`
    #[arg(long, conflicts_with_all = ["queries", "project"])]
    blocked: bool,
    /// Every reference in the library that has no PDF
    #[arg(long, conflicts_with_all = ["queries", "project", "blocked"])]
    all: bool,
    /// Try the download first; open only what fails in your browser, one at a time, and attach the
    /// PDF you download there
    #[arg(long)]
    browser: bool,
    /// With --browser: minutes to wait for each download
    #[arg(long, default_value_t = 5)]
    wait: u64,
}

/// The browser half of `--browser`: where downloads land, and Enter to skip.
struct Session {
    downloads: PathBuf,
    enter: mpsc::Receiver<()>,
    wait: Duration,
}

pub async fn run(context: &Context, args: &GetArgs) -> anyhow::Result<ExitCode> {
    let engine = context.engine().await?;
    let queries = targets(context, engine, args).await?;
    if queries.is_empty() {
        output::hint("Nothing to do: every reference already has a PDF.");
        return Ok(ExitCode::SUCCESS);
    }
    let session = if args.browser { Some(session(engine, args.wait)?) } else { None };
    let total = queries.len();
    let mut missing: Vec<String> = Vec::new();
    for (index, query) in queries.iter().enumerate() {
        output::hint(format!("[{}/{total}] {query}", index + 1));
        match engine.get_pdf(query).await {
            Ok(PdfOutcome::Present(reference, attachments)) => {
                let files = style::paint(style::DIM, format!("{} file(s)", attachments.len()));
                output::report(
                    Tone::Info,
                    "has PDF",
                    format!("{}  {files}", style::paint(style::KEY, &reference.cite_key)),
                );
            }
            Ok(PdfOutcome::Downloaded { attached, source, .. }) => {
                report_attached(&attached, &format!("from {source}"))
            }
            Err(error) => {
                let (tone, label) = if error.is_blocked() { (Tone::Warn, "blocked") } else { (Tone::Error, "no PDF") };
                output::note(tone, label, format!("{query}: {error}"));
                let got = match (&session, engine.show(query).await) {
                    (Some(session), Ok((reference, _))) => in_browser(engine, session, &reference).await?,
                    _ => false,
                };
                if !got {
                    missing.push(query.clone());
                }
            }
        }
    }
    if !missing.is_empty() && session.is_none() {
        output::hint(format!(
            "{} without a PDF. Get them in your browser: `quire pdf get {} --browser`",
            missing.len(),
            selection(args, &missing)
        ));
    }
    Ok(exit_code(missing.len(), total))
}

/// The arguments that select the same references again.
fn selection(args: &GetArgs, missing: &[String]) -> String {
    match (&args.project, args.all, args.blocked) {
        _ if missing.len() <= 5 => missing.join(" "),
        (Some(project), _, _) => format!("-p {project}{}", if args.recursive { " -r" } else { "" }),
        (None, true, _) => "--all".into(),
        _ => "--blocked".into(),
    }
}

/// The references to work on: the arguments, a project's (or the library's) references without
/// a PDF, or the blocked ones.
async fn targets(context: &Context, engine: &Engine, args: &GetArgs) -> anyhow::Result<Vec<String>> {
    if !args.queries.is_empty() {
        return Ok(args.queries.clone());
    }
    let with_pdf = engine.references_with_pdfs().await?;
    if args.blocked {
        let checked = engine.check(None, false).await?;
        let blocked =
            checked.into_iter().filter(|c| c.issues.iter().any(|i| i.code == quire_engine::IssueCode::Blocked));
        return Ok(blocked
            .filter(|c| !with_pdf.contains(&c.reference.id))
            .map(|c| c.reference.cite_key.to_string())
            .collect());
    }
    let (project, recursive) = context.project(args.project.as_ref(), args.recursive, args.all);
    let references = engine.list(project.as_ref(), recursive, false).await?;
    Ok(references.into_iter().filter(|r| !with_pdf.contains(&r.id)).map(|r| r.cite_key.to_string()).collect())
}

fn session(engine: &Engine, minutes: u64) -> anyhow::Result<Session> {
    let downloads = engine.downloads_dir().filter(|d| d.is_dir()).ok_or_else(|| {
        anyhow::anyhow!("no downloads folder found; set it with `quire config set access.downloads <folder>`")
    })?;
    output::hint(format!("PDFs you download are picked up from {}. Enter skips one.", downloads.display()));
    Ok(Session { downloads, enter: browser::enter_presses(), wait: Duration::from_secs(minutes * 60) })
}

/// Opens `reference`'s page and waits for its PDF in the downloads folder; `true` if attached.
/// A PDF of another paper landing there meanwhile (another download) is left alone, and the wait
/// goes on for the right one.
async fn in_browser(engine: &Engine, session: &Session, reference: &Reference) -> anyhow::Result<bool> {
    let key = style::paint(style::KEY, &reference.cite_key);
    let Some(page) = engine.browser_page(reference) else {
        output::note(
            Tone::Error,
            "no page",
            format!(
                "{key} has no DOI, URL or arXiv ID to open; attach a file with `quire pdf import <file> --to {}`",
                reference.cite_key
            ),
        );
        return Ok(false);
    };
    output::note(Tone::Info, "opening", format!("{key}  {page}"));
    // Drop Enter presses made before this page opened.
    while session.enter.try_recv().is_ok() {}
    let since = SystemTime::now();
    let deadline = std::time::Instant::now() + session.wait;
    quire_engine::tools::open(page.as_str())?;
    let mut ignored: HashSet<PathBuf> = HashSet::new();
    loop {
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        let waited = tokio::task::block_in_place(|| {
            browser::wait_for_pdf(&session.downloads, since, left, &session.enter, &ignored)
        });
        match waited {
            Waited::Pdf(file) => match engine.attach_file(reference.cite_key.as_str(), &file, true).await {
                Ok(attached) => {
                    report_attached(&attached, &format!("from {}", file.display()));
                    return Ok(true);
                }
                Err(error) => {
                    output::note(Tone::Warn, "not it", format!("{}: {error}; still waiting", file.display()));
                    ignored.insert(file);
                }
            },
            Waited::Skipped => {
                output::note(Tone::Warn, "skipped", key);
                return Ok(false);
            }
            Waited::TimedOut => {
                output::note(Tone::Warn, "timed out", key);
                return Ok(false);
            }
        }
    }
}

/// `attached  key  15 pages, text  from arxiv`, and a warning if it may be another paper.
pub fn report_attached(attached: &Attached, from: &str) {
    let a = &attached.attachment;
    let kind = match a.pdf_kind {
        Some(PdfKind::Text) => "text",
        Some(PdfKind::Scanned) => "scanned (no text layer)",
        Some(PdfKind::Mixed) => "partly scanned",
        None => "",
    };
    let facts = format!("{} pages, {kind}  {from}", a.pages.unwrap_or(0));
    let label = if attached.new { "attached" } else { "unchanged" };
    output::report(
        Tone::Ok,
        label,
        format!("{}  {}", style::paint(style::KEY, &attached.reference.cite_key), style::paint(style::DIM, facts)),
    );
    if let Some(reason) = attached.verdict.reason() {
        output::note(Tone::Warn, "check", format!("is it the right file? {reason}"));
    }
}
