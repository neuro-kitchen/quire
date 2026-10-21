//! One reference in full, for `show`: a heading (key, type, ID), then one labelled field per line.

use std::path::PathBuf;

use quire_model::{Attachment, Reference, TextField};

use super::style::{self, paint};

/// Width of the label column.
const LABEL_WIDTH: usize = 12;

/// Text fields shown after title, creators and date, in this order.
const FIELDS: &[TextField] = &[
    TextField::ContainerTitle,
    TextField::Volume,
    TextField::Issue,
    TextField::Page,
    TextField::Publisher,
    TextField::PublisherPlace,
    TextField::Edition,
    TextField::Isbn,
    TextField::Issn,
    TextField::Url,
    TextField::Language,
    TextField::Note,
];

/// What `show` adds to a reference's own fields.
#[derive(Debug, Default)]
pub struct Context {
    pub projects: Vec<String>,
    pub pdfs: Vec<(Attachment, PathBuf)>,
    /// Tag names, already colored.
    pub tags: Vec<String>,
    /// Note titles with their IDs.
    pub notes: Vec<(String, String)>,
    /// Related references' cite keys.
    pub related: Vec<String>,
}

/// The block for `reference` with its projects, PDFs, tags, notes and related references.
pub fn block(reference: &Reference, context: &Context) -> String {
    let (projects, pdfs) = (&context.projects, &context.pdfs);
    let data = &reference.data;
    let heading = format!(
        "{}  {}  {}",
        paint(style::KEY.bold(), &reference.cite_key),
        paint(style::DIM, data.kind),
        paint(style::DIM, reference.id)
    );
    let mut lines = vec![heading];
    let mut add = |label: &str, value: String| {
        lines.push(format!("{}{value}", paint(style::LABEL, format!("{label:<LABEL_WIDTH$}"))));
    };
    if let Some(title) = data.title() {
        add("title", title.into());
    }
    for creator in &data.creators {
        add(creator.role.as_str(), creator.display_name());
    }
    if let Some(issued) = data.issued {
        add("issued", issued.to_string());
    }
    for field in FIELDS {
        if let Some(value) = data.get(*field) {
            add(field.as_str(), value.into());
        }
    }
    for id in &data.identifiers {
        add(id.kind().as_str(), id.to_string());
    }
    let projects = if projects.is_empty() {
        paint(style::DIM, "(none)")
    } else {
        projects.iter().map(|p| paint(style::KEY, p)).collect::<Vec<_>>().join(", ")
    };
    add("projects", projects);
    if !context.tags.is_empty() {
        add("tags", context.tags.join(", "));
    }
    if !context.related.is_empty() {
        add("related", context.related.iter().map(|k| paint(style::KEY, k)).collect::<Vec<_>>().join(", "));
    }
    for (id, title) in &context.notes {
        add("note", format!("{title}  {}", paint(style::DIM, id)));
    }
    for (attachment, path) in pdfs {
        let pages = attachment.pages.map(|p| format!("  ({})", super::count(p, "page", "pages"))).unwrap_or_default();
        add("pdf", format!("{}{}", path.display(), paint(style::DIM, pages)));
    }
    if reference.meta.is_deleted() {
        add("status", paint(style::WARN, "in the trash"));
    }
    if let Some(abstract_text) = data.get(TextField::Abstract) {
        lines.push(String::new());
        lines.push(paint(style::DIM, abstract_text));
    }
    lines.join("\n")
}
