//! Reader mode: the paper as one column of text you can tune (`.knowledge/tui.md`, section 1).
//!
//! - [`document`]: the stored Markdown → blocks (headings, paragraphs, lists, table rows).
//! - [`math`]: LaTeX → Unicode for equations from arXiv and Europe PMC.
//! - [`paper`]: blocks → the running text, and figures, tables, equations, references apart.
//! - [`layout`]: blocks → lines for a width and [`Settings`].
//!
//! The reader opens on the abstract (from the library, so it works without a PDF); `←`/`→` move
//! between tabs, each keeping its own scroll position. [`Reader`] is changed by
//! [`Reader::apply`] (pure, tested) from [`ReaderAction`]s. A settings change keeps the reading
//! position at the same block, not the same line number.

pub mod document;
pub mod layout;
pub mod math;
pub mod paper;

use quire_model::ReferenceId;

pub use layout::{Settings, Theme};
pub use paper::Tab;

use self::document::Block;
use self::layout::{Laid, MAX_WIDTH, MIN_WIDTH};

/// What the reader is doing besides showing text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    None,
    /// The table of contents, with the selected heading.
    Contents(usize),
    /// Typing a search.
    Search(String),
}

/// One tab's text and where it is scrolled to.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub tab: Tab,
    /// Items it lists (figures, references), shown next to the tab's name.
    pub count: Option<usize>,
    pub blocks: Vec<Block>,
    pub laid: Laid,
    /// First line shown.
    pub top: usize,
}

impl Page {
    fn new(tab: Tab, count: Option<usize>, blocks: Vec<Block>, settings: &Settings) -> Self {
        let laid = layout::lay_out(&blocks, settings);
        Self { tab, count, blocks, laid, top: 0 }
    }

    /// The block at the top of the screen.
    pub fn current_block(&self) -> usize {
        self.laid.lines.iter().skip(self.top).find_map(|l| l.block).unwrap_or(0)
    }

    fn line_of(&self, block: usize) -> usize {
        self.laid.lines.iter().position(|l| l.block.is_some_and(|b| b >= block)).unwrap_or(0)
    }
}

/// Where the paper's text is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Text {
    /// No PDF: only the abstract.
    Missing,
    Loading,
    Loaded,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reader {
    pub id: ReferenceId,
    pub cite_key: String,
    pub title: String,
    pub text: Text,
    /// Where the text came from (`arXiv HTML`, `PDF`), once loaded.
    pub source: Option<&'static str>,
    /// Abstract first, then the paper and the tabs it has.
    pub pages: Vec<Page>,
    /// Index into `pages`.
    pub tab: usize,
    /// The tab `f` came from, for `Esc` to return to.
    pub back: Option<usize>,
    pub settings: Settings,
    /// Lines that fit on screen (set when drawing).
    pub height: usize,
    pub overlay: Overlay,
    /// The last search and the lines it matched (on the current tab).
    pub query: String,
    pub matches: Vec<usize>,
    pub current_match: usize,
    /// Block of the paper to scroll to once the text is laid out (a remembered position).
    pub resume: Option<usize>,
}

/// What the user did in the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderAction {
    Scroll(isize),
    /// Half pages (positive: down).
    Page(isize),
    Top,
    Bottom,
    /// Next or previous tab.
    Tab(isize),
    /// Follow the next figure, table or equation mentioned (from the top of the screen).
    Follow,
    Wider,
    Narrower,
    Justify,
    Spacing,
    Theme,
    Contents,
    Search,
    Type(char),
    Backspace,
    Submit,
    NextMatch,
    PreviousMatch,
    Escape,
}

/// What the reader asks of its caller after an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderEffect {
    None,
    /// The settings changed: save them.
    SaveSettings,
    /// Leave the reader (saving the position).
    Close,
}

/// Columns moved by `+` / `-`.
const WIDTH_STEP: u16 = 4;

impl Reader {
    /// A reader on the abstract tab (`front`); `has_text`: a PDF exists, its text is loading.
    pub fn new(
        id: ReferenceId,
        cite_key: String,
        title: String,
        front: Vec<Block>,
        has_text: bool,
        settings: Settings,
        resume: Option<usize>,
    ) -> Self {
        let mut pages = vec![Page::new(Tab::Abstract, None, front, &settings)];
        if has_text {
            pages.push(Page::new(Tab::Paper, None, Vec::new(), &settings));
        }
        Self {
            id,
            cite_key,
            title,
            text: if has_text { Text::Loading } else { Text::Missing },
            source: None,
            pages,
            tab: 0,
            back: None,
            settings,
            height: 20,
            overlay: Overlay::None,
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
            resume,
        }
    }

    /// The paper's text arrived; `math`: it has LaTeX math (from arXiv or Europe PMC, not a PDF).
    pub fn loaded(&mut self, markdown: &str, math: bool) {
        let paper = paper::organize(document::parse(markdown, math));
        let extra = paper.extra_tabs();
        let mut body = Page::new(Tab::Paper, None, paper.body, &self.settings);
        if let Some(block) = self.resume.take() {
            body.top = body.line_of(block);
        }
        self.pages.truncate(1);
        self.pages.push(body);
        for (tab, count, blocks) in extra {
            self.pages.push(Page::new(tab, Some(count), blocks, &self.settings));
        }
        self.tab = self.tab.min(self.pages.len() - 1);
        self.text = Text::Loaded;
    }

    pub fn failed(&mut self, error: String) {
        self.text = Text::Failed(error);
    }

    /// No text anywhere: the abstract alone.
    pub fn no_text(&mut self) {
        self.text = Text::Missing;
        self.pages.truncate(1);
        self.tab = 0;
    }

    pub fn page(&self) -> &Page {
        &self.pages[self.tab]
    }

    fn page_mut(&mut self) -> &mut Page {
        &mut self.pages[self.tab]
    }

    /// The block of the paper's text where the reader is (for remembering the position).
    pub fn paper_block(&self) -> Option<usize> {
        let page = self.pages.iter().find(|p| p.tab == Tab::Paper)?;
        (self.text == Text::Loaded).then(|| page.current_block())
    }

    fn last_top(&self) -> usize {
        self.page().laid.lines.len().saturating_sub(self.height.max(1))
    }

    fn scroll_to(&mut self, line: usize) {
        let last = self.last_top();
        self.page_mut().top = line.min(last);
    }

    fn go_to_tab(&mut self, tab: usize) {
        self.tab = tab;
        self.find();
    }

    /// Lays the text out again for new settings, staying at the same block on every tab.
    fn relayout(&mut self) {
        let settings = self.settings;
        let height = self.height.max(1);
        for page in &mut self.pages {
            let block = page.current_block();
            page.laid = layout::lay_out(&page.blocks, &settings);
            page.top = page.line_of(block).min(page.laid.lines.len().saturating_sub(height));
        }
        self.find();
    }

    /// Lines matching the query (case-insensitive) on the current tab.
    fn find(&mut self) {
        let query = self.query.to_lowercase();
        let page = self.page();
        let matches: Vec<usize> = if query.is_empty() {
            Vec::new()
        } else {
            let lines = page.laid.lines.iter().enumerate();
            lines.filter(|(_, l)| l.text().to_lowercase().contains(&query)).map(|(i, _)| i).collect()
        };
        let top = page.top;
        self.current_match = matches.iter().position(|m| *m >= top).unwrap_or(0);
        self.matches = matches;
    }

    /// The first figure, table or equation mentioned from the top of the screen on that has its
    /// tab: (tab, line).
    fn mentioned(&self) -> Option<(usize, usize)> {
        let page = self.page();
        let blocks = page.current_block()..page.blocks.len();
        blocks.into_iter().find_map(|b| {
            let words = &page.blocks.get(b)?.words;
            words.windows(2).filter(|w| w[0].emphasis.mention).find_map(|w| {
                let (tab, number) = paper::mention_target(&w[0].text, &w[1].text)?;
                let index = self.pages.iter().position(|p| p.tab == tab)?;
                let headings = &self.pages[index].laid.headings;
                let line = headings
                    .iter()
                    .find(|h| h.2.split(' ').next_back().map(paper::mention_number) == Some(number.clone()))?
                    .0;
                Some((index, line))
            })
        })
    }

    pub fn apply(&mut self, action: ReaderAction) -> ReaderEffect {
        match (&mut self.overlay, action) {
            (Overlay::Contents(selected), action) => {
                let count = self.pages[self.tab].laid.headings.len();
                match action {
                    ReaderAction::Scroll(by) => {
                        *selected = (*selected as isize + by).clamp(0, count.saturating_sub(1) as isize) as usize
                    }
                    ReaderAction::Submit => {
                        let line = self.pages[self.tab].laid.headings.get(*selected).map(|h| h.0).unwrap_or(0);
                        self.overlay = Overlay::None;
                        self.scroll_to(line);
                    }
                    ReaderAction::Escape | ReaderAction::Contents => self.overlay = Overlay::None,
                    _ => {}
                }
                return ReaderEffect::None;
            }
            (Overlay::Search(text), action) => {
                match action {
                    ReaderAction::Type(c) => text.push(c),
                    ReaderAction::Backspace => {
                        text.pop();
                    }
                    ReaderAction::Submit => {
                        self.query = std::mem::take(text);
                        self.overlay = Overlay::None;
                        self.find();
                        if let Some(line) = self.matches.get(self.current_match).copied() {
                            self.scroll_to(line.saturating_sub(2));
                        }
                    }
                    ReaderAction::Escape => self.overlay = Overlay::None,
                    _ => {}
                }
                return ReaderEffect::None;
            }
            (Overlay::None, _) => {}
        }
        let page = (self.height / 2).max(1) as isize;
        let top = self.page().top as isize;
        match action {
            ReaderAction::Scroll(by) => self.scroll_to((top + by).max(0) as usize),
            ReaderAction::Page(halves) => self.scroll_to((top + halves * page).max(0) as usize),
            ReaderAction::Top => self.page_mut().top = 0,
            ReaderAction::Bottom => self.page_mut().top = self.last_top(),
            ReaderAction::Tab(by) => {
                let tab = (self.tab as isize + by).clamp(0, self.pages.len() as isize - 1) as usize;
                self.back = None;
                self.go_to_tab(tab);
            }
            // Enter on the abstract opens the paper.
            ReaderAction::Submit if self.page().tab == Tab::Abstract && self.pages.len() > 1 => self.go_to_tab(1),
            ReaderAction::Follow => {
                if let Some((tab, line)) = self.mentioned() {
                    self.back = Some(self.tab);
                    self.go_to_tab(tab);
                    self.scroll_to(line);
                }
            }
            ReaderAction::Wider | ReaderAction::Narrower => {
                let width = if action == ReaderAction::Wider {
                    self.settings.width + WIDTH_STEP
                } else {
                    self.settings.width.saturating_sub(WIDTH_STEP)
                };
                self.settings.width = width.clamp(MIN_WIDTH, MAX_WIDTH);
                self.relayout();
                return ReaderEffect::SaveSettings;
            }
            ReaderAction::Justify => {
                self.settings.justify = !self.settings.justify;
                self.relayout();
                return ReaderEffect::SaveSettings;
            }
            ReaderAction::Spacing => {
                self.settings.spacing = if self.settings.spacing == 1 { 2 } else { 1 };
                self.relayout();
                return ReaderEffect::SaveSettings;
            }
            ReaderAction::Theme => {
                self.settings.theme = self.settings.theme.next();
                return ReaderEffect::SaveSettings;
            }
            ReaderAction::Contents if !self.page().laid.headings.is_empty() => {
                let top = self.page().top;
                let current = self.page().laid.headings.iter().rposition(|h| h.0 <= top).unwrap_or(0);
                self.overlay = Overlay::Contents(current);
            }
            ReaderAction::Search => self.overlay = Overlay::Search(String::new()),
            ReaderAction::NextMatch | ReaderAction::PreviousMatch if !self.matches.is_empty() => {
                let count = self.matches.len();
                self.current_match = if action == ReaderAction::NextMatch {
                    (self.current_match + 1) % count
                } else {
                    (self.current_match + count - 1) % count
                };
                self.scroll_to(self.matches[self.current_match].saturating_sub(2));
            }
            ReaderAction::Escape if !self.query.is_empty() => {
                self.query.clear();
                self.matches.clear();
            }
            ReaderAction::Escape if self.back.is_some() => {
                let back = self.back.take().unwrap_or(0);
                self.go_to_tab(back);
            }
            ReaderAction::Escape => return ReaderEffect::Close,
            _ => {}
        }
        ReaderEffect::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::document::{Kind, words};

    fn front() -> Vec<Block> {
        vec![Block { kind: Kind::Heading(1), words: words("Title") }]
    }

    fn open(has_text: bool, settings: Settings, resume: Option<usize>) -> Reader {
        Reader::new(ReferenceId::new(), "key".into(), "Title".into(), front(), has_text, settings, resume)
    }

    /// A reader on the paper tab of `markdown`, 5 lines high, 40 columns.
    fn reader(markdown: &str) -> Reader {
        let mut reader = open(true, Settings { width: 40, ..Settings::default() }, None);
        reader.height = 5;
        reader.loaded(markdown, false);
        reader.apply(ReaderAction::Submit);
        reader
    }

    fn paper() -> String {
        let paragraph = "Words of a paragraph that wraps over several lines when it is laid out narrow. ".repeat(3);
        format!(
            "## Intro\n\n{paragraph}\n\n## Methods\n\n{paragraph} adaptive subtraction\n\n## Results\n\n{paragraph}"
        )
    }

    #[test]
    fn opens_on_the_abstract_then_the_paper() {
        let mut r = open(true, Settings::default(), None);
        assert_eq!((r.page().tab, r.text.clone()), (Tab::Abstract, Text::Loading));
        r.loaded(
            "## Intro\n\nText with Figure 1 in it.\n\nFigure 1: A device.\n\n## References\n\n[1] A. 2020.",
            false,
        );
        assert_eq!(r.page().tab, Tab::Abstract, "still on the abstract");
        let tabs: Vec<Tab> = r.pages.iter().map(|p| p.tab).collect();
        assert_eq!(tabs, [Tab::Abstract, Tab::Paper, Tab::Figures, Tab::References]);
        r.apply(ReaderAction::Submit);
        assert_eq!(r.page().tab, Tab::Paper);
        r.apply(ReaderAction::Follow);
        assert_eq!(r.page().tab, Tab::Figures, "f follows Figure 1");
        assert_eq!(r.apply(ReaderAction::Escape), ReaderEffect::None, "Esc goes back to the text");
        assert_eq!(r.page().tab, Tab::Paper);
        r.apply(ReaderAction::Tab(1));
        r.apply(ReaderAction::Tab(1));
        assert_eq!(r.page().tab, Tab::References);
        assert_eq!(r.apply(ReaderAction::Escape), ReaderEffect::Close);
        let none = open(false, Settings::default(), None);
        assert_eq!((none.pages.len(), none.text), (1, Text::Missing));
    }

    #[test]
    fn scroll_contents_and_search() {
        let mut r = reader(&paper());
        r.apply(ReaderAction::Page(1));
        assert_eq!(r.page().top, 2);
        r.apply(ReaderAction::Contents);
        assert_eq!(r.overlay, Overlay::Contents(0));
        r.apply(ReaderAction::Scroll(1));
        r.apply(ReaderAction::Submit);
        assert_eq!(r.page().top, r.page().laid.headings[1].0, "jumped to Methods");
        r.apply(ReaderAction::Top);
        r.apply(ReaderAction::Search);
        for c in "adaptive".chars() {
            r.apply(ReaderAction::Type(c));
        }
        r.apply(ReaderAction::Submit);
        assert_eq!(r.matches.len(), 1);
        let top = r.page().top;
        assert!(top <= r.matches[0] && r.matches[0] < top + 5);
        assert_eq!(r.apply(ReaderAction::Escape), ReaderEffect::None, "first Esc clears the search");
        assert_eq!(r.apply(ReaderAction::Escape), ReaderEffect::Close);
    }

    #[test]
    fn settings_keep_the_position_and_ask_to_save() {
        let mut r = reader(&paper());
        r.apply(ReaderAction::Contents);
        r.apply(ReaderAction::Scroll(2));
        r.apply(ReaderAction::Submit);
        let block = r.page().current_block();
        assert_eq!(r.apply(ReaderAction::Wider), ReaderEffect::SaveSettings);
        assert_eq!((r.settings.width, r.page().current_block()), (44, block));
        r.apply(ReaderAction::Spacing);
        assert_eq!(r.page().current_block(), block);
        for _ in 0..30 {
            r.apply(ReaderAction::Narrower);
        }
        assert_eq!(r.settings.width, MIN_WIDTH);
    }

    #[test]
    fn resumes_at_a_remembered_block() {
        let mut r = open(true, Settings::default(), Some(4));
        r.height = 3;
        r.loaded(&paper(), false);
        assert_eq!(r.paper_block(), Some(4));
    }
}
