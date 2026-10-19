//! Plain text from another extractor (poppler's `pdftotext`, used when `pdf-inspector` reads
//! only a scan's stamps, [`super::ocr_layer`]) as reader Markdown: pages split at form feeds,
//! paragraphs at blank lines, the lines of a paragraph joined, words broken at a line end mended,
//! then cleaned like `pdf-inspector`'s ([`super::tidy`]).

/// `text` (pages separated by `\f`) as Markdown paragraphs.
pub fn to_markdown(text: &str) -> String {
    let mut paragraphs = Vec::new();
    for page in text.split('\u{c}') {
        for paragraph in page.split("\n\n") {
            let joined = join_lines(paragraph);
            if !joined.is_empty() {
                paragraphs.push(joined);
            }
        }
    }
    super::tidy::tidy(&paragraphs.join("\n\n"))
}

/// The lines of one paragraph as one line; `exam-` + `ple` → `example`.
fn join_lines(paragraph: &str) -> String {
    let mut out = String::new();
    for line in paragraph.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if out.is_empty() {
            out.push_str(line);
        } else if out.ends_with('-') && line.starts_with(char::is_lowercase) {
            out.pop();
            out.push_str(line);
        } else {
            out.push(' ');
            out.push_str(line);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_paragraphs_and_hyphens() {
        let text = "Differential respiratory\nactivity of the dia-\nphragm\n\nMethods were\u{c}Second page.\n";
        assert_eq!(
            to_markdown(text),
            "Differential respiratory activity of the diaphragm\n\nMethods were\n\nSecond page."
        );
    }
}
