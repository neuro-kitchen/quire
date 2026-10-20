//! Figure and table captions made separate `Figure N: …` paragraphs, the form the reader takes
//! out of the text ([`crate::paper_markdown`]):
//!
//! - a caption starting a line inside another block (the converter joined it to a paragraph);
//! - a bold label without punctuation (`**Fig. 1** Adaptive mouse…`, Springer and Nature);
//! - a "table" the converter built from a figure's axis labels and caption
//!   (`|Figure 1:|EMG and (d) contaminated EMG.|(a) ECG, (b)…|`): its caption cells kept.

use super::is_table;
use crate::paper_markdown::{caption_label, label_alone};

pub(super) fn separate(blocks: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(blocks.len());
    for block in blocks {
        if is_table(&block) {
            out.push(caption_in_table(&block).unwrap_or(block));
            continue;
        }
        // Cut before each line (after the first) that starts a caption.
        let mut current: Vec<&str> = Vec::new();
        for line in block.lines() {
            if !current.is_empty() && caption_label(line).is_some() {
                out.push(current.join("\n"));
                current.clear();
            }
            current.push(line);
        }
        if !current.is_empty() {
            out.push(current.join("\n"));
        }
    }
    join_lone_labels(out).into_iter().map(|b| normalize(&b)).collect()
}

/// A block that is only a label (`Table 1`) and the paragraph after it: one caption.
fn join_lone_labels(blocks: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(blocks.len());
    let mut pending: Option<String> = None;
    for block in blocks {
        if let Some(label) = pending.take() {
            if block.starts_with(['#', '|']) || caption_label(&block).is_some() {
                out.push(label);
            } else {
                out.push(format!("{label}: {block}"));
                continue;
            }
        }
        match label_alone(&block) {
            Some(label) => pending = Some(label.name()),
            None => out.push(block),
        }
    }
    out.extend(pending);
    out
}

/// A caption in the reader's form, `Figure 1: …` (`**Fig. 1** Adaptive…`, `Fig. 1. Adaptive…`);
/// other blocks unchanged.
fn normalize(block: &str) -> String {
    let Some(label) = caption_label(block) else { return block.to_string() };
    let mut rest = block[label.len..].trim_start_matches(['*', ' ']).to_string();
    // `**Figure 4 Title…** text`: the bold opened before the label closes in the rest; drop it.
    if block.trim_start().starts_with("**")
        && !block[..label.len].trim_end().ends_with("**")
        && let Some(close) = rest.find("**")
    {
        rest.replace_range(close..close + 2, "");
    }
    format!("{}: {rest}", label.name())
}

/// The caption in a table block made of a figure's labels: the cells from the `Figure N:` cell on.
fn caption_in_table(block: &str) -> Option<String> {
    let cells: Vec<&str> = block
        .lines()
        .flat_map(|line| line.split('|'))
        .map(str::trim)
        .filter(|c| !c.is_empty() && !c.chars().all(|ch| matches!(ch, '-' | ':')))
        // An axis tick can share the label's cell (`0 Figure 4:`).
        .map(|c| c.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == ' '))
        .collect();
    let start = cells.iter().position(|c| caption_label(c).is_some())?;
    Some(normalize(&cells[start..].join(" ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(blocks: &[&str]) -> Vec<String> {
        separate(blocks.iter().map(|b| b.to_string()).collect())
    }

    #[test]
    fn caption_forms() {
        assert_eq!(run(&["**Fig. 1** Adaptive mouse for users."]), ["Figure 1: Adaptive mouse for users."]);
        assert_eq!(run(&["**Figure 2:** A sample of the template."]), ["Figure 2: A sample of the template."]);
        assert_eq!(run(&["Fig. 3. Results of the filter."]), ["Figure 3: Results of the filter."]);
        assert_eq!(
            run(&["Text of a paragraph that ends here.\nFig. 4. The device.\nmore caption"]),
            ["Text of a paragraph that ends here.", "Figure 4: The device.\nmore caption"]
        );
        assert_eq!(
            run(&["|Figure 1:|EMG and (d) contaminated EMG.|(a) ECG, (b) clean|\n|---|---|---|"]),
            ["Figure 1: EMG and (d) contaminated EMG. (a) ECG, (b) clean"]
        );
    }

    #[test]
    fn more_caption_forms() {
        assert_eq!(run(&["Table 1", "Main fiber parameters.", "|a|b|"]), ["Table 1: Main fiber parameters.", "|a|b|"]);
        assert_eq!(run(&["Table 1 Summary of the modes."]), ["Table 1: Summary of the modes."]);
        assert_eq!(
            run(&["**Figure 4 Diaphragm CMAP recorded** from four pairs."]),
            ["Figure 4: Diaphragm CMAP recorded from four pairs."]
        );
        assert_eq!(run(&["Fig. 1-General communications system."]), ["Figure 1: General communications system."]);
        // A label alone before a heading stays as it was.
        assert_eq!(run(&["Table 2", "## Results"]), ["Table 2", "## Results"]);
    }

    #[test]
    fn not_captions() {
        for block in [
            "Figure 2 shows the results.",
            "Table 1 summarizes the data.",
            "|a|b|\n|---|---|\n|1|2|",
            "In Fig. 3 we see",
        ] {
            assert_eq!(run(&[block]), [block], "{block}");
        }
    }
}
