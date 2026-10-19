//! PubMed identifiers: [`Pmid`] (PubMed record) and [`Pmcid`] (PubMed Central full text).

use std::fmt;

use serde::{Deserialize, Serialize};

use super::strip_any_prefix;
use crate::error::{ModelError, Result};

const PMID_PREFIXES: &[&str] = &[
    "https://pubmed.ncbi.nlm.nih.gov/",
    "http://pubmed.ncbi.nlm.nih.gov/",
    "pubmed.ncbi.nlm.nih.gov/",
    "https://www.ncbi.nlm.nih.gov/pubmed/",
    "pmid:",
];

const PMCID_PREFIXES: &[&str] = &[
    "https://pmc.ncbi.nlm.nih.gov/articles/",
    "https://www.ncbi.nlm.nih.gov/pmc/articles/",
    "pmc.ncbi.nlm.nih.gov/articles/",
    "pmcid:",
];

/// Longest PMID or PMCID number accepted (current PMIDs have 8 digits).
const MAX_DIGITS: usize = 10;

/// A PubMed ID, e.g. `31452104`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Pmid(u64);

/// A PubMed Central ID, e.g. `PMC7614242`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Pmcid(u64);

impl Pmid {
    /// Parses digits, `pmid:<digits>` or a PubMed URL.
    pub fn parse(input: &str) -> Result<Self> {
        let text = input.trim();
        let text = strip_any_prefix(text, PMID_PREFIXES).unwrap_or(text).trim().trim_end_matches('/');
        number(text).map(Self).ok_or_else(|| ModelError::InvalidIdentifier {
            kind: "PMID",
            input: input.into(),
            reason: "expected 1 to 10 digits",
        })
    }

    /// `Some` for labelled or URL forms and for bare digits.
    pub(super) fn detect(text: &str) -> Option<Result<Self>> {
        let labelled = strip_any_prefix(text, PMID_PREFIXES).is_some();
        (labelled || number(text).is_some()).then(|| Self::parse(text))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl Pmcid {
    /// Parses `PMC<digits>`, `pmcid:PMC<digits>` or a PubMed Central URL.
    pub fn parse(input: &str) -> Result<Self> {
        let text = input.trim();
        let text = strip_any_prefix(text, PMCID_PREFIXES).unwrap_or(text).trim().trim_end_matches('/');
        strip_any_prefix(text, &["PMC"]).and_then(number).map(Self).ok_or_else(|| ModelError::InvalidIdentifier {
            kind: "PMCID",
            input: input.into(),
            reason: "expected `PMC` followed by 1 to 10 digits",
        })
    }

    /// `Some` for labelled or URL forms and for `PMC<digits>`.
    pub(super) fn detect(text: &str) -> Option<Result<Self>> {
        let labelled = strip_any_prefix(text, PMCID_PREFIXES).is_some();
        let bare = strip_any_prefix(text, &["PMC"]).and_then(number).is_some();
        (labelled || bare).then(|| Self::parse(text))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

/// `text` as a number when it is 1 to [`MAX_DIGITS`] ASCII digits.
fn number(text: &str) -> Option<u64> {
    let digits = !text.is_empty() && text.len() <= MAX_DIGITS && text.bytes().all(|b| b.is_ascii_digit());
    digits.then(|| text.parse().ok()).flatten()
}

impl fmt::Display for Pmid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Pmcid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PMC{}", self.0)
    }
}

impl TryFrom<String> for Pmid {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl TryFrom<String> for Pmcid {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<Pmid> for String {
    fn from(id: Pmid) -> Self {
        id.to_string()
    }
}

impl From<Pmcid> for String {
    fn from(id: Pmcid) -> Self {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pmid_forms() {
        for input in ["31452104", "pmid:31452104", "PMID: 31452104", "https://pubmed.ncbi.nlm.nih.gov/31452104/"] {
            assert_eq!(Pmid::parse(input).unwrap().get(), 31452104, "{input}");
        }
        assert!(Pmid::parse("12a").is_err());
        assert!(Pmid::parse("12345678901").is_err());
    }

    #[test]
    fn pmcid_forms() {
        for input in
            ["PMC7614242", "pmc7614242", "pmcid:PMC7614242", "https://pmc.ncbi.nlm.nih.gov/articles/PMC7614242/"]
        {
            assert_eq!(Pmcid::parse(input).unwrap().to_string(), "PMC7614242", "{input}");
        }
        assert!(Pmcid::parse("7614242").is_err());
    }
}
