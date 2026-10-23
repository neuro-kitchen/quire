//! Reading and writing CSL-JSON values: names, dates and text. Sources are loose (numbers where
//! CSL says strings, arrays where it says one value), so reading accepts the common variants.

use quire_model::{Creator, CreatorRole, Date, Name, PersonName};
use serde_json::{Map, Value, json};

/// A text value: a string, a number, or the first string of an array (Crossref sends titles and
/// ISSNs as arrays). Blank text is `None`.
pub(super) fn text(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(items) => return items.iter().find_map(text),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}

/// The creators in a CSL name list (`"author": [{"family": …, "given": …}, …]`). Entries
/// without a family name or literal are skipped.
pub(super) fn names(role: CreatorRole, value: &Value) -> Vec<Creator> {
    let Some(items) = value.as_array() else { return Vec::new() };
    items.iter().filter_map(Value::as_object).filter_map(|name| creator(role, name)).collect()
}

fn creator(role: CreatorRole, name: &Map<String, Value>) -> Option<Creator> {
    let part = |key: &str| name.get(key).and_then(text);
    if let Some(family) = part("family") {
        let person = PersonName {
            family,
            given: part("given"),
            non_dropping_particle: part("non-dropping-particle"),
            dropping_particle: part("dropping-particle"),
            suffix: part("suffix"),
        };
        return Some(Creator { role, name: Name::Person(person) });
    }
    // An organisation, or a person given only as one string.
    part("literal").or_else(|| part("given")).map(|literal| Creator::literal(role, literal))
}

/// The first date of a CSL date (`{"date-parts": [[2017, 6, 12]]}`; parts may be strings, a
/// single part list may lack the outer array), or its `raw`/`literal` form when it parses.
pub(super) fn date(value: &Value) -> Option<Date> {
    let object = value.as_object()?;
    if let Some(parts) = object.get("date-parts").and_then(Value::as_array) {
        let first = match parts.first() {
            Some(Value::Array(inner)) => inner.as_slice(),
            Some(_) => parts.as_slice(),
            None => return None,
        };
        let part = |i: usize| first.get(i).and_then(text).and_then(|t| t.parse::<u16>().ok());
        let (year, month, day) = (part(0)?, part(1), part(2));
        // `as u8` would wrap (257 → 1): a month or day past 255 is no date at all.
        let small = |n: Option<u16>| n.map(u8::try_from).transpose().ok();
        return Date::new(year, small(month)?, small(day)?).ok();
    }
    ["raw", "literal"].iter().find_map(|key| object.get(*key).and_then(text)).and_then(|t| t.parse().ok())
}

/// A CSL name object for `creator`.
pub(super) fn name_value(creator: &Creator) -> Value {
    match &creator.name {
        Name::Literal { literal } => json!({ "literal": literal }),
        Name::Person(p) => {
            let mut object = Map::new();
            object.insert("family".into(), p.family.clone().into());
            let optional = [
                ("given", &p.given),
                ("non-dropping-particle", &p.non_dropping_particle),
                ("dropping-particle", &p.dropping_particle),
                ("suffix", &p.suffix),
            ];
            for (key, value) in optional {
                if let Some(value) = value {
                    object.insert(key.into(), value.clone().into());
                }
            }
            Value::Object(object)
        }
    }
}

/// A CSL date object for `date`.
pub(super) fn date_value(date: Date) -> Value {
    let mut parts = vec![json!(date.get_year())];
    parts.extend(date.month().map(|m| json!(m)));
    parts.extend(date.day().map(|d| json!(d)));
    json!({ "date-parts": [parts] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_variants() {
        assert_eq!(text(&json!("  Nature ")), Some("Nature".into()));
        assert_eq!(text(&json!(500)), Some("500".into()));
        assert_eq!(text(&json!(["0028-0836", "1476-4687"])), Some("0028-0836".into()));
        assert_eq!(text(&json!("")), None);
        assert_eq!(text(&json!({"a": 1})), None);
    }

    #[test]
    fn name_variants() {
        let list = json!([
            {"family": "Vaswani", "given": "Ashish"},
            {"literal": "WHO"},
            {"given": "Plato"},
            {"sequence": "first"}
        ]);
        let creators = names(CreatorRole::Author, &list);
        assert_eq!(creators.len(), 3);
        assert_eq!(creators[0].sort_name(), "Vaswani");
        assert_eq!(creators[1], Creator::literal(CreatorRole::Author, "WHO"));
        assert_eq!(creators[2].sort_name(), "Plato");
    }

    #[test]
    fn date_variants() {
        let d = |v: Value| date(&v).map(|d| d.to_string());
        assert_eq!(d(json!({"date-parts": [[2013, 7, 31]]})), Some("2013-07-31".into()));
        assert_eq!(d(json!({"date-parts": [["2017", "6"]]})), Some("2017-06".into()));
        assert_eq!(d(json!({"date-parts": [2017]})), Some("2017".into()));
        assert_eq!(d(json!({"date-parts": [[2017], [2018]]})), Some("2017".into()));
        assert_eq!(d(json!({"raw": "2019-03"})), Some("2019-03".into()));
        assert_eq!(d(json!({"date-parts": [[2017, 13]]})), None);
        assert_eq!(d(json!({"literal": "Spring 2017"})), None);
    }

    #[test]
    fn values_round_trip() {
        let person = Creator::person(CreatorRole::Author, "Vaswani", Some("Ashish".into()));
        assert_eq!(names(CreatorRole::Author, &json!([name_value(&person)])), vec![person]);
        let day: Date = "2017-06-12".parse().unwrap();
        assert_eq!(date(&date_value(day)), Some(day));
    }
}
