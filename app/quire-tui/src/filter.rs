//! The table's order: a filter as you type (`nucleo-matcher`, the matcher of the Helix editor)
//! over key, title, authors, year, DOI and tags; best matches first while filtering, otherwise
//! the chosen sort. Each typed word must appear as written (case and accents ignored): with
//! fuzzy subsequence matching, `vas` also matched long titles with a v, an a and an s somewhere.

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use quire_model::{Identifier, TextField};

use crate::state::{Row, Sort};

/// The key and title: a match here ranks above one in the authors or elsewhere.
fn headline(row: &Row) -> String {
    format!("{} {}", row.reference.cite_key, row.reference.data.title().unwrap_or_default())
}

/// The text a row is matched against.
fn haystack(row: &Row) -> String {
    let data = &row.reference.data;
    let authors: Vec<String> = data.creators.iter().map(|c| c.sort_name()).collect();
    let doi = data.identifiers.iter().find_map(|id| match id {
        Identifier::Doi(d) => Some(d.as_str().to_string()),
        _ => None,
    });
    let tags: Vec<&str> = row.tags.iter().map(|(t, _)| t.name.as_str()).collect();
    [
        row.reference.cite_key.to_string(),
        data.title().unwrap_or_default().to_string(),
        authors.join(" "),
        data.issued.map(|d| d.get_year().to_string()).unwrap_or_default(),
        data.get(TextField::ContainerTitle).unwrap_or_default().to_string(),
        doi.unwrap_or_default(),
        tags.join(" "),
    ]
    .join(" ")
}

/// Added to the score of rows whose key or title matches.
const HEADLINE_BONUS: u32 = 10_000;

/// Indices of `rows` to show, in order.
pub fn visible(rows: &[Row], filter: &str, sort: Sort, reverse: bool) -> Vec<usize> {
    if filter.trim().is_empty() {
        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|a, b| compare(&rows[*a], &rows[*b], sort));
        if reverse {
            order.reverse();
        }
        return order;
    }
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::new(filter, CaseMatching::Ignore, Normalization::Smart, AtomKind::Substring);
    let mut buffer = Vec::new();
    let mut scored: Vec<(u32, usize)> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let text = haystack(row);
            let score = pattern.score(Utf32Str::new(&text, &mut buffer), &mut matcher)?;
            let head = headline(row);
            let in_head = pattern.score(Utf32Str::new(&head, &mut buffer), &mut matcher).is_some();
            Some((score + if in_head { HEADLINE_BONUS } else { 0 }, i))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, i)| i).collect()
}

fn compare(a: &Row, b: &Row, sort: Sort) -> std::cmp::Ordering {
    let (x, y) = (&a.reference, &b.reference);
    let author =
        |r: &quire_engine::Reference| r.data.creators.first().map(|c| c.sort_name().to_lowercase()).unwrap_or_default();
    let title = |r: &quire_engine::Reference| r.data.title().unwrap_or_default().to_lowercase();
    match sort {
        Sort::Key => x.cite_key.cmp(&y.cite_key),
        // Newest first reads best for years and additions; `S` reverses.
        Sort::Year => y.data.issued.map(|d| d.get_year()).cmp(&x.data.issued.map(|d| d.get_year())),
        Sort::Author => author(x).cmp(&author(y)),
        Sort::Title => title(x).cmp(&title(y)),
        Sort::Added => y.meta.created_at.cmp(&x.meta.created_at),
        // Stable sort: the order the rows came in (the search's ranking).
        Sort::Relevance => return std::cmp::Ordering::Equal,
    }
    .then_with(|| x.cite_key.cmp(&y.cite_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::row;

    #[test]
    fn fuzzy_filter_ranks_and_sort_orders() {
        let rows = vec![
            row("vaswani2017", "Attention Is All You Need", "Vaswani", 2017),
            row("kucsko2013", "Nanometre thermometry", "Kucsko", 2013),
        ];
        assert_eq!(visible(&rows, "", Sort::Year, false), [0, 1], "newest first");
        assert_eq!(visible(&rows, "", Sort::Key, false), [1, 0]);
        assert_eq!(visible(&rows, "", Sort::Key, true), [0, 1]);
        assert_eq!(visible(&rows, "atten", Sort::Key, false), [0]);
        assert_eq!(visible(&rows, "kucsko 2013", Sort::Key, false), [1]);
        assert_eq!(visible(&rows, "vas", Sort::Key, false), [0], "words, not scattered letters");
        let mut by_author = row("bateman2020", "UniProt", "Bateman", 2020);
        by_author.reference.data.creators.push(quire_model::Creator::person(
            quire_model::CreatorRole::Author,
            "Vasilev",
            None,
        ));
        let rows = vec![by_author, row("vaswani2017", "Attention", "Vaswani", 2017)];
        assert_eq!(visible(&rows, "vas", Sort::Key, false), [1, 0], "key/title matches first");
        assert!(visible(&rows, "zzz", Sort::Key, false).is_empty());
    }
}
