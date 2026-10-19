//! The `?` overlay: every key.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use super::theme;
use crate::keys::HELP;

/// Width of one column of keys.
const COLUMN: u16 = 56;

/// One column, or two side by side when one would not fit the terminal's height.
pub fn draw(frame: &mut Frame, area: Rect) {
    let columns: u16 = if HELP.len() as u16 + 2 > area.height && area.width >= 2 * COLUMN { 2 } else { 1 };
    let per_column = HELP.len().div_ceil(columns as usize);
    let height = (per_column as u16 + 2).min(area.height);
    let [row] = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center).areas(area);
    let [popup] = Layout::horizontal([Constraint::Length(COLUMN * columns)]).flex(Flex::Center).areas(row);
    frame.render_widget(Clear, popup);
    let block = Block::bordered().title(" Keys (Esc closes) ").border_style(theme::border(true));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let parts = Layout::horizontal(vec![Constraint::Fill(1); columns as usize]).split(inner);
    for (part, keys) in parts.iter().zip(HELP.chunks(per_column)) {
        let lines: Vec<Line> = keys
            .iter()
            .map(|(key, what)| Line::from(vec![Span::styled(format!(" {key:<15}"), theme::KEY), Span::raw(*what)]))
            .collect();
        frame.render_widget(Paragraph::new(lines), *part);
    }
}
