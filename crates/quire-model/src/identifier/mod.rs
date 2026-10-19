//! External identifiers of a work (DOI, PMID, PMCID, arXiv ID): parsing from what users paste
//! (bare IDs, `doi:` labels, URLs), and a normalised [`key`](Identifier::key) so the same work is
//! recognised however it was written. See `.knowledge/data-foundations.md` (4).

mod arxiv;
mod doi;
mod pubmed;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub use arxiv::ArxivId;
pub use doi::Doi;
pub use pubmed::{Pmcid, Pmid};

use crate::error::{ModelError, Result};

/// An external identifier of a work.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum Identifier {
    Doi(Doi),
    Pmid(Pmid),
    Pmcid(Pmcid),
    Arxiv(ArxivId),
}

/// The kind of an [`Identifier`]; [`as_str`](Self::as_str) is the name stored in the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IdentifierKind {
    Doi,
    Pmid,
    Pmcid,
    Arxiv,
}

impl IdentifierKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Doi => "doi",
            Self::Pmid => "pmid",
            Self::Pmcid => "pmcid",
            Self::Arxiv => "arxiv",
        }
    }
}

impl Identifier {
    /// Recognises any supported identifier in `input`: bare (`10.1038/nature12373`, `2101.00001`,
    /// `PMC7614242`, `31452104`), labelled (`doi:…`, `arXiv:…`, `pmid:…`) or as a URL (doi.org,
    /// arxiv.org, pubmed.ncbi.nlm.nih.gov, …). An arXiv DOI (`10.48550/arXiv.…`) becomes an arXiv ID,
    /// so both forms give the same key. Bare digits are read as a PMID.
    pub fn parse(input: &str) -> Result<Self> {
        let text = input.trim();
        if let Some(id) = ArxivId::detect(text) {
            return id.map(Self::Arxiv);
        }
        if let Some(doi) = Doi::detect(text) {
            let doi = doi?;
            return Ok(match doi.arxiv_id() {
                Some(arxiv) => Self::Arxiv(arxiv),
                None => Self::Doi(doi),
            });
        }
        if let Some(id) = Pmcid::detect(text) {
            return id.map(Self::Pmcid);
        }
        if let Some(id) = Pmid::detect(text) {
            return id.map(Self::Pmid);
        }
        Err(ModelError::UnknownIdentifier(input.into()))
    }

    pub fn kind(&self) -> IdentifierKind {
        match self {
            Self::Doi(_) => IdentifierKind::Doi,
            Self::Pmid(_) => IdentifierKind::Pmid,
            Self::Pmcid(_) => IdentifierKind::Pmcid,
            Self::Arxiv(_) => IdentifierKind::Arxiv,
        }
    }

    /// Normalised form: equal keys (with equal kinds) mean the same work.
    pub fn key(&self) -> String {
        match self {
            Self::Doi(doi) => doi.key(),
            Self::Pmid(id) => id.to_string(),
            Self::Pmcid(id) => id.to_string(),
            Self::Arxiv(id) => id.key(),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Doi(doi) => write!(f, "doi:{doi}"),
            Self::Pmid(id) => write!(f, "pmid:{id}"),
            Self::Pmcid(id) => write!(f, "pmcid:{id}"),
            Self::Arxiv(id) => write!(f, "arXiv:{id}"),
        }
    }
}

impl FromStr for Identifier {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

/// `text` without the first of `prefixes` it starts with (ASCII case-insensitive).
fn strip_any_prefix<'a>(text: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes.iter().find_map(|prefix| {
        let head = text.get(..prefix.len())?;
        head.eq_ignore_ascii_case(prefix).then(|| &text[prefix.len()..])
    })
}

/// Decodes `%XX` escapes (as in DOIs copied from URLs); returns `text` unchanged if the result
/// would not be UTF-8.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok());
        match (bytes[i], hex.and_then(|h| u8::from_str_radix(h, 16).ok())) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| text.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_and_key(input: &str) -> (IdentifierKind, String) {
        let id = Identifier::parse(input).unwrap();
        (id.kind(), id.key())
    }

    #[test]
    fn detects_each_kind() {
        assert_eq!(kind_and_key("10.1038/nature12373"), (IdentifierKind::Doi, "10.1038/NATURE12373".into()));
        assert_eq!(kind_and_key("2101.00001v2"), (IdentifierKind::Arxiv, "2101.00001".into()));
        assert_eq!(kind_and_key("PMC7614242"), (IdentifierKind::Pmcid, "PMC7614242".into()));
        assert_eq!(kind_and_key("31452104"), (IdentifierKind::Pmid, "31452104".into()));
    }

    #[test]
    fn arxiv_doi_and_arxiv_id_have_the_same_key() {
        assert_eq!(kind_and_key("https://doi.org/10.48550/arXiv.1706.03762"), kind_and_key("arXiv:1706.03762"));
    }

    #[test]
    fn unknown_text_is_an_error() {
        assert!(matches!(Identifier::parse("hello world"), Err(ModelError::UnknownIdentifier(_))));
        assert!(Identifier::parse("https://example.org/paper").is_err());
    }

    #[test]
    fn serde_form() {
        let id = Identifier::parse("doi:10.1038/nature12373").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#"{"kind":"doi","value":"10.1038/nature12373"}"#);
        assert_eq!(serde_json::from_str::<Identifier>(&json).unwrap(), id);
    }

    #[test]
    fn percent_decoding() {
        assert_eq!(percent_decode("10.1002/%28SICI%291097"), "10.1002/(SICI)1097");
        assert_eq!(percent_decode("100%"), "100%");
    }
}
