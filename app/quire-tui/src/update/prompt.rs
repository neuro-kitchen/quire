//! Keys while a prompt is open at the bottom: typing, Enter, Esc. The filter applies as you
//! type; the confirmations take `y` (and `d` for "not duplicates").

use super::{Action, Effect, details, jobs, selected};
use crate::state::{Prompt, Row, Scope, State};

pub(super) fn handle(state: &mut State, action: Action) -> Vec<Effect> {
    let kind = state.prompt.clone().expect("called with a prompt");
    match action {
        Action::Escape => {
            state.prompt = None;
            if kind == Prompt::Filter {
                state.filter.clear();
                state.refresh_visible();
                return details(state);
            }
        }
        Action::Type(c) if kind == Prompt::ConfirmMerge => {
            state.prompt = None;
            let Some(row) = state.current() else { return Vec::new() };
            let (keep, other) = (row.reference.id, row.duplicate_of().unwrap_or_default().to_string());
            let key = row.reference.cite_key.to_string();
            match c {
                'y' => {
                    return vec![jobs::start(
                        state,
                        format!("merge {other} into {key}"),
                        Effect::Merge { keep, other },
                    )];
                }
                'd' => {
                    let label = format!("mark {key} and {other} distinct");
                    return vec![jobs::start(state, label, Effect::Distinct { a: keep, b: other })];
                }
                _ => {}
            }
        }
        Action::Type(c) if kind == Prompt::ConfirmTrash => {
            state.prompt = None;
            if c == 'y' {
                return selected(state).map(Effect::Trash).into_iter().collect();
            }
        }
        Action::Type(c) => {
            state.input.push(c);
            if kind == Prompt::Filter {
                state.filter = state.input.clone();
                state.refresh_visible();
                return details(state);
            }
        }
        Action::Backspace => {
            state.input.pop();
            if kind == Prompt::Filter {
                state.filter = state.input.clone();
                state.refresh_visible();
                return details(state);
            }
        }
        Action::Submit => {
            state.prompt = None;
            let input = std::mem::take(&mut state.input).trim().to_string();
            return submit(state, kind, input);
        }
        _ => {}
    }
    Vec::new()
}

fn submit(state: &mut State, kind: Prompt, input: String) -> Vec<Effect> {
    if input.is_empty() {
        return Vec::new();
    }
    match kind {
        Prompt::Add => {
            let project = match &state.scope {
                Scope::Project(path) => Some(path.clone()),
                _ => None,
            };
            vec![jobs::start(state, format!("add {input}"), Effect::Add { input, project })]
        }
        Prompt::Tag => {
            selected(state).map(|reference| Effect::ToggleTag { reference, tag: input }).into_iter().collect()
        }
        Prompt::Link => match input.parse::<quire_engine::SlugPath>() {
            Ok(path) => selected(state).map(|reference| Effect::Link { reference, path }).into_iter().collect(),
            Err(error) => {
                state.warn(format!("`{input}` is not a project path: {error}"));
                Vec::new()
            }
        },
        Prompt::Search => {
            state.scope = Scope::Search(input);
            state.sort = crate::state::Sort::Relevance;
            state.focus = crate::state::Pane::References;
            state.selected = 0;
            vec![Effect::LoadRows(state.scope.clone())]
        }
        Prompt::SaveSearch => match &state.scope {
            Scope::Search(query) => vec![Effect::SaveSearch { name: input, query: query.clone() }],
            _ => Vec::new(),
        },
        Prompt::Export => {
            let scope = state.scope.clone();
            vec![jobs::start(state, format!("export to {input}"), Effect::Export { scope, path: input })]
        }
        Prompt::Filter | Prompt::ConfirmTrash | Prompt::ConfirmMerge => Vec::new(),
    }
}

/// Whether `kind` needs a selected reference to start.
pub(super) fn needs_selection(kind: &Prompt) -> bool {
    matches!(kind, Prompt::Tag | Prompt::ConfirmTrash | Prompt::Link)
}

/// Whether the selected row has a duplicate to merge.
pub(super) fn has_duplicate(state: &State) -> bool {
    state.current().and_then(Row::duplicate_of).is_some()
}
