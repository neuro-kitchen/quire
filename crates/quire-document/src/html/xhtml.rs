//! Plain (X)HTML → reader Markdown: an EPUB's chapters, and any page without a converter of its
//! own. Headings, paragraphs, quotes, lists, tables and figure captions become blocks; emphasis
//! and sub/superscripts stay; scripts, styles, navigation and figures are left out (an image inside
//! text reads as its `alt`).

use scraper::{ElementRef, Html, Node};

use super::Writer;

/// The `<body>` of `html` as reader Markdown, appended to `out`.
pub(crate) fn body(html: &str, out: &mut Writer) {
    let document = Html::parse_document(html);
    let selector = scraper::Selector::parse("body").expect("valid selector");
    if let Some(body) = document.select(&selector).next() {
        blocks(body, out);
    }
}

fn child_elements(element: ElementRef) -> impl Iterator<Item = ElementRef> {
    element.children().filter_map(ElementRef::wrap)
}

fn blocks(element: ElementRef, out: &mut Writer) {
    // A container holding text of its own (`<div>Text <em>more</em></div>`) is one paragraph.
    let own_text = element.children().any(|c| matches!(c.value(), Node::Text(t) if !t.trim().is_empty()));
    if own_text && !matches!(element.value().name(), "body") {
        out.paragraph(&inline(element));
        return;
    }
    for child in child_elements(element) {
        let name = child.value().name();
        match name {
            "script" | "style" | "nav" | "header" | "footer" | "img" | "svg" | "hr" | "aside" => {}
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                // An EPUB chapter's `<h1>` is a section of the paper, not its title.
                let level: usize = name[1..].parse().unwrap_or(2);
                heading(child, level + 1, out);
            }
            "p" | "blockquote" | "pre" | "figcaption" | "dt" | "dd" => out.paragraph(&inline(child)),
            "ul" | "ol" => {
                let items: Vec<String> =
                    child_elements(child).filter(|c| c.value().name() == "li").map(inline).collect();
                out.list(&items);
            }
            "table" => out.table(&rows(child)),
            _ => blocks(child, out),
        }
    }
}

/// A heading, without the illustration captions some books put inside it (Project Gutenberg:
/// `<h2><a id="c2"/><img/><span class="caption">…</span><br/>CHAPTER II.</h2>`, where HTML parsing
/// leaves the caption inside the unclosed `<a>`); they follow as paragraphs.
fn heading(element: ElementRef, level: usize, out: &mut Writer) {
    out.heading(level, &inline_with(element, true));
    let selector = scraper::Selector::parse(".caption").expect("valid selector");
    for caption in element.select(&selector) {
        out.paragraph(&inline(caption));
    }
}

fn rows(table: ElementRef) -> Vec<Vec<String>> {
    let selector = scraper::Selector::parse("tr").expect("valid selector");
    table
        .select(&selector)
        .map(|row| child_elements(row).filter(|c| matches!(c.value().name(), "td" | "th")).map(inline).collect())
        .collect()
}

/// Inline content as Markdown: emphasis, sub/superscripts, `$math$` (MathML's `alttext`).
fn inline(element: ElementRef) -> String {
    inline_with(element, false)
}

/// [`inline`], leaving out elements of class `caption` at any depth if `skip_captions`.
fn inline_with(element: ElementRef, skip_captions: bool) -> String {
    let mut text = String::new();
    for child in element.children() {
        match child.value() {
            Node::Text(t) => text.push_str(&super::escape(t)),
            Node::Element(_) => {
                let Some(child) = ElementRef::wrap(child) else { continue };
                if skip_captions && child.value().has_class("caption", scraper::CaseSensitivity::AsciiCaseInsensitive) {
                    continue;
                }
                text.push_str(&inline_element(child, skip_captions));
            }
            _ => {}
        }
    }
    text
}

fn inline_element(element: ElementRef, skip_captions: bool) -> String {
    let name = element.value().name();
    match name {
        "math" => {
            return element
                .value()
                .attr("alttext")
                .map(|tex| format!("${tex}$"))
                .unwrap_or_else(|| element.text().collect());
        }
        // An image inside text reads as its `alt` (a decorated initial: `alt="M"`); decorative ones
        // have none.
        "img" => return super::escape(element.value().attr("alt").unwrap_or_default().trim()),
        "svg" | "script" | "style" => return String::new(),
        "br" => return " ".into(),
        _ => {}
    }
    let inner = inline_with(element, skip_captions);
    match name {
        "em" | "i" => super::wrap("*", &inner),
        "b" | "strong" => super::wrap("**", &inner),
        "sup" => format!("<sup>{inner}</sup>"),
        "sub" => format!("<sub>{inner}</sub>"),
        _ => inner,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn markdown(body_html: &str) -> String {
        let mut out = Writer::default();
        body(&format!("<html><body>{body_html}</body></html>"), &mut out);
        out.finish()
    }

    #[test]
    fn a_caption_inside_a_heading_follows_it() {
        let md = markdown(
            r#"<h2><a id="c2"/><img src="i.jpg"/><span class="caption">I hope he will like it.</span><br/>CHAPTER II.</h2><p>Text.</p>"#,
        );
        assert_eq!(md, "### CHAPTER II.\n\nI hope he will like it.\n\nText.\n");
    }

    #[test]
    fn an_initial_drawn_as_an_image_reads_as_its_letter() {
        let md = markdown(
            r#"<p><span class="letra"><img alt="M" src="m.jpg"/></span>R. Bennet <img alt="" src="o.png"/>came.</p>"#,
        );
        assert_eq!(md, "MR. Bennet came.\n");
    }

    #[test]
    fn blocks_and_inline() {
        let md = markdown(
            "<nav>Contents</nav><section><h1>Results</h1><p>CO<sub>2</sub> was <em>high</em>.</p>\
             <ul><li>one</li><li>two</li></ul><table><tr><th>a</th><th>b</th></tr><tr><td>1</td><td>2</td></tr></table>\
             <div>Loose <b>text</b></div><p>Cost: $5</p></section>",
        );
        assert_eq!(
            md,
            "## Results\n\nCO<sub>2</sub> was *high*.\n\n- one\n- two\n\n|a|b|\n|---|---|\n|1|2|\n\nLoose **text**\n\nCost: \\$5\n"
        );
    }
}
