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
        KeyCode::Char('N') => ReaderAction::PreviousMatch,
        _ => return None,
    };
    Some(Action::Reader(action))
}

/// Shown at the bottom of the reader.
pub const READER_HELP: &str = "← → tabs · f follow figure · j/k scroll · t contents · / search · +/- width · J justify · L spacing · T theme · q back";

/// Shown by `?`: key, what it does.
pub const HELP: &[(&str, &str)] = &[
    ("j k ↓ ↑", "move"),
    ("g G", "top, bottom"),
    ("Ctrl-d Ctrl-u", "page down, up"),
    ("Tab h l", "switch pane"),
    ("Enter", "read the paper / open project"),
    ("/", "filter (fuzzy, as you type)"),
    ("Esc", "clear filter, close"),
    ("s S", "sort by next column, reverse"),
    ("a", "add by DOI, arXiv ID, PMID or URL"),
    ("t", "toggle a tag by name"),
    ("1-9", "toggle a colored tag"),
    ("p", "get the PDF"),
    ("o", "open the PDF"),
    ("x", "move to the trash"),
    ("r", "reload"),
    ("?", "this help"),
    ("q", "quit"),
];
