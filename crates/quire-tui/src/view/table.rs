//! The references table: cite key, year, first author, title, colored-tag dots, PDF status,
//! issue mark.

use quire_engine::Severity;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Row as TableRow, Table, TableState};

use super::theme;
use crate::state::{Pane, Row, Scope, State};

/// Widest the cite-key column gets; longer keys are cut so the title keeps room.
const KEY_WIDTH: usize = 24;

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    // The dots column only when some reference has a colored tag.
    let dots = state.visible.iter().any(|i| state.rows[*i].tags.iter().any(|(t, _)| t.color.is_some()));
    let header = TableRow::new(["key", "year", "author", "title", "", "pdf", ""]).style(theme::TITLE);
    let rows = state.visible.iter().map(|i| row(&state.rows[*i]));
    // Fixed columns sized to their content; the title takes the rest (at least 12).
    let key_width = state
        .visible
        .iter()
        .map(|i| state.rows[*i].reference.cite_key.as_str().len())
        .max()
        .unwrap_or(3)
        .clamp(3, KEY_WIDTH) as u16;
    let widths = [
        Constraint::Length(key_width),
        Constraint::Length(4),
        Constraint::Length(12),
        Constraint::Fill(1),
        Constraint::Length(if dots { 3 } else { 0 }),
        Constraint::Length(9),
        Constraint::Length(1),
    ];
    let focused = state.focus == Pane::References;
    let title = match &state.scope {
        Scope::All => " All references ".to_string(),
        Scope::Project(path) => format!(" {path} "),
        Scope::Trash => " Trash ".to_string(),
    };
    let count = if state.filter.is_empty() {
        format!(" {} ", state.visible.len())
    } else {
        format!(" {} of {} · /{} ", state.visible.len(), state.rows.len(), state.filter)
    };
    let block = Block::bordered()
        .title(title)
        .title_bottom(Line::from(count).right_aligned())
        .border_style(theme::border(focused));
    let table =
        Table::new(rows, widths).header(header).block(block).column_spacing(1).row_highlight_style(if focused {
            theme::SELECTED
        } else {
            Style::new().add_modifier(ratatui::style::Modifier::BOLD)
        });
    let mut table_state = TableState::default().with_selected((!state.visible.is_empty()).then_some(state.selected));
    frame.render_stateful_widget(table, area, &mut table_state);
}

fn row(row: &Row) -> TableRow<'static> {
    let data = &row.reference.data;
    let author = match data.creators.as_slice() {
        [] => String::new(),
        [one] => one.sort_name(),
        [first, ..] => format!("{} et al.", first.sort_name()),
    };
    // Colored tags as dots in their colors, as Zotero shows them.
    let dots: Vec<Span> = row
        .tags
        .iter()
        .filter_map(|(tag, _)| theme::tag_color(tag).map(|c| Span::styled("●", Style::new().fg(c))))
        .take(3)
        .collect();
    let (pdf, pdf_style) = theme::pdf(row.pdf);
    let mark = match row.worst() {
        Some(Severity::Error) => Span::styled("!", theme::ERROR),
        Some(Severity::Warning) => Span::styled("~", theme::WARN),
        None => Span::raw(""),
    };
    let trashed = row.reference.meta.is_deleted();
    TableRow::new(vec![
        Cell::from(Span::styled(row.reference.cite_key.to_string(), if trashed { theme::DIM } else { theme::KEY })),
        Cell::from(Span::styled(data.issued.map(|d| d.get_year().to_string()).unwrap_or_default(), theme::DIM)),
        Cell::from(author),
        Cell::from(data.title().unwrap_or("(no title)").to_string()),
        Cell::from(Line::from(dots)),
        Cell::from(Span::styled(pdf, pdf_style)),
        Cell::from(mark),
    ])
}
