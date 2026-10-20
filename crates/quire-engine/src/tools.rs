//! Programs quire uses when they are installed, for what it does not do itself: poppler's
//! `pdftotext` (text of PDFs whose layer pdf-inspector drops) and `pdftoppm` (page images),
//! `tesseract` (OCR of scanned pages), `ocrmypdf` (a text layer added to a scanned PDF). Each is
//! optional: quire says what to install when one is missing. Also [`open`]: the system's default
//! application for a URL or a file.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{EngineError, Result};

/// Opens `target` (a URL or a file) with the system's default application. The application is
/// started in the background with its output discarded: browsers print their own diagnostics
/// (Chrome's zygote errors), which would otherwise land in a front end's screen or prompts.
pub fn open(target: &str) -> Result<()> {
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
    let mut child = command.spawn().map_err(EngineError::file(Path::new(target)))?;
    // `xdg-open` returns once the browser has the URL; reap it on its own thread.
    std::thread::spawn(move || child.wait());
    Ok(())
}

/// Page images for OCR: dots per inch (Tesseract reads best at about 300).
const OCR_DPI: &str = "300";

/// Whether `program` runs (`<program> --version` succeeds).
pub fn installed(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// The text poppler's `pdftotext` reads from `path`, if it is installed and reads more than a
/// stamp.
pub fn poppler_text(path: &Path) -> Option<String> {
    let output = Command::new("pdftotext")
        .args(["-enc", "UTF-8"])
        .arg(path)
        .arg("-")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    (output.status.success() && text.split_whitespace().count() > 100).then_some(text)
}

/// OCR of every page of the PDF at `path`: pages drawn by `pdftoppm`, read by `tesseract` in
/// `language` (Tesseract's codes: `eng`, `deu`, `eng+fra`). Pages are separated by form feeds,
/// as `pdftotext` does. `None` when either program is missing.
pub(crate) fn ocr_text(path: &Path, language: &str) -> Result<Option<String>> {
    if !installed("pdftoppm") || !installed("tesseract") {
        return Ok(None);
    }
    let dir = tempfile::tempdir().map_err(EngineError::file(path))?;
    let prefix = dir.path().join("page");
    run(Command::new("pdftoppm").args(["-r", OCR_DPI, "-png"]).arg(path).arg(&prefix), "pdftoppm")?;
    let mut pages: Vec<_> = std::fs::read_dir(dir.path())
        .map_err(EngineError::file(dir.path()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "png"))
        .collect();
    // `page-01.png`, `page-02.png`, …: zero-padded, so name order is page order.
    pages.sort();
    let mut text = String::new();
    for page in pages {
        let output = Command::new("tesseract")
            .arg(&page)
            .arg("stdout")
            .args(["-l", language])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| EngineError::Pdf(format!("tesseract: {e}")))?;
        if !output.status.success() {
            return Err(EngineError::Pdf(format!(
                "tesseract failed on {} (is the `{language}` language installed?)",
                page.display()
            )));
        }
        text.push_str(&String::from_utf8_lossy(&output.stdout));
        text.push('\x0c');
    }
    Ok(Some(text))
}

/// Writes to `output` the PDF at `input` with a text layer from OCR (`ocrmypdf`; pages that
/// already have text are left alone). `false` when `ocrmypdf` is not installed.
pub fn ocrmypdf(input: &Path, output: &Path, language: &str) -> Result<bool> {
    if !installed("ocrmypdf") {
        return Ok(false);
    }
    run(Command::new("ocrmypdf").args(["--skip-text", "--quiet", "-l", language]).arg(input).arg(output), "ocrmypdf")?;
    Ok(true)
}

fn run(command: &mut Command, name: &str) -> Result<()> {
    let output = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| EngineError::Pdf(format!("{name}: {e}")))?;
    if output.status.success() {
        return Ok(());
    }
    let message = String::from_utf8_lossy(&output.stderr);
    Err(EngineError::Pdf(format!("{name} failed: {}", message.trim())))
}
