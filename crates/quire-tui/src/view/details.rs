//! The details pane: the selected reference's fields, tags, PDF, issues, projects, notes,
//! related references and abstract.

use quire_engine::Severity;
use quire_model::TextField;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

use super::theme;
use crate::state::{Pane, State};

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    let block = Block::bordered().title(" Details ").border_style(theme::border(state.focus == Pane::Details));
    let Some(row) = state.current() else {
        frame.render_widget(Paragraph::new(Line::styled("Nothing selected", theme::DIM)).block(block), area);
        return;
    };
    let reference = &row.reference;
    let data = &reference.data;
    let mut lines =
        vec![Line::styled(data.title().unwrap_or("(no title)").to_string(), theme::TITLE), Line::from(creators(data))];
    let mut place = vec![data.issued.map(|d| d.to_string()).unwrap_or_default()];
    place.extend(data.get(TextField::ContainerTitle).map(str::to_string));
    lines.push(Line::styled(place.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" · "), theme::DIM));
    lines.push(Line::default());
    let field = |label: &str, value: String| {
        Line::from(vec![Span::styled(format!("{label:<9}"), theme::DIM), Span::raw(value)])
    };
    lines.push(Line::from(vec![
        Span::styled(format!("{:<9}", "key"), theme::DIM),
        Span::styled(reference.cite_key.to_string(), theme::KEY),
    ]));
    lines.push(field("type", data.kind.to_string()));
    for id in &data.identifiers {
        lines.push(field(id.kind().as_str(), id.to_string()));
    }
    let (pdf, pdf_style) = theme::pdf(row.pdf);
    lines.push(Line::from(vec![Span::styled(format!("{:<9}", "pdf"), theme::DIM), Span::styled(pdf, pdf_style)]));
    if !row.tags.is_empty() {
        let mut spans = vec![Span::styled(format!("{:<9}", "tags"), theme::DIM)];
        for (i, (tag, _)) in row.tags.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(", "));
            }
            let style = theme::tag_color(tag).map_or(Style::new(), |c| Style::new().fg(c));
            spans.push(Span::styled(tag.name.clone(), style));
        }
        lines.push(Line::from(spans));
    }
    let details = &state.details;
    if !details.projects.is_empty() {
        lines.push(field("projects", details.projects.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")));
    }
    for issue in &row.issues {
        let (mark, style) = match issue.severity() {
            Severity::Error => ("✗ ", theme::ERROR),
            Severity::Warning => ("⚠ ", theme::WARN),
        };
        lines.push(Line::from(vec![Span::styled(mark, style), Span::raw(issue.message())]));
    }
    if !details.notes.is_empty() {
        lines.push(Line::default());
        lines.push(Line::styled("Notes", theme::TITLE));
        lines.extend(details.notes.iter().map(|n| Line::from(format!("• {}", n.title()))));
    }
    if !details.related.is_empty() {
        lines.push(Line::default());
        lines.push(Line::styled("Related", theme::TITLE));
        lines.extend(details.related.iter().map(|r| Line::styled(format!("• {}", r.cite_key), theme::KEY)));
    }
    if let Some(text) = data.get(TextField::Abstract) {
        lines.push(Line::default());
        lines.push(Line::styled("Abstract", theme::TITLE));
        lines.push(Line::from(text.to_string()));
    }
    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: true }), area);
}

/// Authors shown before "… N more".
const CREATORS_SHOWN: usize = 10;

fn creators(data: &quire_model::ReferenceData) -> String {
    let mut names: Vec<String> = data.creators.iter().take(CREATORS_SHOWN).map(|c| c.display_name()).collect();
    if data.creators.len() > CREATORS_SHOWN {
        names.push(format!("… {} more", data.creators.len() - CREATORS_SHOWN));
    }
    names.join("; ")
}
