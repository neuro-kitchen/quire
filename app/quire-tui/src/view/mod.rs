//! Drawing the [`State`]: a header, the scopes pane (all references, projects, trash), the
//! references table and the details of the selected one, a status line; help on top. An open
//! reader takes the whole screen (with figure images on graphics terminals, [`graphics`]).
//!
//! Wide terminals (≥ 140 columns) show all three panes; narrower ones show the details in place of
//! the table when the details pane has the focus.

mod details;
pub mod graphics;
mod help;
mod jobs;
mod reader;
mod sidebar;
mod status;
mod table;
pub mod theme;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

pub use graphics::Graphics;

use crate::state::{Pane, State};

/// Columns from which all three panes fit side by side.
const WIDE: u16 = 140;

pub fn draw(frame: &mut Frame, state: &mut State, graphics: &mut Graphics) {
    if let Some(open) = state.reader.as_mut() {
        return reader::draw(frame, open, graphics);
    }
    let [header, body, footer] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    status::header(frame, header, state);
    let sidebar_width = Constraint::Length(26);
    if body.width >= WIDE {
        let [left, middle, right] =
            Layout::horizontal([sidebar_width, Constraint::Fill(3), Constraint::Fill(2)]).areas(body);
        sidebar::draw(frame, left, state);
        table::draw(frame, middle, state);
        details::draw(frame, right, state);
    } else {
        let [left, main] = Layout::horizontal([sidebar_width, Constraint::Fill(1)]).areas(body);
        sidebar::draw(frame, left, state);
        if state.focus == Pane::Details {
            details::draw(frame, main, state);
        } else {
            table::draw(frame, main, state);
        }
    }
    status::footer(frame, footer, state);
    if state.help {
        help::draw(frame, frame.area());
    }
    if state.show_jobs {
        jobs::draw(frame, frame.area(), state);
    }
}
