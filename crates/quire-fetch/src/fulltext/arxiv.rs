//! arXiv's HTML papers (LaTeXML, `https://arxiv.org/html/<id>`) → reader Markdown ([`quire_io::paper_markdown`]
//! has the format). Seen on 1706.03762v7 (2026-10-19): `<article class="ltx_document">` holds the
//! title, authors and abstract (left out: the reader shows them from the library), then
//! `<section class="ltx_section">`s with `<h2>`–`<h6>` titles, `<div class="ltx_para">` of
//! `<p class="ltx_p">`, `<math alttext="LaTeX">`, equations as `<table class="ltx_equation">`
//! with the number in `.ltx_tag_equation`, `<figure class="ltx_figure|ltx_table">` with
//! `<figcaption>` (and a `<table class="ltx_tabular">`), footnotes as `.ltx_note`, and the
//! bibliography as `<li class="ltx_bibitem">`.

use scraper::{ElementRef, Html, Node};
use url::Url;

use super::Writer;
use quire_io::paper_markdown::{image_paragraph, normalize_label};

/// The paper's sections and references as reader Markdown; figure images (`<img src>`) as URLs
/// resolved against the page's `url`.
pub fn to_markdown(html: &str, url: &Url) -> String {
    let document = Html::parse_document(html);
    let mut out = Writer { base: Some(url.clone()), ..Writer::default() };
    let selector = scraper::Selector::parse("article.ltx_document").expect("valid selector");
    if let Some(article) = document.select(&selector).next() {
        blocks(article, &mut out);
    }
    out.finish()
}

fn has_class(element: ElementRef, class: &str) -> bool {
    element.value().has_class(class, scraper::CaseSensitivity::CaseSensitive)
}

fn child_elements(element: ElementRef) -> impl Iterator<Item = ElementRef> {
    element.children().filter_map(ElementRef::wrap)
}

fn blocks(element: ElementRef, out: &mut Writer) {
    for child in child_elements(element) {
        let name = child.value().name();
        match name {
            "h1" | "nav" | "header" | "footer" | "button" => {}
            "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = name[1..].parse().unwrap_or(2);
                out.heading(level, &inline(child));
            }
            "p" => out.paragraph(&inline(child)),
            "figure" if has_class(child, "ltx_table") => table(child, out),
            "figure" => figure(child, out),
            "table" if has_class(child, "ltx_equation") || has_class(child, "ltx_equationgroup") => {
                equations(child, out)
            }
            "table" if has_class(child, "ltx_tabular") => out.table(&rows(child)),
            "ul" | "ol" if has_class(child, "ltx_biblist") => bibliography(child, out),
            "ul" | "ol" => {
                let items: Vec<String> = child_elements(child).map(inline).collect();
                out.list(&items);
            }
            _ if skipped(child) => {}
            "section" if has_class(child, "ltx_bibliography") => {
                out.heading(2, "References");
                for list in child_elements(child).filter(|c| matches!(c.value().name(), "ul" | "ol")) {
                    bibliography(list, out);
                }
            }
            _ => blocks(child, out),
        }
    }
}

/// Front matter and notes the reader does not show.
fn skipped(element: ElementRef) -> bool {
    ["ltx_authors", "ltx_abstract", "ltx_dates", "ltx_note", "ltx_classification", "ltx_keywords", "ltx_pagination"]
        .iter()
        .any(|class| has_class(element, class))
}

/// The figure's own caption (not a sub-figure's).
fn caption(figure: ElementRef) -> String {
    child_elements(figure).filter(|c| c.value().name() == "figcaption").map(inline).last().unwrap_or_default()
}

fn figure(element: ElementRef, out: &mut Writer) {
    let selector = scraper::Selector::parse("img").expect("valid selector");
    let image = element.select(&selector).find_map(|img| img.value().attr("src"));
    if let Some(url) = image.and_then(|src| out.base.as_ref()?.join(src).ok()) {
        out.paragraph(&image_paragraph(url.as_str()));
    }
    let caption = caption(element);
    out.paragraph(&normalize_caption(&caption, "Figure"));
}

fn table(element: ElementRef, out: &mut Writer) {
    let caption = caption(element);
    out.paragraph(&normalize_caption(&caption, "Table"));
    let selector = scraper::Selector::parse("table.ltx_tabular").expect("valid selector");
    if let Some(tabular) = element.select(&selector).next() {
        out.table(&rows(tabular));
    }
}

/// `Figure 1: The…` stays; a caption without its tag gets `Figure: `.
fn normalize_caption(caption: &str, kind: &str) -> String {
    let caption = caption.trim();
    let (label, rest) = caption.split_once(':').unwrap_or(("", caption));
    if label.is_empty() || label.split_whitespace().count() > 3 {
        return format!("{kind}: {caption}");
    }
    format!("{}: {}", normalize_label(label), rest.trim())
}

fn rows(tabular: ElementRef) -> Vec<Vec<String>> {
    let tr = scraper::Selector::parse("tr").expect("valid selector");
    tabular
        .select(&tr)
        .map(|row| child_elements(row).filter(|c| matches!(c.value().name(), "td" | "th")).map(inline).collect())
        .collect()
}

/// Each row of an equation table: its formulas' LaTeX and its number.
fn equations(table: ElementRef, out: &mut Writer) {
    let tr = scraper::Selector::parse("tr").expect("valid selector");
    let math = scraper::Selector::parse("math").expect("valid selector");
    let tag = scraper::Selector::parse(".ltx_tag_equation").expect("valid selector");
    for row in table.select(&tr) {
        let tex: Vec<&str> = row.select(&math).filter_map(|m| m.value().attr("alttext")).collect();
        let number = row.select(&tag).next().map(|t| t.text().collect::<String>());
        out.equation(&tex.join(" "), number.as_deref());
    }
}

fn bibliography(list: ElementRef, out: &mut Writer) {
    for item in child_elements(list) {
        let tag = scraper::Selector::parse(".ltx_tag_bibitem").expect("valid selector");
        let label = item.select(&tag).next().map(|t| t.text().collect::<String>()).unwrap_or_default();
        let text: Vec<String> = child_elements(item).filter(|c| has_class(*c, "ltx_bibblock")).map(inline).collect();
        let label = label.trim().trim_start_matches('[').trim_end_matches(']');
        let label = if label.is_empty() { String::new() } else { format!("[{label}] ") };
        out.paragraph(&format!("{label}{}", text.join(" ")));
    }
}

/// Inline content as Markdown: emphasis, sub/superscripts, `$math$`, citations as their text;
/// footnotes left out.
fn inline(element: ElementRef) -> String {
    let mut text = String::new();
    for child in element.children() {
        match child.value() {
            Node::Text(t) => text.push_str(&super::escape(t)),
            Node::Element(_) => {
                let Some(child) = ElementRef::wrap(child) else { continue };
                text.push_str(&inline_element(child));
            }
            _ => {}
        }
    }
    text
}

fn inline_element(element: ElementRef) -> String {
    let name = element.value().name();
    if name == "math" {
        return element.value().attr("alttext").map(|tex| format!("${tex}$")).unwrap_or_default();
    }
    if skipped(element) || has_class(element, "ltx_note_mark") || matches!(name, "img" | "svg" | "button") {
        return String::new();
    }
    let inner = inline(element);
    match name {
        "em" | "i" => super::wrap("*", &inner),
        "b" | "strong" => super::wrap("**", &inner),
        "sup" => format!("<sup>{inner}</sup>"),
        "sub" => format!("<sub>{inner}</sub>"),
        "br" => " ".into(),
        _ if has_class(element, "ltx_font_italic") => super::wrap("*", &inner),
        _ if has_class(element, "ltx_font_bold") => super::wrap("**", &inner),
        _ => inner,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transformer_paper() {
        let page = Url::parse("https://arxiv.org/html/1706.03762").unwrap();
        let md = to_markdown(include_str!("../../tests/fixtures/arxiv-html-1706.03762v7.html"), &page);
        assert!(
            md.contains("![Figure](https://arxiv.org/html/1706.03762v7/Figures/ModalNet-21.png)\n\nFigure 1:"),
            "the image just before its caption"
        );
        assert!(!md.contains("Ashish Vaswani"), "authors left out");
        assert!(md.contains("## 1 Introduction\n\nRecurrent neural networks, long short-term memory [13]"));
        assert!(md.contains(r"$$\mathrm{Attention}(Q,K,V)=\mathrm{softmax}(\frac{QK^{T}}{\sqrt{d_{k}}})V$$ (1)"));
        assert!(md.contains("hidden states $h_{t}$"), "inline math");
        assert!(md.contains("Figure 1: The Transformer - model architecture."));
        assert!(md.contains("Table 1: Maximum path lengths"));
        assert!(md.contains("|Layer Type|"), "table rows");
        assert!(md.contains(
            "## References\n\n[1] Jimmy Lei Ba, Jamie Ryan Kiros, and Geoffrey E Hinton. Layer normalization."
        ));
        assert!(!md.contains("Equal contribution"), "footnotes left out");
    }
}
