//! A paper's full text with its structure, from sources that have it (the PDF's text has no
//! structure: equations come out scrambled, captions mid-paragraph):
//!
//! - [`arxiv`]: arXiv's HTML version (LaTeXML), for arXiv papers; equations as their LaTeX.
//! - [`jats`]: Europe PMC's JATS XML, for open-access papers in PubMed Central; equations as
//!   TeX or MathML ([`mathml`] turns MathML into LaTeX).
//!
//! Both become the reader's Markdown, described in [`crate::paper_markdown`] (one place for the
//! format, shared with the PDF cleanup and the reader). Downloading them is `quire-fetch`'s job.

pub mod arxiv;
pub mod jats;
pub mod mathml;
pub(crate) mod xhtml;

/// Builds the Markdown: blocks separated by blank lines, whitespace collapsed.
#[derive(Debug, Default)]
pub(crate) struct Writer {
    blocks: Vec<String>,
    /// The page's URL, for resolving image links.
    pub base: Option<url::Url>,
}

impl Writer {
    pub(crate) fn paragraph(&mut self, text: &str) {
        let text = collapse(text);
        if !text.is_empty() {
            self.blocks.push(text);
        }
    }

    pub(crate) fn heading(&mut self, level: usize, text: &str) {
        let text = collapse(text);
        if !text.is_empty() {
            self.blocks.push(format!("{} {text}", "#".repeat(level.clamp(2, 6))));
        }
    }

    /// `$$tex$$ (label)`.
    pub(crate) fn equation(&mut self, tex: &str, label: Option<&str>) {
        let tex = collapse(tex);
        if tex.is_empty() {
            return;
        }
        let label = label.map(collapse).filter(|l| !l.is_empty());
        let label = label.map(|l| if l.starts_with('(') { l } else { format!("({l})") });
        self.blocks.push(match label {
            Some(label) => format!("$${tex}$$ {label}"),
            None => format!("$${tex}$$"),
        });
    }

    /// Rows as a pipe table (the first as its header).
    pub(crate) fn table(&mut self, rows: &[Vec<String>]) {
        let rows: Vec<String> = rows
            .iter()
            .filter(|r| !r.is_empty())
            .map(|r| format!("|{}|", r.iter().map(|c| collapse(c).replace('|', "/")).collect::<Vec<_>>().join("|")))
            .collect();
        let Some((header, body)) = rows.split_first() else { return };
        let columns = header.matches('|').count().saturating_sub(1);
        let rule = format!("|{}|", vec!["---"; columns].join("|"));
        let mut table = vec![header.clone(), rule];
        table.extend(body.iter().cloned());
        self.blocks.push(table.join("\n"));
    }

    pub(crate) fn list(&mut self, items: &[String]) {
        let items: Vec<String> = items.iter().map(|i| collapse(i)).filter(|i| !i.is_empty()).collect();
        if !items.is_empty() {
            self.blocks.push(items.iter().map(|i| format!("- {i}")).collect::<Vec<_>>().join("\n"));
        }
    }

    pub(crate) fn finish(self) -> String {
        let mut text = self.blocks.join("\n\n");
        text.push('\n');
        text
    }
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A literal `$` in text, escaped so it does not start math.
pub(crate) fn escape(text: &str) -> String {
    text.replace('$', "\\$")
}

/// `*text*` around the words of `text`, its outer spaces kept outside the markers.
pub(crate) fn wrap(marker: &str, text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return text.to_string();
    }
    let start = &text[..text.len() - text.trim_start().len()];
    let end = &text[text.trim_end().len()..];
    format!("{start}{marker}{trimmed}{marker}{end}")
}

/// The formula inside a `<tex-math>`, which often holds a whole LaTeX document
/// (`\documentclass…\begin{document}$$x$$\end{document}`).
pub(crate) fn tex_from_tex_math(text: &str) -> String {
    let body = match (text.find("\\begin{document}"), text.find("\\end{document}")) {
        (Some(start), Some(end)) if start < end => &text[start + "\\begin{document}".len()..end],
        _ => text,
    };
    let body = body.trim();
    let body = body.strip_prefix("$$").and_then(|b| b.strip_suffix("$$")).unwrap_or(body);
    let body = body.strip_prefix('$').and_then(|b| b.strip_suffix('$')).unwrap_or(body);
    let body = body.strip_prefix("\\[").and_then(|b| b.strip_suffix("\\]")).unwrap_or(body);
    collapse(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        assert_eq!(wrap("*", " strain "), " *strain* ");
        assert_eq!(
            tex_from_tex_math(
                r"\documentclass[12pt]{minimal}\usepackage{amsmath}\begin{document}$$x^2$$\end{document}"
            ),
            "x^2"
        );
        let mut w = Writer::default();
        w.equation("a = b", Some("1"));
        w.heading(9, "Deep");
        assert_eq!(w.finish(), "$$a = b$$ (1)\n\n###### Deep\n");
    }
}
