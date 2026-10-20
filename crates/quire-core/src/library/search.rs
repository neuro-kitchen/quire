//! Full-text search: what the user types → an FTS5 query, and the references it finds.
//!
//! Words must all appear (in any field), as word prefixes (`therm` finds "thermometry");
//! `"two words"` is a phrase; `field:word` searches one field: `title:`, `author:`, `abstract:`,
//! `venue:` / `journal:`, `tag:`, `note:`, `key:`, `doi:`/`id:`. Everything else is quoted, so
//! typed punctuation cannot break the query. Without a field, papers' text is searched too.
//!
//! Filters narrow the result (and work alone, without words): `status:unread|reading|read`,
//! `year:2019` or `year:2019..2021` (either end may be left open), `pdf:yes|no`,
//! `type:article-journal` (a CSL type), `in:thesis/chapter-2` (a project and its subprojects).

use std::collections::HashSet;

use quire_model::{EntryKind, ReadingStatus, Reference, ReferenceId, SlugPath};

use crate::error::{CoreError, Result};
use crate::library::projects;
use crate::ports::{Library, ReferenceFilter, SearchHit};

/// Filter names (not index columns).
const FILTERS: &[&str] = &["status", "year", "pdf", "type", "in"];

/// Text-search hits considered before filters narrow them.
const FILTERED_POOL: usize = 100_000;

/// A filter term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    Status(ReadingStatus),
    /// Inclusive; either end may be open.
    Year(Option<u16>, Option<u16>),
    Pdf(bool),
    Kind(EntryKind),
    In(SlugPath),
}

/// What the user typed, split: the FTS5 query (and whether it names a field), and the filters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    pub text: Option<(String, bool)>,
    pub filters: Vec<Filter>,
}

impl Query {
    pub fn parse(input: &str) -> Result<Self> {
        let mut filters = Vec::new();
        for (field, value, _) in tokens(input) {
            let Some(name) = field.filter(|f| FILTERS.contains(&f.as_str())) else { continue };
            filters.push(filter(&name, value.trim())?);
        }
        Ok(Self { text: fts_query(input), filters })
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.filters.is_empty()
    }
}

fn filter(name: &str, value: &str) -> Result<Filter> {
    let invalid = |why: &str| CoreError::InvalidFilter(format!("{name}:{value}: {why}"));
    Ok(match name {
        "status" => Filter::Status(value.parse().map_err(|_| invalid("unread, reading or read"))?),
        "pdf" => match value {
            "yes" | "true" | "1" => Filter::Pdf(true),
            "no" | "false" | "0" => Filter::Pdf(false),
            _ => return Err(invalid("yes or no")),
        },
        "type" => Filter::Kind(value.parse().map_err(|_| invalid("a CSL type, e.g. article-journal"))?),
        "in" => Filter::In(value.parse().map_err(|_| invalid("a project path, e.g. thesis/chapter-2"))?),
        "year" => {
            let year = |text: &str| -> Result<Option<u16>> {
                if text.is_empty() {
                    Ok(None)
                } else {
                    text.parse().map(Some).map_err(|_| invalid("a year or a range a..b"))
                }
            };
            match value.split_once("..") {
                Some((from, to)) => Filter::Year(year(from)?, year(to)?),
                None => {
                    let y = year(value)?.ok_or_else(|| invalid("a year"))?;
                    Filter::Year(Some(y), Some(y))
                }
            }
        }
        _ => unreachable!("only names in FILTERS"),
    })
}

/// Field names users type, and the index column each means.
const FIELDS: &[(&str, &str)] = &[
    ("title", "title"),
    ("author", "authors"),
    ("authors", "authors"),
    ("abstract", "abstract"),
    ("venue", "venue"),
    ("journal", "venue"),
    ("tag", "tags"),
    ("note", "notes"),
    ("notes", "notes"),
    ("key", "cite_key"),
    ("doi", "identifiers"),
    ("id", "identifiers"),
];

/// The FTS5 query for what the user typed, and whether it names a field (then papers' text,
/// which has no fields, is not searched). `None` when nothing searchable was typed.
pub fn fts_query(input: &str) -> Option<(String, bool)> {
    let mut terms = Vec::new();
    let mut fielded = false;
    for (field, text, phrase) in tokens(input) {
        if field.as_deref().is_some_and(|f| FILTERS.contains(&f)) {
            continue;
        }
        let column = field.as_deref().and_then(|f| FIELDS.iter().find(|(name, _)| *name == f)).map(|(_, c)| *c);
        // Inside the quotes FTS5 sees only words; a doubled quote is a literal one.
        let escaped = text.replace('"', "\"\"");
        if escaped.trim().is_empty() {
            continue;
        }
        let term = if phrase { format!("\"{escaped}\"") } else { format!("\"{escaped}\"*") };
        match column {
            Some(column) => {
                fielded = true;
                terms.push(format!("{column} : {term}"));
            }
            None => terms.push(term),
        }
    }
    (!terms.is_empty()).then(|| (terms.join(" AND "), fielded))
}

/// (field, text, is a phrase) for each word or `"quoted phrase"` of `input`.
fn tokens(input: &str) -> Vec<(Option<String>, String, bool)> {
    let mut out = Vec::new();
    let mut rest = input.trim();
    while !rest.is_empty() {
        let (field, after) = match rest.split_once(':') {
            Some((name, after)) if !name.contains(char::is_whitespace) && !name.contains('"') && !name.is_empty() => {
                (Some(name.to_lowercase()), after)
            }
            _ => (None, rest),
        };
        let (text, phrase, next) = if let Some(quoted) = after.strip_prefix('"') {
            let end = quoted.find('"').unwrap_or(quoted.len());
            (quoted[..end].to_string(), true, quoted.get(end + 1..).unwrap_or(""))
        } else {
            let end = after.find(char::is_whitespace).unwrap_or(after.len());
            (after[..end].to_string(), false, &after[end..])
        };
        // An unknown field (`10.1038/x:y`, `http:`) is part of the word.
        if let Some(name) = &field
            && !FIELDS.iter().any(|(f, _)| f == name)
            && !FILTERS.contains(&name.as_str())
        {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            out.push((None, rest[..end].to_string(), false));
            rest = rest[end..].trim_start();
            continue;
        }
        out.push((field, text, phrase));
        rest = next.trim_start();
    }
    out
}

/// The references `input` finds, best first, with their hits (filters alone: by cite key, with
/// empty snippets).
pub fn search(library: &impl Library, input: &str, limit: usize) -> Result<Vec<(Reference, SearchHit)>> {
    let query = Query::parse(input)?;
    let mut found = Vec::new();
    match &query.text {
        Some((text, fielded)) => {
            let pool = if query.filters.is_empty() { limit } else { FILTERED_POOL };
            for hit in library.search(text, !fielded, pool)? {
                if let Some(reference) = library.reference(hit.reference)? {
                    found.push((reference, hit));
                }
            }
        }
        None if !query.filters.is_empty() => {
            for reference in library.references(&ReferenceFilter::default())? {
                let hit = SearchHit { reference: reference.id, score: 0.0, snippet: String::new(), in_text: false };
                found.push((reference, hit));
            }
        }
        None => return Ok(Vec::new()),
    }
    let keep = Matcher::new(library, &query.filters)?;
    let mut kept = Vec::new();
    for (reference, hit) in found {
        if kept.len() == limit {
            break;
        }
        if keep.matches(library, &reference)? {
            kept.push((reference, hit));
        }
    }
    Ok(kept)
}

/// The filters, with what they need looked up once.
struct Matcher<'f> {
    filters: &'f [Filter],
    /// For `in:`: the projects (with subprojects) each filter names.
    projects: Vec<HashSet<quire_model::ProjectId>>,
    with_pdf: HashSet<ReferenceId>,
}

impl<'f> Matcher<'f> {
    fn new(library: &impl Library, filters: &'f [Filter]) -> Result<Self> {
        let mut projects = Vec::new();
        for filter in filters {
            if let Filter::In(path) = filter {
                let project = projects::resolve(library, path)?;
                projects.push(projects::with_descendants(library, project.id, true)?.into_iter().collect());
            }
        }
        let with_pdf = if filters.iter().any(|f| matches!(f, Filter::Pdf(_))) {
            library.references_with_attachments()?.into_iter().collect()
        } else {
            HashSet::new()
        };
        Ok(Self { filters, projects, with_pdf })
    }

    fn matches(&self, library: &impl Library, reference: &Reference) -> Result<bool> {
        let mut projects = self.projects.iter();
        for filter in self.filters {
            let ok = match filter {
                Filter::Status(status) => {
                    let current = library.reading(reference.id)?.map_or(ReadingStatus::Unread, |r| r.status);
                    current == *status
                }
                Filter::Year(from, to) => reference.data.issued.is_some_and(|d| {
                    let year = d.get_year();
                    from.is_none_or(|f| year >= f) && to.is_none_or(|t| year <= t)
                }),
                Filter::Pdf(wanted) => self.with_pdf.contains(&reference.id) == *wanted,
                Filter::Kind(kind) => reference.data.kind == *kind,
                Filter::In(_) => {
                    let members = projects.next().expect("one set per `in:`");
                    library.projects_of(reference.id)?.iter().any(|p| members.contains(p))
                }
            };
            if !ok {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{AddRequest, add};
    use crate::memory::MemoryLibrary;
    use quire_model::{Creator, CreatorRole, EntryKind, Origin, ReferenceData, TextField};

    #[test]
    fn queries() {
        assert_eq!(fts_query("therm cell"), Some(("\"therm\"* AND \"cell\"*".into(), false)));
        assert_eq!(
            fts_query("author:kucsko \"living cell\""),
            Some(("authors : \"kucsko\"* AND \"living cell\"".into(), true))
        );
        assert_eq!(fts_query("10.1038/nature12373"), Some(("\"10.1038/nature12373\"*".into(), false)));
        assert_eq!(fts_query("http://x.org/a"), Some(("\"http://x.org/a\"*".into(), false)), "unknown field: one word");
        assert_eq!(fts_query("say\"hi"), Some(("\"say\"\"hi\"*".into(), false)), "quotes escaped");
        assert_eq!(fts_query("   "), None);
        assert_eq!(fts_query("status:read cell"), Some(("\"cell\"*".into(), false)), "filters are not words");
    }

    #[test]
    fn filters() {
        let query = Query::parse("year:2019..2021 pdf:no status:reading type:book in:thesis/ch-2 year:..2000").unwrap();
        assert_eq!(query.text, None);
        assert_eq!(query.filters[0], Filter::Year(Some(2019), Some(2021)));
        assert_eq!(query.filters[1], Filter::Pdf(false));
        assert_eq!(query.filters[2], Filter::Status(ReadingStatus::Reading));
        assert_eq!(query.filters[3], Filter::Kind(EntryKind::Book));
        assert_eq!(query.filters[4], Filter::In("thesis/ch-2".parse().unwrap()));
        assert_eq!(query.filters[5], Filter::Year(None, Some(2000)));
        assert!(Query::parse("status:skimmed").is_err());
        assert!(Query::parse("year:soon").is_err());
    }

    #[test]
    fn finds_references() {
        let mut library = MemoryLibrary::default();
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "Nanometre-scale thermometry in a living cell");
        data.creators.push(Creator::person(CreatorRole::Author, "Kucsko", None));
        add(&mut library, AddRequest::new(data, Origin::User)).unwrap();
        assert_eq!(search(&library, "living cell", 10).unwrap().len(), 1);
        assert!(search(&library, "attention", 10).unwrap().is_empty());
        // Filters alone, and with words.
        assert_eq!(search(&library, "status:unread", 10).unwrap().len(), 1);
        assert!(search(&library, "living status:read", 10).unwrap().is_empty());
        assert!(search(&library, "year:2000", 10).unwrap().is_empty(), "no date: no year matches");
    }
}
