//! The Markdown of a paper's text for reader mode: what the producers write (arXiv HTML and
//! Europe PMC JATS converters in `quire-fetch`, the PDF cleanup in [`crate::pdf::tidy`]) and what
//! the reader reads (`quire-tui`). One place for the format, so they cannot drift apart:
//!
//! - `## 3 Model` headings (deeper sections, more `#`);
//! - paragraphs with `*italic*`, `**bold**`, `<sup>`/`<sub>`; inline math as `$LaTeX$` and display
//!   equations as their own `$$LaTeX$$ (1)` paragraph (full texts only; a literal dollar is `\$`);
//! - captions as their own paragraph starting with a label: `Figure 1: …`, `Fig. 2. …`,
//!   `Table 1 | …` ([`caption_label`]); a table's rows after its caption as `|a|b|` lines;
//! - a figure's image as its own `![Figure](https://…)` paragraph just before its caption
//!   ([`image_url`]; arXiv HTML only: PMC serves images behind a bot check);
//! - `- item` lists;
//! - a `## References` section ([`is_references_title`]) of `[1] citation` paragraphs;
//! - the front matter (journal header, authors, affiliations, history, copyright) between
//!   [`FRONT_START`] and [`FRONT_END`] (PDF text only).

/// The URL of an image paragraph (`![Figure](https://…)`), if `paragraph` is one.
pub(crate) fn image_url(paragraph: &str) -> Option<&str> {
    let rest = paragraph.trim().strip_prefix("![")?;
    let (_, rest) = rest.split_once("](")?;
    let url = rest.strip_suffix(')')?;
    (url.starts_with("https://") || url.starts_with("http://")).then_some(url)
}

/// An image paragraph for `url`.
pub(crate) fn image_paragraph(url: &str) -> String {
    format!("![Figure]({url})")
}

/// Marks the front matter section (Markdown comments: invisible to other renderers).
pub const FRONT_START: &str = "<!-- front -->";
pub(crate) const FRONT_END: &str = "<!-- /front -->";

/// What a caption labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionKind {
    Figure,
    Table,
}

impl CaptionKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Figure => "Figure",
            Self::Table => "Table",
        }
    }
}

/// The label a caption starts with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptionLabel {
    pub kind: CaptionKind,
    /// `1`, `2a`, `S3`.
    pub number: String,
    /// Bytes of the text the label takes, its punctuation and closing `**` included.
    pub len: usize,
}

impl CaptionLabel {
    /// `Figure 2`, `Table S1`.
    pub fn name(&self) -> String {
        format!("{} {}", self.kind.name(), self.number)
    }
}

/// Words that name a figure or a table, longest first.
const NAMES: &[(&str, CaptionKind)] = &[
    ("Figure", CaptionKind::Figure),
    ("FIGURE", CaptionKind::Figure),
    ("Fig.", CaptionKind::Figure),
    ("FIG.", CaptionKind::Figure),
    ("Fig", CaptionKind::Figure),
    ("Table", CaptionKind::Table),
    ("TABLE", CaptionKind::Table),
    ("Tab.", CaptionKind::Table),
];

/// Punctuation that ends a caption label.
const SEPARATORS: &[char] = &['.', ':', '|', '—', '–'];

/// The caption label `text` starts with, if it is a caption: a figure or table name, its
/// number, then punctuation (`Figure 1:`, `Fig. 2.`, `Table 1 |`), or a bold label alone
/// (`**Fig. 1** Adaptive…`). `Figure 2 shows…` is a sentence, not a caption.
pub fn caption_label(text: &str) -> Option<CaptionLabel> {
    let trimmed = text.trim_start();
    let leading = text.len() - trimmed.len();
    let bold = trimmed.starts_with("**");
    let body = trimmed.trim_start_matches('*');
    let mut at = leading + (trimmed.len() - body.len());
    let (kind, number, after_number) = name_and_number(body)?;
    at += body.len() - after_number.len();
    let closing = after_number.len() - after_number.trim_start_matches(['*', ' ']).len();
    let next = after_number[closing..].chars().next();
    let closes_bold = bold && after_number.trim_start().starts_with("**");
    if let Some(separator) = next.filter(|c| SEPARATORS.contains(c)) {
        at += closing + separator.len_utf8();
        // `**Figure 2:**`: the bold closes after the punctuation.
        let tail = &text[at..];
        at += tail.len() - tail.trim_start_matches('*').len();
    } else if closes_bold {
        // Up to and including the closing `**`.
        at += after_number.len() - after_number.trim_start().len() + 2;
    } else if next == Some('-') && after_number[closing + 1..].starts_with(char::is_uppercase) {
        // Old IEEE: `Fig. 1-General communications system.` (`Figs. 1-3` is a range).
        at += closing + 1;
    } else if starts_title(after_number) {
        // `Table 1 Summary of…`, `**Figure 4 Diaphragm CMAP…**` (Frontiers, BMC, Nature).
        at += after_number.len() - after_number.trim_start().len();
    } else {
        return None;
    }
    Some(CaptionLabel { kind, number, len: at })
}

/// After a label's number, a title rather than a sentence: a space, then a capitalised word of
/// two letters or more (`Figure 2 shows…` is a sentence, `Figure 2 A and B…` names panels).
fn starts_title(after_number: &str) -> bool {
    let Some(rest) = after_number.strip_prefix(' ') else { return false };
    let word: String = rest.trim_start().chars().take_while(|c| c.is_alphabetic()).collect();
    word.chars().next().is_some_and(char::is_uppercase) && word.chars().count() >= 2
}

/// The label if `text` is only a label (`Table 1`, `**Figure 2.**`): its caption is the next
/// paragraph.
pub(crate) fn label_alone(text: &str) -> Option<CaptionLabel> {
    let trimmed = text.trim();
    let body = trimmed.trim_start_matches('*');
    let (kind, number, after_number) = name_and_number(body)?;
    after_number.trim_matches(['*', ' ', '.', ':']).is_empty().then_some(CaptionLabel { kind, number, len: text.len() })
}

/// Whether `text` starts with a figure or table name and a number, punctuated or not (`Fig. 1`,
/// a bold heading that is a caption's label).
pub(crate) fn starts_with_caption_name(text: &str) -> bool {
    name_and_number(text.trim_start().trim_start_matches('*')).is_some()
}

/// (kind, number, the text after the number) for `Fig. 2a…`.
fn name_and_number(text: &str) -> Option<(CaptionKind, String, &str)> {
    let (name, kind) = NAMES.iter().find(|(name, _)| text.starts_with(name))?;
    let rest = &text[name.len()..];
    let digits_at = rest.len() - rest.trim_start().len();
    // `Fig` must be followed by a space or a number, not be the start of a word (`Figures`).
    if digits_at == 0 && !name.ends_with('.') && !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let candidate = &rest[digits_at..];
    let number: String = candidate.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    is_label_number(&number).then(|| (*kind, number.clone(), &candidate[number.len()..]))
}

/// `2`, `10`, `S1`, `3a`.
pub(crate) fn is_label_number(text: &str) -> bool {
    let digits = text.trim_start_matches(|c: char| c.is_ascii_uppercase());
    let digits = digits.trim_end_matches(|c: char| c.is_ascii_lowercase());
    !digits.is_empty() && digits.len() <= 3 && digits.chars().all(|c| c.is_ascii_digit()) && text.len() <= 5
}

/// A label as a publisher writes it, in reader form: `Fig 1` / `Fig. 1.` / `FIGURE 1` →
/// `Figure 1`; `Table 2.` → `Table 2`.
pub(crate) fn normalize_label(label: &str) -> String {
    let label = label.split_whitespace().collect::<Vec<_>>().join(" ");
    let label = label.trim_end_matches([':', '.']);
    let (name, number) = label.split_once(' ').unwrap_or((label, ""));
    let name = match name.trim_end_matches('.').to_lowercase().as_str() {
        "fig" | "figure" => "Figure",
        "table" | "tab" => "Table",
        _ => name,
    };
    format!("{name} {number}").trim().to_string()
}

/// `References`, `Bibliography`… as a section title (numbered or not, with Markdown marks).
pub fn is_references_title(text: &str) -> bool {
    let text: String = text.trim_start_matches('#').replace(['*', '_'], "");
    let text = text.trim().trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ' ').to_lowercase();
    matches!(text.as_str(), "references" | "reference" | "bibliography" | "literature cited" | "references and notes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_paragraphs() {
        let paragraph = image_paragraph("https://arxiv.org/html/1706.03762v7/Figures/ModalNet-21.png");
        assert_eq!(image_url(&paragraph), Some("https://arxiv.org/html/1706.03762v7/Figures/ModalNet-21.png"));
        assert_eq!(image_url("![x](relative.png)"), None);
        assert_eq!(image_url("Figure 1: not an image"), None);
    }

    fn label(text: &str) -> Option<(String, usize)> {
        caption_label(text).map(|l| (l.name(), l.len))
    }

    #[test]
    fn caption_labels() {
        assert_eq!(label("Figure 1: The model."), Some(("Figure 1".into(), 9)));
        assert_eq!(label("Fig. 3. Results."), Some(("Figure 3".into(), 7)));
        assert_eq!(label("**Fig. 1** Adaptive mouse."), Some(("Figure 1".into(), 10)));
        assert_eq!(label("**Figure 2:** A sample."), Some(("Figure 2".into(), 13)));
        assert_eq!(label("Fig. 1 | Overview."), Some(("Figure 1".into(), 8)));
        assert_eq!(label("Table S2: Supplementary."), Some(("Table S2".into(), 9)));
        assert_eq!(label("  FIGURE 3a. Detail"), Some(("Figure 3a".into(), 12)));
        assert_eq!(label("Table 1 Summary of the modes."), Some(("Table 1".into(), 8)));
        assert_eq!(label("**Figure 4 Diaphragm CMAP recorded**"), Some(("Figure 4".into(), 11)));
        assert_eq!(label("Fig. 1-General communications system."), Some(("Figure 1".into(), 7)));
        assert_eq!(label_alone("Table 1").map(|l| l.name()), Some("Table 1".into()));
        assert_eq!(label_alone("Table 1 Summary"), None);
        for sentence in [
            "Figure 2 shows the results.",
            "Table 1 summarizes",
            "Figures 1 and 2.",
            "In Fig. 3 we see",
            "Figure 2 A and B show",
            "Figs. 1-3 show",
            "Fig. 1-3 show",
        ] {
            assert_eq!(label(sentence), None, "{sentence}");
        }
        assert!(starts_with_caption_name("Fig. 1") && !starts_with_caption_name("Fighting"));
    }

    #[test]
    fn labels_and_titles() {
        assert_eq!(normalize_label("Fig. 1."), "Figure 1");
        assert_eq!(normalize_label("FIGURE 3"), "Figure 3");
        assert_eq!(normalize_label("Table 2:"), "Table 2");
        assert!(is_references_title("## 7. References"));
        assert!(is_references_title("**Bibliography**"));
        assert!(!is_references_title("## Results"));
    }
}
