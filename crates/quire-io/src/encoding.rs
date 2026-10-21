//! The character encoding an export file is written in (Zotero's "character encoding" option):
//! UTF-8, UTF-8 with a byte-order mark (some Windows tools look for one), or Windows-1252 (older
//! LaTeX setups with `\usepackage[cp1252]{inputenc}`).

use crate::error::{IoError, Result};
use crate::export::ExportFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Encoding {
    #[default]
    Utf8,
    /// UTF-8 starting with the byte-order mark `EF BB BF`.
    Utf8Bom,
    /// Windows-1252 (Western European). `.bib` only: a character it cannot hold is written as
    /// its LaTeX command.
    Windows1252,
}

impl Encoding {
    pub const ALL: &[Self] = &[Self::Utf8, Self::Utf8Bom, Self::Windows1252];

    /// Name in config files and on the command line.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
            Self::Utf8Bom => "utf-8-bom",
            Self::Windows1252 => "windows-1252",
        }
    }
}

impl std::str::FromStr for Encoding {
    type Err = IoError;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL.iter().copied().find(|e| e.as_str() == s).ok_or_else(|| IoError::UnknownEncoding(s.into()))
    }
}

/// `text` (an export in `format`) as `encoding` can hold it: for Windows-1252, every character it
/// lacks as its LaTeX command. What the file will say, shown before it is written.
pub fn fit(text: &str, encoding: Encoding, format: ExportFormat) -> Result<String> {
    if encoding != Encoding::Windows1252 {
        return Ok(text.to_string());
    }
    if !matches!(format, ExportFormat::Biblatex | ExportFormat::Bibtex) {
        return Err(IoError::Encoding(format!(
            "Windows-1252 is for .bib files; {} is written in UTF-8",
            format.as_str()
        )));
    }
    let mut out = String::with_capacity(text.len());
    let mut missing = Vec::new();
    for c in text.chars() {
        if windows_1252(c).is_some() {
            out.push(c);
            continue;
        }
        let command = crate::bib::to_latex(&c.to_string());
        if command.is_ascii() {
            out.push_str(&command);
        } else if !missing.contains(&c) {
            missing.push(c);
        }
    }
    if !missing.is_empty() {
        let shown: String = missing.iter().take(10).collect();
        return Err(IoError::Encoding(format!("Windows-1252 cannot hold {shown}, nor has LaTeX a command for it; use UTF-8")));
    }
    Ok(out)
}

/// `text` (already [`fit`]) as the file's bytes.
pub fn encode(text: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::Utf8 => text.as_bytes().to_vec(),
        Encoding::Utf8Bom => [&[0xEF, 0xBB, 0xBF][..], text.as_bytes()].concat(),
        // `fit` left only characters it holds; anything else would be a bug, written as `?`.
        Encoding::Windows1252 => text.chars().map(|c| windows_1252(c).unwrap_or(b'?')).collect(),
    }
}

/// The Windows-1252 byte of `c`, if it has one: ASCII and Latin-1 as they are, and the 27
/// characters at 0x80-0x9F (€, curly quotes, dashes…).
fn windows_1252(c: char) -> Option<u8> {
    const HIGH: [(char, u8); 27] = [
        ('€', 0x80), ('‚', 0x82), ('ƒ', 0x83), ('„', 0x84), ('…', 0x85), ('†', 0x86), ('‡', 0x87),
        ('ˆ', 0x88), ('‰', 0x89), ('Š', 0x8A), ('‹', 0x8B), ('Œ', 0x8C), ('Ž', 0x8E), ('‘', 0x91),
        ('’', 0x92), ('“', 0x93), ('”', 0x94), ('•', 0x95), ('–', 0x96), ('—', 0x97), ('˜', 0x98),
        ('™', 0x99), ('š', 0x9A), ('›', 0x9B), ('œ', 0x9C), ('ž', 0x9E), ('Ÿ', 0x9F),
    ];
    match c as u32 {
        0..=0x7F | 0xA0..=0xFF => Some(c as u8),
        _ => HIGH.iter().find(|(h, _)| *h == c).map(|&(_, b)| b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_with_and_without_the_byte_order_mark() {
        assert_eq!(encode("Müller", Encoding::Utf8), "Müller".as_bytes());
        assert_eq!(encode("Müller", Encoding::Utf8Bom), [b"\xEF\xBB\xBF", "Müller".as_bytes()].concat());
    }

    /// Latin letters are one byte each; what the code page lacks becomes LaTeX, so nothing is lost.
    #[test]
    fn windows_1252_holds_western_letters_and_writes_the_rest_as_latex() {
        // ü, ó, – and € are in the code page; Ł and ź are not.
        let text = fit("title = {Müller – Łódź €}", Encoding::Windows1252, ExportFormat::Biblatex).unwrap();
        assert_eq!(text, "title = {Müller – {\\L}ód{\\'z} €}");
    }

    #[test]
    fn windows_1252_bytes() {
        assert_eq!(encode("é–€", Encoding::Windows1252), vec![0xE9, 0x96, 0x80]);
    }

    #[test]
    fn what_neither_windows_1252_nor_latex_can_write_is_refused_not_lost() {
        let error = fit("title = {中文}", Encoding::Windows1252, ExportFormat::Bibtex).unwrap_err().to_string();
        assert!(error.contains("中文") && error.contains("UTF-8"), "{error}");
    }

    #[test]
    fn windows_1252_is_only_for_bib_files() {
        assert!(fit("[]", Encoding::Windows1252, ExportFormat::CslJson).is_err());
        assert_eq!(fit("[]", Encoding::Utf8Bom, ExportFormat::CslJson).unwrap(), "[]");
    }

    #[test]
    fn names_round_trip() {
        for encoding in Encoding::ALL {
            assert_eq!(encoding.as_str().parse::<Encoding>().unwrap(), *encoding);
        }
        assert!("latin-9".parse::<Encoding>().is_err());
    }
}
