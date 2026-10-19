//! Editing text in the user's editor (`$VISUAL`, else `$EDITOR`, else `vi`), shared by `ref edit`
//! and `note add|edit`.

use std::io::{BufRead, Write};
use std::path::Path;
use std::process::Command;

/// Runs the editor on `path` (the variable may hold arguments, e.g. `code --wait`).
pub fn open(path: &Path) -> anyhow::Result<()> {
    let editor = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")).unwrap_or_else(|_| "vi".into());
    let mut words = editor.split_whitespace();
    let program = words.next().ok_or_else(|| anyhow::anyhow!("$EDITOR is empty"))?;
    let status = Command::new(program)
        .args(words)
        .arg(path)
        .status()
        .map_err(|e| anyhow::anyhow!("cannot run `{editor}`: {e}"))?;
    if !status.success() {
        anyhow::bail!("`{editor}` exited with {status}; nothing changed");
    }
    Ok(())
}

/// `initial` edited in a private temporary file with `suffix` (e.g. `.md`); the result.
pub fn edit_text(initial: &str, suffix: &str) -> anyhow::Result<String> {
    let file = tempfile::Builder::new().prefix("quire-").suffix(suffix).tempfile()?;
    std::fs::write(file.path(), initial)?;
    open(file.path())?;
    Ok(std::fs::read_to_string(file.path())?)
}

/// Asks a yes/no question on stderr; anything but `n` is yes.
pub fn ask(question: &str) -> anyhow::Result<bool> {
    eprint!("{question}");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(!answer.trim().eq_ignore_ascii_case("n"))
}
