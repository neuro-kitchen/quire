//! References → `.bib` text. Each reference becomes one `biblatex::Entry`; `biblatex` escapes
//! special characters and, for BibTeX, renames fields and splits `date` into `year`/`month`.

use std::collections::HashSet;

use biblatex::{Chunk, Entry, EntryType, Spanned};
use quire_model::{EntryKind, Identifier, Reference};

use super::fields::{to_biblatex, to_biblatex_value};
use super::import::EXTRA_PREFIX;
use super::names::{NAME_FIELDS, to_chunks};
use super::types;
use crate::error::{IoError, Result};

/// Which dialect to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BibFormat {
    /// BibLaTeX (Biber): `journaltitle`, `date`, `location`, … The default.
    #[default]
    Biblatex,
    /// Classic BibTeX: `journal`, `year`/`month`, `address`, …
    Bibtex,
}

/// How to write a `.bib` file (an export profile's settings, `.knowledge/roadmap.md` M6).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BibOptions {
    pub format: BibFormat,
    /// BibLaTeX field names left out, e.g. `abstract`, `file`.
    pub omit: Vec<String>,
    /// Non-ASCII characters as LaTeX commands (`é` → `{\'e}`), for classic BibTeX.
    pub ascii: bool,
    /// BibTeX months as macros (`month = jul`) instead of numbers.
    pub month_macros: bool,
    /// How preprints (CSL `article`) are written.
    pub preprints: PreprintForm,
}

/// The entry type of a preprint (an arXiv paper not yet in a journal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreprintForm {
    /// `@online` with `eprint`: what Biber's styles expect. The default.
    #[default]
    Online,
    /// `@article` with `journaltitle = {arXiv}`: what Typst renders as a preprint (it reads
    /// `@online` as a web page, without year or eprint; tested 2026-10-19, Typst 0.15.1).
    Article,
}

/// The `.bib` text for `references`, in the given order. Fails if two share a cite key.
pub fn to_bib(references: &[Reference], format: BibFormat) -> Result<String> {
    to_bib_with(references, &BibOptions { format, ..BibOptions::default() })
}

/// [`to_bib`] with an export profile's settings.
pub fn to_bib_with(references: &[Reference], options: &BibOptions) -> Result<String> {
    let mut out = String::new();
    for entry in entries(references, &options.omit, options.preprints)? {
        let text = match options.format {
            BibFormat::Biblatex => entry.to_biblatex_string(),
            BibFormat::Bibtex => entry
                .to_bibtex_string()
                .map_err(|e| IoError::BibtexWrite { key: entry.key.clone(), message: e.to_string() })?,
        };
        let mut text = indent_fields(&unescape_math(&text));
        if options.ascii {
            text = super::ascii::to_latex(&text);
        }
        if options.month_macros && options.format == BibFormat::Bibtex {
            text = month_macros(&text);
        }
        out.push_str(&text);
        out.push_str("\n\n");
    }
    Ok(out)
}

/// The entries for `references` without the `omit` fields, preprints in `preprints` form; fails if
/// two share a cite key.
pub fn entries(references: &[Reference], omit: &[String], preprints: PreprintForm) -> Result<Vec<Entry>> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(references.len());
    for reference in references {
        let key = reference.cite_key.as_str();
        if !seen.insert(key) {
            return Err(IoError::DuplicateCiteKey(key.into()));
        }
        let mut entry = to_entry(reference);
        if preprints == PreprintForm::Article && reference.data.kind == EntryKind::Article {
            preprint_as_article(&mut entry);
        }
        for field in omit {
            entry.remove(field);
        }
        out.push(entry);
    }
    Ok(out)
}

/// `@online{…, publisher = {arXiv}}` → `@article{…, journaltitle = {arXiv}}`.
fn preprint_as_article(entry: &mut Entry) {
    entry.entry_type = EntryType::Article;
    if entry.get("journaltitle").is_none() {
        let venue = entry.remove("publisher").or_else(|| entry.remove("organization"));
        let venue =
            venue.unwrap_or_else(|| text_chunks(if entry.get("eprint").is_some() { "arXiv" } else { "Preprint" }));
        entry.set("journaltitle", venue);
    }
}

/// BibTeX's month macros, the form styles expect for sorting and abbreviating.
const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];

/// `month = {07},` → `month = jul,`.
fn month_macros(entry: &str) -> String {
    let lines = entry.lines().map(|line| {
        let Some(value) = line.trim_start().strip_prefix("month = {").and_then(|rest| rest.strip_suffix("},")) else {
            return line.to_string();
        };
        match value.parse::<usize>().ok().and_then(|m| MONTHS.get(m.wrapping_sub(1))) {
            Some(name) => format!("  month = {name},"),
            None => line.to_string(),
        }
    });
    lines.collect::<Vec<_>>().join("\n")
}

/// Characters `biblatex` escapes with a backslash (`biblatex::resolve::is_escapable`).
const ESCAPED: &[char] = &['{', '}', '\\', '~', '^', '#', '&', '%', '$', '_'];

/// Removes the backslashes `biblatex` 0.12 adds inside `$…$` math (it escapes math chunks like
/// text, writing `$T\^2$` for `$T^2$`, which LaTeX reads as an accent). Outside math nothing
/// changes; an escaped `\$` is text, not a math delimiter.
fn unescape_math(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_math = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek().is_some_and(|next| ESCAPED.contains(next)) => {
                let next = chars.next().expect("peeked");
                if !in_math {
                    out.push('\\');
                }
                out.push(next);
            }
            '$' => {
                in_math = !in_math;
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// `biblatex` writes fields flush left; indent the lines between `@type{key,` and `}`.
fn indent_fields(entry: &str) -> String {
    let lines: Vec<&str> = entry.lines().collect();
    let last = lines.len().saturating_sub(1);
    let indented = lines
        .iter()
        .enumerate()
        .map(|(i, line)| if i == 0 || i == last { line.to_string() } else { format!("  {line}") });
    indented.collect::<Vec<_>>().join("\n")
}

/// The BibLaTeX entry for `reference`.
pub fn to_entry(reference: &Reference) -> Entry {
    let data = &reference.data;
    let mut entry = Entry::new(reference.cite_key.to_string(), EntryType::new(types::to_biblatex(data.kind)));

    for (name, role) in NAME_FIELDS {
        if let Some(chunks) = to_chunks(&data.creators, *role) {
            entry.set(name, chunks);
        }
    }
    if let Some(issued) = data.issued {
        set_text(&mut entry, "date", &issued.to_string());
    }
    for (field, value) in &data.text {
        if let Some(name) = to_biblatex(data.kind, *field, value) {
            set_text(&mut entry, name, &to_biblatex_value(*field, value));
        }
    }
    write_identifiers(&mut entry, &data.identifiers);
    // Fields kept from an imported `.bib` that quire does not model go back out unchanged.
    for (key, value) in &data.extra {
        if let Some(name) = key.strip_prefix(EXTRA_PREFIX) {
            set_text(&mut entry, name, value);
        }
    }
    entry
}

/// `doi`, `eprint` + `eprinttype` (arXiv), `pmid`, `pmcid`. An arXiv paper also gets its DOI
/// unless it has another one (a published version).
fn write_identifiers(entry: &mut Entry, identifiers: &[Identifier]) {
    let has_doi = identifiers.iter().any(|id| matches!(id, Identifier::Doi(_)));
    for id in identifiers {
        match id {
            Identifier::Doi(doi) => set_text(entry, "doi", doi.as_str()),
            Identifier::Pmid(pmid) => set_text(entry, "pmid", &pmid.to_string()),
            Identifier::Pmcid(pmcid) => set_text(entry, "pmcid", &pmcid.to_string()),
            Identifier::Arxiv(arxiv) => {
                set_text(entry, "eprint", arxiv.id());
                set_text(entry, "eprinttype", "arxiv");
                if !has_doi {
                    set_text(entry, "doi", arxiv.doi().as_str());
                }
            }
        }
    }
}

fn set_text(entry: &mut Entry, name: &str, value: &str) {
    entry.set(name, text_chunks(value));
}

/// `value` as chunks, with `$…$` spans as math so they stay LaTeX math (arXiv titles and
/// abstracts use it) instead of being escaped to `\$T\^2\$`, which LaTeX reads as an accent.
/// An unpaired `$` makes the whole value plain text.
fn text_chunks(value: &str) -> Vec<Spanned<Chunk>> {
    let parts: Vec<&str> = value.split('$').collect();
    if parts.len() < 3 || parts.len().is_multiple_of(2) {
        return vec![Spanned::detached(Chunk::Normal(value.into()))];
    }
    let chunk = |(i, part): (usize, &&str)| match i % 2 {
        0 => Chunk::Normal(part.to_string()),
        _ => Chunk::Math(part.to_string()),
    };
    parts.iter().enumerate().filter(|(_, part)| !part.is_empty()).map(chunk).map(Spanned::detached).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_model::{CiteKey, Creator, CreatorRole, Date, DeviceId, EntryKind, Origin, ReferenceData, TextField};

    fn reference(key: &str) -> Reference {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.creators.push(Creator::person(CreatorRole::Author, "Kucsko", Some("G.".into())));
        data.issued = Some(Date::new(2013, Some(7), Some(31)).unwrap());
        data.set(TextField::Title, "Nanometre-scale thermometry in a living cell");
        data.set(TextField::ContainerTitle, "Nature");
        data.set(TextField::Page, "54-58");
        data.set(TextField::Issue, "7460");
        data.identifiers.push(Identifier::parse("10.1038/nature12373").unwrap());
        Reference::new(CiteKey::parse(key).unwrap(), data, Origin::User, DeviceId::new())
    }

    #[test]
    fn biblatex_entry() {
        let text = to_bib(&[reference("kucsko2013")], BibFormat::Biblatex).unwrap();
        insta::assert_snapshot!(text);
    }

    #[test]
    fn bibtex_entry() {
        let text = to_bib(&[reference("kucsko2013")], BibFormat::Bibtex).unwrap();
        insta::assert_snapshot!(text);
    }

    #[test]
    fn profile_options() {
        let mut r = reference("kucsko2013");
        r.data.set(TextField::Title, "Thermométrie");
        r.data.set(TextField::Abstract, "Long text");
        let options = BibOptions {
            format: BibFormat::Bibtex,
            omit: vec!["abstract".into()],
            ascii: true,
            month_macros: true,
            ..BibOptions::default()
        };
        let text = to_bib_with(&[r], &options).unwrap();
        assert!(text.contains("month = jul,") && !text.contains("abstract"), "{text}");
        assert!(text.contains("title = {Thermom{\\'e}trie}"), "{text}");
    }

    #[test]
    fn duplicate_keys_are_an_error() {
        let refs = [reference("same"), reference("same")];
        assert!(matches!(to_bib(&refs, BibFormat::Biblatex), Err(IoError::DuplicateCiteKey(k)) if k == "same"));
    }

    #[test]
    fn math_stays_math() {
        let mut r = reference("m2013");
        r.data.set(TextField::Title, "Spectra on $T^2$ and $S^2$");
        r.data.set(TextField::Abstract, "Costs $5 and more");
        let text = to_bib(&[r], BibFormat::Biblatex).unwrap();
        assert!(text.contains("title = {Spectra on $T^2$ and $S^2$}"), "{text}");
        assert!(text.contains(r"abstract = {Costs \$5 and more}"), "{text}");
    }

    #[test]
    fn unescapes_only_inside_math() {
        assert_eq!(unescape_math(r"$T\^2$ and $x\_1 + \\frac\{a\}\{b\}$"), r"$T^2$ and $x_1 + \frac{a}{b}$");
        assert_eq!(unescape_math(r"50\% of R\&D, \$5, $a\_b$"), r"50\% of R\&D, \$5, $a_b$");
    }

    #[test]
    fn special_characters_are_escaped() {
        let mut r = reference("x2013");
        r.data.set(TextField::Title, "50% of R&D in C#");
        let text = to_bib(&[r], BibFormat::Biblatex).unwrap();
        assert!(text.contains(r"title = {50\% of R\&D in C\#}"), "{text}");
    }
}
