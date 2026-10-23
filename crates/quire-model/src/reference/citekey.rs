//! [`CiteKey`]: the key used in LaTeX (`\cite{smithstudyexamples2024}`). Generation (Better
//! BibTeX formula) lives in `quire-core`; this type only guarantees a key is safe in `.bib` files.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{ModelError, Result};

/// Longest key accepted.
pub(super) const MAX_LEN: usize = 100;

/// A cite key: 1 to [`MAX_LEN`] characters, ASCII letters, digits and `_ - : .` only (no spaces,
/// braces, commas, quotes, `#`, `%` or `\`, which break BibTeX/Biber).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CiteKey(String);

impl CiteKey {
    /// Longest key accepted, in bytes.
    pub const MAX_LEN: usize = MAX_LEN;

    pub fn parse(input: &str) -> Result<Self> {
        let invalid = |reason| ModelError::InvalidCiteKey { input: input.into(), reason };
        if input.is_empty() || input.len() > MAX_LEN {
            return Err(invalid("must be 1 to 100 characters"));
        }
        if !input.bytes().all(is_key_byte) {
            return Err(invalid("only ASCII letters, digits and `_ - : .` are allowed"));
        }
        Ok(Self(input.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether `byte` may appear in a cite key.
pub fn is_key_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.')
}

impl fmt::Display for CiteKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for CiteKey {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<String> for CiteKey {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<CiteKey> for String {
    fn from(key: CiteKey) -> Self {
        key.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_keys() {
        for key in ["smithstudyexamples2024", "smith2024a", "Smith_2024", "doe:2020-x.1"] {
            assert!(CiteKey::parse(key).is_ok(), "{key}");
        }
    }

    #[test]
    fn invalid_keys() {
        for key in ["", "smith 2024", "smith,2024", "{smith}", "smith%", "müller2020", &"a".repeat(101)] {
            assert!(CiteKey::parse(key).is_err(), "{key}");
        }
    }
}
