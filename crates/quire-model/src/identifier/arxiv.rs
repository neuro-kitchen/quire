//! [`ArxivId`]: an arXiv paper ID, new style (`2101.00001`, since 2007) or old style
//! (`hep-th/9901001`, `math.GT/0309136`), with an optional version (`v2`).
//! arXiv IDs map to DataCite DOIs `10.48550/arXiv.<id>` (verified for new-style IDs, 2026-10-18;
//! old style untested, see `.knowledge/metadata-sources.md`).

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{strip_any_prefix, Doi};
use crate::error::{ModelError, Result};

const PREFIXES: &[&str] = &[
    "https://arxiv.org/abs/",
    "http://arxiv.org/abs/",
    "https://arxiv.org/pdf/",
    "http://arxiv.org/pdf/",
    "arxiv.org/abs/",
    "arxiv.org/pdf/",
    "arxiv:",
];

/// An arXiv ID without version, plus the version if one was given.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ArxivId {
    id: String,
    version: Option<u32>,
}

impl ArxivId {
    /// Parses a bare ID, `arXiv:<id>`, or an arxiv.org `abs`/`pdf` URL.
    pub fn parse(input: &str) -> Result<Self> {
        let text = input.trim();
        let text = strip_any_prefix(text, PREFIXES).unwrap_or(text).trim();
        let text = text.strip_suffix(".pdf").unwrap_or(text).trim_end_matches('/');
        let (id, version) = split_version(text);
        let id = if is_new_style(id) { Some(id.to_string()) } else { canonical_old_style(id) };
        let id = id.ok_or_else(|| ModelError::InvalidIdentifier {
            kind: "arXiv ID",
            input: input.into(),
            reason: "expected `YYMM.NNNNN` or `archive/YYMMNNN`, optionally with `vN`",
        })?;
        Ok(Self { id, version })
    }

    /// `Some` for labelled or URL forms and for bare new-style IDs (old-style bare IDs need the
    /// `arXiv:` label, to avoid guessing on any `word/digits` text).
    pub(super) fn detect(text: &str) -> Option<Result<Self>> {
        let labelled = strip_any_prefix(text, PREFIXES).is_some();
        (labelled || is_new_style(split_version(text).0)).then(|| Self::parse(text))
    }

    /// The ID without version, e.g. `2101.00001`.
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> Option<u32> {
        self.version
    }

    /// Normalised form: lower case, without version (all versions are one work).
    pub fn key(&self) -> String {
        self.id.to_ascii_lowercase()
    }

    /// The DataCite DOI arXiv registered for this paper.
    pub fn doi(&self) -> Doi {
        Doi::parse(&format!("10.48550/arXiv.{}", self.id)).expect("arXiv DOI is well-formed")
    }
}

/// Splits a trailing `vN` off `text`.
fn split_version(text: &str) -> (&str, Option<u32>) {
    if let Some(at) = text.rfind(['v', 'V']) {
        let digits = &text[at + 1..];
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            return (&text[..at], digits.parse().ok());
        }
    }
    (text, None)
}

/// `YYMM.NNNN` (2007–2014) or `YYMM.NNNNN` (2015 on), month 01–12.
fn is_new_style(id: &str) -> bool {
    let Some((yymm, number)) = id.split_once('.') else { return false };
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    let month_ok = yymm.get(2..).and_then(|mm| mm.parse::<u8>().ok()).is_some_and(|mm| (1..=12).contains(&mm));
    yymm.len() == 4 && digits(yymm) && month_ok && matches!(number.len(), 4 | 5) && digits(number)
}

/// `archive[.XX]/YYMMNNN` in canonical case (archive lower, subject class upper), e.g.
/// `hep-th/9901001`, `math.GT/0309136`. Any case is accepted: DOIs are case-insensitive and
/// DataCite returns arXiv DOIs upper-cased (`10.48550/ARXIV.HEP-TH/9901001`).
fn canonical_old_style(id: &str) -> Option<String> {
    let (archive, number) = id.split_once('/')?;
    let (name, class) = archive.split_once('.').unwrap_or((archive, ""));
    let name_ok = !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphabetic() || b == b'-');
    let class_ok = class.is_empty() || (class.len() == 2 && class.bytes().all(|b| b.is_ascii_alphabetic()));
    let number_ok = number.len() == 7 && number.bytes().all(|b| b.is_ascii_digit());
    if !(name_ok && class_ok && number_ok) {
        return None;
    }
    let name = name.to_ascii_lowercase();
    Some(match class {
        "" => format!("{name}/{number}"),
        class => format!("{name}.{}/{number}", class.to_ascii_uppercase()),
    })
}

impl fmt::Display for ArxivId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.version {
            Some(v) => write!(f, "{}v{v}", self.id),
            None => f.write_str(&self.id),
        }
    }
}

impl TryFrom<String> for ArxivId {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<ArxivId> for String {
    fn from(id: ArxivId) -> Self {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_style_forms() {
        for input in [
            "1706.03762",
            "arXiv:1706.03762",
            "https://arxiv.org/abs/1706.03762",
            "https://arxiv.org/pdf/1706.03762.pdf",
            "0704.0001",
        ] {
            assert!(ArxivId::parse(input).is_ok(), "{input}");
        }
        assert_eq!(ArxivId::parse("https://arxiv.org/abs/1706.03762").unwrap().id(), "1706.03762");
    }

    #[test]
    fn version_is_kept_but_not_in_the_key() {
        let id = ArxivId::parse("arXiv:2101.00001v3").unwrap();
        assert_eq!((id.id(), id.version(), id.key()), ("2101.00001", Some(3), "2101.00001".into()));
        assert_eq!(id.to_string(), "2101.00001v3");
    }

    #[test]
    fn old_style() {
        assert_eq!(ArxivId::parse("arXiv:hep-th/9901001").unwrap().id(), "hep-th/9901001");
        assert_eq!(ArxivId::parse("math.GT/0309136v1").unwrap().version(), Some(1));
        assert!(ArxivId::detect("hep-th/9901001").is_none(), "bare old style needs the label");
    }

    #[test]
    fn old_style_case_is_normalised() {
        assert_eq!(ArxivId::parse("HEP-TH/9901001").unwrap().id(), "hep-th/9901001");
        assert_eq!(ArxivId::parse("arXiv:MATH.gt/0309136").unwrap().id(), "math.GT/0309136");
        let doi = Doi::parse("10.48550/ARXIV.HEP-TH/9901001").unwrap();
        assert_eq!(doi.arxiv_id().unwrap().id(), "hep-th/9901001");
    }

    #[test]
    fn rejects_malformed() {
        for input in ["1713.00001", "17060.3762", "1706.123", "arXiv:hello", "hep-th/990100", "hep_th/9901001"] {
            assert!(ArxivId::parse(input).is_err(), "{input}");
        }
    }

    #[test]
    fn doi() {
        assert_eq!(ArxivId::parse("1706.03762v5").unwrap().doi().as_str(), "10.48550/arXiv.1706.03762");
    }
}
