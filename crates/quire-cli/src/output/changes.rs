//! Field changes of a refetch or edit, one per line:
//!
//! ```text
//!   title       Old title → New title
//!   issued      2014 (yours, kept; source has 2013)
//! ```

use quire_engine::{FieldChange, Outcome, Updated};

use super::{style, truncate};

/// Width of the field-name column.
const FIELD_WIDTH: usize = 16;
/// Longest value shown; longer ones (abstracts) are cut.
const VALUE_WIDTH: usize = 70;

/// The lines for `updated`'s changes, indented.
pub fn lines(updated: &Updated) -> Vec<String> {
    let mut out = Vec::new();
    if let Some((old, new)) = &updated.cite_key {
        out.push(format!(
            "    {}{} → {}",
            label("cite key"),
            style::paint(style::DIM, old),
            style::paint(style::KEY, new)
        ));
    }
    out.extend(updated.changes.iter().map(line));
    out
}

fn label(name: &str) -> String {
    style::paint(style::LABEL, format!("{name:<FIELD_WIDTH$}"))
}

fn line(change: &FieldChange) -> String {
    let show =
        |value: &Option<String>| value.as_deref().map(|v| truncate(v, VALUE_WIDTH)).unwrap_or_else(|| "(none)".into());
    let (old, new) = (show(&change.old), show(&change.new));
    match change.outcome {
        Outcome::Updated => format!("    {}{} → {}", label(change.field.as_str()), style::paint(style::DIM, old), new),
        Outcome::KeptUserValue => {
            format!(
                "    {}{} {}",
                label(change.field.as_str()),
                old,
                style::paint(style::WARN, format!("(yours, kept; source has {new})"))
            )
        }
    }
}
