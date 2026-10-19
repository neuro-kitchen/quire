//! The keys `config get|set|unset` accept, and how their values are checked and written. A `*`
//! segment stands for a name the user chooses (a library, an export profile), which must be a
//! valid slug (`thesis`, `lab-2`).

use quire_model::Slug;

use crate::error::{EngineError, Result};

/// What a key holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Email,
    Text,
    Path,
    /// The name of a library (a slug).
    LibraryName,
    Bool,
    /// Comma-separated on the command line, an array in the file.
    List,
    /// One of a fixed set of words.
    Choice(&'static [&'static str]),
    /// A whole number between the two bounds.
    Number(u32, u32),
}

/// A key pattern with its value kind and a one-line description for `config keys`.
#[derive(Debug, Clone, Copy)]
pub struct KeySpec {
    pub pattern: &'static str,
    pub kind: ValueKind,
    pub help: &'static str,
}

const fn key(pattern: &'static str, kind: ValueKind, help: &'static str) -> KeySpec {
    KeySpec { pattern, kind, help }
}

/// Every settable key.
pub const KEYS: &[KeySpec] = &[
    key("user.email", ValueKind::Email, "contact email sent to Crossref, NCBI and Unpaywall"),
    key("user.name", ValueKind::Text, "your name"),
    key("library.default", ValueKind::LibraryName, "library used when none is chosen"),
    key("libraries.*.path", ValueKind::Path, "folder of a registered library (see `quire library new`)"),
    key("access.proxy", ValueKind::Text, "institutional proxy prefix, e.g. https://login.proxy.uni.edu/login?url="),
    key("access.ca", ValueKind::Path, "PEM file with intermediate certificates your proxy does not send"),
    key("access.downloads", ValueKind::Path, "folder your browser saves downloads to (default: ~/Downloads)"),
    key(
        "export.*.format",
        ValueKind::Choice(&["biblatex", "bibtex", "csl-json", "hayagriva"]),
        "export profile: output format",
    ),
    key("export.*.omit", ValueKind::List, "export profile: fields left out, e.g. abstract,file,note"),
    key("export.*.ascii", ValueKind::Bool, "export profile: non-ASCII as LaTeX commands (.bib)"),
    key("export.*.months", ValueKind::Choice(&["number", "macro"]), "export profile: BibTeX months as 7 or jul"),
    key(
        "export.*.preprints",
        ValueKind::Choice(&["online", "article"]),
        "export profile: preprints as @online (Biber) or @article (Typst)",
    ),
    key("reader.width", ValueKind::Number(40, 120), "reader mode: text column width"),
    key("reader.justify", ValueKind::Bool, "reader mode: justified text"),
    key("reader.spacing", ValueKind::Choice(&["1", "2"]), "reader mode: line spacing"),
    key("reader.theme", ValueKind::Choice(&["dark", "light", "sepia", "contrast"]), "reader mode: colors"),
];

/// The spec matching `key`, checking `*` segments are valid names.
pub fn spec(key: &str) -> Result<&'static KeySpec> {
    let parts: Vec<&str> = key.split('.').collect();
    KEYS.iter()
        .find(|spec| {
            let pattern: Vec<&str> = spec.pattern.split('.').collect();
            pattern.len() == parts.len()
                && pattern.iter().zip(&parts).all(|(p, k)| if *p == "*" { Slug::parse(k).is_ok() } else { p == k })
        })
        .ok_or_else(|| {
            let known: Vec<&str> = KEYS.iter().map(|s| s.pattern).collect();
            EngineError::Config(format!("unknown key `{key}` (known: {})", known.join(", ")))
        })
}

/// `text` checked and converted for `kind`.
pub fn to_toml(kind: ValueKind, text: &str) -> Result<toml_edit::Value> {
    let bad = |what: &str| EngineError::Config(format!("`{text}` is not {what}"));
    Ok(match kind {
        ValueKind::Email if !looks_like_email(text) => return Err(bad("an email address")),
        ValueKind::LibraryName if Slug::parse(text).is_err() => {
            return Err(bad("a library name (lowercase letters, digits and `-`)"));
        }
        ValueKind::Bool => match text {
            "true" | "yes" | "on" => true.into(),
            "false" | "no" | "off" => false.into(),
            _ => return Err(bad("true or false")),
        },
        ValueKind::List => {
            let items = text.split(',').map(str::trim).filter(|item| !item.is_empty());
            toml_edit::Value::Array(items.collect())
        }
        ValueKind::Number(min, max) => match text.parse::<u32>() {
            Ok(n) if (min..=max).contains(&n) => i64::from(n).into(),
            _ => return Err(bad(&format!("a number from {min} to {max}"))),
        },
        ValueKind::Choice(words) if !words.contains(&text) => return Err(bad(&format!("one of {}", words.join(", ")))),
        _ => text.into(),
    })
}

fn looks_like_email(text: &str) -> bool {
    matches!(text.split_once('@'), Some((user, domain)) if !user.is_empty() && domain.contains('.') && !text.contains(char::is_whitespace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns_match_names() {
        assert_eq!(spec("libraries.lab.path").unwrap().kind, ValueKind::Path);
        assert!(spec("libraries.Lab.path").is_err());
        assert!(spec("libraries.path").is_err());
        assert!(spec("user.mail").is_err());
    }

    #[test]
    fn values_are_checked() {
        assert!(to_toml(ValueKind::Email, "not an email").is_err());
        assert!(to_toml(ValueKind::LibraryName, "My Lib").is_err());
        assert_eq!(to_toml(ValueKind::Bool, "yes").unwrap().as_bool(), Some(true));
        let list = to_toml(ValueKind::List, "abstract, file,").unwrap();
        assert_eq!(list.as_array().unwrap().len(), 2);
        assert!(to_toml(ValueKind::Choice(&["a", "b"]), "c").is_err());
    }
}
