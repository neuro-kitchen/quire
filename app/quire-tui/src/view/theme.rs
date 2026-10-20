//! Colors and styles of the TUI, in one place.

use quire_engine::{PdfAttempt, PdfStatus, Tag};
use ratatui::style::{Color, Modifier, Style};

pub(crate) const ACCENT: Color = Color::Cyan;
pub const DIM: Style = Style::new().fg(Color::DarkGray);
pub const KEY: Style = Style::new().fg(Color::Cyan);
pub(crate) const TITLE: Style = Style::new().add_modifier(Modifier::BOLD);
pub const OK: Style = Style::new().fg(Color::Green);
pub const WARN: Style = Style::new().fg(Color::Yellow);
pub const ERROR: Style = Style::new().fg(Color::Red);
pub(crate) const SELECTED: Style = Style::new().bg(Color::Rgb(40, 50, 70)).add_modifier(Modifier::BOLD);

/// The border style of a pane: bright when it has the focus.
pub fn border(focused: bool) -> Style {
    if focused { Style::new().fg(ACCENT) } else { DIM }
}

/// A tag's color, if it has one.
pub(crate) fn tag_color(tag: &Tag) -> Option<Color> {
    let hex = tag.color.as_deref()?.strip_prefix('#')?;
    let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    Some(Color::Rgb(byte(0)?, byte(2)?, byte(4)?))
}

/// The PDF column's text and style.
pub fn pdf(status: Option<PdfStatus>) -> (String, Style) {
    match status {
        Some(PdfStatus::Available { pages: Some(p) }) => (format!("✓ {p} p"), OK),
        Some(PdfStatus::Available { pages: None }) => ("✓".into(), OK),
        Some(PdfStatus::Missing(PdfAttempt::Blocked)) => ("blocked".into(), WARN),
        Some(PdfStatus::Missing(PdfAttempt::NotFound)) => ("not found".into(), DIM),
        Some(PdfStatus::Missing(PdfAttempt::NoIdentifier)) => ("no DOI".into(), DIM),
        Some(PdfStatus::Missing(PdfAttempt::Error)) => ("error".into(), ERROR),
        None => ("–".into(), DIM),
    }
}
