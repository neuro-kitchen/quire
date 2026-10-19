//! `.bib` text → cite keys + [`ReferenceData`]. Fields quire does not model are kept in `extra`
//! as `biblatex.<field>` and written back on export, so a `.bib` round trip loses nothing.

use biblatex::{Bibliography, ChunksExt, DateValue, Entry, Person, PermissiveType};
use quire_model::{Date, Doi, EntryKind, Identifier, Pmcid, Pmid, ReferenceData};

use super::fields::{from_biblatex, READ_ORDER};
use super::names::{from_person, NAME_FIELDS};
use super::types::from_biblatex as kind_from_biblatex;
use crate::error::{IoError, Result};

/// Prefix of `extra` keys holding unmapped BibLaTeX fields.
pub const EXTRA_PREFIX: &str = "biblatex.";

/// Fields read into structured parts (dates, identifiers), not text fields or `extra`.
const STRUCTURED: &[&str] = &["date", "year", "month", "day", "doi", "eprint", "eprinttype", "archiveprefix", "pmid", "pmcid"];

/// One entry of a `.bib` file: its cite key as written (it may not be a valid quire key) and
/// its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibRecord {
    pub cite_key: String,
    pub data: ReferenceData,
}

/// Parses `.bib` text (BibTeX or BibLaTeX; `crossref`/`xdata` inheritance is resolved).
pub fn from_bib(text: &str) -> Result<Vec<BibRecord>> {
    let bibliography = Bibliography::parse(text).map_err(|e| IoError::BibParse(e.to_string()))?;
    Ok(bibliography.iter().map(record).collect())
}

fn record(entry: &Entry) -> BibRecord {
    let bib_type = entry.entry_type.to_string().to_ascii_lowercase();
    let mut data = ReferenceData::new(kind_from_biblatex(&bib_type));

    for (name, role) in NAME_FIELDS {
        if let Ok(people) = entry.get_as::<Vec<Person>>(name) {
            data.creators.extend(people.iter().map(|p| from_person(*role, p)));
        }
    }
    data.issued = issued(entry);
    data.identifiers = identifiers(entry);
    // quire writes preprints (CSL `article`) as `@online` with an arXiv `eprint`; read them back
    // as preprints, not web pages.
    if bib_type == "online" && data.identifiers.iter().any(|id| matches!(id, Identifier::Arxiv(_))) {
        data.kind = EntryKind::Article;
    }

    let names = NAME_FIELDS.iter().map(|(n, _)| *n);
    let skip: Vec<&str> = names.chain(STRUCTURED.iter().copied()).collect();
    let ordered = READ_ORDER.iter().copied().filter(|f| entry.fields.contains_key(*f));
    let rest = entry.fields.keys().map(String::as_str).filter(|f| !READ_ORDER.contains(f));
    for field in ordered.chain(rest).filter(|f| !skip.contains(f)) {
        let value = entry.fields[field].format_verbatim();
        if !from_biblatex(&bib_type, field, &value, &mut data) {
            data.extra.insert(format!("{EXTRA_PREFIX}{field}"), value);
        }
    }
    BibRecord { cite_key: entry.key.clone(), data }
}

/// `date`, or `year`/`month`/`day` (via `biblatex`, which numbers months and days from 0). For a
/// range, the start.
fn issued(entry: &Entry) -> Option<Date> {
    let PermissiveType::Typed(date) = entry.date().ok()? else { return None };
    let at = match date.value {
        DateValue::At(at) | DateValue::After(at) | DateValue::Before(at) | DateValue::Between(at, _) => at,
    };
    let year = u16::try_from(at.year).ok()?;
    Date::new(year, at.month.map(|m| m + 1), at.day.map(|d| d + 1)).ok()
}

/// `doi`, `eprint` with `eprinttype`/`archiveprefix` `arxiv` or `pubmed`, `pmid`, `pmcid`.
fn identifiers(entry: &Entry) -> Vec<Identifier> {
    let get = |name: &str| entry.get(name).map(|c| c.format_verbatim()).filter(|v| !v.trim().is_empty());
    let mut ids = Vec::new();
    if let Some(doi) = get("doi").and_then(|v| Doi::parse(&v).ok()) {
        ids.push(doi.arxiv_id().map_or(Identifier::Doi(doi), Identifier::Arxiv));
    }
    let eprint_type = get("eprinttype").or_else(|| get("archiveprefix")).map(|t| t.to_ascii_lowercase());
    if let Some(eprint) = get("eprint") {
        let parsed = match eprint_type.as_deref() {
            Some("arxiv") => Identifier::parse(&format!("arXiv:{eprint}")).ok(),
            Some("pubmed") => Pmid::parse(&eprint).ok().map(Identifier::Pmid),
            _ => None,
        };
        ids.extend(parsed.filter(|id| !ids.iter().any(|known| known.key() == id.key())));
    }
    ids.extend(get("pmid").and_then(|v| Pmid::parse(&v).ok()).map(Identifier::Pmid));
    ids.extend(get("pmcid").and_then(|v| Pmcid::parse(&v).ok()).map(Identifier::Pmcid));
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_model::{CiteKey, DeviceId, EntryKind, IdentifierKind, Origin, Reference, TextField};

    use crate::bib::{to_bib, BibFormat};

    const BIB: &str = r#"
@article{kucsko2013,
  author = {Kucsko, G. and Maurer, P. C. and {World Health Organization}},
  title = {Nanometre-scale thermometry in a living cell},
  journal = {Nature},
  year = {2013},
  month = jul,
  volume = {500},
  number = {7460},
  pages = {54--58},
  doi = {10.1038/nature12373},
  keywords = {thermometry, NV centres},
}
@misc{vaswani2017,
  author = {Vaswani, Ashish},
  title = {Attention Is All You Need},
  date = {2017-06-12},
  eprint = {1706.03762},
  eprinttype = {arXiv},
}
"#;

    #[test]
    fn reads_bibtex_and_biblatex_entries() {
        let records = from_bib(BIB).unwrap();
        assert_eq!(records.len(), 2);
        let nature = &records[0].data;
        assert_eq!((records[0].cite_key.as_str(), nature.kind), ("kucsko2013", EntryKind::ArticleJournal));
        assert_eq!(nature.creators.len(), 3);
        assert_eq!(nature.creators[2].sort_name(), "World Health Organization");
        assert_eq!(nature.issued.unwrap().to_string(), "2013-07");
        assert_eq!(nature.get(TextField::ContainerTitle), Some("Nature"));
        assert_eq!(nature.get(TextField::Issue), Some("7460"));
        assert_eq!(nature.get(TextField::Page), Some("54-58"));
        assert_eq!(nature.extra.get("biblatex.keywords").map(String::as_str), Some("thermometry, NV centres"));

        let arxiv = &records[1].data;
        assert_eq!(arxiv.issued.unwrap().to_string(), "2017-06-12");
        assert_eq!(arxiv.identifiers[0].kind(), IdentifierKind::Arxiv);
    }

    #[test]
    fn export_then_import_keeps_the_content() {
        for record in from_bib(BIB).unwrap() {
            let key = CiteKey::parse(&record.cite_key).unwrap();
            let reference = Reference::new(key, record.data.clone(), Origin::User, DeviceId::new());
            let text = to_bib(&[reference], BibFormat::Biblatex).unwrap();
            let back = from_bib(&text).unwrap().remove(0);
            assert_eq!(back.data.creators, record.data.creators, "{text}");
            assert_eq!(back.data.text, record.data.text, "{text}");
            assert_eq!(back.data.issued, record.data.issued, "{text}");
            assert_eq!(back.data.extra, record.data.extra, "{text}");
            let keys = |d: &ReferenceData| d.identifiers.iter().map(Identifier::key).collect::<Vec<_>>();
            assert_eq!(keys(&back.data), keys(&record.data), "{text}");
        }
    }

    #[test]
    fn online_with_arxiv_eprint_is_a_preprint() {
        let bib = "@online{v, title = {T}, eprint = {1706.03762}, eprinttype = {arxiv}}\n@online{w, title = {Blog}, url = {https://x.org}}";
        let records = from_bib(bib).unwrap();
        assert_eq!((records[0].data.kind, records[1].data.kind), (EntryKind::Article, EntryKind::Webpage));
    }

    #[test]
    fn math_round_trips() {
        let records = from_bib("@article{m, title = {Spectra on $T^2$}}").unwrap();
        assert_eq!(records[0].data.title(), Some("Spectra on $T^2$"));
    }

    #[test]
    fn invalid_bib_is_an_error() {
        assert!(matches!(from_bib("@article{x, title = {unclosed"), Err(IoError::BibParse(_))));
    }
}
