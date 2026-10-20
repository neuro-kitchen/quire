//! The cite keys a manuscript uses (`quire folder check`, the cite-key language server): Typst
//! (`@key`, `#cite(<key>)`), LaTeX (`\cite{a,b}`, natbib and biblatex variants) and Pandoc
//! Markdown (`@key`, `[@a; -@b]`, `@{key}`). Comments and code are skipped, and so are
//! references to labels the document defines itself (`<fig-a>` in Typst, `{#fig:a}` in
//! Markdown, `\label{…}` in LaTeX), which use the same `@` syntax.

use std::collections::HashSet;
use std::path::Path;

/// The markup of a manuscript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Markup {
    Typst,
    Latex,
    Markdown,
}

impl Markup {
    /// The markup a file's extension names (`.typ`, `.tex`/`.ltx`, `.md`/`.qmd`/`.Rmd`).
    pub fn of(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "typ" => Some(Self::Typst),
            "tex" | "ltx" => Some(Self::Latex),
            "md" | "markdown" | "qmd" | "rmd" => Some(Self::Markdown),
            _ => None,
        }
    }
}

/// One use of a cite key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub key: String,
    /// 1-based line.
    pub line: usize,
    /// 0-based column, in characters, of the key's first character.
    pub column: usize,
}

/// Every cite key `text` uses, in order (repeats included).
pub fn citations(text: &str, markup: Markup) -> Vec<Citation> {
    let found = match markup {
        Markup::Typst => typst(text),
        Markup::Latex => latex(text),
        Markup::Markdown => markdown(text),
    };
    let lines = LineIndex::new(text);
    found.into_iter().map(|(offset, key)| lines.citation(text, offset, key)).collect()
}

/// A key found at a byte offset.
type Found = Vec<(usize, String)>;

fn typst(text: &str) -> Found {
    let text = blank_out(text, &[("//", "\n"), ("/*", "*/"), ("```", "```"), ("`", "`")], |text, at| {
        // `https://…` is a link in Typst markup, not a comment.
        text[..at].ends_with(':')
    });
    let mut labels = HashSet::new();
    let mut found = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => {
                let start = i + 1;
                let end = start + label_len(&text[start..], is_typst_key_char);
                if end > start && bytes.get(end) == Some(&b'>') {
                    let key = text[start..end].to_string();
                    if text[..i].trim_end().ends_with("cite(") {
                        found.push((start, key));
                    } else {
                        labels.insert(key);
                    }
                    i = end;
                }
            }
            b'@' if !escaped_or_word(&text, i) => {
                let start = i + 1;
                let key = trim_key(&text[start..start + label_len(&text[start..], is_typst_key_char)]);
                if !key.is_empty() {
                    found.push((start, key.to_string()));
                }
            }
            _ => {}
        }
        i += 1;
    }
    found.retain(|(_, key)| !labels.contains(key));
    found
}

fn latex(text: &str) -> Found {
    let text = blank_out(text, &[("%", "\n")], |text, at| text[..at].ends_with('\\'));
    let labels: HashSet<String> = command_arguments(&text, |name| name == "label")
        .into_iter()
        .flat_map(|(_, group)| split_keys(&group).into_iter().map(|(_, key)| key))
        .collect();
    let mut found = Vec::new();
    for (offset, group) in command_arguments(&text, |name| name.to_ascii_lowercase().contains("cite")) {
        for (at, key) in split_keys(&group) {
            if key != "*" && !labels.contains(&key) {
                found.push((offset + at, key));
            }
        }
    }
    found
}

/// The `{…}` arguments (offset of their content, content) of every command whose name `wanted`
/// accepts, skipping `*`, `[…]` and `(…)` options; commands ending in `s` (`\cites`,
/// `\parencites`) take several.
fn command_arguments(text: &str, wanted: impl Fn(&str) -> bool) -> Vec<(usize, String)> {
    let bytes = text.as_bytes();
    let mut arguments = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            i += 1;
            continue;
        }
        let name_start = i + 1;
        let mut j = name_start;
        while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
            j += 1;
        }
        let name = &text[name_start..j];
        if name.is_empty() || !wanted(name) {
            i = j.max(i + 1);
            continue;
        }
        let several = name.ends_with('s');
        if bytes.get(j) == Some(&b'*') {
            j += 1;
        }
        loop {
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            match bytes.get(j) {
                Some(b'[') => j = closing(bytes, j, b'[', b']'),
                Some(b'(') => j = closing(bytes, j, b'(', b')'),
                Some(b'{') => {
                    let end = closing(bytes, j, b'{', b'}');
                    arguments.push((j + 1, text[j + 1..end.saturating_sub(1).max(j + 1)].to_string()));
                    j = end;
                    if !several {
                        break;
                    }
                }
                _ => break,
            }
        }
        i = j.max(i + 1);
    }
    arguments
}

/// The index after the bracket matching the one at `open_at`.
fn closing(bytes: &[u8], open_at: usize, open: u8, close: u8) -> usize {
    let mut depth = 0;
    for (k, &byte) in bytes.iter().enumerate().skip(open_at) {
        if byte == open {
            depth += 1;
        } else if byte == close {
            depth -= 1;
            if depth == 0 {
                return k + 1;
            }
        }
    }
    bytes.len()
}

/// `a, b ,c` → the keys with their offsets in `group`.
fn split_keys(group: &str) -> Vec<(usize, String)> {
    let mut keys = Vec::new();
    let mut offset = 0;
    for part in group.split(',') {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            keys.push((offset + part.find(trimmed).unwrap_or(0), trimmed.to_string()));
        }
        offset += part.len() + 1;
    }
    keys
}

fn markdown(text: &str) -> Found {
    let text = blank_out(text, &[("```", "```"), ("~~~", "~~~"), ("`", "`"), ("<!--", "-->")], |_, _| false);
    let bytes = text.as_bytes();
    let mut labels = HashSet::new();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            // `{#fig:a}`, `{#sec:intro .unnumbered}`: an element's own label.
            b'{' if bytes.get(i + 1) == Some(&b'#') => {
                let start = i + 2;
                let len = label_len(&text[start..], is_pandoc_key_char);
                labels.insert(trim_key(&text[start..start + len]).to_string());
            }
            b'@' if !escaped_or_word(&text, i) => {
                let start = i + 1;
                if bytes.get(start) == Some(&b'{') {
                    let end = closing(bytes, start, b'{', b'}');
                    let key = text[start + 1..end.saturating_sub(1).max(start + 1)].trim();
                    if !key.is_empty() {
                        found.push((start + 1, key.to_string()));
                    }
                } else if text[start..].starts_with(|c: char| c.is_alphanumeric() || c == '_') {
                    let key = trim_key(&text[start..start + label_len(&text[start..], is_pandoc_key_char)]);
                    found.push((start, key.to_string()));
                }
            }
            _ => {}
        }
        i += 1;
    }
    found.retain(|(_, key)| !labels.contains(key));
    found
}

fn is_typst_key_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | ':' | '.')
}

/// Pandoc: letters, digits and `_`, with `:.#$%&-+?<>~/` inside.
fn is_pandoc_key_char(c: char) -> bool {
    c.is_alphanumeric()
        || matches!(c, '_' | ':' | '.' | '#' | '$' | '%' | '&' | '-' | '+' | '?' | '<' | '>' | '~' | '/')
}

/// Bytes of the leading run of `text` whose characters `allowed` accepts.
fn label_len(text: &str, allowed: fn(char) -> bool) -> usize {
    text.find(|c: char| !allowed(c)).unwrap_or(text.len())
}

/// Punctuation that ends a sentence is not part of a key (`see @smith2020.`).
fn trim_key(key: &str) -> &str {
    key.trim_end_matches(|c: char| !(c.is_alphanumeric() || c == '_'))
}

/// `\@` (escaped) or `name@host` (an email address): not a citation.
fn escaped_or_word(text: &str, at: usize) -> bool {
    text[..at].chars().next_back().is_some_and(|c| c == '\\' || c.is_alphanumeric())
}

/// `text` with every span from an opener to its closer (comments, code) and every URL replaced
/// by spaces (one per byte, newlines kept), so offsets and line numbers stay valid.
/// `not_here(text, at)`: an opener at `at` that does not count.
fn blank_out(text: &str, spans: &[(&str, &str)], not_here: impl Fn(&str, usize) -> bool) -> String {
    let mut out = String::with_capacity(text.len());
    let blank = |out: &mut String, span: &str| {
        for c in span.chars() {
            if c == '\n' {
                out.push('\n');
            } else {
                out.extend(std::iter::repeat_n(' ', c.len_utf8()));
            }
        }
    };
    let mut i = 0;
    'scan: while i < text.len() {
        let rest = &text[i..];
        if rest.starts_with("https://") || rest.starts_with("http://") {
            let end = i + rest.find(char::is_whitespace).unwrap_or(rest.len());
            blank(&mut out, &text[i..end]);
            i = end;
            continue;
        }
        for (open, close) in spans {
            if rest.starts_with(open) && !not_here(text, i) {
                let body = i + open.len();
                let end = text[body..].find(close).map_or(text.len(), |k| body + k + close.len());
                blank(&mut out, &text[i..end]);
                i = end;
                continue 'scan;
            }
        }
        let c = rest.chars().next().unwrap_or(' ');
        out.push(c);
        i += c.len_utf8();
    }
    out
}

/// Byte offsets → line and column.
struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let starts = std::iter::once(0).chain(text.match_indices('\n').map(|(i, _)| i + 1)).collect();
        Self { starts }
    }

    fn citation(&self, text: &str, offset: usize, key: String) -> Citation {
        let line = self.starts.partition_point(|&start| start <= offset);
        let start = self.starts[line - 1];
        let column = text.get(start..offset).map_or(0, |prefix| prefix.chars().count());
        Citation { key, line, column }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(text: &str, markup: Markup) -> Vec<String> {
        citations(text, markup).into_iter().map(|c| c.key).collect()
    }

    #[test]
    fn typst_citations() {
        let text = r#"= Intro <intro>
As @smith2020 showed (see @jones-2019[p. 4]) and #cite(<lee:2018>), unlike @intro.
Mail me@example.com or \@escaped. See @fig-a.
// @commented
#figure(image("a.png"), caption: [A]) <fig-a>
Link https://example.com/@user and `@code`.
/* @block
   comment */ @last.
"#;
        assert_eq!(keys(text, Markup::Typst), ["smith2020", "jones-2019", "lee:2018", "last"]);
    }

    #[test]
    fn latex_citations() {
        let text = r"\section{Intro}\label{sec:intro}
See \cite{smith2020, jones2019} and \citep[p.~4]{lee2018}.
\parencite*[see][12]{kim2021} \textcites(pre)(post)[a]{one}[b]{two}
% \cite{commented}
50\% of \nocite{*} cases \autocite{last}
";
        assert_eq!(keys(text, Markup::Latex), ["smith2020", "jones2019", "lee2018", "kim2021", "one", "two", "last"]);
    }

    #[test]
    fn markdown_citations() {
        let text = "As @smith2020 says [@jones2019, p. 4; -@lee2018].\n\
                    ![A figure](a.png){#fig:a}\n\
                    See @fig:a and @{odd key}.\n\
                    ```\n@code\n```\n\
                    Email me@example.com. Ends with @last.\n";
        assert_eq!(keys(text, Markup::Markdown), ["smith2020", "jones2019", "lee2018", "odd key", "last"]);
    }

    #[test]
    fn lines_and_columns() {
        let text = "Über @a\nthen\n  @b";
        let found = citations(text, Markup::Typst);
        assert_eq!(found[0], Citation { key: "a".into(), line: 1, column: 6 });
        assert_eq!(found[1], Citation { key: "b".into(), line: 3, column: 3 });
    }

    #[test]
    fn comments_keep_offsets_with_unicode() {
        let text = "// é ü\n@after";
        assert_eq!(citations(text, Markup::Typst)[0], Citation { key: "after".into(), line: 2, column: 1 });
    }

    #[test]
    fn markup_from_extension() {
        assert_eq!(Markup::of(Path::new("paper.typ")), Some(Markup::Typst));
        assert_eq!(Markup::of(Path::new("thesis.TEX")), Some(Markup::Latex));
        assert_eq!(Markup::of(Path::new("notes.qmd")), Some(Markup::Markdown));
        assert_eq!(Markup::of(Path::new("refs.bib")), None);
    }
}
