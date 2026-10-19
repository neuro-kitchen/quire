//! Full-text search: what the user types → an FTS5 query, and the references it finds.
//!
//! Words must all appear (in any field), as word prefixes (`therm` finds "thermometry");
//! `"two words"` is a phrase; `field:word` searches one field: `title:`, `author:`, `abstract:`,
//! `venue:` / `journal:`, `tag:`, `note:`, `key:`, `doi:`/`id:`. Everything else is quoted, so
//! typed punctuation cannot break the query. Without a field, papers' text is searched too.

use quire_model::Reference;

use crate::error::Result;
use crate::ports::{Library, SearchHit};

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

/// The references `input` finds, best first, with their hits.
pub fn search(library: &impl Library, input: &str, limit: usize) -> Result<Vec<(Reference, SearchHit)>> {
    let Some((query, fielded)) = fts_query(input) else { return Ok(Vec::new()) };
    let hits = library.search(&query, !fielded, limit)?;
    let mut found = Vec::with_capacity(hits.len());
    for hit in hits {
        if let Some(reference) = library.reference(hit.reference)? {
            found.push((reference, hit));
        }
    }
    Ok(found)
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
    }
}
