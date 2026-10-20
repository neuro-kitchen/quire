//! Blocks → lines of a centred column of `width` characters: words wrapped (and justified if
//! asked), headings set apart, list items with a hanging bullet, tables in aligned columns.
//! Recomputed only when the text, width or settings change.

use super::document::{Block, Emphasis, Kind, Word};

/// How the reader lays the text out (saved in `config.toml` `[reader]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// Characters per line (the measure).
    pub width: u16,
    pub justify: bool,
    /// 1 = single, 2 = a blank line after each line.
    pub spacing: u8,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self { width: 72, justify: false, spacing: 1, theme: Theme::Dark }
    }
}

/// Narrowest and widest measure.
pub const MIN_WIDTH: u16 = 40;
pub(crate) const MAX_WIDTH: u16 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
    Sepia,
    Contrast,
}

impl Theme {
    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Sepia,
            Self::Sepia => Self::Contrast,
            Self::Contrast => Self::Dark,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Sepia => "sepia",
            Self::Contrast => "contrast",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Dark, Self::Light, Self::Sepia, Self::Contrast].into_iter().find(|t| t.as_str() == text)
    }
}

/// A piece of a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub emphasis: Emphasis,
    /// Index of the word in its block (`None` for spaces, bullets and table rows): lets the view
    /// mark highlighted and selected words.
    pub word: Option<usize>,
}

impl Piece {
    fn plain(text: String) -> Self {
        Self { text, emphasis: Emphasis::default(), word: None }
    }
}

/// One screen line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub pieces: Vec<Piece>,
    /// The block it belongs to (`None` for spacing).
    pub block: Option<usize>,
    pub kind: Option<Kind>,
}

impl Line {
    fn blank() -> Self {
        Self { pieces: Vec::new(), block: None, kind: None }
    }

    pub fn text(&self) -> String {
        self.pieces.iter().map(|p| p.text.as_str()).collect()
    }
}

/// The laid-out text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Laid {
    pub lines: Vec<Line>,
    /// Headings: (line, level, text), for the table of contents.
    pub headings: Vec<(usize, u8, String)>,
}

pub(crate) fn lay_out(blocks: &[Block], settings: &Settings) -> Laid {
    let width = settings.width as usize;
    let mut laid = Laid::default();
    let columns = table_columns(blocks, width);
    for (index, block) in blocks.iter().enumerate() {
        let next_row = block.kind == Kind::TableRow && index > 0 && blocks[index - 1].kind == Kind::TableRow;
        if !laid.lines.is_empty() && !next_row {
            laid.lines.push(Line::blank());
            if matches!(block.kind, Kind::Heading(level) if level <= 3) {
                laid.lines.push(Line::blank());
            }
        }
        let (indent, first_prefix) = match block.kind {
            Kind::ListItem => (2, "• "),
            Kind::Equation => (4, "    "),
            _ => (0, ""),
        };
        if let Kind::Heading(level) = block.kind {
            laid.headings.push((laid.lines.len(), level, block.text()));
        }
        if block.kind == Kind::TableRow {
            let widths = &columns[index];
            let header = !next_row && blocks.get(index + 1).is_some_and(|b| b.kind == Kind::TableRow);
            let emphasis = Emphasis { bold: header, ..Emphasis::default() };
            let cells: Vec<String> =
                widths.iter().enumerate().map(|(i, w)| cell(block.words.get(i).map_or("", |c| &c.text), *w)).collect();
            laid.lines.push(Line {
                pieces: vec![Piece { text: cells.join(" │ ").trim_end().to_string(), emphasis, word: None }],
                block: Some(index),
                kind: Some(block.kind),
            });
            continue;
        }
        let justify = settings.justify && block.kind == Kind::Paragraph;
        let rows = wrap(block, width.saturating_sub(indent).max(10));
        let count = rows.len();
        for (row, words) in rows.into_iter().enumerate() {
            let prefix = if row == 0 { first_prefix.to_string() } else { " ".repeat(indent) };
            let mut pieces = vec![Piece::plain(prefix)];
            let last = row + 1 == count;
            pieces.extend(join(&words, width.saturating_sub(indent), justify && !last));
            laid.lines.push(Line { pieces, block: Some(index), kind: Some(block.kind) });
            if settings.spacing > 1 && !last {
                laid.lines.push(Line::blank());
            }
        }
    }
    laid
}

/// Column widths of the table each table row belongs to (empty for other blocks): each column as
/// wide as its longest cell, the widest ones narrowed until the table fits `width`.
fn table_columns(blocks: &[Block], width: usize) -> Vec<Vec<usize>> {
    let mut columns = vec![Vec::new(); blocks.len()];
    let mut start = 0;
    while start < blocks.len() {
        if blocks[start].kind != Kind::TableRow {
            start += 1;
            continue;
        }
        let end = (start..blocks.len()).find(|i| blocks[*i].kind != Kind::TableRow).unwrap_or(blocks.len());
        let mut widths: Vec<usize> = Vec::new();
        for row in &blocks[start..end] {
            for (i, cell) in row.words.iter().enumerate() {
                let len = cell.text.chars().count();
                if i < widths.len() {
                    widths[i] = widths[i].max(len);
                } else {
                    widths.push(len);
                }
            }
        }
        let separators = 3 * widths.len().saturating_sub(1);
        while widths.iter().sum::<usize>() + separators > width {
            let Some(widest) = widths.iter_mut().max().filter(|w| **w > MIN_CELL) else { break };
            *widest -= 1;
        }
        columns[start..end].fill(widths);
        start = end;
    }
    columns
}

/// Narrowest a table column gets.
const MIN_CELL: usize = 4;

/// `text` padded or cut (with `…`) to `width` characters.
fn cell(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len <= width {
        format!("{text}{}", " ".repeat(width - len))
    } else {
        text.chars().take(width.saturating_sub(1)).chain(['…']).collect()
    }
}

/// Greedy wrapping of `block`'s words into rows of at most `width` characters (a longer word
/// gets a row of its own).
fn wrap(block: &Block, width: usize) -> Vec<Vec<(usize, &Word)>> {
    let mut rows: Vec<Vec<(usize, &Word)>> = vec![Vec::new()];
    let mut used = 0;
    for (index, word) in block.words.iter().enumerate() {
        let len = word.text.chars().count();
        let row = rows.last_mut().expect("never empty");
        let needed = if row.is_empty() { len } else { used + 1 + len };
        if !row.is_empty() && needed > width {
            rows.push(vec![(index, word)]);
            used = len;
        } else {
            row.push((index, word));
            used = needed;
        }
    }
    rows.retain(|r| !r.is_empty());
    rows
}

/// A row's words as pieces separated by spaces; justified rows spread the extra space over the
/// gaps, the first gaps getting one more.
fn join(words: &[(usize, &Word)], width: usize, justify: bool) -> Vec<Piece> {
    let gaps = words.len().saturating_sub(1);
    let letters: usize = words.iter().map(|(_, w)| w.text.chars().count()).sum();
    let spare = width.saturating_sub(letters + gaps);
    let mut pieces = Vec::with_capacity(words.len() * 2);
    for (i, (index, word)) in words.iter().enumerate() {
        if i > 0 {
            let extra = if justify && gaps > 0 { spare / gaps + usize::from(i - 1 < spare % gaps) } else { 0 };
            pieces.push(Piece::plain(" ".repeat(1 + extra)));
        }
        pieces.push(Piece { text: word.text.clone(), emphasis: word.emphasis, word: Some(*index) });
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::document::parse;

    #[test]
    fn wraps_justifies_and_indexes_headings() {
        let blocks = parse(
            "## Title\n\nThe quick brown fox jumps over the lazy dog again and again.\n\n- a list item that is long enough to wrap",
            false,
        );
        let settings = Settings { width: 20, ..Settings::default() };
        let laid = lay_out(&blocks, &settings);
        let texts: Vec<String> = laid.lines.iter().map(Line::text).collect();
        assert!(texts.iter().all(|t| t.chars().count() <= 20), "{texts:?}");
        assert_eq!(laid.headings, [(0, 2, "Title".to_string())]);
        assert!(texts.iter().any(|t| t.starts_with("• a list")) && texts.iter().any(|t| t.starts_with("  ")));
        let justified = lay_out(&blocks, &Settings { justify: true, ..settings });
        let first = justified.lines.iter().find(|l| l.kind == Some(Kind::Paragraph)).unwrap().text();
        assert_eq!(first.chars().count(), 20, "justified to the width: {first:?}");
        let double = lay_out(&blocks, &Settings { spacing: 2, ..settings });
        assert!(double.lines.len() > laid.lines.len());
    }

    #[test]
    fn tables_line_up() {
        let blocks =
            parse("|Layer Type|Complexity|\n|---|---|\n|Self-Attention|O(n<sup>2</sup>)|\n|RNN|*O*(n)|", false);
        let laid = lay_out(&blocks, &Settings { width: 40, ..Settings::default() });
        let texts: Vec<String> = laid.lines.iter().map(Line::text).collect();
        assert_eq!(texts, ["Layer Type     │ Complexity", "Self-Attention │ O(n²)", "RNN            │ O(n)"]);
        assert!(laid.lines[0].pieces[0].emphasis.bold, "header row");
        let narrow = lay_out(&blocks, &Settings { width: 16, ..Settings::default() });
        assert!(narrow.lines.iter().all(|l| l.text().chars().count() <= 16));
        assert!(narrow.lines[1].text().contains('…'));
    }
}
