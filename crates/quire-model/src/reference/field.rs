//! Field names: [`TextField`] for the plain-text variables a reference stores (CSL variable names),
//! and [`Field`] for every part of a reference that can have its own provenance.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{ModelError, Result};

name_enum! {
    /// A plain-text CSL 1.0.2 variable stored on a reference. Other CSL variables are kept in
    /// [`ReferenceData::extra`](super::ReferenceData::extra).
    pub enum TextField (unknown: ModelError::UnknownField) {
        Title => "title",
        TitleShort => "title-short",
        ContainerTitle => "container-title",
        ContainerTitleShort => "container-title-short",
        CollectionTitle => "collection-title",
        CollectionNumber => "collection-number",
        Volume => "volume",
        NumberOfVolumes => "number-of-volumes",
        Issue => "issue",
        Page => "page",
        NumberOfPages => "number-of-pages",
        Number => "number",
        Edition => "edition",
        Publisher => "publisher",
        PublisherPlace => "publisher-place",
        EventTitle => "event-title",
        EventPlace => "event-place",
        Genre => "genre",
        Medium => "medium",
        Version => "version",
        Status => "status",
        Abstract => "abstract",
        Language => "language",
        Url => "URL",
        Issn => "ISSN",
        Isbn => "ISBN",
        Note => "note",
    }
}

/// A part of a reference with its own provenance: the type, the creators, the issued date, the
/// identifiers, or one text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Field {
    Kind,
    Creators,
    Issued,
    Identifiers,
    Text(TextField),
}

impl Field {
    /// Name as stored: `type`, `creators`, `issued`, `identifiers`, or the text field's CSL name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Kind => "type",
            Self::Creators => "creators",
            Self::Issued => "issued",
            Self::Identifiers => "identifiers",
            Self::Text(field) => field.as_str(),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Field {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "type" => Self::Kind,
            "creators" => Self::Creators,
            "issued" => Self::Issued,
            "identifiers" => Self::Identifiers,
            text => Self::Text(text.parse()?),
        })
    }
}

impl Serialize for Field {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_names_round_trip() {
        let fields = [Field::Kind, Field::Creators, Field::Issued, Field::Identifiers];
        let text = TextField::ALL.iter().map(|f| Field::Text(*f));
        for field in fields.into_iter().chain(text) {
            assert_eq!(field.as_str().parse::<Field>().unwrap(), field);
        }
        assert!("titel".parse::<Field>().is_err());
    }
}
