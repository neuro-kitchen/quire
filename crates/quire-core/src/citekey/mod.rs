//! Cite keys as Better BibTeX makes them (`.knowledge/library-model.md`): generated once from the
//! default formula `auth.lower + shorttitle(3,3) + year`, made unique across the library with a
//! letter postfix (`a`, `b`, …), and never changed automatically afterwards.

mod formula;
mod skipwords;

use quire_model::{CiteKey, ReferenceData};

pub use formula::{auth_lower, base_key, short_title, year};

use crate::error::Result;

/// A key for `data` that `taken` does not report as used: the formula's key, else the first free
/// of `key` + `a`, `b`, …, `z`, `aa`, `ab`, …
pub(crate) fn unique_key(data: &ReferenceData, taken: impl Fn(&str) -> Result<bool>) -> Result<CiteKey> {
    free_key(&base_key(data), taken)
}

/// The first free key starting from `base` (used as is when free; postfixed keys are cut to the
/// longest length allowed).
pub fn free_key(base: &str, taken: impl Fn(&str) -> Result<bool>) -> Result<CiteKey> {
    if !taken(base)? {
        return Ok(CiteKey::parse(base)?);
    }
    for n in 0.. {
        // The postfix replaces the base's end when the two would be too long together.
        let postfix = postfix(n);
        let mut keep = base.len().min(CiteKey::MAX_LEN - postfix.len());
        while !base.is_char_boundary(keep) {
            keep -= 1;
        }
        let candidate = format!("{}{postfix}", &base[..keep]);
        if !taken(&candidate)? {
            return Ok(CiteKey::parse(&candidate)?);
        }
    }
    unreachable!("the postfix sequence is infinite")
}

/// `a`, `b`, …, `z`, `aa`, `ab`, … for 0, 1, …
fn postfix(mut n: usize) -> String {
    let mut letters = Vec::new();
    loop {
        letters.push(b'a' + (n % 26) as u8);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    letters.reverse();
    String::from_utf8(letters).expect("ASCII letters")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A taken key of the longest length allowed (a `.bib` file's own key) once failed: its
    /// postfixed form was too long, so the whole import stopped.
    #[test]
    fn a_long_taken_key_still_gets_a_free_one() {
        let long = "k".repeat(100);
        let key = free_key(&long, |k| Ok(k == long)).unwrap();
        assert_eq!((key.as_str().len(), key.as_str().ends_with('a')), (100, true));
        let shorter = free_key(&long[..99], |k| Ok(k.len() == 99)).unwrap();
        assert_eq!(shorter.as_str(), format!("{}a", &long[..99]));
    }
    use std::collections::HashSet;

    #[test]
    fn postfix_sequence() {
        let got: Vec<String> = [0, 1, 25, 26, 27, 51, 52].into_iter().map(postfix).collect();
        assert_eq!(got, ["a", "b", "z", "aa", "ab", "az", "ba"]);
    }

    #[test]
    fn clashes_get_letters() {
        let used: HashSet<&str> = ["smith2020", "smith2020a"].into();
        let key = free_key("smith2020", |k| Ok(used.contains(k))).unwrap();
        assert_eq!(key.as_str(), "smith2020b");
        assert_eq!(free_key("doe2021", |k| Ok(used.contains(k))).unwrap().as_str(), "doe2021");
    }
}
