//! What the output goes to: a terminal (tables, colours, progress) or a pipe (plain lines).

use std::io::IsTerminal;

/// Width used when the terminal's cannot be read.
const FALLBACK_WIDTH: usize = 100;

pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

pub fn stderr_is_terminal() -> bool {
    std::io::stderr().is_terminal()
}

/// The terminal's width in columns: the window, else `$COLUMNS`, else 100.
pub fn width() -> usize {
    terminal_size::terminal_size()
        .map(|(width, _)| usize::from(width.0))
        .or_else(|| std::env::var("COLUMNS").ok()?.parse().ok())
        .unwrap_or(FALLBACK_WIDTH)
}
