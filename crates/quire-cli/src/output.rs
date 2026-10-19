//! How references are printed: one line per reference in lists, a block for `show`.

use quire_model::{Reference, TextField};

/// Longest title shown in one-line summaries, in characters.
const TITLE_WIDTH: usize = 70;

/// `cite-key  year  first author  title`, e.g.
/// `vaswaniAttentionAllYou2017  2017  Vaswani  Attention Is All You Need`.
pub fn line(reference: &Reference) -> String {
    let data = &reference.data;
    let year = data.issued.map(|d| d.get_year().to_string()).unwrap_or_else(|| "----".into());
    let author = data.creators.first().map(|c| c.sort_name()).unwrap_or_else(|| "–".into());
    let title = truncate(data.title().unwrap_or("(no title)"), TITLE_WIDTH);
    let trashed = if reference.meta.is_deleted() { "  [trash]" } else { "" };
    format!("{}  {year}  {author}  {title}{trashed}", reference.cite_key)
}

/// Width of the label column in `show`.
const LABEL_WIDTH: usize = 16;

/// Field per line, for `show`.
pub fn block(reference: &Reference, projects: &[String]) -> String {
    let data = &reference.data;
    let mut lines = Vec::new();
    let mut add = |label: &str, value: String| lines.push(format!("{label:<LABEL_WIDTH$}{value}"));
    add("cite key", reference.cite_key.to_string());
    add("id", reference.id.to_string());
    add("type", data.kind.to_string());
    if let Some(title) = data.title() {
        add("title", title.into());
    }
    let creators: Vec<String> = data.creators.iter().map(|c| format!("{} ({})", c.sort_name(), c.role)).collect();
    if !creators.is_empty() {
        add("creators", creators.join("; "));
    }
    if let Some(issued) = data.issued {
        add("issued", issued.to_string());
    }
    for field in [TextField::ContainerTitle, TextField::Volume, TextField::Issue, TextField::Page, TextField::Publisher, TextField::Url] {
        if let Some(value) = data.get(field) {
            add(field.as_str(), value.into());
        }
    }
    for id in &data.identifiers {
        add("identifier", id.to_string());
    }
    add("projects", if projects.is_empty() { "(none)".into() } else { projects.join(", ") });
    if reference.meta.is_deleted() {
        add("status", "in the trash".into());
    }
    lines.join("\n")
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.into();
    }
    let cut: String = text.chars().take(width - 1).collect();
    format!("{}…", cut.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_titles() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("a long title here", 8), "a long…");
    }
}
