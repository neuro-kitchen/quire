//! Rows for tests.

use quire_model::{
    CiteKey, Creator, CreatorRole, Date, DeviceId, EntryKind, Origin, Reference, ReferenceData, TextField,
};

use crate::state::Row;

pub fn row(key: &str, title: &str, author: &str, year: u16) -> Row {
    let mut data = ReferenceData::new(EntryKind::ArticleJournal);
    data.set(TextField::Title, title);
    data.creators.push(Creator::person(CreatorRole::Author, author, None));
    data.issued = Some(Date::year(year));
    let reference = Reference::new(CiteKey::parse(key).unwrap(), data, Origin::User, DeviceId::new());
    Row { reference, issues: Vec::new(), pdf: None, tags: Vec::new(), hit: None }
}
