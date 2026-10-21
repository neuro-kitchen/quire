//! A paper's highlights and notes as a Markdown file for a notes app (Obsidian, Logseq, any
//! Markdown folder): YAML front matter with the paper's details (Obsidian shows them as
//! properties, Logseq reads them as page properties), then the highlights as quotes with their
//! comments, then the notes.

use quire_model::{Highlight, Note, Reference, TextField};

/// The Markdown file for `reference`.
pub fn to_markdown(reference: &Reference, tags: &[String], highlights: &[Highlight], notes: &[Note]) -> String {
    let data = &reference.data;
    let title = data.title().unwrap_or("(no title)");
    let mut out = String::from("---\n");
    out.push_str(&format!("citekey: {}\n", yaml(reference.cite_key.as_str())));
    out.push_str(&format!("title: {}\n", yaml(title)));
    if !data.creators.is_empty() {
        out.push_str("authors:\n");
        for creator in &data.creators {
            out.push_str(&format!("  - {}\n", yaml(&creator.display_name())));
        }
    }
    if let Some(issued) = data.issued {
        out.push_str(&format!("year: {}\n", issued.get_year()));
    }
    if let Some(venue) = data.get(TextField::ContainerTitle) {
        out.push_str(&format!("venue: {}\n", yaml(venue)));
    }
    for identifier in &data.identifiers {
        let (kind, key) =
            identifier.to_string().split_once(':').map(|(k, v)| (k.to_lowercase(), v.to_string())).unwrap_or_default();
        out.push_str(&format!("{kind}: {}\n", yaml(&key)));
    }
    if !tags.is_empty() {
        out.push_str("tags:\n");
        for tag in tags {
            // Note apps' tags cannot hold spaces.
            out.push_str(&format!("  - {}\n", yaml(&tag.replace(' ', "-"))));
        }
    }
    out.push_str("---\n\n");
    out.push_str(&format!("# {title}\n"));
    if !highlights.is_empty() {
        out.push_str("\n## Highlights\n");
        for highlight in highlights {
            out.push('\n');
            for line in highlight.quote.lines() {
                out.push_str(&format!("> {line}\n"));
            }
            if let Some(comment) = &highlight.comment {
                out.push_str(&format!("\n{comment}\n"));
            }
        }
    }
    if !notes.is_empty() {
        out.push_str("\n## Notes\n");
        for note in notes {
            out.push_str(&format!("\n{}\n", note.body.trim()));
        }
    }
    out
}

/// A YAML scalar: plain when safe, else double-quoted with escapes.
fn yaml(text: &str) -> String {
    let plain = !text.is_empty()
        && !text.starts_with(|c: char| "-?:,[]{}#&*!|>'\"%@`".contains(c) || c.is_whitespace())
        && !text.ends_with(char::is_whitespace)
        && !text.contains(char::is_control)
        && !text.contains(": ")
        && !text.contains(" #")
        && !matches!(text.to_ascii_lowercase().as_str(), "true" | "false" | "null" | "yes" | "no" | "~")
        && text.parse::<f64>().is_err();
    if plain {
        return text.to_string();
    }
    let mut quoted = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            c if c.is_control() => quoted.push_str(&format!("\\u{:04X}", c as u32)),
            c => quoted.push(c),
        }
    }
    format!("\"{quoted}\"")
}

#[cfg(test)]
mod tests {
    use quire_model::{CiteKey, Creator, CreatorRole, Date, DeviceId, EntryKind, Identifier, Origin, ReferenceData};

    use super::*;

    /// A line break or tab in a value (a title pasted with its wrap) was written plain, which ends
    /// the YAML value early or breaks the front matter.
    #[test]
    fn control_characters_are_quoted() {
        assert_eq!(yaml("Line one\nLine two"), "\"Line one\\nLine two\"");
        assert_eq!(yaml("a\tb\r"), "\"a\\tb\\r\"");
        assert_eq!(yaml("Plain title"), "Plain title");
    }

    #[test]
    fn front_matter_highlights_notes() {
        let device = DeviceId::new();
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "Attention: is all you need");
        data.set(TextField::ContainerTitle, "NeurIPS");
        data.creators.push(Creator::person(CreatorRole::Author, "Vaswani", Some("Ashish".into())));
        data.issued = Some(Date::year(2017));
        data.identifiers.push(Identifier::parse("arXiv:1706.03762").unwrap());
        let reference = Reference::new(CiteKey::parse("vaswani2017").unwrap(), data, Origin::User, device);
        let highlight = Highlight::new(reference.id, "Attention is all\nyou need", "", Some("key claim"), device);
        let note = Note::new(Some(reference.id), "Read section 3 again.", device);
        let markdown = to_markdown(&reference, &["to read".into()], &[highlight], &[note]);
        assert_eq!(
            markdown,
            "---\ncitekey: vaswani2017\ntitle: \"Attention: is all you need\"\nauthors:\n  - Vaswani, Ashish\n\
             year: 2017\nvenue: NeurIPS\narxiv: \"1706.03762\"\ntags:\n  - to-read\n---\n\n\
             # Attention: is all you need\n\n## Highlights\n\n> Attention is all\n> you need\n\nkey claim\n\n\
             ## Notes\n\nRead section 3 again.\n"
        );
    }

    #[test]
    fn yaml_scalars() {
        assert_eq!(yaml("plain text"), "plain text");
        assert_eq!(yaml("2017"), "\"2017\"");
        assert_eq!(yaml("yes"), "\"yes\"");
        assert_eq!(yaml("#tag"), "\"#tag\"");
        assert_eq!(yaml("say \"hi\""), "say \"hi\"");
        assert_eq!(yaml("\"quoted\""), "\"\\\"quoted\\\"\"");
    }
}
