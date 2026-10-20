//! References: [`ReferenceData`] is the bibliographic content (what fetchers and importers
//! produce, modelled on CSL-JSON); [`Reference`] is a stored record of it, with ID, cite key,
//! per-field provenance and record metadata.

mod citekey;
mod field;
mod kind;
pub mod parts;
mod provenance;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use citekey::{CiteKey, is_key_byte};
pub use field::{Field, TextField};
pub use kind::EntryKind;
pub use provenance::{Origin, Provenance};

use crate::creator::Creator;
use crate::date::Date;
use crate::id::{DeviceId, ReferenceId};
use crate::identifier::Identifier;
use crate::record::{RecordMeta, Timestamp};

/// The bibliographic content of a work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceData {
    pub kind: EntryKind,
    /// In order of appearance, all roles together.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub creators: Vec<Creator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issued: Option<Date>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<Identifier>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub text: BTreeMap<TextField, String>,
    /// Source variables quire has no field for (name → value as text), kept so nothing is lost.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

impl ReferenceData {
    pub fn new(kind: EntryKind) -> Self {
        Self {
            kind,
            creators: Vec::new(),
            issued: None,
            identifiers: Vec::new(),
            text: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }

    pub fn get(&self, field: TextField) -> Option<&str> {
        self.text.get(&field).map(String::as_str)
    }

    /// Sets `field`, or removes it when `value` is blank. Values are trimmed.
    pub fn set(&mut self, field: TextField, value: impl Into<String>) {
        let value = value.into();
        match value.trim() {
            "" => self.text.remove(&field),
            trimmed => self.text.insert(field, trimmed.into()),
        };
    }

    pub fn title(&self) -> Option<&str> {
        self.get(TextField::Title)
    }

    /// The fields that hold a value, for recording their provenance.
    pub fn present_fields(&self) -> Vec<Field> {
        let mut fields = vec![Field::Kind];
        if !self.creators.is_empty() {
            fields.push(Field::Creators);
        }
        if self.issued.is_some() {
            fields.push(Field::Issued);
        }
        if !self.identifiers.is_empty() {
            fields.push(Field::Identifiers);
        }
        fields.extend(self.text.keys().map(|f| Field::Text(*f)));
        fields
    }
}

/// A reference stored in the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub id: ReferenceId,
    pub cite_key: CiteKey,
    pub data: ReferenceData,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub provenance: BTreeMap<Field, Provenance>,
    /// When each part last changed (a field name, `cite-key`, `extra`, `trash`), for merging two
    /// devices' edits field by field. A part without a clock last changed at `meta.updated_at`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub clocks: BTreeMap<String, Timestamp>,
    pub meta: RecordMeta,
}

impl Reference {
    /// A new record holding `data`, with every present field attributed to `origin`.
    pub fn new(cite_key: CiteKey, data: ReferenceData, origin: Origin, device: DeviceId) -> Self {
        let stamp = Provenance::now(origin);
        let provenance = data.present_fields().into_iter().map(|f| (f, stamp.clone())).collect();
        let meta = RecordMeta::new(device);
        // Every part dates from now: a later edit of one part does not make the others look new.
        let clocks = parts::names().into_iter().map(|part| (part, meta.created_at)).collect();
        Self { id: ReferenceId::new(), cite_key, data, provenance, clocks, meta }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_trims_and_removes_blank() {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "  Attention Is All You Need ");
        assert_eq!(data.title(), Some("Attention Is All You Need"));
        data.set(TextField::Title, "   ");
        assert_eq!(data.title(), None);
    }

    #[test]
    fn new_reference_records_provenance_of_present_fields() {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "A title");
        data.issued = Some(Date::year(2017));
        let origin = Origin::Fetched { source: "doi.org".into() };
        let reference = Reference::new(CiteKey::parse("a2017").unwrap(), data, origin, DeviceId::new());
        let fields: Vec<_> = reference.provenance.keys().copied().collect();
        assert_eq!(fields, vec![Field::Kind, Field::Issued, Field::Text(TextField::Title)]);
    }
}
