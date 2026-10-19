//! Everything the TUI shows and remembers: the data loaded from the engine and the interface's
//! own state (focus, selection, filter, sort, prompts). Changed only by [`crate::update`].

use quire_engine::{Issue, Note, PdfStatus, ProjectSummary, Reference, Severity, SlugPath, Tag, TagKind};
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
}

impl Row {
    /// The worst issue, if any.
    pub fn worst(&self) -> Option<Severity> {
        self.issues.iter().map(Issue::severity).max()
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
}

impl Sort {
    pub fn next(self) -> Self {
        match self {
            Self::Key => Self::Year,
            Self::Year => Self::Author,
            Self::Author => Self::Title,
            Self::Title => Self::Added,
            Self::Added => Self::Key,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Year => "year",
            Self::Author => "author",
            Self::Title => "title",
            Self::Added => "added",
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
    pub jobs: usize,
    pub quit: bool,
    /// The open reader, on top of the library.
    pub reader: Option<Reader>,
    /// Reader layout (from the config; changed in the reader).
    pub reader_settings: Settings,
    /// Where each paper was left (block index), by reference.
    pub positions: HashMap<ReferenceId, usize>,
}

impl State {
    pub fn new(library: String) -> Self {
        Self {
            library,
            projects: Vec::new(),
            colored: Vec::new(),
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
            jobs: 0,
            quit: false,
            reader: None,
            reader_settings: Settings::default(),
            positions: HashMap::new(),
        }
    }

    /// Entries of the left pane: all, the projects, the trash.
    pub fn scope_count(&self) -> usize {
        self.projects.len() + 2
    }

    /// The scope at `index` of the left pane.
    pub fn scope_at(&self, index: usize) -> Scope {
        match index {
            0 => Scope::All,
            i if i <= self.projects.len() => Scope::Project(self.projects[i - 1].path.clone()),
            _ => Scope::Trash,
        }
    }

    /// The selected row, if the table is not empty.
    pub fn current(&self) -> Option<&Row> {
        self.visible.get(self.selected).map(|i| &self.rows[*i])
    }

    /// Re-applies filter and sort, keeping the selected reference selected when it is still shown.
    pub fn refresh_visible(&mut self) {
        let keep = self.current().map(|r| r.reference.id);
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
