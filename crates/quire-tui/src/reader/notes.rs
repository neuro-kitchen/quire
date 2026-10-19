//! The reader's annotations: highlights placed on the paper's words (found again by their quote,
//! [`annotate::locate`]), the Notes tab listing notes and highlights, and the keys used while
//! selecting sentences (`v` starts; `j`/`k` move, `l`/`h` extend or shrink, `m` highlights, `c`
//! highlights with a comment, `x` removes the highlight there, `Esc` stops).

use quire_model::{Highlight, HighlightId, Note};

use super::annotate::{self, Marked, Selection};
use super::document::{Block, Kind, words};
use super::{Overlay, Page, Reader, ReaderAction, ReaderEffect, Tab, Text};

/// Highlights and notes of the paper being read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Annotations {
    pub highlights: Vec<Highlight>,
    pub notes: Vec<Note>,
    /// Where each highlight is in the paper's text (highlights not found are left out).
    pub marks: Vec<(HighlightId, Marked)>,
}

impl Annotations {
    /// The highlight covering any of `marked`'s words, if one does.
    pub fn at(&self, marked: &Marked) -> Option<HighlightId> {
        self.marks
            .iter()
            .find(|(_, m)| {
                m.block == marked.block && m.words.start < marked.words.end && marked.words.start < m.words.end
            })
            .map(|(id, _)| *id)
    }
}

impl Reader {
    /// The highlights and notes arrived (or changed).
    pub fn set_annotations(&mut self, highlights: Vec<Highlight>, notes: Vec<Note>) {
        self.annotations = Annotations { highlights, notes, marks: Vec::new() };
        self.place_annotations();
    }

    /// Places the highlights on the paper's words and rebuilds the Notes tab.
    pub(super) fn place_annotations(&mut self) {
        let current = self.page().tab;
        if let Some(paper) = self.pages.iter().find(|p| p.tab == Tab::Paper) {
            let marks = self
                .annotations
                .highlights
                .iter()
                .filter_map(|h| annotate::locate(&paper.blocks, &h.quote, &h.prefix).map(|m| (h.id, m)))
                .collect();
            self.annotations.marks = marks;
        }
        self.pages.retain(|p| p.tab != Tab::Notes);
        let count = self.annotations.highlights.len() + self.annotations.notes.len();
        if count > 0 {
            let blocks = self.notes_blocks();
            self.pages.push(Page::new(Tab::Notes, Some(count), blocks, &self.settings));
        }
        self.tab = self.pages.iter().position(|p| p.tab == current).unwrap_or(0);
    }

    /// The Notes tab: each note, then each highlight (quote in italics, comment below).
    fn notes_blocks(&self) -> Vec<Block> {
        let mut blocks = Vec::new();
        for note in &self.annotations.notes {
            blocks.push(Block { kind: Kind::Heading(4), words: words(&format!("Note: {}", note.title())) });
            blocks.extend(super::document::parse(&note.body, false));
        }
        for highlight in &self.annotations.highlights {
            let placed = self.annotations.marks.iter().any(|(id, _)| *id == highlight.id);
            let title = if placed { "Highlight" } else { "Highlight (not found in this text)" };
            blocks.push(Block { kind: Kind::Heading(4), words: words(title) });
            let mut quote = words(&highlight.quote);
            quote.iter_mut().for_each(|w| w.emphasis.italic = true);
            blocks.push(Block { kind: Kind::Paragraph, words: quote });
            if let Some(comment) = &highlight.comment {
                blocks.extend(super::document::parse(comment, false));
            }
        }
        blocks
    }

    /// `v`: a selection on the first sentence at the top of the paper's screen.
    pub(super) fn start_selecting(&mut self) {
        let page = self.page();
        if page.tab != Tab::Paper || self.text != Text::Loaded {
            return;
        }
        self.selection = Selection::at(&page.blocks, page.current_block());
    }

    /// The selected words, if selecting.
    pub fn selected(&self) -> Option<Marked> {
        let paper = self.pages.iter().find(|p| p.tab == Tab::Paper)?;
        Some(self.selection?.marked(&paper.blocks))
    }

    /// Keys while selecting.
    pub(super) fn apply_selecting(&mut self, action: ReaderAction) -> ReaderEffect {
        let Some(selection) = self.selection else { return ReaderEffect::None };
        let Some(paper) = self.pages.iter().position(|p| p.tab == Tab::Paper) else { return ReaderEffect::None };
        let blocks = &self.pages[paper].blocks;
        let moved = match action {
            ReaderAction::Scroll(by) if by > 0 => selection.next(blocks),
            ReaderAction::Scroll(_) => selection.previous(blocks),
            ReaderAction::Tab(by) if by > 0 => selection.extend(blocks),
            ReaderAction::Tab(_) => selection.shrink(),
            ReaderAction::Mark => {
                let (quote, prefix) = annotate::quote_of(blocks, &selection.marked(blocks));
                self.selection = None;
                return ReaderEffect::SaveHighlight { quote, prefix, comment: None };
            }
            ReaderAction::MarkWithComment => {
                self.overlay = Overlay::Comment(String::new());
                return ReaderEffect::None;
            }
            ReaderAction::Unmark => {
                let found = self.annotations.at(&selection.marked(blocks));
                self.selection = None;
                return found.map_or(ReaderEffect::None, ReaderEffect::RemoveHighlight);
            }
            ReaderAction::Escape | ReaderAction::Select => {
                self.selection = None;
                return ReaderEffect::None;
            }
            _ => return ReaderEffect::None,
        };
        self.selection = Some(moved);
        self.keep_selection_visible();
        ReaderEffect::None
    }

    /// Scrolls the paper so the selection's first line is on screen.
    fn keep_selection_visible(&mut self) {
        let Some(selection) = self.selection else { return };
        let page = self.page();
        let Some(line) = page.laid.lines.iter().position(|l| l.block == Some(selection.block)) else { return };
        let (top, height) = (page.top, self.height.max(1));
        if line < top || line >= top + height {
            let last = page.laid.lines.len().saturating_sub(height);
            self.pages[self.tab].top = line.saturating_sub(2).min(last);
        }
    }

    /// Keys while typing a comment or a note.
    pub(super) fn apply_typing(&mut self, action: ReaderAction) -> ReaderEffect {
        let (Overlay::Comment(text) | Overlay::Note(text)) = &mut self.overlay else { return ReaderEffect::None };
        match action {
            ReaderAction::Type(c) => text.push(c),
            ReaderAction::Backspace => {
                text.pop();
            }
            ReaderAction::Escape => self.overlay = Overlay::None,
            ReaderAction::Submit => {
                let overlay = std::mem::replace(&mut self.overlay, Overlay::None);
                return match overlay {
                    Overlay::Note(text) if !text.trim().is_empty() => ReaderEffect::AddNote(text),
                    Overlay::Comment(text) => {
                        let Some(marked) = self.selected() else { return ReaderEffect::None };
                        let paper = self.pages.iter().find(|p| p.tab == Tab::Paper).expect("selected on the paper");
                        let (quote, prefix) = annotate::quote_of(&paper.blocks, &marked);
                        self.selection = None;
                        let comment = Some(text).filter(|t| !t.trim().is_empty());
                        ReaderEffect::SaveHighlight { quote, prefix, comment }
                    }
                    _ => ReaderEffect::None,
                };
            }
            _ => {}
        }
        ReaderEffect::None
    }
}
