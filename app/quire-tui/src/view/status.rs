//! The header (library, scope, counts, jobs) and the footer (prompt, message or key hints).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::theme;
use crate::state::{Prompt, State};

pub(super) fn header(frame: &mut Frame, area: Rect, state: &State) {
    let mut spans = vec![
        Span::styled(" quire ", Style::new().fg(ratatui::style::Color::Black).bg(theme::ACCENT)),
        Span::raw(" "),
        Span::styled(state.library.clone(), theme::KEY),
        Span::styled(
            format!("  sorted by {}{}", state.sort.label(), if state.reverse { " ↑" } else { "" }),
            theme::DIM,
        ),
    ];
    if !state.jobs.is_empty() {
        spans.push(Span::styled(format!("  ⟳ {} running (J)", state.jobs.len()), theme::WARN));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub(super) fn footer(frame: &mut Frame, area: Rect, state: &State) {
    let line = match &state.prompt {
        Some(Prompt::Filter) => prompt_line("/", &state.input),
        Some(Prompt::Add) => prompt_line("add (DOI, arXiv ID, PMID, URL): ", &state.input),
        Some(Prompt::Tag) => prompt_line("tag (toggle): ", &state.input),
        Some(Prompt::Link) => prompt_line("link to project (path, e.g. thesis/chapter-1): ", &state.input),
        Some(Prompt::Export) => prompt_line("export to (.bib, .yml or .json): ", &state.input),
        Some(Prompt::SaveSearch) => prompt_line("save this search as: ", &state.input),
        Some(Prompt::Search) => {
            prompt_line("search (words, \"a phrase\", author:, title:, tag:, note:): ", &state.input)
        }
        Some(Prompt::ConfirmTrash) => {
            let key = state.current().map(|r| r.reference.cite_key.to_string()).unwrap_or_default();
            Line::from(vec![Span::styled(format!(" move {key} to the trash? "), theme::WARN), Span::raw("y / n")])
        }
        Some(Prompt::ConfirmMerge) => {
            let row = state.current();
            let key = row.map(|r| r.reference.cite_key.to_string()).unwrap_or_default();
            let other = row.and_then(|r| r.duplicate_of()).unwrap_or_default();
            Line::from(vec![
                Span::styled(format!(" merge {other} into {key}? "), theme::WARN),
                Span::raw("y merge · d not duplicates · n cancel"),
            ])
        }
        None => match &state.message {
            Some(message) => {
                Line::styled(format!(" {}", message.text), if message.error { theme::ERROR } else { theme::OK })
            }
            None => Line::styled(" ? help  / filter  a add  t tag  p get PDF  o open  s sort  q quit", theme::DIM),
        },
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn prompt_line(label: &str, input: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {label}"), theme::KEY),
        Span::raw(input.to_string()),
        Span::styled("█", theme::DIM),
    ])
}
