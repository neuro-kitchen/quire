//! Keys → [`Action`]s. Vim-like with arrow-key equivalents; inside a prompt, keys type.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::reader::ReaderAction;
use crate::state::Prompt;
use crate::update::{Action, PAGE};

/// The action for `key`, `None` for keys that do nothing. `typing`: a prompt is open.
pub fn action(key: KeyEvent, typing: bool) -> Option<Action> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl && matches!(key.code, KeyCode::Char('c')) {
        return Some(Action::Quit);
    }
    if ctrl && matches!(key.code, KeyCode::Char('f')) && !typing {
        return Some(Action::StartPrompt(Prompt::Search));
    }
    if typing {
        return match key.code {
            KeyCode::Esc => Some(Action::Escape),
            KeyCode::Enter => Some(Action::Submit),
            KeyCode::Backspace => Some(Action::Backspace),
            KeyCode::Char(c) => Some(Action::Type(c)),
            _ => None,
        };
    }
    Some(match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::Help,
        KeyCode::Char('j') | KeyCode::Down => Action::Move(1),
        KeyCode::Char('k') | KeyCode::Up => Action::Move(-1),
        KeyCode::Char('d') if ctrl => Action::Move(PAGE),
        KeyCode::Char('u') if ctrl => Action::Move(-PAGE),
        KeyCode::PageDown => Action::Move(PAGE),
        KeyCode::PageUp => Action::Move(-PAGE),
        KeyCode::Char('g') | KeyCode::Home => Action::Top,
        KeyCode::Char('G') | KeyCode::End => Action::Bottom,
        KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => Action::NextPane,
        KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => Action::PreviousPane,
        KeyCode::Enter => Action::Open,
        KeyCode::Esc => Action::Escape,
        KeyCode::Char('/') => Action::StartPrompt(Prompt::Filter),
        KeyCode::Char('a') => Action::StartPrompt(Prompt::Add),
        KeyCode::Char('t') => Action::StartPrompt(Prompt::Tag),
        KeyCode::Char('x') | KeyCode::Delete => Action::StartPrompt(Prompt::ConfirmTrash),
        KeyCode::Char('s') => Action::Sort,
        KeyCode::Char('S') => Action::Reverse,
        KeyCode::Char('o') => Action::OpenPdf,
        KeyCode::Char('p') => Action::GetPdf,
        KeyCode::Char('E') => Action::ExtractText,
        KeyCode::Char('M') => Action::StartPrompt(Prompt::ConfirmMerge),
        KeyCode::Char('e') => Action::Edit,
        KeyCode::Char('F') => Action::Refetch,
        KeyCode::Char('L') => Action::StartPrompt(Prompt::Link),
        KeyCode::Char('U') => Action::Unlink,
        KeyCode::Char('X') => Action::StartPrompt(Prompt::Export),
        KeyCode::Char('Y') => Action::CopyKey,
        KeyCode::Char('m') => Action::CycleReading,
        KeyCode::Char('w') => Action::StartPrompt(Prompt::SaveSearch),
        KeyCode::Char('J') => Action::ToggleJobs,
        KeyCode::Char('r') => Action::Reload,
        KeyCode::Char(c @ '1'..='9') => Action::ToggleTag(c as u8 - b'0'),
        _ => return None,
    })
}

/// The action for `key` in the reader. `typing`: the search box is open.
pub fn reader_action(key: KeyEvent, typing: bool) -> Option<Action> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl && matches!(key.code, KeyCode::Char('c')) {
        return Some(Action::Quit);
    }
    let action = match key.code {
        KeyCode::Esc => ReaderAction::Escape,
        KeyCode::Enter => ReaderAction::Submit,
        KeyCode::Backspace => ReaderAction::Backspace,
        KeyCode::Char('j') if !typing => ReaderAction::Scroll(1),
        KeyCode::Down => ReaderAction::Scroll(1),
        KeyCode::Char('k') if !typing => ReaderAction::Scroll(-1),
        KeyCode::Up => ReaderAction::Scroll(-1),
        KeyCode::Char(c) if typing => ReaderAction::Type(c),
        KeyCode::Char('q') => ReaderAction::Escape,
        KeyCode::Char('d') if ctrl => ReaderAction::Page(1),
        KeyCode::Char('u') if ctrl => ReaderAction::Page(-1),
        KeyCode::Char(' ') | KeyCode::PageDown => ReaderAction::Page(2),
        KeyCode::Char('b') | KeyCode::PageUp => ReaderAction::Page(-2),
        KeyCode::Char('g') | KeyCode::Home => ReaderAction::Top,
        KeyCode::Char('G') | KeyCode::End => ReaderAction::Bottom,
        KeyCode::Char('+') | KeyCode::Char('=') => ReaderAction::Wider,
        KeyCode::Char('-') => ReaderAction::Narrower,
        KeyCode::Char('J') => ReaderAction::Justify,
        KeyCode::Char('L') => ReaderAction::Spacing,
        KeyCode::Char('T') => ReaderAction::Theme,
        KeyCode::Char('t') => ReaderAction::Contents,
        KeyCode::Char('/') => ReaderAction::Search,
        KeyCode::Char('n') => ReaderAction::NextMatch,
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => ReaderAction::Tab(1),
        KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => ReaderAction::Tab(-1),
        KeyCode::Char('f') => ReaderAction::Follow,
        KeyCode::Char('R') | KeyCode::Char('E') => ReaderAction::Refresh,
        KeyCode::Char('v') => ReaderAction::Select,
        KeyCode::Char('m') => ReaderAction::Mark,
        KeyCode::Char('c') => ReaderAction::MarkWithComment,
        KeyCode::Char('x') => ReaderAction::Unmark,
        KeyCode::Char('a') => ReaderAction::AddNote,
        KeyCode::Char('N') => ReaderAction::PreviousMatch,
        _ => return None,
    };
    Some(Action::Reader(action))
}

/// Shown while selecting sentences to highlight.
pub const SELECT_HELP: &str = "selecting: j/k next/previous sentence · l/h longer/shorter · m highlight · c highlight with comment · x remove highlight · Esc stop";

/// Shown at the bottom of the reader.
pub const READER_HELP: &str = "← → tabs · f follow figure · v highlight · a note · j/k scroll · t contents · / search · +/- width · J/L/T layout · R re-extract · q back";

/// Shown by `?`: key, what it does.
pub const HELP: &[(&str, &str)] = &[
    ("j k ↓ ↑", "move"),
    ("g G", "top, bottom"),
    ("Ctrl-d Ctrl-u", "page down, up"),
    ("Tab h l", "switch pane"),
    ("Enter", "read the paper / open project"),
    ("/", "filter (fuzzy, as you type)"),
    ("Ctrl-f", "search everything, papers' text too"),
    ("Esc", "clear filter, close"),
    ("s S", "sort by next column, reverse"),
    ("a", "add by DOI, arXiv ID, PMID or URL"),
    ("t", "toggle a tag by name"),
    ("1-9", "toggle a colored tag"),
    ("p", "get the PDF"),
    ("E", "extract the text again"),
    ("o", "open the PDF"),
    ("x", "move to the trash"),
    ("M", "merge a duplicate (≈) or mark distinct"),
    ("e F", "edit in $EDITOR, refetch"),
    ("L U", "link to a project, unlink from this one"),
    ("X", "export this view (.bib .yml .json)"),
    ("Y", "copy the cite key"),
    ("m", "reading: unread → reading → read"),
    ("w", "save this search (in the left pane)"),
    ("J", "jobs running"),
    ("r", "reload"),
    ("?", "this help"),
    ("q", "quit"),
];
