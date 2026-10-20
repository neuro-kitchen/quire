//! The pure part of the TUI: an [`Action`] (a key, or a result from the engine) changes the
//! [`State`] and asks for [`Effect`]s (engine work, run elsewhere). No I/O here, so every
//! behaviour is a unit test.
//!
//! - [`prompt`]: keys while a prompt is open; [`reader`]: the reader's keys and results;
//!   [`jobs`]: background work shown in the jobs panel.

mod jobs;
mod prompt;
mod reader;

use quire_engine::{ProjectSummary, ReadingStatus, SavedSearch, SlugPath, Tag};
use quire_model::ReferenceId;

use crate::reader::{ReaderAction, Settings};
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
    /// Extract the selected paper's text again.
    ExtractText,
    /// Edit the selected reference in the user's editor.
    Edit,
    /// Look the selected reference up again (fields the user edited are kept).
    Refetch,
    /// Take the selected reference out of the open project.
    Unlink,
    /// Copy the selected reference's cite key.
    CopyKey,
    /// Move the selected paper to the next reading status (unread → reading → read).
    CycleReading,
    /// Open or close the jobs panel.
    ToggleJobs,
    Reload,
    /// Results from the engine.
    Loaded {
        projects: Vec<ProjectSummary>,
        colored: Vec<Tag>,
        duplicate_pairs: usize,
        saved: Vec<SavedSearch>,
    },
    Rows(Vec<Row>),
    DetailsLoaded(Details),
    Done {
        text: String,
        ok: bool,
    },
    /// A background job ended (its result came as another action).
    JobFinished(u64),
    /// The library changed elsewhere (a job, another program).
    Changed,
    /// Open the selected reference in the reader.
    Read,
    /// Keys inside the reader.
    Reader(ReaderAction),
    /// The reader's text arrived (or failed).
    /// The terminal said whether it shows images.
    GraphicsDetected(bool),
    /// A figure image's file arrived (or failed).
    FigureLoaded {
        url: String,
        result: Result<std::path::PathBuf, String>,
    },
    TextLoaded {
        id: ReferenceId,
        text: Result<quire_engine::PaperText, String>,
    },
    /// Settings or positions were written (nothing to show).
    Saved,
    /// A paper's highlights and notes (loaded or changed).
    Annotations {
        id: ReferenceId,
        highlights: Vec<quire_engine::Highlight>,
        notes: Vec<quire_engine::Note>,
    },
}

/// Engine work to run after an update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    LoadSidebar,
    LoadRows(Scope),
    LoadDetails(ReferenceId),
    Add {
        input: String,
        project: Option<quire_engine::SlugPath>,
    },
    ToggleTag {
        reference: ReferenceId,
        tag: String,
    },
    Trash(ReferenceId),
    /// Merge the reference `other` names into `keep`.
    Merge {
        keep: ReferenceId,
        other: String,
    },
    /// Record that two references are not duplicates.
    Distinct {
        a: ReferenceId,
        b: String,
    },
    OpenPdf(ReferenceId),
    GetPdf(ReferenceId),
    ExtractText(ReferenceId),
    LoadText(ReferenceId),
    /// Forget the stored text and extract it again.
    RefreshText(ReferenceId),
    /// Load a paper's highlights and notes for the reader.
    LoadAnnotations(ReferenceId),
    /// Download a figure image (cached by the engine).
    LoadFigure(String),
    /// Ask the terminal whether it shows images: the event loop pauses reading keys for it.
    DetectGraphics,
    AddHighlight {
        reference: ReferenceId,
        quote: String,
        prefix: String,
        comment: Option<String>,
    },
    RemoveHighlight {
        reference: ReferenceId,
        highlight: quire_engine::HighlightId,
    },
    AddNote {
        reference: ReferenceId,
        body: String,
    },
    SaveReaderSettings(Settings),
    SavePositions(Vec<(ReferenceId, usize)>),
    /// `effect`, listed in the jobs panel while it runs.
    Job {
        id: u64,
        effect: Box<Effect>,
    },
    /// Edit in the user's editor: the event loop leaves the screen to it.
    Edit(ReferenceId),
    Refetch(ReferenceId),
    Link {
        reference: ReferenceId,
        path: SlugPath,
    },
    Unlink {
        reference: ReferenceId,
        path: SlugPath,
    },
    /// Export the scope's references to `path` (format from its extension).
    Export {
        scope: Scope,
        path: String,
    },
    /// Put `text` on the clipboard: the event loop writes to the terminal (OSC 52).
    Clipboard {
        text: String,
        what: String,
    },
    SetReading {
        reference: ReferenceId,
        status: ReadingStatus,
    },
    /// The reader opened a paper: mark it reading if unread.
    Opened(ReferenceId),
    SaveSearch {
        name: String,
        query: String,
    },
}

/// Lines moved by page up / down.
pub const PAGE: isize = 10;

pub fn update(state: &mut State, action: Action) -> Vec<Effect> {
    match action {
        Action::Read => return reader::open(state),
        Action::TextLoaded { id, text } => {
            reader::text_loaded(state, id, text);
            return reader::figure(state).into_iter().collect();
        }
        Action::Reader(action) => {
            let mut effects = reader::action(state, action);
            effects.extend(reader::figure(state));
            return effects;
        }
        Action::GraphicsDetected(available) => {
            state.graphics = Some(available);
            return reader::figure(state).into_iter().collect();
        }
        Action::FigureLoaded { url, result } => {
            if let Some(open) = state.reader.as_mut() {
                open.figure_loaded(url, result);
            }
            return Vec::new();
        }
        Action::JobFinished(id) => {
            jobs::finished(state, id);
            return Vec::new();
        }
        Action::Saved => return Vec::new(),
        Action::Annotations { id, highlights, notes } => {
            if let Some(reader) = state.reader.as_mut().filter(|r| r.id == id) {
                reader.set_annotations(highlights, notes);
            }
            return Vec::new();
        }
        _ => {}
    }
    if state.prompt.is_some()
        && !matches!(
            action,
            Action::Loaded { .. } | Action::Rows(_) | Action::DetailsLoaded(_) | Action::Done { .. } | Action::Changed
        )
    {
        return prompt::handle(state, action);
    }
    match action {
        Action::Quit => state.quit = true,
        Action::Help => state.help = !state.help,
        Action::Escape if state.help => state.help = false,
        Action::Escape if state.show_jobs => state.show_jobs = false,
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
            Pane::References => return reader::open(state),
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
            if prompt::needs_selection(&kind) && state.current().is_none() {
                return Vec::new();
            }
            if kind == Prompt::ConfirmMerge && !prompt::has_duplicate(state) {
                state.warn("no duplicate to merge (≈ marks possible duplicates)");
                return Vec::new();
            }
            if kind == Prompt::SaveSearch && !matches!(state.scope, Scope::Search(_)) {
                state.warn("search first (Ctrl-f), then w saves the search");
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
        Action::GetPdf => return job_on_selected(state, "get the PDF of", Effect::GetPdf),
        Action::ExtractText => return job_on_selected(state, "extract the text of", Effect::ExtractText),
        Action::Refetch => return job_on_selected(state, "refetch", Effect::Refetch),
        Action::Edit => return selected(state).map(Effect::Edit).into_iter().collect(),
        Action::CycleReading => {
            let Some(row) = state.current() else { return Vec::new() };
            let status = match row.reading.unwrap_or(ReadingStatus::Unread) {
                ReadingStatus::Unread => ReadingStatus::Reading,
                ReadingStatus::Reading => ReadingStatus::Read,
                ReadingStatus::Read => ReadingStatus::Unread,
            };
            return vec![Effect::SetReading { reference: row.reference.id, status }];
        }
        Action::CopyKey => {
            return state
                .current()
                .map(|row| {
                    let key = row.reference.cite_key.to_string();
                    Effect::Clipboard { what: format!("cite key {key}"), text: key }
                })
                .into_iter()
                .collect();
        }
        Action::Unlink => {
            let Scope::Project(path) = state.scope.clone() else {
                state.warn("open a project first: unlink takes a reference out of the open project");
                return Vec::new();
            };
            return selected(state).map(|reference| Effect::Unlink { reference, path }).into_iter().collect();
        }
        Action::ToggleJobs => state.show_jobs = !state.show_jobs,
        Action::Reload | Action::Changed => {
            return vec![Effect::LoadSidebar, Effect::LoadRows(state.scope.clone())];
        }
        Action::Loaded { projects, colored, duplicate_pairs, saved } => {
            state.projects = projects;
            state.colored = colored;
            state.duplicate_pairs = duplicate_pairs;
            state.saved = saved;
            state.scope_index = state.scope_index.min(state.scope_count() - 1);
        }
        Action::Rows(rows) => {
            state.set_rows(rows);
            return details(state);
        }
        Action::DetailsLoaded(loaded) => {
            if loaded.id == state.current().map(|r| r.reference.id) {
                state.details = loaded;
            }
        }
        Action::Done { text, ok } => {
            if ok {
                state.tell(text)
            } else {
                state.warn(text)
            }
        }
        // Prompt-only actions outside a prompt: nothing.
        Action::Type(_) | Action::Backspace | Action::Submit => {}
        Action::Read
        | Action::Reader(_)
        | Action::TextLoaded { .. }
        | Action::FigureLoaded { .. }
        | Action::GraphicsDetected(_)
        | Action::Saved
        | Action::Annotations { .. }
        | Action::JobFinished(_) => {
            unreachable!("handled above")
        }
    }
    Vec::new()
}

/// A job on the selected reference, labelled `<verb> <cite key>`.
fn job_on_selected(state: &mut State, verb: &str, effect: fn(ReferenceId) -> Effect) -> Vec<Effect> {
    let Some(row) = state.current() else { return Vec::new() };
    let (id, label) = (row.reference.id, format!("{verb} {}", row.reference.cite_key));
    vec![jobs::start(state, label, effect(id))]
}

pub(super) fn selected(state: &State) -> Option<ReferenceId> {
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
pub(super) fn details(state: &mut State) -> Vec<Effect> {
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
        let [Effect::Job { id, effect }] = effects.as_slice() else { panic!("{effects:?}") };
        assert_eq!(**effect, Effect::Add { input: "10.1/x".into(), project: Some("thesis".parse().unwrap()) });
        assert_eq!(state.jobs.len(), 1);
        assert_eq!(state.jobs[0].label, "add 10.1/x");
        update(&mut state, Action::JobFinished(*id));
        assert!(state.jobs.is_empty());
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
        assert_eq!(state.scope_index, 2, "all, duplicates, trash");
        assert_eq!(update(&mut state, Action::Open), [Effect::LoadRows(Scope::Trash)]);
        assert_eq!(state.focus, Pane::References);
        state.focus = Pane::Scopes;
        update(&mut state, Action::Move(-1));
        assert_eq!(update(&mut state, Action::Open), [Effect::LoadRows(Scope::Duplicates)]);
    }

    #[test]
    fn link_unlink_export_and_jobs() {
        let mut state = loaded();
        assert!(update(&mut state, Action::Unlink).is_empty());
        assert!(state.message.as_ref().unwrap().error, "unlink needs an open project");
        state.scope = Scope::Project("thesis".parse().unwrap());
        assert!(matches!(update(&mut state, Action::Unlink).as_slice(), [Effect::Unlink { .. }]));

        update(&mut state, Action::StartPrompt(Prompt::Link));
        for c in "thesis/ch-1".chars() {
            update(&mut state, Action::Type(c));
        }
        let effects = update(&mut state, Action::Submit);
        assert!(matches!(effects.as_slice(), [Effect::Link { path, .. }] if path.to_string() == "thesis/ch-1"));

        update(&mut state, Action::StartPrompt(Prompt::Export));
        for c in "refs.bib".chars() {
            update(&mut state, Action::Type(c));
        }
        let effects = update(&mut state, Action::Submit);
        assert!(
            matches!(effects.as_slice(), [Effect::Job { effect, .. }] if matches!(&**effect, Effect::Export { path, .. } if path == "refs.bib"))
        );

        assert!(matches!(update(&mut state, Action::Refetch).as_slice(), [Effect::Job { .. }]));
        assert_eq!(state.jobs.len(), 2);
        update(&mut state, Action::ToggleJobs);
        assert!(state.show_jobs);
        update(&mut state, Action::Escape);
        assert!(!state.show_jobs);
        assert!(matches!(update(&mut state, Action::Edit).as_slice(), [Effect::Edit(_)]));
    }

    #[test]
    fn copy_key() {
        let mut state = loaded();
        let key = state.current().unwrap().reference.cite_key.to_string();
        let effects = update(&mut state, Action::CopyKey);
        assert!(matches!(effects.as_slice(), [Effect::Clipboard { text, .. }] if *text == key));
    }

    #[test]
    fn reading_status_cycles_and_searches_save() {
        let mut state = loaded();
        let id = state.current().unwrap().reference.id;
        let effects = update(&mut state, Action::CycleReading);
        assert_eq!(effects, [Effect::SetReading { reference: id, status: ReadingStatus::Reading }]);
        update(&mut state, Action::StartPrompt(Prompt::SaveSearch));
        assert_eq!(state.prompt, None, "only in a search");
        state.scope = Scope::Search("status:unread".into());
        update(&mut state, Action::StartPrompt(Prompt::SaveSearch));
        for c in "queue".chars() {
            update(&mut state, Action::Type(c));
        }
        let effects = update(&mut state, Action::Submit);
        assert_eq!(effects, [Effect::SaveSearch { name: "queue".into(), query: "status:unread".into() }]);
    }

    #[test]
    fn full_text_search_opens_ranked_results() {
        let mut state = loaded();
        update(&mut state, Action::StartPrompt(Prompt::Search));
        for c in "ecg artifact".chars() {
            update(&mut state, Action::Type(c));
        }
        let effects = update(&mut state, Action::Submit);
        assert_eq!(effects, [Effect::LoadRows(Scope::Search("ecg artifact".into()))]);
        assert_eq!(state.sort, crate::state::Sort::Relevance);
        // Rows keep the ranking they arrive in.
        let rows = vec![row("z2020", "Last key", "Z", 2020), row("a2020", "First key", "A", 2020)];
        update(&mut state, Action::Rows(rows));
        assert_eq!(key(&state), "z2020", "best match first, not sorted by key");
        // A reload keeps the selected reference, wherever it moved.
        update(&mut state, Action::Move(1));
        // The same references (same IDs) plus a new one first.
        let mut rows = vec![row("new2024", "New", "N", 2024)];
        rows.extend(state.rows.iter().cloned());
        update(&mut state, Action::Rows(rows));
        assert_eq!(key(&state), "a2020");
    }

    #[test]
    fn merging_a_duplicate() {
        let mut state = loaded();
        update(&mut state, Action::StartPrompt(Prompt::ConfirmMerge));
        assert!(state.prompt.is_none() && state.message.as_ref().unwrap().error, "no duplicate: refused");
        let mut row = state.rows[0].clone();
        row.issues.push(quire_engine::Issue {
            detail: Some("other2017 (same title)".into()),
            ..quire_engine::Issue::new(quire_engine::IssueCode::PossibleDuplicate)
        });
        update(&mut state, Action::Rows(vec![row]));
        update(&mut state, Action::StartPrompt(Prompt::ConfirmMerge));
        let effects = update(&mut state, Action::Type('y'));
        assert!(
            matches!(effects.as_slice(), [Effect::Job { effect, .. }] if matches!(&**effect, Effect::Merge { other, .. } if other == "other2017"))
        );
        update(&mut state, Action::StartPrompt(Prompt::ConfirmMerge));
        let effects = update(&mut state, Action::Type('d'));
        assert!(
            matches!(effects.as_slice(), [Effect::Job { effect, .. }] if matches!(&**effect, Effect::Distinct { .. }))
        );
    }
}
