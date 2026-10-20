//! A reference's parts, as sync merges them one by one: each [`Field`] (type, creators, issued,
//! identifiers, each text field), the cite key, the other variables (`extra`), and whether it is
//! in the trash. Comparing and copying one part, by its name.

use super::{Field, Reference, TextField};

/// The parts that are not fields.
pub const CITE_KEY: &str = "cite-key";
pub const EXTRA: &str = "extra";
pub const TRASH: &str = "trash";

/// Every part's name.
pub fn names() -> Vec<String> {
    [Field::Kind, Field::Creators, Field::Issued, Field::Identifiers]
        .into_iter()
        .chain(TextField::ALL.iter().map(|f| Field::Text(*f)))
        .map(|f| f.as_str().to_string())
        .chain([CITE_KEY, EXTRA, TRASH].map(str::to_string))
        .collect()
}

impl Reference {
    /// Whether `self` and `other` hold the same value for `part`.
    pub fn same_part(&self, other: &Reference, part: &str) -> bool {
        match part {
            CITE_KEY => self.cite_key == other.cite_key,
            EXTRA => self.data.extra == other.data.extra,
            TRASH => self.meta.deleted_at.is_some() == other.meta.deleted_at.is_some(),
            name => match name.parse::<Field>() {
                Ok(Field::Kind) => self.data.kind == other.data.kind,
                Ok(Field::Creators) => self.data.creators == other.data.creators,
                Ok(Field::Issued) => self.data.issued == other.data.issued,
                Ok(Field::Identifiers) => self.data.identifiers == other.data.identifiers,
                Ok(Field::Text(text)) => self.data.get(text) == other.data.get(text),
                Err(_) => true,
            },
        }
    }

    /// Takes `part` (and a field's provenance) from `from`.
    pub fn copy_part(&mut self, from: &Reference, part: &str) {
        match part {
            CITE_KEY => self.cite_key = from.cite_key.clone(),
            EXTRA => self.data.extra = from.data.extra.clone(),
            TRASH => self.meta.deleted_at = from.meta.deleted_at,
            name => {
                let Ok(field) = name.parse::<Field>() else { return };
                match field {
                    Field::Kind => self.data.kind = from.data.kind,
                    Field::Creators => self.data.creators = from.data.creators.clone(),
                    Field::Issued => self.data.issued = from.data.issued,
                    Field::Identifiers => self.data.identifiers = from.data.identifiers.clone(),
                    Field::Text(text) => match from.data.get(text) {
                        Some(value) => self.data.set(text, value),
                        None => {
                            self.data.text.remove(&text);
                        }
                    },
                }
                match from.provenance.get(&field) {
                    Some(provenance) => self.provenance.insert(field, provenance.clone()),
                    None => self.provenance.remove(&field),
                };
            }
        }
    }
}
