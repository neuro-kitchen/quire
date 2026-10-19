//! Non-ASCII text as LaTeX commands, for `.bib` files read by classic BibTeX (Better BibTeX's
//! "export unicode as plain-text latex commands"): `é` → `{\'e}`, `ß` → `{\ss}`, `–` → `--`.
//! A letter is decomposed (NFD) into its base and accents; an accent LaTeX has a command for is
//! written as that command. Characters without a LaTeX form stay as they are.

use unicode_normalization::UnicodeNormalization;

/// LaTeX accent commands by combining mark.
fn accent(mark: char) -> Option<&'static str> {
    Some(match mark {
        '\u{300}' => "`",
        '\u{301}' => "'",
        '\u{302}' => "^",
        '\u{303}' => "~",
        '\u{304}' => "=",
        '\u{306}' => "u",
        '\u{307}' => ".",
        '\u{308}' => "\"",
        '\u{30A}' => "r",
        '\u{30B}' => "H",
        '\u{30C}' => "v",
        '\u{327}' => "c",
        '\u{328}' => "k",
        _ => return None,
    })
}

/// Letters and signs that are not a base letter plus accents.
fn special(c: char) -> Option<&'static str> {
    Some(match c {
        'ß' => "{\\ss}",
        'æ' => "{\\ae}",
        'Æ' => "{\\AE}",
        'œ' => "{\\oe}",
        'Œ' => "{\\OE}",
        'ø' => "{\\o}",
        'Ø' => "{\\O}",
        'ł' => "{\\l}",
        'Ł' => "{\\L}",
        'ı' => "{\\i}",
        'ð' => "{\\dh}",
        'Ð' => "{\\DH}",
        'þ' => "{\\th}",
        'Þ' => "{\\TH}",
        '–' => "--",
        '—' => "---",
        '‘' => "`",
        '’' => "'",
        '“' => "``",
        '”' => "''",
        '…' => "\\ldots{}",
        '\u{a0}' => "~",
        '§' => "{\\S}",
        '©' => "{\\textcopyright}",
        '°' => "{\\textdegree}",
        '×' => "{\\texttimes}",
        _ => return None,
    })
}

/// `text` with every non-ASCII character LaTeX can write replaced by its command.
pub fn to_latex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii() {
            out.push(c);
        } else if let Some(command) = special(c) {
            out.push_str(command);
        } else {
            out.push_str(&accented(c).unwrap_or_else(|| c.to_string()));
        }
    }
    out
}

/// `{\'e}` for `é`: an ASCII base letter with accents LaTeX knows, applied inside out.
fn accented(c: char) -> Option<String> {
    let mut parts = std::iter::once(c).nfd();
    let base = parts.next().filter(char::is_ascii_alphabetic)?;
    let mut written = match base {
        'i' => "\\i".to_string(),
        other => other.to_string(),
    };
    let mut any = false;
    for mark in parts {
        let command = accent(mark)?;
        let letter_command = command.chars().all(|c| c.is_ascii_alphabetic());
        written = if letter_command { format!("\\{command}{{{written}}}") } else { format!("\\{command}{written}") };
        any = true;
    }
    if !any {
        return None;
    }
    // A dotless i is only needed under an accent; plain `i` keeps its dot otherwise.
    Some(format!("{{{written}}}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_and_specials() {
        assert_eq!(to_latex("Gödel, Escher, Bach"), "G{\\\"o}del, Escher, Bach");
        assert_eq!(to_latex("Erdős"), "Erd{\\H{o}}s");
        assert_eq!(to_latex("façade naïve"), "fa{\\c{c}}ade na{\\\"\\i}ve");
        assert_eq!(to_latex("Straße – Łódź"), "Stra{\\ss}e -- {\\L}{\\'o}d{\\'z}");
        assert_eq!(to_latex("plain"), "plain");
        assert_eq!(to_latex("π ≈ 3"), "π ≈ 3", "no LaTeX text form: kept");
    }
}
