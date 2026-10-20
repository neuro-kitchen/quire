//! The colours and text styles of every command, in one place. Text is printed through
//! `anstream`, which drops the styles when the output is not a terminal or `NO_COLOR` is set.

use std::fmt::Display;

use anstyle::{AnsiColor, Style};
use clap::builder::Styles;

/// Success: added, linked, written.
pub const OK: Style = AnsiColor::Green.on_default().bold();
/// Something to look at, but not a failure: already there, skipped.
pub const WARN: Style = AnsiColor::Yellow.on_default().bold();
pub const ERROR: Style = AnsiColor::Red.on_default().bold();
/// Neutral status labels: moved, renamed.
pub const INFO: Style = AnsiColor::Blue.on_default().bold();
/// Cite keys, project paths and library names: what the user types back.
pub const KEY: Style = AnsiColor::Cyan.on_default();
/// IDs, counts, paths: secondary information.
pub const DIM: Style = Style::new().dimmed();
/// Field labels in `show`, table headers.
pub const LABEL: Style = Style::new().bold();

/// `text` wrapped in `style`.
pub fn paint(style: Style, text: impl Display) -> String {
    format!("{style}{text}{style:#}")
}

/// clap's help in the same colours.
pub const CLAP: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().bold().underline())
    .usage(AnsiColor::Green.on_default().bold())
    .literal(AnsiColor::Cyan.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default())
    .error(ERROR)
    .valid(OK)
    .invalid(WARN);
