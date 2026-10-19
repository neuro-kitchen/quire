//! The scopes pane: all references, the projects tree, the trash.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState};

use super::theme;
use crate::state::{Pane, Scope, State};

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    let mut items = vec![ListItem::new(entry("All references", None, state.scope == Scope::All))];
    for summary in &state.projects {
        let depth = summary.path.segments().len() - 1;
        let label = format!("{}{}", "  ".repeat(depth), summary.project.name);
        let current = state.scope == Scope::Project(summary.path.clone());
        items.push(ListItem::new(entry(&label, Some(summary.references), current)));
    }
    items.push(ListItem::new(entry("Trash", None, state.scope == Scope::Trash)));
    let focused = state.focus == Pane::Scopes;
    let block = Block::bordered().title(" Library ").border_style(theme::border(focused));
    let list = List::new(items).block(block).highlight_style(if focused { theme::SELECTED } else { theme::DIM });
    let mut list_state = ListState::default().with_selected(Some(state.scope_index));
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// `name  12`, the open scope in the accent color.
fn entry(name: &str, count: Option<usize>, current: bool) -> Line<'static> {
    let style =
        if current { theme::KEY.add_modifier(ratatui::style::Modifier::BOLD) } else { ratatui::style::Style::new() };
    let mut spans = vec![Span::styled(name.to_string(), style)];
    if let Some(count) = count {
        spans.push(Span::styled(format!("  {count}"), theme::DIM));
    }
    Line::from(spans)
}
