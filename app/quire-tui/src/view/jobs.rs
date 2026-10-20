//! The jobs panel (`J`): background work still running, with how long it has taken.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use super::theme;
use crate::state::State;

pub(super) fn draw(frame: &mut Frame, area: Rect, state: &State) {
    let lines: Vec<Line> = if state.jobs.is_empty() {
        vec![Line::styled(" nothing running", theme::DIM)]
    } else {
        state
            .jobs
            .iter()
            .map(|job| {
                let seconds = job.started.elapsed().as_secs();
                Line::from(vec![Span::styled(format!(" {seconds:>4}s  "), theme::DIM), Span::raw(job.label.clone())])
            })
            .collect()
    };
    let height = (lines.len() as u16 + 2).min(area.height);
    let [row] = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center).areas(area);
    let [popup] = Layout::horizontal([Constraint::Length(60)]).flex(Flex::Center).areas(row);
    frame.render_widget(Clear, popup);
    let title = format!(" Jobs ({} running, J closes) ", state.jobs.len());
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(title).border_style(theme::border(true))),
        popup,
    );
}
