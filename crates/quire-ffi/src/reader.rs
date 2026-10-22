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
    let mut tabs = vec![tab(Tab::Abstract, None, paper::front_matter(&reference), Vec::new())];
    let (source, text_error) = match engine.reader_text(id).await {
        Ok((_, text)) => {
            let source = text.source;
            for content in paper::tabs(&text.markdown, source.has_math()) {
                tabs.push(tab(content.tab, content.count, content.blocks, content.images));
            }
            (Some(source.label().to_string()), None)
        }
        Err(e) => (None, Some(e.to_string())),
    };
    Ok(PaperRecord { tabs, source, text_error })
}

/// `images`: each figure's image, shown before the block at its index (its label): an
/// `Image` block there. Dropping them lost every full text's figure images (2026-10-22).
fn tab(tab: Tab, count: Option<usize>, blocks: Vec<Block>, images: Vec<(usize, String)>) -> ReaderTab {
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
    let mut records = Vec::with_capacity(blocks.len() + images.len());
    let mut images = images.into_iter().peekable();
    for (i, b) in blocks.into_iter().enumerate() {
        while let Some((_, url)) = images.next_if(|(at, _)| *at <= i) {
            records.push(BlockRecord { kind: BlockKind::Image { url }, words: Vec::new() });
        }
        records.push(block(b));
    }
    records.extend(images.map(|(_, url)| BlockRecord { kind: BlockKind::Image { url }, words: Vec::new() }));
    ReaderTab { name: name.into(), count: count.map(|c| c as u32), blocks: records }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A full text's figure image reaches the app: an `Image` block right before its label.
    #[test]
    fn figure_images_cross_the_bridge() {
        let markdown = "## Results\n\nText.\n\n![Figure](https://pmc.ncbi.nlm.nih.gov/articles/instance/1/bin/f1.jpg)\n\n\
                        Figure 1: Schematic.\n\nFigure 2: No image.\n";
        let figures = paper::tabs(markdown, false)
            .into_iter()
            .find(|t| t.tab == Tab::Figures)
            .map(|t| tab(t.tab, t.count, t.blocks, t.images))
            .expect("a Figures tab");
        let kinds: Vec<_> = figures.blocks.iter().map(|b| &b.kind).collect();
        assert_eq!(kinds[0], &BlockKind::Image { url: "https://pmc.ncbi.nlm.nih.gov/articles/instance/1/bin/f1.jpg".into() });
        assert_eq!(kinds[1], &BlockKind::Heading { level: 4 });
        assert_eq!(kinds.iter().filter(|k| matches!(k, BlockKind::Image { .. })).count(), 1, "{kinds:?}");
    }
}
