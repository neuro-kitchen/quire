//! [`Creator`]: a person or organisation with a role in a work (author, editor, …), with names
//! split as in CSL-JSON.

use serde::{Deserialize, Serialize};

use crate::error::ModelError;

name_enum! {
    /// A CSL 1.0.2 name variable (Appendix IV).
    pub enum CreatorRole (unknown: ModelError::UnknownCreatorRole) {
        Author => "author",
        Chair => "chair",
        CollectionEditor => "collection-editor",
        Compiler => "compiler",
        Composer => "composer",
        ContainerAuthor => "container-author",
        Contributor => "contributor",
        Curator => "curator",
        Director => "director",
        Editor => "editor",
        EditorialDirector => "editorial-director",
        EditorTranslator => "editor-translator",
        ExecutiveProducer => "executive-producer",
        Guest => "guest",
        Host => "host",
        Illustrator => "illustrator",
        Interviewer => "interviewer",
        Narrator => "narrator",
        Organizer => "organizer",
        OriginalAuthor => "original-author",
        Performer => "performer",
        Producer => "producer",
        Recipient => "recipient",
        ReviewedAuthor => "reviewed-author",
        ScriptWriter => "script-writer",
        SeriesCreator => "series-creator",
        Translator => "translator",
    }
}

/// A person's name in parts, e.g. Ludwig van Beethoven: family `Beethoven`, given `Ludwig`,
/// non-dropping particle `van`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonName {
    pub family: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    /// Particle kept with the family name when sorting (`van` in "van Gogh" in Dutch use).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_dropping_particle: Option<String>,
    /// Particle dropped when sorting (`de` in "Jean de La Fontaine").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropping_particle: Option<String>,
    /// `Jr.`, `III`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

/// A creator's name: in parts for people, one string for organisations and unparsed names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Name {
    Person(PersonName),
    Literal { literal: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Creator {
    pub role: CreatorRole,
    #[serde(flatten)]
    pub name: Name,
}

impl Creator {
    /// A person with `family` and optional `given` name.
    pub fn person(role: CreatorRole, family: impl Into<String>, given: Option<String>) -> Self {
        let name = PersonName {
            family: family.into(),
            given,
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        };
        Self { role, name: Name::Person(name) }
    }

    /// An organisation or a name kept as written.
    pub fn literal(role: CreatorRole, name: impl Into<String>) -> Self {
        Self { role, name: Name::Literal { literal: name.into() } }
    }

    /// The name used for sorting and cite keys: family name (with a non-dropping particle) for a
    /// person, the whole name otherwise.
    pub fn sort_name(&self) -> String {
        match &self.name {
            Name::Person(p) => match &p.non_dropping_particle {
                Some(particle) => format!("{particle} {}", p.family),
                None => p.family.clone(),
            },
            Name::Literal { literal } => literal.clone(),
        }
    }

    /// The name as written in bibliographies: `van Gogh, Vincent, Jr.` for a person, the whole
    /// name otherwise.
    pub fn display_name(&self) -> String {
        match &self.name {
            Name::Person(p) => {
                let given = [p.given.as_deref(), p.dropping_particle.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");
                let parts = [Some(self.sort_name()), Some(given).filter(|g| !g.is_empty()), p.suffix.clone()];
                parts.into_iter().flatten().collect::<Vec<_>>().join(", ")
            }
            Name::Literal { literal } => literal.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_27_csl_name_roles() {
        assert_eq!(CreatorRole::ALL.len(), 27);
    }

    #[test]
    fn serde_matches_csl_json_names() {
        let person = Creator::person(CreatorRole::Author, "Vaswani", Some("Ashish".into()));
        let json = serde_json::to_string(&person).unwrap();
        assert_eq!(json, r#"{"role":"author","family":"Vaswani","given":"Ashish"}"#);
        assert_eq!(serde_json::from_str::<Creator>(&json).unwrap(), person);

        let org = Creator::literal(CreatorRole::Author, "WHO");
        let json = serde_json::to_string(&org).unwrap();
        assert_eq!(json, r#"{"role":"author","literal":"WHO"}"#);
        assert_eq!(serde_json::from_str::<Creator>(&json).unwrap(), org);
    }

    #[test]
    fn sort_name_keeps_non_dropping_particle() {
        let mut creator = Creator::person(CreatorRole::Author, "Gogh", Some("Vincent".into()));
        if let Name::Person(p) = &mut creator.name {
            p.non_dropping_particle = Some("van".into());
        }
        assert_eq!(creator.sort_name(), "van Gogh");
        assert_eq!(creator.display_name(), "van Gogh, Vincent");
    }
}
