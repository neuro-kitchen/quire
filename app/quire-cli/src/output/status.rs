//! One-line status reports: a coloured label, right-aligned like cargo's, then the message:
//!
//! ```text
//!      added  vaswaniAttentionAllYou2017  2017  Vaswani  Attention Is All You Need
//!   existing  kucskoNanometreScaleThermometry2013  …
//!     failed  10.1000/nope: not found: DOI 10.1000/nope
//! ```
//!
//! Results go to stdout ([`report`]); messages about the run go to stderr ([`note`]).

use std::fmt::Display;

use anstyle::Style;

use super::style;

/// Width of the label column.
const LABEL_WIDTH: usize = 10;

/// What a line reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Ok,
    Warn,
    Error,
    Info,
}

impl Tone {
    fn style(self) -> Style {
        match self {
            Self::Ok => style::OK,
            Self::Warn => style::WARN,
            Self::Error => style::ERROR,
            Self::Info => style::INFO,
        }
    }
}

/// `label` right-aligned and coloured, then `message`.
pub fn line(tone: Tone, label: &str, message: impl Display) -> String {
    format!("{} {message}", style::paint(tone.style(), format!("{label:>LABEL_WIDTH$}")))
}

/// A result line on stdout.
pub fn report(tone: Tone, label: &str, message: impl Display) {
    crate::out!("{}", line(tone, label, message));
}

/// A message on stderr.
pub fn note(tone: Tone, label: &str, message: impl Display) {
    crate::err!("{}", line(tone, label, message));
}

/// A plain hint on stderr, dimmed.
pub fn hint(message: impl Display) {
    crate::err!("{}", style::paint(style::DIM, message));
}
