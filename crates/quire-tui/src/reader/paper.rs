//! The paper split into what the reader shows on separate tabs: the running text, figure and
//! table captions (with the table's rows), numbered equations and the reference list. The PDF's
//! text puts captions wherever they sat on the page, in the middle of paragraphs; here they leave
//! the text, and mentions like "Figure 2" are marked so `f` can jump to them.
//!
//! Seen in `pdf-inspector` output (2026-10-19):
//! - captions as their own paragraph: `Figure 1: …`, `Fig. 3.`, `Table 2: …`, `Fig. 1 | …`, often
//!   cut in two (`… consists of several` / `attention layers running in parallel.`);
//! - a table's rows right after its caption;
//! - display equations as one short paragraph with the number in it: `… = softmax( √)V (1) dk`;
//! - the reference list under a `References` heading, an appendix possibly after it.

use quire_engine::Reference;
use quire_engine::paper_markdown::{
    CaptionKind, FRONT_END, FRONT_START, caption_label, is_label_number, is_references_title,
};
use quire_model::TextField;

use super::document::{Block, Kind, Word, words};

/// The reader's tabs, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Abstract,
    Paper,
    /// The PDF's front matter (journal header, authors, affiliations, history, copyright).
    Front,
    Figures,
    Tables,
    Equations,
    References,
    /// The user's notes on the paper and highlights in it.
    Notes,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Abstract => "Abstract",
            Self::Paper => "Paper",
            Self::Front => "Front matter",
            Self::Figures => "Figures",
            Self::Tables => "Tables",
            Self::Equations => "Equations",
            Self::References => "References",
            Self::Notes => "Notes",
        }
    }
}

/// `markdown` cut into its front matter section (`<!-- front -->` … `<!-- /front -->`, written by
/// the PDF cleanup) and the rest.
pub fn split_front(markdown: &str) -> (&str, String) {
    let Some(start) = markdown.find(FRONT_START) else { return ("", markdown.to_string()) };
    let after = &markdown[start + FRONT_START.len()..];
    let Some(end) = after.find(FRONT_END) else { return ("", markdown.to_string()) };
    let body = format!("{}{}", &markdown[..start], &after[end + FRONT_END.len()..]);
    (after[..end].trim(), body)
}

/// A figure, table or equation: its label (`Figure 2`, `(1)`) and what is shown for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub label: String,
    pub blocks: Vec<Block>,
    /// A figure's image (`![Figure](url)` before its caption; arXiv HTML).
    pub image: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Paper {
    pub body: Vec<Block>,
    pub figures: Vec<Item>,
    pub tables: Vec<Item>,
    pub equations: Vec<Item>,
    pub references: Vec<Block>,
}

/// A tab after `Paper`: its items as blocks (a heading per item) and the images they show (each
/// from the block of its item's heading).
pub struct ExtraTab {
    pub tab: Tab,
    pub count: usize,
    pub blocks: Vec<Block>,
    pub images: Vec<(usize, String)>,
}

impl Paper {
    /// The tabs after `Paper` that have something.
    pub fn extra_tabs(&self) -> Vec<ExtraTab> {
        let items = |items: &[Item]| -> (Vec<Block>, Vec<(usize, String)>) {
            let (mut blocks, mut images) = (Vec::new(), Vec::new());
            for item in items {
                if let Some(url) = &item.image {
                    images.push((blocks.len(), url.clone()));
                }
                blocks.push(Block { kind: Kind::Heading(4), words: words(&item.label) });
                blocks.extend(item.blocks.iter().cloned());
            }
            (blocks, images)
        };
        let tab = |tab, count, (blocks, images)| ExtraTab { tab, count, blocks, images };
        [
            tab(Tab::Figures, self.figures.len(), items(&self.figures)),
            tab(Tab::Tables, self.tables.len(), items(&self.tables)),
            tab(Tab::Equations, self.equations.len(), items(&self.equations)),
            tab(Tab::References, self.references.len(), (self.references.clone(), Vec::new())),
        ]
        .into_iter()
        .filter(|extra| extra.count > 0)
        .collect()
    }
}

/// Longest paragraph taken as the second half of a cut caption.
const MAX_CONTINUATION_WORDS: usize = 60;
/// Longest paragraph taken for a display equation.
const MAX_EQUATION_WORDS: usize = 30;

/// Splits the blocks of a paper into text, captions, equations and references.
pub fn organize(blocks: Vec<Block>) -> Paper {
    let mut paper = Paper::default();
    let mut in_references = false;
    // Whether the list's entries are numbered (`[1] …`, `1. …`), from its first entry.
    let mut numbered: Option<bool> = None;
    // Rows of the display equation being read.
    let mut rows: Vec<Block> = Vec::new();
    // A figure's image, waiting for the caption after it.
    let mut image: Option<String> = None;
    let mut i = 0;
    while i < blocks.len() {
        let mut block = blocks[i].clone();
        i += 1;
        if let Kind::Heading(_) = block.kind {
            in_references = is_references_title(&block.text());
            numbered = None;
            if in_references {
                continue;
            }
        }
        // A numbered list ends at the first block that is not an entry (figure text, an
        // appendix without a heading); captions inside it still go to their tab.
        if in_references && caption(&block).is_none() {
            let entry = is_numbered_entry(&block);
            if *numbered.get_or_insert(entry) == entry {
                paper.references.push(block);
                continue;
            }
            in_references = false;
        }
        if block.kind == Kind::Image {
            image = Some(block.text());
            continue;
        }
        if let Some((of, label, rest)) = caption(&block) {
            let mut caption = Block { kind: Kind::Paragraph, words: rest };
            let mut rows = Vec::new();
            // The second half of a caption cut in two.
            if let Some(next) = blocks.get(i)
                && !ends_sentence(&caption)
                && next.kind == Kind::Paragraph
                && next.words.len() <= MAX_CONTINUATION_WORDS
                && (starts_lowercase(next) || (of == CaptionKind::Table && blocks.get(i + 1).is_some_and(is_row)))
            {
                caption.words.extend(next.words.iter().cloned());
                i += 1;
            }
            if of == CaptionKind::Table {
                while let Some(row) = blocks.get(i).filter(|b| is_row(b)) {
                    rows.push(row.clone());
                    i += 1;
                }
            }
            let mut shown = vec![caption];
            shown.extend(rows);
            let image = if of == CaptionKind::Figure { image.take() } else { None };
            let item = Item { label, blocks: shown, image };
            match of {
                CaptionKind::Figure => paper.figures.push(item),
                CaptionKind::Table => paper.tables.push(item),
            }
            continue;
        }
        // An image belongs to the caption right after it; any other block drops it.
        image = None;
        if block.kind == Kind::Equation {
            // A display equation's rows (from `$$…$$`); its number is on the last one.
            rows.push(block.clone());
            if let Some(number) = block.words.last().and_then(|w| equation_label(&w.text)) {
                paper.equations.push(Item {
                    label: format!("({number})"),
                    blocks: std::mem::take(&mut rows),
                    image: None,
                });
            }
        } else {
            rows.clear();
            if let Some(number) = equation_number(&block) {
                block.kind = Kind::Equation;
                paper.equations.push(Item { label: format!("({number})"), blocks: vec![block.clone()], image: None });
            }
        }
        paper.body.push(block);
    }
    for block in &mut paper.body {
        mark_mentions(&mut block.words);
    }
    paper
}

/// The abstract tab: title, authors, venue and year, identifiers, then the abstract.
pub fn front_matter(reference: &Reference) -> Vec<Block> {
    let data = &reference.data;
    let mut blocks = vec![Block { kind: Kind::Heading(1), words: words(data.title().unwrap_or("(no title)")) }];
    let authors: Vec<String> = data.creators.iter().map(|c| c.display_name()).collect();
    if !authors.is_empty() {
        let mut line = words(&authors.join(", "));
        line.iter_mut().for_each(|w| w.emphasis.italic = true);
        blocks.push(Block { kind: Kind::Paragraph, words: line });
    }
    let venue = [
        data.get(TextField::ContainerTitle).map(str::to_string),
        data.issued.map(|d| d.get_year().to_string()),
        data.identifiers.first().map(|id| id.to_string()),
    ];
    let venue: Vec<String> = venue.into_iter().flatten().collect();
    if !venue.is_empty() {
        blocks.push(Block { kind: Kind::Paragraph, words: words(&venue.join(" · ")) });
    }
    blocks.push(Block { kind: Kind::Heading(3), words: words("Abstract") });
    match data.get(TextField::Abstract) {
        Some(text) => blocks.extend(
            text.split("\n\n")
                .filter(|p| !p.trim().is_empty())
                .map(|p| Block { kind: Kind::Paragraph, words: words(&p.replace('\n', " ")) }),
        ),
        None => blocks.push(Block {
            kind: Kind::Paragraph,
            words: words("No abstract stored. `quire ref abstracts` looks for one."),
        }),
    }
    blocks
}

/// `[12] …`, `12. …`.
fn is_numbered_entry(block: &Block) -> bool {
    let Some(first) = block.words.first() else { return false };
    let number = first.text.strip_prefix('[').and_then(|t| t.strip_suffix(']')).or(first.text.strip_suffix('.'));
    number.is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

fn is_row(block: &Block) -> bool {
    block.kind == Kind::TableRow
}

fn ends_sentence(block: &Block) -> bool {
    block.words.last().is_some_and(|w| w.text.ends_with(['.', '!', '?']))
}

fn starts_lowercase(block: &Block) -> bool {
    block.words.first().and_then(|w| w.text.chars().next()).is_some_and(char::is_lowercase)
}

/// A caption paragraph ([`caption_label`]): what it labels, the label (`Figure 2`), the rest.
fn caption(block: &Block) -> Option<(CaptionKind, String, Vec<Word>)> {
    if block.kind != Kind::Paragraph {
        return None;
    }
    let text = block.text();
    let label = caption_label(&text)?;
    let label_words = text[..label.len].split_whitespace().count();
    Some((label.kind, label.name(), block.words[label_words.min(block.words.len())..].to_vec()))
}

/// A short paragraph with `=` and a `(N)` word: a display equation, and its number.
fn equation_number(block: &Block) -> Option<String> {
    if block.kind != Kind::Paragraph || block.words.len() > MAX_EQUATION_WORDS || !block.text().contains('=') {
        return None;
    }
    block.words.iter().find_map(|w| equation_label(&w.text))
}

/// `(12)` → `12` (also `(3a)`, `(S2)`).
fn equation_label(word: &str) -> Option<String> {
    let inner = word.strip_prefix('(')?.strip_suffix(')')?;
    is_label_number(inner).then(|| inner.to_string())
}

/// Words that name a figure, table or equation when followed by a number.
const MENTIONS: &[&str] = &["figure", "figures", "fig.", "figs.", "table", "tables", "tab.", "eq.", "eqs.", "equation"];

/// Marks `Figure 2`, `Fig. 3a`, `Table 1`, `Eq. (4)` so they stand out and `f` can follow them.
fn mark_mentions(words: &mut [Word]) {
    for i in 0..words.len().saturating_sub(1) {
        let name = words[i].text.trim_start_matches('(').to_lowercase();
        if MENTIONS.contains(&name.as_str()) && is_label_number(&mention_number(&words[i + 1].text)) {
            words[i].emphasis.mention = true;
            words[i + 1].emphasis.mention = true;
        }
    }
}

/// `2,` / `(4))` / `3a.` → `2` / `4` / `3a`.
pub fn mention_number(text: &str) -> String {
    text.trim_matches(|c: char| !c.is_alphanumeric()).to_string()
}

/// What a mention points to: the tab and the number (`Figure 2` → Figures, `2`).
pub fn mention_target(name: &str, number: &str) -> Option<(Tab, String)> {
    let name = name.trim_start_matches('(').to_lowercase();
    let tab = match name.as_str() {
        "figure" | "figures" | "fig." | "figs." => Tab::Figures,
        "table" | "tables" | "tab." => Tab::Tables,
        "eq." | "eqs." | "equation" => Tab::Equations,
        _ => return None,
    };
    Some((tab, mention_number(number)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::document::parse;

    #[test]
    fn figure_images_go_with_their_caption() {
        let md = "Text.\n\n![Figure](https://arxiv.org/html/x/f1.png)\n\nFigure 1: A model.\n\n\
                  Figure 2: No image.\n\nTable 1: A table.";
        let paper = organize(parse(md, false));
        assert_eq!(paper.body.len(), 1, "the image line is not text");
        assert_eq!(paper.figures[0].image.as_deref(), Some("https://arxiv.org/html/x/f1.png"));
        assert_eq!(paper.figures[1].image, None);
        let orphan = organize(parse("![Figure](https://x.org/a.png)\n\nText.\n\nFigure 1: Late.", false));
        assert_eq!(orphan.figures[0].image, None, "only the caption right after the image");
        let figures = paper.extra_tabs().into_iter().find(|e| e.tab == Tab::Figures).unwrap();
        assert_eq!(figures.images, [(0, "https://arxiv.org/html/x/f1.png".to_string())]);
    }

    #[test]
    fn captions_tables_equations_and_references_leave_the_text() {
        let md = "#### 3 Model\n\nSee the left of Figure 1, and Table 1.\n\nFigure 1: The Transformer-model architecture.\n\n\
            Figure 2: (left) Scaled Dot-Product Attention. (right) Multi-Head Attention consists of several\n\n\
            attention layers running in parallel.\n\nAttention(*Q,K,V*) = softmax( *√*)*V* (1) *d<sub>k</sub>*\n\n\
            Table 1: Maximum path lengths\n\nfor different layer types.\n\n|Layer|Complexity|\n|---|---|\n|Self|O(1)|\n\n\
            Table 2 summarizes our results.\n\n#### References\n\n[1] Jimmy Lei Ba. Layer normalization. 2016.\n\n\
            [2] Dzmitry Bahdanau. Neural machine translation. 2014.\n\nFigure 3:\n\nin in this this spirit\n\n\
            Input-Input Layer5\n\n# Appendix\n\nMore figures.";
        let paper = organize(parse(md, false));
        let labels = |items: &[Item]| items.iter().map(|i| i.label.clone()).collect::<Vec<_>>();
        assert_eq!(labels(&paper.figures), ["Figure 1", "Figure 2", "Figure 3"], "captions after the references too");
        assert!(paper.figures[1].blocks[0].text().ends_with("running in parallel."), "cut caption joined");
        assert_eq!(labels(&paper.tables), ["Table 1"]);
        assert_eq!(paper.tables[0].blocks.len(), 3, "caption and two rows");
        assert!(paper.tables[0].blocks[0].text().ends_with("layer types."));
        assert_eq!(labels(&paper.equations), ["(1)"]);
        assert_eq!(paper.references.len(), 2);
        let body: Vec<String> = paper.body.iter().map(Block::text).collect();
        assert_eq!(
            body,
            [
                "3 Model",
                "See the left of Figure 1, and Table 1.",
                "Attention(Q,K,V) = softmax( √)V (1) dₖ",
                "Table 2 summarizes our results.",
                "Input-Input Layer5",
                "Appendix",
                "More figures."
            ]
        );
        assert_eq!(paper.body[2].kind, Kind::Equation);
        let marked: Vec<&str> =
            paper.body[1].words.iter().filter(|w| w.emphasis.mention).map(|w| w.text.as_str()).collect();
        assert_eq!(marked, ["Figure", "1,", "Table", "1."]);
        let tabs: Vec<Tab> = paper.extra_tabs().into_iter().map(|extra| extra.tab).collect();
        assert_eq!(tabs, [Tab::Figures, Tab::Tables, Tab::Equations, Tab::References]);
    }

    #[test]
    fn caption_forms() {
        let label = |md: &str| organize(parse(md, false)).figures.first().map(|i| i.label.clone());
        assert_eq!(label("Fig. 3. Results."), Some("Figure 3".into()));
        assert_eq!(label("**Fig. 1 |** Overview of the device."), Some("Figure 1".into()));
        assert_eq!(label("Figure S2: Supplementary."), Some("Figure S2".into()));
        assert_eq!(label("Figure 2 shows the results."), None, "a sentence, not a caption");
        assert_eq!(mention_target("(Fig.", "2b)"), Some((Tab::Figures, "2b".into())));
    }
}
