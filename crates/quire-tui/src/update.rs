//! The pure part of the TUI: an [`Action`] (a key, or a result from the engine) changes the
//! [`State`] and asks for [`Effect`]s (engine work, run elsewhere). No I/O here, so every
//! behaviour is a unit test.

use quire_engine::{ProjectSummary, Tag};
use quire_model::ReferenceId;

use crate::reader::{Reader, ReaderAction, ReaderEffect, Settings, paper};
use crate::state::{Details, Pane, Prompt, Row, Scope, State};

/// What happened.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    Help,
    /// Move the selection by this many lines (negative: up).
    Move(isize),
    Top,
    Bottom,
    NextPane,
    PreviousPane,
    /// Enter: open the scope under the cursor, or focus the details.
    Open,
    Escape,
    Sort,
    Reverse,
    StartPrompt(Prompt),
    Type(char),
    Backspace,
    Submit,
    /// Toggle the colored tag with this number (1–9) on the selected reference.
    ToggleTag(u8),
    OpenPdf,
    GetPdf,
    Reload,
    /// Results from the engine.
    Loaded {
        projects: Vec<ProjectSummary>,
        colored: Vec<Tag>,
    },
    Rows(Vec<Row>),
    DetailsLoaded(Details),
    Done {
        text: String,
        ok: bool,
    },
    /// The library changed elsewhere (a job, another program).
    Changed,
    /// Open the selected reference in the reader.
    Read,
    /// Keys inside the reader.
    Reader(ReaderAction),
    /// The reader's text arrived (or failed).
    TextLoaded {
        id: ReferenceId,
        text: Result<quire_engine::PaperText, String>,
    },
    /// Settings or positions were written (nothing to show).
    Saved,
}

/// Engine work to run after an update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    LoadSidebar,
    LoadRows(Scope),
    LoadDetails(ReferenceId),
    Add { input: String, project: Option<quire_engine::SlugPath> },
    ToggleTag { reference: ReferenceId, tag: String },
    Trash(ReferenceId),
    OpenPdf(ReferenceId),
    GetPdf(ReferenceId),
    LoadText(ReferenceId),
    SaveReaderSettings(Settings),
    SavePositions(Vec<(ReferenceId, usize)>),
}

/// Lines moved by page up / down.
pub const PAGE: isize = 10;

pub fn update(state: &mut State, action: Action) -> Vec<Effect> {
    match action {
        Action::Read => return open_reader(state),
        Action::TextLoaded { id, text } => {
            let has_pdf = state.rows.iter().any(|r| r.reference.id == id && has_pdf(r));
            if let Some(reader) = state.reader.as_mut().filter(|r| r.id == id) {
                match text {
                    Ok(text) => {
                        reader.loaded(&text.markdown, text.source.has_math());
                        reader.source = Some(text.source.label());
                    }
                    // Without a PDF, "no text" is not an error: the abstract is all there is.
                    Err(_) if !has_pdf => reader.no_text(),
                    Err(error) => reader.failed(error),
                }
            }
            return Vec::new();
        }
        Action::Reader(action) => return reader_action(state, action),
        Action::Saved => return Vec::new(),
        _ => {}
    }
    if state.prompt.is_some()
        && !matches!(
            action,
            Action::Loaded { .. } | Action::Rows(_) | Action::DetailsLoaded(_) | Action::Done { .. } | Action::Changed
        )
    {
        return prompt(state, action);
    }
    match action {
        Action::Quit => state.quit = true,
        Action::Help => state.help = !state.help,
        Action::Escape if state.help => state.help = false,
        Action::Escape if !state.filter.is_empty() => {
            state.filter.clear();
            state.refresh_visible();
            return details(state);
        }
        Action::Escape => state.focus = Pane::References,
        Action::Move(by) => return move_by(state, by),
        Action::Top => return move_by(state, isize::MIN / 2),
        Action::Bottom => return move_by(state, isize::MAX / 2),
        Action::NextPane => state.focus = next(state.focus),
        Action::PreviousPane => state.focus = next(next(state.focus)),
        Action::Open => match state.focus {
            Pane::Scopes => {
                state.scope = state.scope_at(state.scope_index);
                state.focus = Pane::References;
                state.selected = 0;
                return vec![Effect::LoadRows(state.scope.clone())];
            }
            Pane::References => return open_reader(state),
            Pane::Details => state.focus = Pane::References,
        },
        Action::Sort => {
            state.sort = state.sort.next();
            state.refresh_visible();
            state.tell(format!("sorted by {}", state.sort.label()));
        }
        Action::Reverse => {
            state.reverse = !state.reverse;
            state.refresh_visible();
        }
        Action::StartPrompt(kind) => {
            if matches!(kind, Prompt::Tag | Prompt::ConfirmTrash) && state.current().is_none() {
                return Vec::new();
            }
            state.input = if kind == Prompt::Filter { state.filter.clone() } else { String::new() };
            state.prompt = Some(kind);
        }
        Action::ToggleTag(number) => {
            let Some(row) = state.current() else { return Vec::new() };
            let Some(tag) = state.colored.iter().find(|t| t.position == Some(number)) else {
                state.warn(format!("no colored tag {number}: `quire tag color <tag> <color> --key {number}`"));
                return Vec::new();
            };
            return vec![Effect::ToggleTag { reference: row.reference.id, tag: tag.name.clone() }];
        }
        Action::OpenPdf => return selected(state).map(Effect::OpenPdf).into_iter().collect(),
        Action::GetPdf => {
            let effects: Vec<Effect> = selected(state).map(Effect::GetPdf).into_iter().collect();
            state.jobs += effects.len();
            return effects;
        }
        Action::Reload | Action::Changed => {
            return vec![Effect::LoadSidebar, Effect::LoadRows(state.scope.clone())];
        }
        Action::Loaded { projects, colored } => {
            state.projects = projects;
            state.colored = colored;
            state.scope_index = state.scope_index.min(state.scope_count() - 1);
        }
        Action::Rows(rows) => {
            state.rows = rows;
            state.refresh_visible();
            return details(state);
        }
        Action::DetailsLoaded(loaded) => {
            if loaded.id == state.current().map(|r| r.reference.id) {
                state.details = loaded;
            }
        }
        Action::Done { text, ok } => {
            state.jobs = state.jobs.saturating_sub(1);
            if ok { state.tell(text) } else { state.warn(text) }
        }
        // Prompt-only actions outside a prompt: nothing.
        Action::Type(_) | Action::Backspace | Action::Submit => {}
        Action::Read | Action::Reader(_) | Action::TextLoaded { .. } | Action::Saved => unreachable!("handled above"),
    }
    Vec::new()
}

fn has_pdf(row: &Row) -> bool {
    matches!(row.pdf, Some(quire_engine::PdfStatus::Available { .. }))
}

/// Opens the selected reference in the reader, on its abstract; its text (arXiv, Europe PMC or
/// the PDF) loads in the background.
fn open_reader(state: &mut State) -> Vec<Effect> {
    let Some(row) = state.current() else { return Vec::new() };
    let has_text = has_pdf(row) || quire_engine::online_candidate(&row.reference);
    let reference = &row.reference;
    let id = reference.id;
    let title = reference.data.title().unwrap_or("(no title)").to_string();
    let front = paper::front_matter(reference);
    let resume = state.positions.get(&id).copied();
    let key = reference.cite_key.to_string();
    state.reader = Some(Reader::new(id, key, title, front, has_text, state.reader_settings, resume));
    if has_text { vec![Effect::LoadText(id)] } else { Vec::new() }
}

fn reader_action(state: &mut State, action: ReaderAction) -> Vec<Effect> {
    let Some(reader) = state.reader.as_mut() else { return Vec::new() };
    match reader.apply(action) {
        ReaderEffect::None => Vec::new(),
        ReaderEffect::SaveSettings => {
            state.reader_settings = reader.settings;
            vec![Effect::SaveReaderSettings(reader.settings)]
        }
        ReaderEffect::Close => {
            let (id, block) = (reader.id, reader.paper_block());
            state.reader = None;
            let Some(block) = block else { return Vec::new() };
            state.positions.insert(id, block);
            vec![Effect::SavePositions(state.positions.iter().map(|(k, v)| (*k, *v)).collect())]
        }
    }
}

/// Keys while a prompt is open: typing, Enter, Esc. The filter applies as you type.
fn prompt(state: &mut State, action: Action) -> Vec<Effect> {
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
            match kind {
                Prompt::Filter => {}
                Prompt::Add if !input.is_empty() => {
                    state.jobs += 1;
                    let project = match &state.scope {
                        Scope::Project(path) => Some(path.clone()),
                        _ => None,
                    };
                    return vec![Effect::Add { input, project }];
                }
                Prompt::Tag if !input.is_empty() => {
                    return selected(state)
                        .map(|reference| Effect::ToggleTag { reference, tag: input })
                        .into_iter()
                        .collect();
                }
                _ => {}
            }
        }
        _ => {}
    }
    Vec::new()
}

fn selected(state: &State) -> Option<ReferenceId> {
    state.current().map(|r| r.reference.id)
}

fn next(pane: Pane) -> Pane {
    match pane {
        Pane::Scopes => Pane::References,
        Pane::References => Pane::Details,
        Pane::Details => Pane::Scopes,
    }
}

fn move_by(state: &mut State, by: isize) -> Vec<Effect> {
    match state.focus {
        Pane::Scopes => {
            let last = state.scope_count() as isize - 1;
            state.scope_index = (state.scope_index as isize).saturating_add(by).clamp(0, last) as usize;
            Vec::new()
        }
        Pane::References | Pane::Details => {
            let last = state.visible.len().saturating_sub(1) as isize;
            let before = state.selected;
            state.selected = (state.selected as isize).saturating_add(by).clamp(0, last.max(0)) as usize;
            if state.selected != before { details(state) } else { Vec::new() }
        }
    }
}

/// Load the selected reference's details (cleared until they arrive).
fn details(state: &mut State) -> Vec<Effect> {
    match state.current().map(|r| r.reference.id) {
        Some(id) if state.details.id != Some(id) => {
            state.details = Details { id: Some(id), ..Details::default() };
            vec![Effect::LoadDetails(id)]
        }
        Some(_) => Vec::new(),
        None => {
            state.details = Details::default();
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Sort;
    use crate::test_support::row;

    fn loaded() -> State {
        let mut state = State::new("test".into());
        let rows = vec![
            row("b2020", "Beta", "Beta", 2020),
            row("a2021", "Alpha", "Alpha", 2021),
            row("c2019", "Gamma", "Gamma", 2019),
        ];
        update(&mut state, Action::Rows(rows));
        state
    }

    fn key(state: &State) -> String {
        state.current().unwrap().reference.cite_key.to_string()
    }

    #[test]
    fn rows_sort_and_selection_loads_details() {
        let mut state = State::new("test".into());
        let effects = update(&mut state, Action::Rows(vec![row("b2020", "Beta", "Beta", 2020)]));
        assert!(matches!(effects.as_slice(), [Effect::LoadDetails(_)]));
        let mut state = loaded();
        assert_eq!(key(&state), "a2021", "sorted by key");
        assert!(matches!(update(&mut state, Action::Move(1)).as_slice(), [Effect::LoadDetails(_)]));
        assert_eq!(key(&state), "b2020");
        update(&mut state, Action::Bottom);
        assert_eq!(key(&state), "c2019");
        assert!(update(&mut state, Action::Move(1)).is_empty(), "already at the bottom");
        update(&mut state, Action::Sort);
        assert_eq!((state.sort, key(&state)), (Sort::Year, "c2019".to_string()), "selection kept");
    }

    #[test]
    fn filter_as_you_type_and_escape_clears() {
        let mut state = loaded();
        update(&mut state, Action::StartPrompt(Prompt::Filter));
        for c in "gam".chars() {
            update(&mut state, Action::Type(c));
        }
        assert_eq!((state.visible.len(), key(&state)), (1, "c2019".to_string()));
        update(&mut state, Action::Submit);
        assert!(state.prompt.is_none() && state.filter == "gam");
        update(&mut state, Action::Escape);
        assert_eq!(state.visible.len(), 3);
    }

    #[test]
    fn add_goes_to_the_current_project() {
        let mut state = loaded();
        state.scope = Scope::Project("thesis".parse().unwrap());
        update(&mut state, Action::StartPrompt(Prompt::Add));
        for c in "10.1/x".chars() {
            update(&mut state, Action::Type(c));
        }
        let effects = update(&mut state, Action::Submit);
        assert_eq!(effects, [Effect::Add { input: "10.1/x".into(), project: Some("thesis".parse().unwrap()) }]);
        assert_eq!(state.jobs, 1);
        update(&mut state, Action::Done { text: "added".into(), ok: true });
        assert_eq!(state.jobs, 0);
    }

    #[test]
    fn trash_needs_y_and_tags_need_a_color() {
        let mut state = loaded();
        update(&mut state, Action::StartPrompt(Prompt::ConfirmTrash));
        assert!(update(&mut state, Action::Type('n')).is_empty());
        update(&mut state, Action::StartPrompt(Prompt::ConfirmTrash));
        assert!(matches!(update(&mut state, Action::Type('y')).as_slice(), [Effect::Trash(_)]));
        assert!(update(&mut state, Action::ToggleTag(1)).is_empty());
        assert!(state.message.as_ref().unwrap().error);
    }

    #[test]
    fn opening_a_project_loads_it() {
        let mut state = loaded();
        state.focus = Pane::Scopes;
        update(&mut state, Action::Move(5));
        assert_eq!(state.scope_index, 1, "all + trash only");
        assert_eq!(update(&mut state, Action::Open), [Effect::LoadRows(Scope::Trash)]);
        assert_eq!(state.focus, Pane::References);
    }
}
