//! The user's editor for `ref edit` and `note add|edit` (the engine runs it, shared with the TUI),
//! and yes/no questions.

use std::io::{BufRead, Write};
use std::path::Path;

/// Runs the editor on `path` ([`quire_engine::editor`]).
pub fn open(path: &Path) -> anyhow::Result<()> {
    Ok(quire_engine::editor::open(path)?)
}

/// `initial` edited in a private temporary file with `suffix` (e.g. `.md`); the result.
pub fn edit_text(initial: &str, suffix: &str) -> anyhow::Result<String> {
    Ok(quire_engine::editor::edit_text(initial, suffix)?)
}

/// Asks a yes/no question on stderr; anything but `n` is yes.
pub fn ask(question: &str) -> anyhow::Result<bool> {
    eprint!("{question}");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(!answer.trim().eq_ignore_ascii_case("n"))
}
