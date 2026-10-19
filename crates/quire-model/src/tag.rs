//! [`Tag`]: a label on references, as Zotero's tags. A tag is manual (the user added it) or
//! automatic (it came with imported data, e.g. a `.bib` file's `keywords`). Up to nine tags can be
//! coloured; their position (1–9) is the key that toggles them in the TUI.

use serde::{Deserialize, Serialize};

use crate::error::{ModelError, Result};
use crate::id::{DeviceId, TagId};
use crate::record::RecordMeta;

/// Longest tag name, in characters.
pub const MAX_TAG_LEN: usize = 100;

/// Colored tags have positions 1 to this.
pub const MAX_COLORED: u8 = 9;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagId,
    /// As the user wrote it (case kept); unique ignoring case.
    pub name: String,
    /// `#rrggbb`, for colored tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// 1–9 for colored tags: their order and key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u8>,
    pub meta: RecordMeta,
}

name_enum! {
    /// Who put a tag on a reference.
    pub enum TagKind (unknown: ModelError::UnknownTagKind) {
        Manual => "manual",
        /// From imported data; can be hidden or removed all at once.
        Automatic => "automatic",
    }
}

impl Tag {
    /// A new tag named `name` (trimmed; checked).
    pub fn new(name: &str, device: DeviceId) -> Result<Self> {
        Ok(Self {
            id: TagId::new(),
            name: check_name(name)?,
            color: None,
            position: None,
            meta: RecordMeta::new(device),
        })
    }

    /// The name compared for uniqueness: lowercase (Unicode).
    pub fn key(&self) -> String {
        name_key(&self.name)
    }
}

/// `name` trimmed and checked: 1–100 characters, no line breaks or control characters.
pub fn check_name(name: &str) -> Result<String> {
    let name = name.trim();
    let invalid = |reason| ModelError::InvalidTag { input: name.into(), reason };
    if name.is_empty() || name.chars().count() > MAX_TAG_LEN {
        return Err(invalid("must be 1 to 100 characters"));
    }
    if name.chars().any(char::is_control) {
        return Err(invalid("no line breaks or control characters"));
    }
    Ok(name.to_string())
}

/// The uniqueness key of a tag name.
pub fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// `#rrggbb` (lowercase) from `#RRGGBB`, `rrggbb`, or a basic color name.
pub fn parse_color(text: &str) -> Result<String> {
    let named = match text.trim().to_ascii_lowercase().as_str() {
        "red" => "#e5484d",
        "orange" => "#f76b15",
        "yellow" => "#ffc53d",
        "green" => "#30a46c",
        "blue" => "#0090ff",
        "purple" => "#8e4ec6",
        "pink" => "#d6409f",
        "gray" | "grey" => "#8b8d98",
        other => {
            let hex = other.strip_prefix('#').unwrap_or(other);
            if hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Ok(format!("#{hex}"));
            }
            return Err(ModelError::InvalidTag {
                input: text.into(),
                reason: "a color is #rrggbb or red, orange, yellow, green, blue, purple, pink, gray",
            });
        }
    };
    Ok(named.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_colors() {
        assert_eq!(check_name("  to read ").unwrap(), "to read");
        assert!(check_name("").is_err() && check_name("a\nb").is_err());
        assert_eq!(name_key("Méthodes"), "méthodes");
        assert_eq!(parse_color("Blue").unwrap(), "#0090ff");
        assert_eq!(parse_color("#AABBCC").unwrap(), "#aabbcc");
        assert!(parse_color("#abc").is_err());
    }
}
