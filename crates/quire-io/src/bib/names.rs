//! Creators ↔ BibLaTeX name lists (`author = {Doe, Jane and {World Health Organization}}`).

use biblatex::{Chunk, Chunks, Person, Spanned};
use quire_model::{Creator, CreatorRole, Name, PersonName};

/// BibLaTeX name fields and the roles they hold.
pub const NAME_FIELDS: &[(&str, CreatorRole)] = &[
    ("author", CreatorRole::Author),
    ("editor", CreatorRole::Editor),
    ("translator", CreatorRole::Translator),
    ("bookauthor", CreatorRole::ContainerAuthor),
];

/// The name list for the creators in `role`, or `None` if there are none. People are written
/// `von Last, Jr, First`; organisations and one-part names are braced so they are not split.
pub fn to_chunks(creators: &[Creator], role: CreatorRole) -> Option<Chunks> {
    let mut chunks = Vec::new();
    for creator in creators.iter().filter(|c| c.role == role) {
        if !chunks.is_empty() {
            chunks.push(normal(" and "));
        }
        match &creator.name {
            Name::Literal { literal } => chunks.push(verbatim(literal)),
            Name::Person(p) => match &p.given {
                None if p.non_dropping_particle.is_none() && p.suffix.is_none() => chunks.push(verbatim(&p.family)),
                _ => chunks.push(normal(&person_text(p))),
            },
        }
    }
    (!chunks.is_empty()).then_some(chunks)
}

fn person_text(p: &PersonName) -> String {
    let mut text = String::new();
    if let Some(particle) = &p.non_dropping_particle {
        text.push_str(particle);
        text.push(' ');
    }
    text.push_str(&p.family);
    if let Some(suffix) = &p.suffix {
        text.push_str(", ");
        text.push_str(suffix);
    }
    text.push_str(", ");
    let given = [&p.given, &p.dropping_particle].into_iter().flatten().map(String::as_str);
    text.push_str(&given.collect::<Vec<_>>().join(" "));
    text
}

/// A creator from a parsed BibLaTeX person. A name with only a last part (as `{WHO}` parses)
/// becomes a literal name.
pub fn from_person(role: CreatorRole, person: &Person) -> Creator {
    let some = |s: &str| (!s.trim().is_empty()).then(|| s.trim().to_string());
    if person.given_name.trim().is_empty() && person.prefix.trim().is_empty() && person.suffix.trim().is_empty() {
        return Creator::literal(role, person.name.trim());
    }
    let name = PersonName {
        family: person.name.trim().into(),
        given: some(&person.given_name),
        non_dropping_particle: some(&person.prefix),
        dropping_particle: None,
        suffix: some(&person.suffix),
    };
    Creator { role, name: Name::Person(name) }
}

fn normal(text: &str) -> Spanned<Chunk> {
    Spanned::detached(Chunk::Normal(text.into()))
}

fn verbatim(text: &str) -> Spanned<Chunk> {
    Spanned::detached(Chunk::Verbatim(text.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use biblatex::ChunksExt;

    fn written(creators: &[Creator]) -> String {
        to_chunks(creators, CreatorRole::Author).unwrap().to_biblatex_string(false)
    }

    #[test]
    fn writes_people_and_organisations() {
        let mut beethoven = Creator::person(CreatorRole::Author, "Beethoven", Some("Ludwig".into()));
        if let Name::Person(p) = &mut beethoven.name {
            p.non_dropping_particle = Some("van".into());
        }
        let creators = [
            Creator::person(CreatorRole::Author, "García Márquez", Some("Gabriel".into())),
            Creator::literal(CreatorRole::Author, "World Health Organization"),
            beethoven,
            Creator::person(CreatorRole::Editor, "Roe", None),
        ];
        assert_eq!(
            written(&creators),
            "{García Márquez, Gabriel and {World Health Organization} and van Beethoven, Ludwig}"
        );
        assert!(to_chunks(&creators, CreatorRole::Translator).is_none());
    }

    #[test]
    fn reads_back_what_it_writes() {
        let creators = [
            Creator::person(CreatorRole::Author, "García Márquez", Some("Gabriel".into())),
            Creator::literal(CreatorRole::Author, "World Health Organization"),
        ];
        let people: Vec<Person> = to_chunks(&creators, CreatorRole::Author).unwrap().parse().unwrap();
        let back: Vec<Creator> = people.iter().map(|p| from_person(CreatorRole::Author, p)).collect();
        assert_eq!(back, creators);
    }
}
