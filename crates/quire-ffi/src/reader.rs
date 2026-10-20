//! The reader: a paper's tabs as styled blocks (the same reading model as the terminal reader,
//! `quire_document::reading`), its highlights and notes, and its reading status.

use quire_engine::reading::document::{Block, Kind};
use quire_engine::reading::paper::{self, Tab};
use quire_engine::{Engine, Highlight, Note, ReadingStatus};
use quire_model::ReferenceId;

use crate::{QuireError, Result};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PaperRecord {
    /// The abstract tab first, then the paper's text tabs (if its text was found).
    pub tabs: Vec<ReaderTab>,
    /// Where the text came from: `arXiv HTML`, `Europe PMC`, `PDF`; `None` without text.
    pub source: Option<String>,
    /// Why there is no text (no PDF, nothing online), when there is none.
    pub text_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ReaderTab {
    /// `abstract`, `paper`, `front`, `figures`, `tables`, `equations`, `references`.
    pub name: String,
    /// Figures, tables, equations or references on it; `None` for running text.
    pub count: Option<u32>,
    pub blocks: Vec<BlockRecord>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum BlockKind {
    Heading {
        level: u8,
    },
    Paragraph,
    ListItem,
    TableRow,
    Equation,
    /// A figure's image; `url` is its address (see [`crate::QuireLibrary::figure`]).
    Image {
        url: String,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct BlockRecord {
    pub kind: BlockKind,
    pub words: Vec<WordRecord>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct WordRecord {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    /// Names a figure, table or equation (`Figure 2`).
    pub mention: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct HighlightRecord {
    pub id: String,
    pub quote: String,
    /// Text just before the quote, telling repeated quotes apart.
    pub prefix: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct NoteRecord {
    pub id: String,
    /// Markdown.
    pub body: String,
    /// `YYYY-MM-DD`.
    pub updated: String,
}

pub(crate) async fn paper(engine: &Engine, id: &str) -> Result<PaperRecord> {
    let (reference, _) = engine.show(id).await?;
    let mut tabs = vec![tab(Tab::Abstract, None, paper::front_matter(&reference))];
    let (source, text_error) = match engine.reader_text(id).await {
        Ok((_, text)) => {
            let source = text.source;
            for content in paper::tabs(&text.markdown, source.has_math()) {
                tabs.push(tab(content.tab, content.count, content.blocks));
            }
            (Some(source.label().to_string()), None)
        }
        Err(e) => (None, Some(e.to_string())),
    };
    Ok(PaperRecord { tabs, source, text_error })
}

fn tab(tab: Tab, count: Option<usize>, blocks: Vec<Block>) -> ReaderTab {
    let name = match tab {
        Tab::Abstract => "abstract",
        Tab::Paper => "paper",
        Tab::Front => "front",
        Tab::Figures => "figures",
        Tab::Tables => "tables",
        Tab::Equations => "equations",
        Tab::References => "references",
        Tab::Notes => "notes",
    };
    ReaderTab { name: name.into(), count: count.map(|c| c as u32), blocks: blocks.into_iter().map(block).collect() }
}

fn block(block: Block) -> BlockRecord {
    let kind = match block.kind {
        Kind::Heading(level) => BlockKind::Heading { level },
        Kind::Paragraph => BlockKind::Paragraph,
        Kind::ListItem => BlockKind::ListItem,
        Kind::TableRow => BlockKind::TableRow,
        Kind::Equation => BlockKind::Equation,
        Kind::Image => BlockKind::Image { url: block.text() },
    };
    let words = match kind {
        BlockKind::Image { .. } => Vec::new(),
        _ => block
            .words
            .into_iter()
            .map(|w| WordRecord {
                text: w.text,
                bold: w.emphasis.bold,
                italic: w.emphasis.italic,
                mention: w.emphasis.mention,
            })
            .collect(),
    };
    BlockRecord { kind, words }
}

pub(crate) fn highlight(h: Highlight) -> HighlightRecord {
    HighlightRecord { id: h.id.to_string(), quote: h.quote, prefix: h.prefix, comment: h.comment }
}

pub(crate) fn note(n: Note) -> NoteRecord {
    NoteRecord { id: n.id.to_string(), body: n.body, updated: n.meta.updated_at.date().to_string() }
}

pub(crate) fn reading_status(text: &str) -> Result<ReadingStatus> {
    text.parse().map_err(|_| QuireError::Invalid(format!("`{text}` is not unread, reading or read")))
}

pub(crate) fn reference_id(text: &str) -> Result<ReferenceId> {
    text.parse().map_err(|_| QuireError::Invalid(format!("`{text}` is not a reference ID")))
}
