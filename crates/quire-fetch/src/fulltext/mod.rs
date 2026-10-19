//! A paper's full text with its structure, for reader mode, from sources that have it (the PDF's
//! text has no structure: equations come out scrambled, captions mid-paragraph):
//!
//! - [`arxiv`]: arXiv's HTML version (LaTeXML), for arXiv papers; equations as their LaTeX.
//! - [`jats`]: Europe PMC's JATS XML, for open-access papers in PubMed Central; equations as
//!   TeX or MathML ([`mathml`] turns MathML into LaTeX).
//!
//! Both become the reader's Markdown, described in [`quire_io::paper_markdown`] (one place for
//! the format, shared with the PDF cleanup and the reader).

pub mod arxiv;
pub mod jats;
pub mod mathml;

use quire_model::{ArxivId, Identifier};
use serde_json::Value;
use url::Url;

use crate::error::{FetchError, Result};
use crate::fetcher::Fetcher;

/// Where a full text came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullTextSource {
    ArxivHtml,
    EuropePmc,
}

impl FullTextSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArxivHtml => "arxiv-html",
            Self::EuropePmc => "europepmc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullText {
    pub source: FullTextSource,
    pub markdown: String,
}

/// Shortest Markdown taken as a full text (an HTML page with only an error is shorter).
const MIN_LENGTH: usize = 2000;

impl Fetcher {
    /// The structured full text of the work with `identifiers`: arXiv's HTML for an arXiv ID,
    /// else Europe PMC's for an open-access paper (by PMCID, or found by DOI or PMID). `None`
    /// when neither has it.
    pub async fn full_text(&self, identifiers: &[Identifier]) -> Result<Option<FullText>> {
        let arxiv = identifiers.iter().find_map(|id| match id {
            Identifier::Arxiv(id) => Some(id),
            _ => None,
        });
        if let Some(id) = arxiv
            && let Some(markdown) = self.arxiv_html(id).await?
        {
            return Ok(Some(FullText { source: FullTextSource::ArxivHtml, markdown }));
        }
        let Some(pmcid) = self.open_access_pmcid(identifiers).await? else { return Ok(None) };
        let url = self.endpoints.europepmc.join(&format!("{pmcid}/fullTextXML")).map_err(|_| invalid())?;
        let xml = match self.http.get_text(&url, "application/xml").await {
            Ok(xml) => xml,
            Err(FetchError::NotFound(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        let markdown = jats::to_markdown(&xml).map_err(|e| FetchError::parse("europepmc", e))?;
        Ok((markdown.len() >= MIN_LENGTH).then_some(FullText { source: FullTextSource::EuropePmc, markdown }))
    }

    /// arXiv's HTML version of `id` as reader Markdown; `None` when arXiv has none (papers whose
    /// LaTeX it could not convert).
    async fn arxiv_html(&self, id: &ArxivId) -> Result<Option<String>> {
        let url = self.endpoints.arxiv_html.join(&id.to_string()).map_err(|_| invalid())?;
        let html = match self.http.get_text(&url, "text/html").await {
            Ok(html) => html,
            Err(FetchError::NotFound(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        let markdown = arxiv::to_markdown(&html);
        Ok((markdown.len() >= MIN_LENGTH).then_some(markdown))
    }

    /// The PMCID of the work, if it is in Europe PMC's open-access subset (whose full text it serves).
    async fn open_access_pmcid(&self, identifiers: &[Identifier]) -> Result<Option<String>> {
        let query = identifiers.iter().find_map(|id| match id {
            Identifier::Pmcid(pmcid) => Some(format!("PMCID:PMC{}", pmcid.get())),
            _ => None,
        });
        let query = query.or_else(|| {
            identifiers.iter().find_map(|id| match id {
                Identifier::Doi(doi) => Some(format!("DOI:\"{}\"", doi.as_str())),
                Identifier::Pmid(pmid) => Some(format!("EXT_ID:{} AND SRC:MED", pmid.get())),
                _ => None,
            })
        });
        let Some(query) = query else { return Ok(None) };
        let mut url: Url = self.endpoints.europepmc.join("search").map_err(|_| invalid())?;
        url.query_pairs_mut()
            .append_pair("query", &format!("{query} AND OPEN_ACCESS:y"))
            .append_pair("resultType", "lite")
            .append_pair("format", "json")
            .append_pair("pageSize", "1");
        let body = self.http.get_text(&url, "application/json").await?;
        let value: Value = serde_json::from_str(&body).map_err(|e| FetchError::parse("europepmc", e))?;
        Ok(value.pointer("/resultList/result/0/pmcid").and_then(Value::as_str).map(str::to_string))
    }
}

fn invalid() -> FetchError {
    FetchError::InvalidUrl("full text".into())
}

/// Builds the Markdown: blocks separated by blank lines, whitespace collapsed.
#[derive(Debug, Default)]
pub(crate) struct Writer {
    blocks: Vec<String>,
}

impl Writer {
    pub fn paragraph(&mut self, text: &str) {
        let text = collapse(text);
        if !text.is_empty() {
            self.blocks.push(text);
        }
    }

    pub fn heading(&mut self, level: usize, text: &str) {
        let text = collapse(text);
        if !text.is_empty() {
            self.blocks.push(format!("{} {text}", "#".repeat(level.clamp(2, 6))));
        }
    }

    /// `$$tex$$ (label)`.
    pub fn equation(&mut self, tex: &str, label: Option<&str>) {
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
    pub fn table(&mut self, rows: &[Vec<String>]) {
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

    pub fn list(&mut self, items: &[String]) {
        let items: Vec<String> = items.iter().map(|i| collapse(i)).filter(|i| !i.is_empty()).collect();
        if !items.is_empty() {
            self.blocks.push(items.iter().map(|i| format!("- {i}")).collect::<Vec<_>>().join("\n"));
        }
    }

    pub fn finish(self) -> String {
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
