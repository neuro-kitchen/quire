//! The `?` overlay: every key.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use super::theme;
use crate::keys::HELP;

pub fn draw(frame: &mut Frame, area: Rect) {
    let height = HELP.len() as u16 + 2;
    let [row] = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center).areas(area);
    let [popup] = Layout::horizontal([Constraint::Length(52)]).flex(Flex::Center).areas(row);
    let lines: Vec<Line> = HELP
        .iter()
        .map(|(key, what)| Line::from(vec![Span::styled(format!(" {key:<15}"), theme::KEY), Span::raw(*what)]))
        .collect();
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Keys (Esc closes) ").border_style(theme::border(true))),
        popup,
    );
}
