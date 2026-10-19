//! Projects: named groups of references inside the library, nestable like Zotero
//! subcollections (`.knowledge/library-model.md`). A project is addressed by its [`SlugPath`],
//! e.g. `thesis/chapter-2`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{ModelError, Result};
use crate::id::{DeviceId, ProjectId};
use crate::record::RecordMeta;

/// Longest slug accepted, in characters.
pub const MAX_SLUG_LEN: usize = 64;

/// A project's short name for the command line: lowercase letters (any script) and digits,
/// separated by single `-`, e.g. `chapter-2`, `introducción`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Slug(String);

impl Slug {
    pub fn parse(input: &str) -> Result<Self> {
        let invalid = |reason| ModelError::InvalidSlug { input: input.into(), reason };
        let len = input.chars().count();
        if len == 0 || len > MAX_SLUG_LEN {
            return Err(invalid("must be 1 to 64 characters"));
        }
        if !input.chars().all(|c| c == '-' || (c.is_alphanumeric() && !c.is_uppercase())) {
            return Err(invalid("only lowercase letters, digits and `-` are allowed"));
        }
        if input.starts_with('-') || input.ends_with('-') || input.contains("--") {
            return Err(invalid("`-` only between letters or digits"));
        }
        Ok(Self(input.into()))
    }

    /// A slug made from a display name: lowercased, every run of other characters becomes `-`,
    /// e.g. `"Chapter 2: Methods"` → `chapter-2-methods`. Fails if nothing usable is left.
    pub fn from_name(name: &str) -> Result<Self> {
        let mut slug = String::new();
        for c in name.chars().flat_map(char::to_lowercase) {
            if c.is_alphanumeric() {
                slug.push(c);
            } else if !slug.is_empty() && !slug.ends_with('-') {
                slug.push('-');
            }
        }
        let slug: String = slug.trim_end_matches('-').chars().take(MAX_SLUG_LEN).collect();
        Self::parse(slug.trim_end_matches('-')).map_err(|_| ModelError::InvalidSlug {
            input: name.into(),
            reason: "no letters or digits to build a slug from",
        })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Slug {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<String> for Slug {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<Slug> for String {
    fn from(slug: Slug) -> Self {
        slug.0
    }
}

/// Slugs from the top-level project down, written `thesis/chapter-2`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlugPath(Vec<Slug>);

impl SlugPath {
    pub fn segments(&self) -> &[Slug] {
        &self.0
    }

    /// The last segment: the project's own slug.
    pub fn leaf(&self) -> &Slug {
        self.0.last().expect("a slug path has at least one segment")
    }

    /// The path of the parent project, `None` at the top level.
    pub fn parent(&self) -> Option<SlugPath> {
        (self.0.len() > 1).then(|| Self(self.0[..self.0.len() - 1].to_vec()))
    }
}

impl FromStr for SlugPath {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self> {
        s.trim_matches('/').split('/').map(Slug::parse).collect::<Result<Vec<_>>>().map(Self)
    }
}

impl fmt::Display for SlugPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for slug in &self.0 {
            if !first {
                f.write_str("/")?;
            }
            f.write_str(slug.as_str())?;
            first = false;
        }
        Ok(())
    }
}

/// A project. Slugs are unique among siblings (same parent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub slug: Slug,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<ProjectId>,
    pub meta: RecordMeta,
}

impl Project {
    /// A new project named `name`, with a slug made from it, under `parent` (top level if `None`).
    pub fn new(name: &str, parent: Option<ProjectId>, device: DeviceId) -> Result<Self> {
        Ok(Self {
            id: ProjectId::new(),
            name: name.trim().into(),
            slug: Slug::from_name(name)?,
            parent,
            meta: RecordMeta::new(device),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_rules() {
        for ok in ["thesis", "chapter-2", "introducción", "2024"] {
            assert!(Slug::parse(ok).is_ok(), "{ok}");
        }
        for bad in ["", "Thesis", "chapter 2", "-a", "a-", "a--b", "a/b", &"a".repeat(65)] {
            assert!(Slug::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn slug_from_name() {
        let cases = [
            ("Chapter 2: Methods", "chapter-2-methods"),
            ("  Introducción ", "introducción"),
            ("PhD — Thesis!", "phd-thesis"),
        ];
        for (name, slug) in cases {
            assert_eq!(Slug::from_name(name).unwrap().as_str(), slug, "{name}");
        }
        assert!(Slug::from_name("!!!").is_err());
        let long = Slug::from_name(&"ab ".repeat(40)).unwrap();
        assert!(long.as_str().chars().count() <= MAX_SLUG_LEN && !long.as_str().ends_with('-'), "{long}");
    }

    #[test]
    fn slug_path() {
        let path: SlugPath = "thesis/chapter-2".parse().unwrap();
        assert_eq!(path.leaf().as_str(), "chapter-2");
        assert_eq!(path.parent().unwrap().to_string(), "thesis");
        assert!(path.parent().unwrap().parent().is_none());
        assert_eq!(path.to_string(), "thesis/chapter-2");
        assert!("thesis//x".parse::<SlugPath>().is_err());
    }
}
