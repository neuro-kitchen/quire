//! Presentation MathML → LaTeX, for papers whose full text has equations only as MathML (PLOS and
//! most of PubMed Central, 2026-10-19). The LaTeX is what reader mode turns into Unicode, so only
//! the structure matters: fractions, roots, scripts, accents, tables of rows.

use roxmltree::Node;

/// The LaTeX of a `<math>` element (any namespace prefix).
pub(crate) fn to_tex(math: Node) -> String {
    element(math).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Element children (MathML puts text only inside token elements).
fn children<'a, 'i>(node: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children().filter(Node::is_element)
}

fn nth(node: Node, n: usize) -> String {
    children(node).nth(n).map(element).unwrap_or_default()
}

/// The children one after the other: letters of a product (`ma`) stay together, operators bring
/// their own spaces.
fn all(node: Node) -> String {
    children(node).map(element).collect()
}

/// The text of a token element, without invisible operators (function application, invisible
/// times and separators).
fn token(node: Node) -> String {
    let text: String = node.descendants().filter(Node::is_text).filter_map(|t| t.text()).collect();
    text.chars().filter(|c| !matches!(c, '\u{2061}'..='\u{2064}' | '\u{200b}')).collect::<String>().trim().to_string()
}

/// `\hat`, `\bar`… for an accent character over a base.
fn accent(mark: &str) -> Option<&'static str> {
    Some(match mark {
        "^" | "ˆ" | "\u{302}" => "\\hat",
        "¯" | "‾" | "\u{304}" | "―" | "_" => "\\bar",
        "~" | "˜" | "\u{303}" => "\\tilde",
        "˙" | "\u{307}" | "." => "\\dot",
        "¨" | "\u{308}" => "\\ddot",
        "→" | "\u{20d7}" => "\\vec",
        _ => return None,
    })
}

fn element(node: Node) -> String {
    match node.tag_name().name() {
        // A name (`exp`, `max`) is followed by a space so it does not run into its argument.
        "mi" => {
            let name = token(node);
            if name.chars().count() > 1 && name.chars().all(char::is_alphabetic) { format!("{name} ") } else { name }
        }
        "mn" | "mtext" | "ms" => token(node),
        "mo" => {
            let op = token(node);
            // Spaces around relations and binary operators keep the converted text readable.
            if matches!(op.as_str(), "=" | "<" | ">" | "≤" | "≥" | "≠" | "≈" | "+" | "−" | "-" | "±" | "×" | "→" | "∈")
            {
                format!(" {op} ")
            } else {
                op
            }
        }
        "mspace" => " ".into(),
        "annotation" | "annotation-xml" | "mphantom" | "none" | "mprescripts" => String::new(),
        "semantics" => nth(node, 0),
        "msub" => format!("{{{}}}_{{{}}}", nth(node, 0), nth(node, 1)),
        "msup" => format!("{{{}}}^{{{}}}", nth(node, 0), nth(node, 1)),
        "msubsup" => format!("{{{}}}_{{{}}}^{{{}}}", nth(node, 0), nth(node, 1), nth(node, 2)),
        "mfrac" => format!("\\frac{{{}}}{{{}}}", nth(node, 0), nth(node, 1)),
        "msqrt" => format!("\\sqrt{{{}}}", all(node)),
        "mroot" => format!("\\sqrt[{}]{{{}}}", nth(node, 1), nth(node, 0)),
        // Over or under nothing (an `<mspace>`): a typesetting trick to place a character, as in
        // MDPI's slanted fraction `¹/√|a|`. The character alone.
        "mover" | "munder" if nth(node, 0).trim().is_empty() => nth(node, 1),
        "mover" => {
            let over = children(node).nth(1).map(token).unwrap_or_default();
            match accent(&over) {
                Some(command) => format!("{command}{{{}}}", nth(node, 0)),
                None => format!("{{{}}}^{{{}}}", nth(node, 0), nth(node, 1)),
            }
        }
        "munder" => format!("{{{}}}_{{{}}}", nth(node, 0), nth(node, 1)),
        "munderover" => format!("{{{}}}_{{{}}}^{{{}}}", nth(node, 0), nth(node, 1), nth(node, 2)),
        "mfenced" => {
            let open = node.attribute("open").unwrap_or("(");
            let close = node.attribute("close").unwrap_or(")");
            let separator = node.attribute("separators").map_or(",", |s| s.trim()).chars().next().unwrap_or(',');
            let parts: Vec<String> = children(node).map(element).collect();
            format!("{open}{}{close}", parts.join(&separator.to_string()))
        }
        "mtable" => children(node).map(element).filter(|r| !r.trim().is_empty()).collect::<Vec<_>>().join(" \\\\ "),
        // A labelled row's first cell is its equation number: left out (the label is shown apart).
        "mlabeledtr" => children(node).skip(1).map(element).collect::<Vec<_>>().join(" & "),
        "mtr" => children(node).map(element).collect::<Vec<_>>().join(" & "),
        _ => all(node),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tex(xml: &str) -> String {
        let xml = format!(r#"<math xmlns="http://www.w3.org/1998/Math/MathML">{xml}</math>"#);
        let doc = roxmltree::Document::parse(&xml).unwrap();
        to_tex(doc.root_element())
    }

    #[test]
    fn structures() {
        assert_eq!(
            tex(
                "<mrow><mi>M</mi><mo>=</mo><mfrac><msub><mi>ε</mi><mi>a</mi></msub><msqrt><mi>d</mi></msqrt></mfrac></mrow>"
            ),
            r"M = \frac{{ε}_{a}}{\sqrt{d}}"
        );
        assert_eq!(tex("<mover><mi>x</mi><mo>^</mo></mover><mo>\u{2061}</mo>"), r"\hat{x}");
        assert_eq!(tex("<munderover><mo>∑</mo><mi>i</mi><mi>n</mi></munderover>"), "{∑}_{i}^{n}");
        assert_eq!(
            tex(
                "<mtable><mtr><mtd><mi>a</mi></mtd><mtd><mi>b</mi></mtd></mtr><mtr><mtd><mi>c</mi></mtd></mtr></mtable>"
            ),
            r"a & b \\ c"
        );
        assert_eq!(
            tex(
                r#"<mover accent="true"><mspace height=".6"/><mn>1</mn></mover><mo>/</mo><munder><mspace depth="-.8"/><msqrt><mi>a</mi></msqrt></munder>"#
            ),
            r"1/\sqrt{a}"
        );
        assert_eq!(tex("<semantics><mi>y</mi><annotation encoding=\"x\">ignored</annotation></semantics>"), "y");
    }
}
