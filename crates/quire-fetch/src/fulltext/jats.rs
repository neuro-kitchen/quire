//! JATS XML (Europe PMC's open-access full text) → reader Markdown ([`super`] has the format).
//! Sections nest with their `<title>`; `<disp-formula>`, `<fig>`, `<table-wrap>` and lists can sit
//! inside a `<p>`, which is then cut around them.

use roxmltree::{Document, Node};

use super::{Writer, figure_label, tex_from_tex_math};
use crate::fulltext::mathml;

/// The article's body and reference list as reader Markdown.
pub fn to_markdown(xml: &str) -> Result<String, roxmltree::Error> {
    let doc = Document::parse_with_options(xml, roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() })?;
    let mut out = Writer::default();
    let article = doc.root_element();
    if let Some(body) = article.children().find(|n| n.has_tag_name("body")) {
        blocks(body, 1, &mut out);
    }
    if let Some(back) = article.children().find(|n| n.has_tag_name("back")) {
        for part in back.children().filter(Node::is_element) {
            match part.tag_name().name() {
                "ack" | "app-group" | "app" | "sec" => blocks(part, 1, &mut out),
                "ref-list" => references(part, &mut out),
                _ => {}
            }
        }
    }
    Ok(out.finish())
}

/// Block-level content of `node`; `depth`: section nesting (1 → `##`).
fn blocks(node: Node, depth: usize, out: &mut Writer) {
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "sec" | "app" | "ack" | "app-group" => {
                if let Some(title) = child.children().find(|n| n.has_tag_name("title")) {
                    let label = child.children().find(|n| n.has_tag_name("label")).map(text).unwrap_or_default();
                    out.heading(depth + 1, &format!("{label} {}", inline(title)));
                }
                blocks(child, depth + 1, out);
            }
            "p" => paragraph(child, depth, out),
            "disp-formula" => formula(child, out),
            "fig" => figure(child, out),
            "table-wrap" => table(child, out),
            "list" => list(child, out),
            "fig-group" | "table-wrap-group" | "disp-formula-group" | "boxed-text" | "disp-quote" => {
                blocks(child, depth, out)
            }
            "title" | "label" | "sec-meta" | "supplementary-material" | "ref-list" | "fn-group" => {}
            _ => blocks(child, depth, out),
        }
    }
}

/// A `<p>`: its inline text, cut where a block (an equation, a figure) sits inside it.
fn paragraph(p: Node, depth: usize, out: &mut Writer) {
    let mut text = String::new();
    for child in p.children() {
        let block = child.is_element()
            && matches!(
                child.tag_name().name(),
                "disp-formula" | "fig" | "table-wrap" | "list" | "disp-quote" | "boxed-text"
            );
        if block {
            out.paragraph(&std::mem::take(&mut text));
            match child.tag_name().name() {
                "disp-formula" => formula(child, out),
                "fig" => figure(child, out),
                "table-wrap" => table(child, out),
                "list" => list(child, out),
                _ => blocks(child, depth, out),
            }
        } else {
            text.push_str(&inline(child));
        }
    }
    out.paragraph(&text);
}

fn formula(node: Node, out: &mut Writer) {
    let label = node.children().find(|n| n.has_tag_name("label")).map(text);
    if let Some(tex) = math_of(node) {
        out.equation(&tex, label.as_deref());
    }
}

/// The LaTeX of a formula: its `<tex-math>`, else its MathML.
fn math_of(node: Node) -> Option<String> {
    if let Some(tex) = node.descendants().find(|n| n.has_tag_name("tex-math")) {
        return Some(tex_from_tex_math(&text(tex)));
    }
    node.descendants().find(|n| n.has_tag_name("math")).map(mathml::to_tex)
}

/// `Figure N: title. caption`.
fn caption(node: Node, default: &str) -> String {
    let label = node.children().find(|n| n.has_tag_name("label")).map(text).unwrap_or_else(|| default.into());
    let caption = node.children().find(|n| n.has_tag_name("caption")).map(|c| {
        let parts: Vec<String> = c.children().filter(Node::is_element).map(inline).collect();
        parts.iter().map(|p| p.trim()).filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ")
    });
    format!("{}: {}", figure_label(&label), caption.unwrap_or_default())
}

fn figure(node: Node, out: &mut Writer) {
    out.paragraph(&caption(node, "Figure"));
}

fn table(node: Node, out: &mut Writer) {
    out.paragraph(&caption(node, "Table"));
    let rows: Vec<Vec<String>> = node
        .descendants()
        .filter(|n| n.has_tag_name("tr"))
        .map(|tr| tr.children().filter(|c| c.has_tag_name("td") || c.has_tag_name("th")).map(inline).collect())
        .collect();
    out.table(&rows);
}

fn list(node: Node, out: &mut Writer) {
    let items: Vec<String> = node
        .children()
        .filter(|n| n.has_tag_name("list-item"))
        .map(|item| item.children().filter(Node::is_element).map(inline).collect::<Vec<_>>().join(" "))
        .collect();
    out.list(&items);
}

fn references(node: Node, out: &mut Writer) {
    out.heading(2, "References");
    for (i, reference) in node.children().filter(|n| n.has_tag_name("ref")).enumerate() {
        let label = reference.children().find(|n| n.has_tag_name("label")).map(text).unwrap_or((i + 1).to_string());
        let citation = reference.children().find(|n| n.has_tag_name("mixed-citation")).map(inline).or_else(|| {
            // `<element-citation>` has no punctuation: its parts joined.
            let parts = reference.children().find(|n| n.has_tag_name("element-citation"))?;
            let texts: Vec<String> = parts.children().filter(Node::is_element).map(text).collect();
            Some(texts.iter().map(|t| t.trim()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(", "))
        });
        let label = label.trim().trim_start_matches('[').trim_end_matches([']', '.']);
        out.paragraph(&format!("[{label}] {}", citation.unwrap_or_default()));
    }
}

/// All the text under `node`.
fn text(node: Node) -> String {
    node.descendants().filter(Node::is_text).filter_map(|t| t.text()).collect()
}

/// Inline content as Markdown: emphasis, sub/superscripts, `$math$`, citations as their text.
fn inline(node: Node) -> String {
    if node.is_text() {
        return super::escape(node.text().unwrap_or_default());
    }
    let inner = || node.children().map(inline).collect::<String>();
    match node.tag_name().name() {
        "italic" => super::wrap("*", &inner()),
        "bold" => super::wrap("**", &inner()),
        "sup" => format!("<sup>{}</sup>", inner()),
        "sub" => format!("<sub>{}</sub>", inner()),
        "inline-formula" => math_of(node).map(|tex| format!("${tex}$")).unwrap_or_default(),
        "break" => " ".into(),
        "fn" | "graphic" | "inline-graphic" | "label" | "object-id" | "alt-text" => String::new(),
        _ => inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_formulas_figures_tables_references() {
        let xml = r#"<article xmlns:mml="http://www.w3.org/1998/Math/MathML"><body>
            <sec><title>1. Introduction</title><p>Injury is common [<xref rid="r1">1</xref>] and costs $5.
              See <italic>strain</italic> <inline-formula><mml:math><mml:msub><mml:mi>ε</mml:mi><mml:mi>a</mml:mi></mml:msub></mml:math></inline-formula>:
              <disp-formula><label>(1)</label><alternatives><graphic/><mml:math display="block"><mml:mi>M</mml:mi><mml:mo>=</mml:mo><mml:mn>2</mml:mn></mml:math></alternatives></disp-formula>
              which holds.</p>
              <sec><title>1.1 Model</title><p>Text.</p>
              <fig id="f1"><label>Fig 1</label><caption><title>Overall framework.</title><p><bold>(a)</bold> Axon.</p></caption></fig>
              </sec></sec></body>
            <back><ref-list><title>References</title><ref id="r1"><label>1</label><mixed-citation><surname>Gennarelli</surname> TA. <article-title>Diffuse axonal injury</article-title>. <year>1982</year>.</mixed-citation></ref></ref-list></back></article>"#;
        let md = to_markdown(xml).unwrap();
        insta::assert_snapshot!(md);
    }

    #[test]
    fn tables() {
        let xml = r#"<article><body><table-wrap><label>Table 1</label><caption><title>Conditions.</title></caption>
            <table><thead><tr><th>Case</th><th>Strain</th></tr></thead><tbody><tr><td>A</td><td>0.<italic>1</italic></td></tr></tbody></table>
            </table-wrap></body></article>"#;
        assert_eq!(to_markdown(xml).unwrap(), "Table 1: Conditions.\n\n|Case|Strain|\n|---|---|\n|A|0.*1*|\n");
    }
}
