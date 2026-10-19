//! [`Provenance`]: where a field's value came from and when, so a re-fetch never overwrites what
//! the user typed (`.knowledge/data-foundations.md`, 5).

use serde::{Deserialize, Serialize};

use crate::record::Timestamp;

/// Where a value came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "origin", rename_all = "lowercase")]
pub enum Origin {
    /// Fetched from a metadata service, e.g. `source = "doi.org"`.
    Fetched { source: String },
    /// Read from a file, e.g. a `.bib` the user imported.
    Imported { file: String },
    /// Typed or edited by the user.
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(flatten)]
    pub origin: Origin,
    pub at: Timestamp,
}

impl Provenance {
    pub fn now(origin: Origin) -> Self {
        Self { origin, at: Timestamp::now() }
    }

    /// Whether an automatic update (re-fetch, re-import) may replace the value.
    pub fn replaceable(&self) -> bool {
        !matches!(self.origin, Origin::User)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_values_are_not_replaceable() {
        assert!(!Provenance::now(Origin::User).replaceable());
        assert!(Provenance::now(Origin::Fetched { source: "doi.org".into() }).replaceable());
    }

    #[test]
    fn serde_form() {
        let p = Provenance { origin: Origin::Fetched { source: "doi.org".into() }, at: Timestamp::from_millis(1) };
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, r#"{"origin":"fetched","source":"doi.org","at":1}"#);
        assert_eq!(serde_json::from_str::<Provenance>(&json).unwrap(), p);
    }
}
