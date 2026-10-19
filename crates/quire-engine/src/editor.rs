//! Editing text in the user's editor (`$VISUAL`, else `$EDITOR`, else `vi`), shared by the front
//! ends: `ref edit` and `note add|edit` in the CLI, `e` in the TUI (which leaves the screen to it).

use std::path::Path;
use std::process::Command;

use crate::error::{EngineError, Result};

/// Runs the editor on `path` (the variable may hold arguments, e.g. `code --wait`).
pub fn open(path: &Path) -> Result<()> {
    let editor = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")).unwrap_or_else(|_| "vi".into());
    let mut words = editor.split_whitespace();
    let program = words.next().ok_or_else(|| EngineError::Editor("$EDITOR is empty".into()))?;
    let status = Command::new(program)
        .args(words)
        .arg(path)
        .status()
        .map_err(|e| EngineError::Editor(format!("cannot run `{editor}`: {e}")))?;
    if !status.success() {
        return Err(EngineError::Editor(format!("`{editor}` exited with {status}; nothing changed")));
    }
    Ok(())
}

/// `initial` edited in a private temporary file (0600, random name) with `suffix` (e.g. `.md`).
pub fn edit_text(initial: &str, suffix: &str) -> Result<String> {
    let file = tempfile::Builder::new()
        .prefix("quire-")
        .suffix(suffix)
        .tempfile()
        .map_err(EngineError::file(std::env::temp_dir()))?;
    std::fs::write(file.path(), initial).map_err(EngineError::file(file.path()))?;
    open(file.path())?;
    std::fs::read_to_string(file.path()).map_err(EngineError::file(file.path()))
}
