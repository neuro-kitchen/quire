//! The parts of Better BibTeX's default formula `auth.lower + shorttitle(3,3) + year`:
//! - `auth.lower`: first author's family name (editor's, then any creator's, if there is no
//!   author), lowercased;
//! - `shorttitle(3,3)`: the first 3 title words that are not skip words, each capitalised;
//! - `year`: the issued year.
//!
//! Text is transliterated to ASCII (`Müller` → `Muller`) and reduced to letters and digits, so the
//! key is always valid. If all parts are empty the key is `ref`.

use deunicode::deunicode;
use quire_model::{CreatorRole, ReferenceData};

use super::skipwords::is_skip_word;

/// Number of title words taken and capitalised.
const TITLE_WORDS: usize = 3;

/// Key used when the formula gives nothing.
const FALLBACK: &str = "ref";

/// Longest base key, leaving room for postfixes within the cite-key limit.
const MAX_BASE_LEN: usize = 90;

/// The formula's key for `data`, before uniqueness postfixes.
pub fn base_key(data: &ReferenceData) -> String {
    let mut key = format!("{}{}{}", auth_lower(data), short_title(data), year(data));
    key.truncate(MAX_BASE_LEN);
    if key.is_empty() { FALLBACK.into() } else { key }
}

/// `auth.lower`.
pub fn auth_lower(data: &ReferenceData) -> String {
    let first_with = |role| data.creators.iter().find(|c| c.role == role);
    let creator = first_with(CreatorRole::Author).or_else(|| first_with(CreatorRole::Editor)).or(data.creators.first());
    creator.map(|c| ascii_alnum(&c.sort_name()).to_lowercase()).unwrap_or_default()
}

/// `shorttitle(3,3)`.
pub fn short_title(data: &ReferenceData) -> String {
    let Some(title) = data.title() else { return String::new() };
    let ascii = deunicode(title);
    let words = ascii.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty() && !is_skip_word(w));
    words.take(TITLE_WORDS).map(capitalise).collect()
}

/// `year`.
pub fn year(data: &ReferenceData) -> String {
    data.issued.map(|d| d.get_year().to_string()).unwrap_or_default()
}

/// ASCII letters and digits of `text` after transliteration.
fn ascii_alnum(text: &str) -> String {
    deunicode(text).chars().filter(char::is_ascii_alphanumeric).collect()
}

fn capitalise(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map(|first| first.to_ascii_uppercase().to_string() + chars.as_str()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_model::{Creator, Date, EntryKind, TextField};

    fn data(family: &str, title: &str, year: Option<u16>) -> ReferenceData {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.creators.push(Creator::person(CreatorRole::Author, family, Some("A.".into())));
        data.set(TextField::Title, title);
        data.issued = year.map(Date::year);
        data
    }

    #[test]
    fn default_formula() {
        assert_eq!(base_key(&data("Vaswani", "Attention Is All You Need", Some(2017))), "vaswaniAttentionAllYou2017");
        let nature = data("Kucsko", "Nanometre-scale thermometry in a living cell", Some(2013));
        assert_eq!(base_key(&nature), "kucskoNanometreScaleThermometry2013");
    }

    #[test]
    fn transliterates_and_strips() {
        assert_eq!(base_key(&data("Müller-Lüdenscheidt", "Über die Ästhetik", Some(1999))), "mullerludenscheidtUberAsthetik1999");
        assert_eq!(base_key(&data("O'Brien", "", None)), "obrien");
    }

    #[test]
    fn falls_back_to_editor_then_ref() {
        let mut edited = ReferenceData::new(EntryKind::Book);
        edited.creators.push(Creator::person(CreatorRole::Editor, "Roe", None));
        assert_eq!(auth_lower(&edited), "roe");
        assert_eq!(base_key(&ReferenceData::new(EntryKind::Book)), "ref");
    }

    #[test]
    fn organisation_author() {
        let mut d = ReferenceData::new(EntryKind::Report);
        d.creators.push(Creator::literal(CreatorRole::Author, "World Health Organization"));
        d.issued = Some(Date::year(2020));
        assert_eq!(base_key(&d), "worldhealthorganization2020");
    }
}
