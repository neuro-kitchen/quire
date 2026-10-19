//! [`Doi`]: a Digital Object Identifier. DOIs are case-insensitive for ASCII, so equality and the
//! key use ASCII upper case (DOI Handbook, "Case insensitivity of the DOI name").

use std::fmt;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use super::{percent_decode, strip_any_prefix, ArxivId};
use crate::error::{ModelError, Result};

/// Forms in which DOIs are pasted, removed before parsing (case-insensitive).
const PREFIXES: &[&str] = &[
    "https://doi.org/",
    "http://doi.org/",
    "https://dx.doi.org/",
    "http://dx.doi.org/",
    "doi.org/",
    "dx.doi.org/",
    "doi:",
];

/// arXiv's DOI prefix (DataCite): `10.48550/arXiv.<id>`.
pub(super) const ARXIV_DOI_PREFIX: &str = "10.48550/arxiv.";

/// A DOI as written (without `doi:` or resolver URL), e.g. `10.1038/nature12373`.
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Doi(String);

impl Doi {
    /// Parses a bare, labelled (`doi:`) or URL form. A DOI is `10.<registrant>/<suffix>`: the
    /// registrant is digits and dots, the suffix any non-empty text without whitespace.
    pub fn parse(input: &str) -> Result<Self> {
        let text = input.trim();
        let text = strip_any_prefix(text, PREFIXES).unwrap_or(text).trim();
        let text = percent_decode(text);
        let invalid = |reason| ModelError::InvalidIdentifier { kind: "DOI", input: input.into(), reason };

        let rest = strip_any_prefix(&text, &["10."]).ok_or_else(|| invalid("must start with `10.`"))?;
        let (registrant, suffix) = rest.split_once('/').ok_or_else(|| invalid("missing `/` after the prefix"))?;
        if registrant.is_empty() || !registrant.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
            return Err(invalid("the part between `10.` and `/` must be digits and dots"));
        }
        if suffix.is_empty() {
            return Err(invalid("empty suffix after `/`"));
        }
        if text.chars().any(char::is_whitespace) {
            return Err(invalid("contains whitespace"));
        }
        Ok(Self(text))
    }

    /// `Some` when `text` looks like a DOI (labelled, resolver URL or starting with `10.`), with
    /// the parse result; `None` when it is clearly something else.
    pub(super) fn detect(text: &str) -> Option<Result<Self>> {
        let looks_like = strip_any_prefix(text, PREFIXES).is_some() || strip_any_prefix(text, &["10."]).is_some();
        looks_like.then(|| Self::parse(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Normalised form for comparison: ASCII upper case.
    pub fn key(&self) -> String {
        self.0.to_ascii_uppercase()
    }

    /// Resolver URL, e.g. `https://doi.org/10.1038/nature12373`.
    pub fn url(&self) -> String {
        format!("https://doi.org/{}", self.0)
    }

    /// The arXiv ID when this is an arXiv DOI (`10.48550/arXiv.<id>`).
    pub fn arxiv_id(&self) -> Option<ArxivId> {
        let id = strip_any_prefix(&self.0, &[ARXIV_DOI_PREFIX])?;
        ArxivId::parse(id).ok()
    }
}

impl PartialEq for Doi {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for Doi {}

impl Hash for Doi {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl fmt::Display for Doi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Doi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Doi({})", self.0)
    }
}

impl TryFrom<String> for Doi {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<Doi> for String {
    fn from(doi: Doi) -> Self {
        doi.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_pasted_forms() {
        for input in [
            "10.1038/nature12373",
            " doi:10.1038/nature12373 ",
            "DOI: 10.1038/nature12373",
            "https://doi.org/10.1038/nature12373",
            "http://dx.doi.org/10.1038/nature12373",
            "doi.org/10.1038/nature12373",
        ] {
            assert_eq!(Doi::parse(input).unwrap().as_str(), "10.1038/nature12373", "{input}");
        }
    }

    #[test]
    fn decodes_url_escapes() {
        let doi = Doi::parse("https://doi.org/10.1002/%28SICI%291097-4636").unwrap();
        assert_eq!(doi.as_str(), "10.1002/(SICI)1097-4636");
    }

    #[test]
    fn case_insensitive_equality() {
        assert_eq!(Doi::parse("10.1038/ABC").unwrap(), Doi::parse("10.1038/abc").unwrap());
        assert_eq!(Doi::parse("10.1038/abc").unwrap().key(), "10.1038/ABC");
    }

    #[test]
    fn rejects_malformed() {
        for input in ["11.1038/x", "10.1038", "10./x", "10.ab/x", "10.1038/", "10.1038/a b"] {
            assert!(Doi::parse(input).is_err(), "{input}");
        }
    }

    #[test]
    fn arxiv_doi() {
        let doi = Doi::parse("10.48550/arXiv.1706.03762").unwrap();
        assert_eq!(doi.arxiv_id().unwrap().key(), "1706.03762");
        assert!(Doi::parse("10.1038/nature12373").unwrap().arxiv_id().is_none());
    }
}
