//! The paper's Markdown (from `pdf-inspector`) as blocks for the reader: headings, paragraphs,
//! list items, table rows. Inline `**bold**`, `*italic*`, `<sup>`/`<sub>` and links become styled
//! words; the arXiv margin stamp (`arXiv:1706.03762v7 [cs.CL] 2 Aug 2023`, read as a heading in
//! the middle of the text) is dropped.

use super::math::{self, SUBSCRIPTS, SUPERSCRIPTS};

/// How a word is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Emphasis {
    pub bold: bool,
    pub italic: bool,
    /// Names a figure, table or equation (`Figure 2`), set by [`super::paper::organize`].
    pub mention: bool,
}

/// A word with its emphasis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub emphasis: Emphasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Level 1–6.
    Heading(u8),
    Paragraph,
    ListItem,
    /// A table row, shown as written (not reflowed).
    TableRow,
    /// A display equation (as the PDF's text has it), set by [`super::paper::organize`].
    Equation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: Kind,
    pub words: Vec<Word>,
}

impl Block {
    /// The block's text, for search and the table of contents.
    pub fn text(&self) -> String {
        self.words.iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ")
    }
}

/// Words above which a "heading" is a line of text.
const MAX_HEADING_WORDS: usize = 14;

/// The blocks of `markdown`. With `math` (text from arXiv or Europe PMC, [`quire_fetch::fulltext`]),
/// `$…$` is inline LaTeX and a `$$…$$ (1)` paragraph a display equation, both shown as Unicode
/// ([`super::math`]); PDF text has no math markup, and a `$` there is a dollar.
pub fn parse(markdown: &str, math: bool) -> Vec<Block> {
    let mut blocks = Vec::new();
    for chunk in markdown.split("\n\n").map(str::trim).filter(|c| !c.is_empty()) {
        if chunk.lines().all(|l| l.trim_start().starts_with('|')) {
            for line in chunk.lines().filter(|l| !l.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))) {
                blocks.push(Block { kind: Kind::TableRow, words: cells(line, math) });
            }
            continue;
        }
        let list = chunk.lines().all(|l| is_list_line(l.trim_start()));
        if list {
            for line in chunk.lines() {
                let text = line.trim_start().trim_start_matches(['-', '*', '•']).trim_start();
                let text = text
                    .split_once(". ")
                    .filter(|(n, _)| n.chars().all(|c| c.is_ascii_digit()))
                    .map_or(text, |(_, rest)| rest);
                blocks.push(Block { kind: Kind::ListItem, words: inline(text, math) });
            }
            continue;
        }
        if math && chunk.starts_with("$$") {
            blocks.extend(display_equation(chunk));
            continue;
        }
        let level = chunk.chars().take_while(|c| *c == '#').count();
        let (mut kind, text) = match level {
            1..=6 => (Kind::Heading(level as u8), chunk[level..].trim()),
            _ => (Kind::Paragraph, chunk),
        };
        // The PDF converter reads some large text as headings: equations and long lines are not.
        if !math
            && kind != Kind::Paragraph
            && (text.contains('=') || text.split_whitespace().count() > MAX_HEADING_WORDS)
        {
            kind = Kind::Paragraph;
        }
        if kind != Kind::Paragraph && is_arxiv_stamp(text) {
            continue;
        }
        let words = inline(&text.replace('\n', " "), math);
        if !words.is_empty() {
            blocks.push(Block { kind, words });
        }
    }
    blocks
}

fn is_list_line(line: &str) -> bool {
    line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("• ")
        || line
            .split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.len() <= 3 && n.chars().all(|c| c.is_ascii_digit()))
}

/// `arXiv:1706.03762v7 [cs.CL] 2 Aug 2023`: arXiv's margin stamp.
fn is_arxiv_stamp(text: &str) -> bool {
    let text = text.trim_matches(['*', '_', ' ']);
    text.starts_with("arXiv:") && text.contains('[') && text.split_whitespace().count() <= 6
}

/// `$$LaTeX$$ (1)`: one equation block per row of the formula, the number after the last.
fn display_equation(chunk: &str) -> Vec<Block> {
    let chunk = chunk.replace('\n', " ");
    let body = &chunk[2..];
    let (tex, rest) = body.split_once("$$").unwrap_or((body, ""));
    let rows = math::to_unicode(tex);
    let mut rows: Vec<String> = rows.lines().map(str::to_string).collect();
    if let (Some(last), number) = (rows.last_mut(), rest.trim())
        && !number.is_empty()
    {
        last.push_str("   ");
        last.push_str(number);
    }
    rows.into_iter()
        .map(|row| Block { kind: Kind::Equation, words: row.split_whitespace().map(plain).collect() })
        .collect()
}

fn plain(text: &str) -> Word {
    Word { text: text.to_string(), emphasis: Emphasis::default() }
}

/// Words of `text`; with `math`, `$…$` spans become Unicode words (no emphasis parsing inside),
/// glued to the text around them when there is no space (`$x$-axis` → `x-axis`).
pub fn inline(text: &str, math: bool) -> Vec<Word> {
    if !math {
        return words(text);
    }
    let mut out: Vec<Word> = Vec::new();
    let mut emphasis = Emphasis::default();
    let mut glue = false;
    for (is_math, segment) in math_segments(text) {
        let mut new: Vec<Word> = if is_math {
            math::to_unicode(&segment).split_whitespace().map(plain).collect()
        } else {
            words_with(&segment, &mut emphasis)
        };
        // Emphasis marks are not text: `and *$x$*` has a space before the math.
        let bare = segment.trim_matches(['*', '_']);
        if glue
            && !bare.starts_with(char::is_whitespace)
            && !new.is_empty()
            && let Some(last) = out.last_mut()
        {
            last.text.push_str(&new.remove(0).text);
        }
        out.extend(new);
        glue = !bare.ends_with(char::is_whitespace);
    }
    out
}

/// `text` cut into (is math, piece): `$…$` spans and the text between (`\$` is a dollar).
fn math_segments(text: &str) -> Vec<(bool, String)> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_math = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'$') => {
                chars.next();
                current.push(if in_math { ' ' } else { '$' });
            }
            '$' => {
                segments.push((in_math, std::mem::take(&mut current)));
                in_math = !in_math;
            }
            _ => current.push(c),
        }
    }
    // An unclosed `$` was a dollar after all.
    if in_math {
        let open = segments.pop().map(|(_, text)| text).unwrap_or_default();
        current = format!("{open}${current}");
        in_math = false;
    }
    segments.push((in_math, current));
    segments.retain(|(_, text)| !text.is_empty());
    segments
}

/// A table row's cells (one word each), without emphasis marks.
fn cells(line: &str, math: bool) -> Vec<Word> {
    let line = line.trim().trim_start_matches('|').trim_end_matches('|');
    line.split('|')
        .map(|cell| {
            let text = inline(cell, math).into_iter().map(|w| w.text).collect::<Vec<_>>().join(" ");
            Word { text, emphasis: Emphasis::default() }
        })
        .collect()
}

/// The words of a paragraph, with emphasis from `**`/`*`/`_` markers, `<sup>`/`<sub>` as
/// superscript/subscript characters where Unicode has them (else `^x` / `_x`), links as their text,
/// `<u>` (the converter's underline, often a fraction bar) dropped.
pub fn words(text: &str) -> Vec<Word> {
    words_with(text, &mut Emphasis::default())
}

/// [`words`] continuing from the emphasis open before `text`.
fn words_with(text: &str, emphasis: &mut Emphasis) -> Vec<Word> {
    let text = scripts(&links(&text.replace("<u>", "").replace("</u>", "")));
    let mut out = Vec::new();
    for raw in text.split_whitespace() {
        let mut word = String::new();
        let mut chars = raw.chars().peekable();
        let mut start = *emphasis;
        let mut first = true;
        let mut previous: Option<char> = None;
        while let Some(c) = chars.next() {
            // A marker next to a non-letter opens or closes (`(*x*)`); between letters, `_` is a
            // letter (`d_model`) and `*` marks only when it pairs up (`d*model*`).
            let edge =
                previous.is_none_or(|p| !p.is_alphanumeric()) || chars.peek().is_none_or(|n| !n.is_alphanumeric());
            let pairs = emphasis.italic || chars.clone().any(|n| n == '*');
            match c {
                '*' if chars.peek() == Some(&'*') => {
                    chars.next();
                    emphasis.bold = !emphasis.bold;
                }
                '*' | '_' if edge || (c == '*' && pairs) => emphasis.italic = !emphasis.italic,
                other => {
                    previous = Some(other);
                    if first {
                        start = *emphasis;
                        first = false;
                    }
                    word.push(other);
                }
            }
        }
        if !word.is_empty() {
            out.push(Word { text: word, emphasis: start });
        }
    }
    out
}

/// `[label](target)` → `label`.
fn links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let Some(mid) = rest[open..].find(']') else { break };
        // `[13]` is a citation, not a link: keep it.
        let target = &rest[open + mid + 1..];
        let close = target.strip_prefix('(').and_then(|t| t.find(')'));
        let Some(close) = close else {
            out.push_str(&rest[..open + mid + 1]);
            rest = target;
            continue;
        };
        out.push_str(&rest[..open]);
        out.push_str(&rest[open + 1..open + mid]);
        rest = &target[close + 2..];
    }
    out.push_str(rest);
    out
}

/// `<sup>2</sup>` → `²`, `<sub>t</sub>` → `ₜ`; characters Unicode lacks keep a `^` / `_` mark.
fn scripts(text: &str) -> String {
    let mut out = text.to_string();
    for (tag, map) in [("sup", SUPERSCRIPTS), ("sub", SUBSCRIPTS)] {
        let (open, close) = (format!("<{tag}>"), format!("</{tag}>"));
        while let Some(start) = out.find(&open) {
            let Some(end) = out[start..].find(&close) else { break };
            let inner = out[start + open.len()..start + end].replace('*', "");
            let mapped: Option<String> =
                inner.chars().map(|c| map.iter().find(|(from, _)| *from == c).map(|(_, to)| *to)).collect();
            let replacement = mapped.unwrap_or_else(|| format!("{}{inner}", if tag == "sup" { '^' } else { '_' }));
            out.replace_range(start..start + end + close.len(), &replacement);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_and_cleanup() {
        let md = "## Attention Is All You Need\n\n**Ashish Vaswani<sup>∗</sup>** Google\n\n#### 1 Introduction\n\nRecurrent *neural* networks [13] see [the paper](https://x.org).\n\n# arXiv:1706.03762v7 [cs.CL] 2 Aug 2023\n\n- first\n- second\n\n| a | b |\n|---|---|\n| 1 | 2 |";
        let blocks = parse(md, false);
        let kinds: Vec<Kind> = blocks.iter().map(|b| b.kind).collect();
        assert_eq!(
            kinds,
            [
                Kind::Heading(2),
                Kind::Paragraph,
                Kind::Heading(4),
                Kind::Paragraph,
                Kind::ListItem,
                Kind::ListItem,
                Kind::TableRow,
                Kind::TableRow
            ]
        );
        assert_eq!(blocks[1].words[1].text, "Vaswani∗");
        assert!(blocks[1].words[0].emphasis.bold);
        assert_eq!(blocks[3].text(), "Recurrent neural networks [13] see the paper.");
        assert!(blocks[3].words[1].emphasis.italic && !blocks[3].words[2].emphasis.italic);
        let maths = words("LayerNorm(*x* + f(*x*)) of d*model* and d_k, ## not");
        let text: Vec<&str> = maths.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(text, ["LayerNorm(x", "+", "f(x))", "of", "dmodel", "and", "d_k,", "##", "not"]);
        assert!(!maths[5].emphasis.italic, "closed after d*model*");
        assert_eq!(parse("## MultiHead(Q,K,V) = Concat(head)", false)[0].kind, Kind::Paragraph);
    }

    #[test]
    fn math_from_full_texts() {
        let md = "States $h_{t}$-wise cost \\$5 and *$x$*.\n\n$$a=b \\\\ \\frac{c}{d+1}=e$$ (2)\n\n|$n^{2}$|x|";
        let blocks = parse(md, true);
        assert_eq!(blocks[0].text(), "States hₜ-wise cost $5 and x.");
        assert_eq!(blocks[1].kind, Kind::Equation);
        assert_eq!((blocks[1].text(), blocks[2].text()), ("a = b".into(), "c/(d+1) = e (2)".into()));
        assert_eq!(blocks[3].words[0].text, "n²");
        assert_eq!(parse("Costs $5 and $6.", false)[0].text(), "Costs $5 and $6.", "PDF text: dollars");
        assert_eq!(parse("Costs $5 only.", true)[0].text(), "Costs $5 only.", "an unclosed $ is a dollar");
    }

    #[test]
    fn super_and_subscripts() {
        assert_eq!(scripts("h<sub>t−1</sub> and T<sup>2</sup> and x<sup>ab</sup>"), "hₜ₋₁ and T² and xᵃᵇ");
        assert_eq!(scripts("y<sub>q</sub> m<sup>-1</sup>"), "y_q m⁻¹");
    }
}
