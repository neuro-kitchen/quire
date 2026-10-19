//! [`Issue`]: something wrong or doubtful about a reference's metadata, found by checking its
//! fields (lints, computed when asked) or recorded when it was fetched (a fallback source was
//! used, the publisher blocked the request). `quire ref check` reports both.

use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::record::Timestamp;

name_enum! {
    /// What kind of issue.
    pub enum IssueCode (unknown: ModelError::UnknownIssue) {
        /// No title.
        NoTitle => "no-title",
        /// No author, editor or other creator.
        NoCreators => "no-creators",
        /// No publication date.
        NoDate => "no-date",
        /// An article or chapter without the journal or book it appeared in.
        NoContainer => "no-container",
        /// Type `document`: the source did not say what the work is.
        UnknownType => "unknown-type",
        /// The title is written in capitals.
        TitleAllCaps => "title-all-caps",
        /// No DOI, PMID, PMCID or arXiv ID: it cannot be refetched or deduplicated.
        NoIdentifier => "no-identifier",
        /// The preferred source failed; the metadata came from a lesser one (detail says which).
        Fallback => "fallback",
        /// The publisher answered with a challenge page ("are you human") instead of the data.
        Blocked => "blocked",
        /// The attached PDF prints another DOI or arXiv ID than the reference's.
        PdfMismatch => "pdf-mismatch",
    }
}

/// How much an issue matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Worth a look; the reference is usable.
    Warning,
    /// The reference is probably wrong or incomplete for citing.
    Error,
}

impl IssueCode {
    pub fn severity(self) -> Severity {
        match self {
            Self::NoTitle | Self::NoCreators | Self::NoDate | Self::Blocked => Severity::Error,
            _ => Severity::Warning,
        }
    }

    /// Whether the issue is recorded when it happens (not computable from the fields later).
    pub fn is_recorded(self) -> bool {
        matches!(self, Self::Fallback | Self::Blocked | Self::PdfMismatch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub code: IssueCode,
    /// E.g. which source failed and why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// When it was recorded; `None` for lints (computed now).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<Timestamp>,
}

impl Issue {
    pub fn new(code: IssueCode) -> Self {
        Self { code, detail: None, at: None }
    }

    /// A recorded issue, stamped now.
    pub fn recorded(code: IssueCode, detail: impl Into<String>) -> Self {
        Self { code, detail: Some(detail.into()), at: Some(Timestamp::now()) }
    }

    pub fn severity(&self) -> Severity {
        self.code.severity()
    }

    /// What it means, for people.
    pub fn message(&self) -> String {
        let text = match self.code {
            IssueCode::NoTitle => "no title",
            IssueCode::NoCreators => "no authors or editors",
            IssueCode::NoDate => "no date",
            IssueCode::NoContainer => "no journal or book title",
            IssueCode::UnknownType => "type unknown (document)",
            IssueCode::TitleAllCaps => "title in capitals",
            IssueCode::NoIdentifier => "no DOI, PMID or arXiv ID",
            IssueCode::Fallback => "metadata from a fallback source",
            IssueCode::Blocked => "blocked by the publisher",
            IssueCode::PdfMismatch => "the PDF shows another identifier",
        };
        match &self.detail {
            Some(detail) => format!("{text}: {detail}"),
            None => text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip_and_serde() {
        for code in IssueCode::ALL {
            assert_eq!(code.as_str().parse::<IssueCode>().unwrap(), *code);
        }
        let issue = Issue { code: IssueCode::Fallback, detail: Some("doi.org: HTTP 503".into()), at: None };
        let json = serde_json::to_string(&issue).unwrap();
        assert_eq!(json, r#"{"code":"fallback","detail":"doi.org: HTTP 503"}"#);
        assert_eq!(issue.message(), "metadata from a fallback source: doi.org: HTTP 503");
    }
}
