//! The user's own browser for what publishers block (`.knowledge/metadata-sources.md`, challenge
//! pages): open the page, let the user pass the check or log in and download the PDF, and pick
//! the new file up from the downloads folder. The browser keeps the user's logins and passes bot
//! checks as a person; quire never automates them.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

/// How often the downloads folder is looked at.
const POLL: Duration = Duration::from_millis(500);

/// Opens `target` (a URL or a file) with the system's default application. The application is
/// started in the background with its output discarded: browsers print their own diagnostics
/// (Chrome's zygote errors), which would otherwise land in quire's prompts.
pub fn open(target: &str) -> anyhow::Result<()> {
    let mut command = if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(target_os = "windows") {
        let mut start = Command::new("cmd");
        start.args(["/C", "start", ""]);
        start
    } else {
        Command::new("xdg-open")
    };
    command.arg(target).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = command.spawn().map_err(|e| anyhow::anyhow!("cannot open {target}: {e}"))?;
    // `xdg-open` returns once the browser has the URL; reap it on its own thread.
    std::thread::spawn(move || child.wait());
    Ok(())
}

/// What happened while waiting for a download.
#[derive(Debug, PartialEq, Eq)]
pub enum Waited {
    /// A new, finished PDF appeared.
    Pdf(PathBuf),
    /// The user pressed Enter.
    Skipped,
    TimedOut,
}

/// Waits for a PDF to appear in `dir` (newer than `since`, no longer growing, not a browser's
/// partial file, not in `ignored`), or for a line on `skip`.
pub fn wait_for_pdf(
    dir: &Path,
    since: SystemTime,
    timeout: Duration,
    skip: &mpsc::Receiver<()>,
    ignored: &std::collections::HashSet<PathBuf>,
) -> Waited {
    let start = std::time::Instant::now();
    let mut sizes: Vec<(PathBuf, u64)> = Vec::new();
    while start.elapsed() < timeout {
        if skip.try_recv().is_ok() {
            return Waited::Skipped;
        }
        let current: Vec<(PathBuf, u64)> =
            new_pdfs(dir, since).into_iter().filter(|(p, _)| !ignored.contains(p)).collect();
        // Finished: the same size as half a second ago.
        if let Some((path, _)) = current.iter().find(|(path, size)| *size > 0 && sizes.contains(&(path.clone(), *size)))
        {
            return Waited::Pdf(path.clone());
        }
        sizes = current;
        std::thread::sleep(POLL);
    }
    Waited::TimedOut
}

/// PDFs in `dir` modified after `since`, with their sizes; files a browser is still writing
/// (`x.pdf.part`, `x.pdf.crdownload`, …) are not PDFs yet.
fn new_pdfs(dir: &Path, since: SystemTime) -> Vec<(PathBuf, u64)> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_pdf = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
            let meta = entry.metadata().ok()?;
            let fresh = meta.modified().ok().is_some_and(|m| m >= since);
            (is_pdf && fresh && meta.is_file()).then_some((path, meta.len()))
        })
        .collect()
}

/// A channel that receives `()` each time the user presses Enter (read on its own thread).
pub fn enter_presses() -> mpsc::Receiver<()> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        while std::io::stdin().read_line(&mut line).is_ok_and(|n| n > 0) {
            if sender.send(()).is_err() {
                break;
            }
            line.clear();
        }
    });
    receiver
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_up_a_finished_pdf_only() {
        let dir = tempfile::tempdir().unwrap();
        let since = SystemTime::now() - Duration::from_secs(1);
        std::fs::write(dir.path().join("notes.txt"), "x").unwrap();
        std::fs::write(dir.path().join("paper.pdf.part"), "x").unwrap();
        let (_tx, rx) = mpsc::channel();
        let none = std::collections::HashSet::new();
        assert_eq!(wait_for_pdf(dir.path(), since, Duration::from_millis(1200), &rx, &none), Waited::TimedOut);
        // Another download lands too; it was already found not to be this paper.
        std::fs::write(dir.path().join("other.pdf"), "%PDF-1.4 other").unwrap();
        std::fs::write(dir.path().join("paper.pdf"), "%PDF-1.4").unwrap();
        let ignored: std::collections::HashSet<PathBuf> = [dir.path().join("other.pdf")].into();
        assert_eq!(
            wait_for_pdf(dir.path(), since, Duration::from_secs(3), &rx, &ignored),
            Waited::Pdf(dir.path().join("paper.pdf"))
        );
    }

    #[test]
    fn enter_skips() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = mpsc::channel();
        tx.send(()).unwrap();
        let none = std::collections::HashSet::new();
        assert_eq!(wait_for_pdf(dir.path(), SystemTime::now(), Duration::from_secs(5), &rx, &none), Waited::Skipped);
    }
}
