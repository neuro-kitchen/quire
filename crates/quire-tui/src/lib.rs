//! `quire tui`: the library in the terminal (`.knowledge/tui.md`).
//!
//! Elm-style: [`state::State`] is changed only by [`update::update`] from [`update::Action`]s
//! (keys, engine results, library changes); the engine work it asks for ([`update::Effect`]) runs
//! in background tasks ([`effects`]) that answer with actions, so the screen never waits.
//!
//! - [`state`]: what is shown and remembered; [`filter`]: fuzzy filter and sort.
//! - [`update`]: the pure state machine; [`keys`]: key bindings.
//! - [`effects`]: engine work; [`view`]: drawing.
//! - [`reader`]: reader mode, the paper's text in one column.

pub mod effects;
pub mod filter;
pub mod keys;
pub mod reader;
pub mod state;
pub mod update;
pub mod view;

#[cfg(test)]
mod test_support;

use std::sync::Arc;

use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::StreamExt;
use quire_engine::Engine;
use tokio::sync::broadcast::error::RecvError;

use crate::state::{Scope, State};
use crate::update::{Action, Effect};

/// Runs the TUI on `engine` until the user quits; the terminal is restored even on a panic.
pub async fn run(engine: Arc<Engine>) -> anyhow::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, engine).await;
    ratatui::restore();
    result
}

async fn event_loop(terminal: &mut ratatui::DefaultTerminal, engine: Arc<Engine>) -> anyhow::Result<()> {
    let (send, mut results) = tokio::sync::mpsc::unbounded_channel();
    let mut changes = engine.subscribe();
    let mut events = EventStream::new();
    let mut state = State::new(engine.library().label());
    state.reader_settings = effects::reader_settings(engine.config());
    state.positions = effects::load_positions(&engine.library_dir().join(effects::POSITIONS));
    for effect in [Effect::LoadSidebar, Effect::LoadRows(Scope::All)] {
        effects::spawn(&engine, effect, &send);
    }
    while !state.quit {
        terminal.draw(|frame| view::draw(frame, &mut state))?;
        let action = tokio::select! {
            event = events.next() => match event {
                Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => match &state.reader {
                    Some(reader) => keys::reader_action(key, matches!(reader.overlay, reader::Overlay::Search(_))),
                    None => keys::action(key, state.prompt.is_some()),
                },
                Some(Ok(_)) => None, // resize and others: just redraw
                Some(Err(error)) => return Err(error.into()),
                None => break,
            },
            Some(action) = results.recv() => Some(action),
            change = changes.recv() => match change {
                Ok(_) | Err(RecvError::Lagged(_)) => Some(Action::Changed),
                Err(RecvError::Closed) => None,
            },
        };
        if let Some(action) = action {
            for effect in update::update(&mut state, action) {
                effects::spawn(&engine, effect, &send);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use crate::state::State;
    use crate::test_support::row;
    use crate::update::{Action, update};

    fn screen(state: &mut State, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| crate::view::draw(frame, state)).unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn library_browser_screens() {
        let mut state = State::new("personal".into());
        update(
            &mut state,
            Action::Rows(vec![
                row("vaswani2017", "Attention Is All You Need", "Vaswani", 2017),
                row("kucsko2013", "Nanometre-scale thermometry in a living cell", "Kucsko", 2013),
            ]),
        );
        insta::assert_snapshot!("wide", screen(&mut state, 150, 16));
        update(&mut state, Action::Help);
        insta::assert_snapshot!("help", screen(&mut state, 150, 24));
        update(&mut state, Action::Escape);
        update(&mut state, Action::StartPrompt(crate::state::Prompt::Filter));
        update(&mut state, Action::Type('k'));
        insta::assert_snapshot!("narrow_filtered", screen(&mut state, 80, 12));
    }

    #[test]
    fn reader_screens() {
        use crate::reader::ReaderAction;
        let mut state = State::new("personal".into());
        let mut paper = row("vaswani2017", "Attention Is All You Need", "Vaswani", 2017);
        update(&mut state, Action::Rows(vec![paper.clone()]));
        assert!(update(&mut state, Action::Read).is_empty(), "no PDF: only the abstract");
        insta::assert_snapshot!("reader_abstract_only", screen(&mut state, 80, 10));
        assert!(update(&mut state, Action::Reader(ReaderAction::Escape)).is_empty(), "no position to save");
        paper.pdf = Some(quire_engine::PdfStatus::Available { pages: Some(15) });
        update(&mut state, Action::Rows(vec![paper]));
        let effects = update(&mut state, Action::Open);
        assert!(matches!(effects[..], [crate::update::Effect::LoadText(_)]));
        let id = state.reader.as_ref().unwrap().id;
        let markdown = "## 1 Introduction\n\nRecurrent neural networks, long short-term memory and **gated** recurrent \
            neural networks have been firmly established (Figure 1) as *state of the art* approaches.\n\n\
            Figure 1: The Transformer-model architecture.\n\n## 2 Background\n\n\
            The goal of reducing sequential computation also forms the foundation of the Extended Neural GPU.";
        let text = quire_engine::PaperText { markdown: markdown.into(), source: quire_engine::TextSource::Pdf };
        update(&mut state, Action::TextLoaded { id, text: Ok(text) });
        insta::assert_snapshot!("reader_opens_on_abstract", screen(&mut state, 80, 10));
        update(&mut state, Action::Reader(ReaderAction::Submit));
        update(&mut state, Action::Reader(ReaderAction::Search));
        for c in "neural".chars() {
            update(&mut state, Action::Reader(ReaderAction::Type(c)));
        }
        update(&mut state, Action::Reader(ReaderAction::Submit));
        insta::assert_snapshot!("reader", screen(&mut state, 80, 14));
        update(&mut state, Action::Reader(ReaderAction::Contents));
        insta::assert_snapshot!("reader_contents", screen(&mut state, 80, 14));
        update(&mut state, Action::Reader(ReaderAction::Escape));
        update(&mut state, Action::Reader(ReaderAction::Escape));
        update(&mut state, Action::Reader(ReaderAction::Follow));
        insta::assert_snapshot!("reader_figures", screen(&mut state, 80, 10));
        update(&mut state, Action::Reader(ReaderAction::Escape));
        assert_eq!(state.reader.as_ref().unwrap().page().tab, crate::reader::Tab::Paper, "back to the text");
        let effects = update(&mut state, Action::Reader(ReaderAction::Escape));
        assert!(state.reader.is_none());
        assert!(matches!(effects[..], [crate::update::Effect::SavePositions(_)]));
        assert!(state.positions.contains_key(&id));
    }
}
