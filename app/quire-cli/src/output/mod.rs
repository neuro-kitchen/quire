//! How the CLI prints: styles and status lines shared by every command, and the views of
//! references and projects.
//!
//! - [`style`]: colours, and clap's help styles.
//! - [`status`]: `added  …` / `failed  …` lines.
//! - [`table`]: references as a table (or tab-separated when piped).
//! - [`reference`]: one reference in full.
//! - [`changes`]: field changes of a refetch or edit.
//! - [`tree`]: projects as a tree.
//! - [`terminal`]: terminal detection and width.

pub mod changes;
pub mod reference;
pub mod status;
pub mod style;
pub mod table;
pub mod terminal;
pub mod tree;

pub use status::{Tone, hint, note, report};

use quire_model::Reference;

/// `key  year  author  title`, for status lines.
pub fn summary(reference: &Reference) -> String {
    let data = &reference.data;
    let year = data.issued.map(|d| d.get_year().to_string()).unwrap_or_else(|| "----".into());
    let author = data.creators.first().map(|c| c.sort_name()).unwrap_or_else(|| "–".into());
    let title = truncate(data.title().unwrap_or("(no title)"), 60);
    format!("{}  {}  {author}  {title}", style::paint(style::KEY, &reference.cite_key), style::paint(style::DIM, year))
}

/// `text` cut to `width` characters, ending in `…` when cut.
pub fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.into();
    }
    let cut: String = text.chars().take(width - 1).collect();
    format!("{}…", cut.trim_end())
}

/// `n` with its noun: `1 reference`, `2 references`, `0 references`.
pub fn count(n: impl TryInto<u64>, one: &str, many: &str) -> String {
    let n: u64 = n.try_into().unwrap_or(u64::MAX);
    format!("{n} {}", if n == 1 { one } else { many })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Messages said "1 entries", "trashed thesis and 1 subprojects", "1 pages".
    #[test]
    fn counts_agree_with_their_noun() {
        assert_eq!(count(1usize, "entry", "entries"), "1 entry");
        assert_eq!(count(2u32, "entry", "entries"), "2 entries");
        assert_eq!(count(0u64, "page", "pages"), "0 pages");
    }
    #[test]
    fn truncates_long_titles() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("a long title here", 8), "a long…");
    }
}
