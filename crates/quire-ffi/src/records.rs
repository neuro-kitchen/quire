//! The records and enums crossing to Kotlin. Strings rather than model types: IDs, cite keys and
//! project paths as text, kinds and roles by their CSL names (`article-journal`, `author`).

use quire_engine::{ReadingStatus, Reference, TagKind};
use quire_model::{Identifier, TextField};

#[derive(Debug, Clone, uniffi::Record)]
pub struct LibraryInfo {
    /// `None` for a folder not registered in the config.
    pub name: Option<String>,
    pub path: String,
    pub config_file: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Creator {
    /// CSL role: `author`, `editor`, …
    pub role: String,
    /// Family name (with particle), or the whole name of an organisation.
    pub sort_name: String,
    /// "Family, Given".
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ReferenceRecord {
    pub id: String,
    pub cite_key: String,
    /// CSL type: `article-journal`, `book`, `paper-conference`, …
    pub kind: String,
    pub title: String,
    pub creators: Vec<Creator>,
    pub year: Option<u16>,
    /// The issued date as far as known: `1952`, `1952-08`, `1952-08-28`.
    pub date: Option<String>,
    /// Journal, proceedings or book it appeared in.
    pub container_title: Option<String>,
    pub publisher: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    /// The bare DOI, `10.1113/…`.
    pub doi: Option<String>,
    /// Every identifier, labelled: `doi:…`, `arXiv:…`, `pmid:…`.
    pub identifiers: Vec<String>,
    pub url: Option<String>,
    pub abstract_text: Option<String>,
    /// Tag names (manual and automatic).
    pub tags: Vec<String>,
    /// Paths of the projects it is in (`thesis/chapter-2`).
    pub projects: Vec<String>,
    pub has_pdf: bool,
    /// `unread`, `reading` or `read`.
    pub reading: String,
    /// `YYYY-MM-DD`.
    pub date_added: String,
}

impl ReferenceRecord {
    pub(crate) fn new(
        reference: &Reference,
        tags: Vec<String>,
        projects: Vec<String>,
        has_pdf: bool,
        reading: ReadingStatus,
    ) -> Self {
        let data = &reference.data;
        let text = |field: TextField| data.get(field).map(str::to_string);
        Self {
            id: reference.id.to_string(),
            cite_key: reference.cite_key.to_string(),
            kind: data.kind.as_str().to_string(),
            title: data.title().unwrap_or_default().to_string(),
            creators: data
                .creators
                .iter()
                .map(|c| Creator {
                    role: c.role.as_str().to_string(),
                    sort_name: c.sort_name(),
                    display_name: c.display_name(),
                })
                .collect(),
            year: data.issued.map(|d| d.get_year()),
            date: data.issued.map(|d| d.to_string()),
            container_title: text(TextField::ContainerTitle),
            publisher: text(TextField::Publisher),
            volume: text(TextField::Volume),
            issue: text(TextField::Issue),
            pages: text(TextField::Page),
            doi: data.identifiers.iter().find_map(|id| match id {
                Identifier::Doi(doi) => Some(doi.to_string()),
                _ => None,
            }),
            identifiers: data.identifiers.iter().map(ToString::to_string).collect(),
            url: text(TextField::Url),
            abstract_text: text(TextField::Abstract),
            tags,
            projects,
            has_pdf,
            reading: reading.as_str().to_string(),
            date_added: reference.meta.created_at.date().to_string(),
        }
    }
}

/// Tag names of a reference, manual and automatic alike, by name.
pub(crate) fn tag_names(tags: &[(quire_engine::Tag, TagKind)]) -> Vec<String> {
    let mut names: Vec<String> = tags.iter().map(|(tag, _)| tag.name.clone()).collect();
    names.sort_by_key(|n| n.to_lowercase());
    names
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ProjectRecord {
    /// `thesis/chapter-2`; identifies the project.
    pub path: String,
    pub name: String,
    pub parent: Option<String>,
    /// References in this project itself (not its subprojects).
    pub references: u64,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct TagRecord {
    pub name: String,
    /// `#rrggbb` for colored tags.
    pub color: Option<String>,
    pub count: u64,
}

/// Which references a [`Query`] starts from.
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum Source {
    All,
    /// In no project.
    Unfiled,
    /// In this project itself (`thesis/chapter-2`), not its subprojects.
    Project {
        path: String,
    },
    Trash,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum SortColumn {
    Title,
    Creator,
    Year,
    DateAdded,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Query {
    pub source: Source,
    /// Full-text search words (the engine's search syntax); empty: no search.
    pub text: String,
    /// References must carry every one of these tags.
    pub tags: Vec<String>,
    pub sort: SortColumn,
    pub ascending: bool,
}
