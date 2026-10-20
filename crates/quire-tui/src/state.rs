//! Everything the TUI shows and remembers: the data loaded from the engine and the interface's
//! own state (focus, selection, filter, sort, prompts). Changed only by [`crate::update`].

use quire_engine::{
    Issue, IssueCode, Note, PdfStatus, ProjectSummary, ReadingStatus, Reference, SavedSearch, Severity, SlugPath, Tag,
    TagKind,
};
use std::collections::HashMap;

use quire_model::ReferenceId;

use crate::filter;
use crate::reader::{Reader, Settings};

/// One reference in the table, with what the list shows about it.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub reference: Reference,
    pub issues: Vec<Issue>,
    pub pdf: Option<PdfStatus>,
    pub tags: Vec<(Tag, TagKind)>,
    /// For a full-text search result: the matching words in context, and whether only the
    /// paper's text matched.
    pub hit: Option<(String, bool)>,
    /// Reading status (unread when `None`).
    pub reading: Option<ReadingStatus>,
}

impl Row {
    /// The worst issue, if any.
    pub fn worst(&self) -> Option<Severity> {
        self.issues.iter().map(Issue::severity).max()
    }

    /// The cite key of a reference this one looks like a duplicate of (the first found).
    pub fn duplicate_of(&self) -> Option<&str> {
        let issue = self.issues.iter().find(|i| i.code == IssueCode::PossibleDuplicate)?;
        issue.detail.as_deref()?.split_whitespace().next()
    }
}

/// The selected reference's extra information, loaded on selection.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Details {
    pub id: Option<ReferenceId>,
    pub projects: Vec<SlugPath>,
    pub notes: Vec<Note>,
    pub related: Vec<Reference>,
}

/// What the left pane lists: every reference, a project, or a special view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    All,
    Project(SlugPath),
    /// Full-text search results, best first.
    Search(String),
    /// A saved search, by name (run again when opened).
    Saved(String),
    /// References that look like the same work as another.
    Duplicates,
    Trash,
}

/// Which pane has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Scopes,
    References,
    Details,
}

/// What the table is sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Key,
    Year,
    Author,
    Title,
    Added,
    /// As the search ranked them (search results only).
    Relevance,
}

impl Sort {
    pub fn next(self) -> Self {
        match self {
            Self::Key => Self::Year,
            Self::Year => Self::Author,
            Self::Author => Self::Title,
            Self::Title => Self::Added,
            Self::Added | Self::Relevance => Self::Key,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Year => "year",
            Self::Author => "author",
            Self::Title => "title",
            Self::Added => "added",
            Self::Relevance => "relevance",
        }
    }
}

/// A one-line text prompt at the bottom (add, filter) and what it is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prompt {
    /// Fuzzy filter, applied as you type.
    Filter,
    /// DOI, arXiv ID, PMID, PMCID or URL to add.
    Add,
    /// A tag name to toggle on the selected reference.
    Tag,
    /// `y` to move the selected reference to the trash.
    ConfirmTrash,
    /// `y` to merge the selected reference's duplicate into it, `d` to mark them distinct.
    ConfirmMerge,
    /// A project path to link the selected reference to.
    Link,
    /// A file to export the open scope to (format from the extension).
    Export,
    /// Words to search the whole library for (full text).
    Search,
    /// A name to keep the open search under.
    SaveSearch,
}

/// Engine work running in the background (a lookup, a download, a merge), for the jobs panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: u64,
    pub label: String,
    pub started: std::time::Instant,
}

/// A message in the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub text: String,
    pub error: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub library: String,
    /// Projects, depth first.
    pub projects: Vec<ProjectSummary>,
    /// Colored tags, by position (their number keys).
    pub colored: Vec<Tag>,
    /// How many pairs of references look like duplicates (shown in the left pane).
    pub duplicate_pairs: usize,
    /// Saved searches, by name (after the projects in the left pane).
    pub saved: Vec<SavedSearch>,
    pub scope: Scope,
    /// Index into the left pane: 0 = all, 1..=projects = projects, last = trash.
    pub scope_index: usize,
    pub rows: Vec<Row>,
    /// Indices into `rows` in display order (filtered and sorted).
    pub visible: Vec<usize>,
    /// Index into `visible`.
    pub selected: usize,
    pub details: Details,
    pub focus: Pane,
    pub sort: Sort,
    pub reverse: bool,
    pub filter: String,
    pub prompt: Option<Prompt>,
    pub input: String,
    pub help: bool,
    pub message: Option<Message>,
    /// Background jobs running (lookups, downloads).
    pub jobs: Vec<Job>,
    /// The ID the next job gets.
    pub next_job: u64,
    /// The jobs panel is open (`J`).
    pub show_jobs: bool,
    pub quit: bool,
    /// The open reader, on top of the library.
    pub reader: Option<Reader>,
    /// Reader layout (from the config; changed in the reader).
    pub reader_settings: Settings,
    /// Where each paper was left (block index), by reference.
    pub positions: HashMap<ReferenceId, usize>,
    /// Whether the terminal shows images (kitty, sixel, iTerm2), once asked (`None`: not yet).
    /// Figures are downloaded for the reader only when it does.
    pub graphics: Option<bool>,
}

impl State {
    pub fn new(library: String) -> Self {
        Self {
            library,
            projects: Vec::new(),
            colored: Vec::new(),
            duplicate_pairs: 0,
            saved: Vec::new(),
            scope: Scope::All,
            scope_index: 0,
            rows: Vec::new(),
            visible: Vec::new(),
            selected: 0,
            details: Details::default(),
            focus: Pane::References,
            sort: Sort::Key,
            reverse: false,
            filter: String::new(),
            prompt: None,
            input: String::new(),
            help: false,
            message: None,
            jobs: Vec::new(),
            next_job: 0,
            show_jobs: false,
            quit: false,
            reader: None,
            reader_settings: Settings::default(),
            positions: HashMap::new(),
            graphics: None,
        }
    }

    /// Entries of the left pane: all, the projects, the saved searches, duplicates, the trash.
    pub fn scope_count(&self) -> usize {
        self.projects.len() + self.saved.len() + 3
    }

    /// The scope at `index` of the left pane.
    pub fn scope_at(&self, index: usize) -> Scope {
        let (projects, saved) = (self.projects.len(), self.saved.len());
        match index {
            0 => Scope::All,
            i if i <= projects => Scope::Project(self.projects[i - 1].path.clone()),
            i if i <= projects + saved => Scope::Saved(self.saved[i - projects - 1].name.clone()),
            i if i == projects + saved + 1 => Scope::Duplicates,
            _ => Scope::Trash,
        }
    }

    /// The selected row, if the table is not empty.
    pub fn current(&self) -> Option<&Row> {
        // `get`: `visible` may still index the previous rows while new ones are being shown.
        self.visible.get(self.selected).and_then(|i| self.rows.get(*i))
    }

    /// Shows new rows, keeping the selected reference selected if it is among them (taken
    /// before the rows change: the old positions do not point into the new rows).
    pub fn set_rows(&mut self, rows: Vec<Row>) {
        let keep = self.current().map(|r| r.reference.id);
        self.rows = rows;
        self.show(keep);
    }

    /// Re-applies filter and sort, keeping the selected reference selected when it is still shown.
    pub fn refresh_visible(&mut self) {
        let keep = self.current().map(|r| r.reference.id);
        self.show(keep);
    }

    fn show(&mut self, keep: Option<ReferenceId>) {
        self.visible = filter::visible(&self.rows, &self.filter, self.sort, self.reverse);
        self.selected =
            keep.and_then(|id| self.visible.iter().position(|i| self.rows[*i].reference.id == id)).unwrap_or(0);
        self.clamp();
    }

    /// Keeps `selected` inside the table.
    pub fn clamp(&mut self) {
        self.selected = self.selected.min(self.visible.len().saturating_sub(1));
    }

    pub fn tell(&mut self, text: impl Into<String>) {
        self.message = Some(Message { text: text.into(), error: false });
    }

    pub fn warn(&mut self, text: impl Into<String>) {
        self.message = Some(Message { text: text.into(), error: true });
    }
}
