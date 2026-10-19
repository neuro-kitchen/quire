//! A reference as an editable TOML document, for `quire ref edit`: written with comments that
//! explain the format, read back after the user saved it.
//!
//! ```toml
//! cite-key = "kucskoNanometreScaleThermometry2013"
//! type = "article-journal"
//! issued = "2013-07-31"
//! identifiers = ["doi:10.1038/nature12373"]
//!
//! [creators]
//! author = ["Kucsko, G.", "Maurer, P. C."]
//!
//! [fields]
//! title = "Nanometre-scale thermometry in a living cell"
//! container-title = "Nature"
//! ```
//!
//! Names are `Family, Given` (`Family, Given, Jr.`); `{World Health Organization}` keeps a name
//! whole. Fields quire has no name for (`extra`) are not shown and stay as they are.

use quire_model::{
    CiteKey, Creator, CreatorRole, Date, EntryKind, Identifier, Name, PersonName, Reference, ReferenceData, TextField,
};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

use crate::error::{IoError, Result};

const HEADER: &str = "\
# Edit this reference, save and close the editor. Close without saving to cancel.
# Fields you change become yours: `quire ref refetch` will not overwrite them.
# Remove a line to clear a field. Names: \"Family, Given\" or \"{Organisation}\".
";

/// The editable document for `reference`.
pub fn to_edit_toml(reference: &Reference) -> String {
    let data = &reference.data;
    let mut document = DocumentMut::new();
    document["cite-key"] = toml_edit::value(reference.cite_key.as_str());
    document["type"] = toml_edit::value(data.kind.as_str());
    if let Some(issued) = data.issued {
        document["issued"] = toml_edit::value(issued.to_string());
    }
    document["identifiers"] = toml_edit::value(data.identifiers.iter().map(ToString::to_string).collect::<Array>());

    let mut creators = Table::new();
    for role in roles_in_order(data) {
        let names: Array = data.creators.iter().filter(|c| c.role == role).map(name_text).collect();
        creators[role.as_str()] = toml_edit::value(names);
    }
    document["creators"] = Item::Table(creators);

    let mut fields = Table::new();
    for (field, value) in &data.text {
        fields[field.as_str()] = toml_edit::value(value.as_str());
    }
    document["fields"] = Item::Table(fields);
    if let Some(Item::Value(Value::String(kind))) = document.get_mut("type") {
        kind.decor_mut().set_suffix("   # CSL type: article-journal, book, chapter, paper-conference, …");
    }
    if let Some(Item::Value(Value::String(issued))) = document.get_mut("issued") {
        issued.decor_mut().set_suffix("   # YYYY, YYYY-MM or YYYY-MM-DD");
    }
    format!("{HEADER}{document}")
}

/// Reads an edited document back: the cite key and the bibliographic data (without `extra`).
pub fn from_edit_toml(text: &str) -> Result<(CiteKey, ReferenceData)> {
    let document: DocumentMut = text.parse().map_err(|e| IoError::Edit(format!("not valid TOML: {e}")))?;
    let string = |key: &str| document.get(key).and_then(Item::as_str).map(str::trim).filter(|s| !s.is_empty());
    let cite_key = CiteKey::parse(string("cite-key").ok_or_else(|| edit("`cite-key` is missing"))?).map_err(edit)?;
    let kind: EntryKind = string("type").ok_or_else(|| edit("`type` is missing"))?.parse().map_err(edit)?;
    let mut data = ReferenceData::new(kind);
    data.issued = string("issued").map(|t| t.parse::<Date>()).transpose().map_err(edit)?;
    for id in strings(document.get("identifiers"), "identifiers")? {
        data.identifiers.push(Identifier::parse(&id).map_err(edit)?);
    }
    if let Some(table) = document.get("creators").and_then(Item::as_table) {
        for (role, names) in table {
            let role: CreatorRole = role.parse().map_err(edit)?;
            for name in strings(Some(names), &format!("creators.{role}"))? {
                data.creators.push(parse_name(role, &name));
            }
        }
    }
    if let Some(table) = document.get("fields").and_then(Item::as_table) {
        for (name, value) in table {
            let field: TextField =
                name.parse().map_err(|_| edit(format!("unknown field `{name}` (known: {})", known_fields())))?;
            let value = value.as_str().ok_or_else(|| edit(format!("`fields.{name}` must be text in quotes")))?;
            data.set(field, value);
        }
    }
    Ok((cite_key, data))
}

fn edit(message: impl ToString) -> IoError {
    IoError::Edit(message.to_string())
}

fn known_fields() -> String {
    TextField::ALL.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ")
}

/// A list of strings at `item` (a single string is a list of one); missing is empty.
fn strings(item: Option<&Item>, key: &str) -> Result<Vec<String>> {
    let Some(item) = item else { return Ok(Vec::new()) };
    if let Some(one) = item.as_str() {
        return Ok(vec![one.to_string()]);
    }
    let array = item.as_array().ok_or_else(|| edit(format!("`{key}` must be a list of text, e.g. [\"…\"]")))?;
    array
        .iter()
        .map(|v| v.as_str().map(str::to_string).ok_or_else(|| edit(format!("`{key}` must hold text in quotes"))))
        .collect()
}

/// Roles in order of first appearance.
fn roles_in_order(data: &ReferenceData) -> Vec<CreatorRole> {
    let mut roles = Vec::new();
    for creator in &data.creators {
        if !roles.contains(&creator.role) {
            roles.push(creator.role);
        }
    }
    roles
}

fn name_text(creator: &Creator) -> String {
    match &creator.name {
        Name::Literal { literal } => format!("{{{literal}}}"),
        Name::Person(_) => creator.display_name(),
    }
}

/// `{Whole name}`, `Family, Given[, Suffix]`, or `Given Family`.
fn parse_name(role: CreatorRole, text: &str) -> Creator {
    let text = text.trim();
    if let Some(literal) = text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
        return Creator::literal(role, literal.trim());
    }
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    let some = |s: &str| Some(s.to_string()).filter(|s| !s.is_empty());
    let (family, given, suffix) = match parts.as_slice() {
        [family, given, suffix, ..] => (family.to_string(), some(given), some(suffix)),
        [family, given] => (family.to_string(), some(given), None),
        _ => match text.rsplit_once(' ') {
            Some((given, family)) => (family.to_string(), some(given), None),
            None => (text.to_string(), None, None),
        },
    };
    let person = PersonName { family, given, non_dropping_particle: None, dropping_particle: None, suffix };
    Creator { role, name: Name::Person(person) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_model::{DeviceId, Origin};

    fn reference() -> Reference {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.creators.push(Creator::person(CreatorRole::Author, "Kucsko", Some("G.".into())));
        data.creators.push(Creator::literal(CreatorRole::Author, "The NV Team"));
        data.creators.push(Creator::person(CreatorRole::Editor, "Doe", Some("Jane".into())));
        data.issued = Some(Date::new(2013, Some(7), Some(31)).unwrap());
        data.set(TextField::Title, "Nanometre-scale \"thermometry\"");
        data.set(TextField::ContainerTitle, "Nature");
        data.identifiers.push(Identifier::parse("10.1038/nature12373").unwrap());
        Reference::new(CiteKey::parse("kucsko2013").unwrap(), data, Origin::User, DeviceId::new())
    }

    #[test]
    fn round_trip() {
        let original = reference();
        let text = to_edit_toml(&original);
        assert!(text.starts_with("# Edit this reference"), "{text}");
        assert!(text.contains("author = [\"Kucsko, G.\", \"{The NV Team}\"]"), "{text}");
        let (key, data) = from_edit_toml(&text).unwrap();
        assert_eq!(key, original.cite_key);
        assert_eq!(data, original.data);
    }

    #[test]
    fn edits_and_errors() {
        let text =
            to_edit_toml(&reference()).replace("container-title = \"Nature\"\n", "").replace("2013-07-31", "2014");
        let (_, data) = from_edit_toml(&text).unwrap();
        assert_eq!((data.get(TextField::ContainerTitle), data.issued.unwrap().to_string()), (None, "2014".into()));
        let bad = to_edit_toml(&reference()).replace("[fields]", "[fields]\njournal = \"x\"");
        assert!(from_edit_toml(&bad).unwrap_err().to_string().contains("unknown field `journal`"));
        assert!(from_edit_toml("cite-key = \"x\"\ntype = \"nope\"").is_err());
        assert!(from_edit_toml("not toml [").is_err());
    }

    #[test]
    fn name_forms() {
        let name = |t: &str| parse_name(CreatorRole::Author, t).display_name();
        assert_eq!(name("Doe, Jane, Jr."), "Doe, Jane, Jr.");
        assert_eq!(name("Jane Doe"), "Doe, Jane");
        assert_eq!(name("{WHO}"), "WHO");
    }
}
