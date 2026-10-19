//! Person names given as one string by web pages and the arXiv API.

use quire_model::{Creator, CreatorRole};

/// `Last, First` or `First Last` (the last word is the family name); one word is a family name.
pub fn person(name: &str) -> Creator {
    let given = |g: &str| Some(g.trim().to_string()).filter(|g| !g.is_empty());
    match name.split_once(',') {
        Some((family, rest)) => Creator::person(CreatorRole::Author, family.trim(), given(rest)),
        None => match name.trim().rsplit_once(' ') {
            Some((first, family)) => Creator::person(CreatorRole::Author, family, given(first)),
            None => Creator::person(CreatorRole::Author, name.trim(), None),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms() {
        assert_eq!(person("Doe, Jane").display_name(), "Doe, Jane");
        assert_eq!(person("Aidan N. Gomez").display_name(), "Gomez, Aidan N.");
        assert_eq!(person("Plato").display_name(), "Plato");
    }
}
