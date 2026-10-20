//! Plain text from what sources send: whitespace collapsed, markup dropped (JATS and HTML tags
//! in Crossref and Europe PMC titles and abstracts, MathML in titles), character references
//! decoded.

/// Trims and collapses runs of whitespace (line breaks included) to one space.
pub fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `text` without its tags (each becomes a space), entities decoded, whitespace collapsed:
/// `<jats:p>Heat <i>in vivo</i></jats:p>` → `Heat in vivo`. A `<` not starting a tag stays.
pub fn plain(text: &str) -> String {
    if !text.contains(['<', '&']) {
        return squash(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let is_tag = after.starts_with(|c: char| c.is_ascii_alphabetic() || matches!(c, '/' | '!' | '?'));
        match after.find('>').filter(|_| is_tag) {
            Some(end) => {
                out.push(' ');
                rest = &after[end + 1..];
            }
            None => {
                out.push('<');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    squash(&decode(&out))
}

/// HTML character references: the five named ones, `&nbsp;`, and numeric ones; anything else is
/// left as written.
pub fn decode(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let end = tail.find(';').filter(|e| *e <= 10);
        let decoded = end.and_then(|e| match &tail[1..e] {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" => Some(' '),
            entity => {
                let number = entity.strip_prefix('#')?;
                let code = match number.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                    None => number.parse().ok()?,
                };
                char::from_u32(code)
            }
        });
        match (decoded, end) {
            (Some(c), Some(e)) => {
                out.push(c);
                rest = &tail[e + 1..];
            }
            _ => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_and_entities() {
        assert_eq!(squash("  a\n  b\tc "), "a b c");
        assert_eq!(plain("<jats:p>Heat &amp; <i>in vivo</i></jats:p>"), "Heat & in vivo");
        assert_eq!(plain("<h4>Background</h4>Temperature<sup>2</sup>  rises."), "Background Temperature 2 rises.");
        assert_eq!(plain("heating in\n <mml:math><mml:mn>88</mml:mn></mml:math> Sr"), "heating in 88 Sr");
        assert_eq!(plain("a < b and c > d"), "a < b and c > d");
        assert_eq!(decode("&#8212; &#x2014; &bogus; AT&T"), "— — &bogus; AT&T");
    }
}
