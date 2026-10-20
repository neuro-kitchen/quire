//! References as a table. On a terminal: a bordered table fitted to the window width, the widest
//! column (usually the title) cut first. Piped: tab-separated values, one reference per line, no
//! header, so `cut` and `awk` work (`--json` for anything richer).

use std::collections::HashMap;

use clap::ValueEnum;
use quire_engine::{Checked, PdfAttempt, PdfStatus, Severity, Tag, TagKind};
use quire_model::{Identifier, ReferenceId, TextField};
use tabled::builder::Builder;
use tabled::settings::object::{Columns, Rows};
use tabled::settings::{Color, Style, Width};

use super::{style, terminal};

/// A column of the reference table (`list --columns`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Column {
    /// Cite key
    Key,
    /// First creator's family name (+ "et al.")
    Author,
    Year,
    Title,
    /// Entry type (CSL)
    Type,
    /// Journal, book or proceedings title
    Container,
    Doi,
    /// Reference ID (ref_…)
    Id,
    /// PDF: ✓ pages, blocked, not found, no DOI, error, – (never tried; `quire pdf get`)
    Pdf,
    /// Tags (colored ones in their color)
    Tags,
    /// Marks: ! = metadata errors, ~ = warnings (see `ref check`), T = in the trash
    Flags,
}

/// Columns shown when none are chosen.
pub const DEFAULT_COLUMNS: &[Column] =
    &[Column::Key, Column::Year, Column::Author, Column::Title, Column::Type, Column::Pdf, Column::Flags];

impl Column {
    fn header(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Author => "author",
            Self::Year => "year",
            Self::Title => "title",
            Self::Type => "type",
            Self::Container => "in",
            Self::Doi => "doi",
            Self::Id => "id",
            Self::Pdf => "pdf",
            Self::Tags => "tags",
            Self::Flags => "",
        }
    }

    fn value(self, checked: &Checked, extras: &Extras) -> String {
        let pdfs = &extras.pdfs;
        let reference = &checked.reference;
        let data = &reference.data;
        match self {
            Self::Key => reference.cite_key.to_string(),
            Self::Author => match data.creators.as_slice() {
                [] => String::new(),
                [one] => one.sort_name(),
                [first, _] => format!("{} & {}", first.sort_name(), data.creators[1].sort_name()),
                [first, ..] => format!("{} et al.", first.sort_name()),
            },
            Self::Year => data.issued.map(|d| d.get_year().to_string()).unwrap_or_default(),
            Self::Title => data.title().unwrap_or_default().to_string(),
            Self::Type => data.kind.to_string(),
            Self::Container => data.get(TextField::ContainerTitle).unwrap_or_default().to_string(),
            Self::Doi => data
                .identifiers
                .iter()
                .find_map(|id| match id {
                    Identifier::Doi(doi) => Some(doi.to_string()),
                    _ => None,
                })
                .unwrap_or_default(),
            Self::Id => reference.id.to_string(),
            Self::Flags => {
                let worst = checked.issues.iter().map(|i| i.severity()).max();
                let issue = match worst {
                    Some(Severity::Error) => "!",
                    Some(Severity::Warning) => "~",
                    None => "",
                };
                format!("{issue}{}", if reference.meta.is_deleted() { "T" } else { "" })
            }
            Self::Tags => extras
                .tags
                .get(&reference.id)
                .map(|tags| tags.iter().map(|(tag, _)| tag.name.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default(),
            Self::Pdf => match pdfs.get(&reference.id) {
                Some(PdfStatus::Available { pages: Some(pages) }) => format!("✓ {pages} p"),
                Some(PdfStatus::Available { pages: None }) => "✓".into(),
                Some(PdfStatus::Missing(PdfAttempt::Blocked)) => "blocked".into(),
                Some(PdfStatus::Missing(PdfAttempt::NotFound)) => "not found".into(),
                Some(PdfStatus::Missing(PdfAttempt::NoIdentifier)) => "no DOI".into(),
                Some(PdfStatus::Missing(PdfAttempt::Error)) => "error".into(),
                None => "–".into(),
            },
        }
    }
}

impl Column {
    /// How much a column keeps when the table is too wide: 0 = never cut (keys, IDs, years);
    /// higher keeps more (the title keeps twice what author, container and type keep).
    fn weight(self) -> usize {
        match self {
            Self::Title => 2,
            Self::Container | Self::Author | Self::Type | Self::Tags => 1,
            _ => 0,
        }
    }
}

/// What the table shows besides the references themselves.
#[derive(Debug, Default)]
pub struct Extras {
    pub pdfs: HashMap<ReferenceId, PdfStatus>,
    pub tags: HashMap<ReferenceId, Vec<(Tag, TagKind)>>,
}

/// Narrowest a shrinking column is cut to.
const MIN_WIDTH: usize = 8;

/// The table text for `references` (without a trailing newline). `Flags` is left out when no
/// reference has a flag.
pub fn references(references: &[Checked], columns: &[Column], extras: &Extras) -> String {
    let columns: Vec<Column> = columns
        .iter()
        .copied()
        .filter(|c| *c != Column::Flags || references.iter().any(|r| !c.value(r, extras).is_empty()))
        .collect();
    let rows: Vec<Vec<String>> =
        references.iter().map(|r| columns.iter().map(|c| c.value(r, extras)).collect()).collect();
    if !terminal::stdout_is_terminal() {
        let line = |row: &Vec<String>| row.iter().map(|v| v.replace(['\t', '\n'], " ")).collect::<Vec<_>>().join("\t");
        return rows.iter().map(line).collect::<Vec<_>>().join("\n");
    }
    let mut builder = Builder::new();
    builder.push_record(columns.iter().map(|c| c.header()));
    let natural: Vec<usize> = (0..columns.len())
        .map(|i| rows.iter().map(|row| row[i].chars().count()).chain([columns[i].header().len()]).max().unwrap_or(0))
        .collect();
    for row in rows {
        builder.push_record(row.into_iter().zip(&columns).map(|(value, column)| styled(*column, value)));
    }
    let mut table = builder.build();
    table.with(Style::rounded());
    let widths = fit(&natural, &columns, terminal::width());
    for (index, column) in columns.iter().enumerate() {
        if widths[index] < natural[index] {
            table.modify(Columns::one(index), Width::truncate(widths[index]).suffix("…"));
        }
        match column {
            Column::Key => table.modify(Columns::one(index), Color::FG_CYAN),
            Column::Year | Column::Id | Column::Type => {
                table.modify(Columns::one(index), Color::new("\u{1b}[2m", "\u{1b}[22m"))
            }
            Column::Flags => table.modify(Columns::one(index), Color::FG_YELLOW),
            _ => &mut table,
        };
    }
    table.modify(Rows::first(), Color::BOLD);
    table.to_string()
}

/// A cell's text with its colour, for cells whose colour depends on the value (the PDF status).
fn styled(column: Column, value: String) -> String {
    if column != Column::Pdf {
        return value;
    }
    let colour = match value.as_str() {
        v if v.starts_with('✓') => style::OK,
        "blocked" => style::WARN,
        "error" => style::ERROR,
        _ => style::DIM,
    };
    style::paint(colour, value)
}

/// Column widths that fit `total` terminal columns. Columns with a [`Column::weight`] are cut one
/// character at a time, always the one with the most width per weight, down to [`MIN_WIDTH`];
/// the others keep their width (the table may then overflow).
fn fit(natural: &[usize], columns: &[Column], total: usize) -> Vec<usize> {
    // Each column takes its width + 2 spaces of padding + 1 border; plus the closing border.
    let frame = natural.len() * 3 + 1;
    let mut widths = natural.to_vec();
    let mut over = (widths.iter().sum::<usize>() + frame).saturating_sub(total);
    while over > 0 {
        let candidates = (0..widths.len()).filter(|i| columns[*i].weight() > 0 && widths[*i] > MIN_WIDTH);
        // Most width per weight; ties go to the lighter column.
        let Some(index) =
            candidates.max_by_key(|i| (widths[*i] * 2 / columns[*i].weight(), usize::MAX - columns[*i].weight()))
        else {
            break;
        };
        widths[index] -= 1;
        over -= 1;
    }
    widths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_cuts_by_weight_and_keeps_keys() {
        let columns = [Column::Key, Column::Author, Column::Title];
        // 30 + 20 + 60 + frame 10 = 120; fit into 100 → title loses 20.
        assert_eq!(fit(&[30, 20, 60], &columns, 100), [30, 20, 40]);
        // Into 70: the title keeps twice the author's width, key untouched.
        assert_eq!(fit(&[30, 20, 60], &columns, 70), [30, 10, 20]);
        // Too narrow even at the minimum: key stays whole.
        assert_eq!(fit(&[30, 20, 60], &columns, 20), [30, MIN_WIDTH, MIN_WIDTH]);
    }
}
