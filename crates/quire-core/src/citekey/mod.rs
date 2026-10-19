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
pub fn unique_key(data: &ReferenceData, taken: impl Fn(&str) -> Result<bool>) -> Result<CiteKey> {
    free_key(&base_key(data), taken)
}

/// The first free key starting from `base` (used as is when free).
pub fn free_key(base: &str, taken: impl Fn(&str) -> Result<bool>) -> Result<CiteKey> {
    if !taken(base)? {
        return Ok(CiteKey::parse(base)?);
    }
    for n in 0.. {
        let candidate = format!("{base}{}", postfix(n));
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
