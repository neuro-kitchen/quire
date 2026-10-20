//! Cleaning `pdf-inspector`'s Markdown for reading. Seen on 16 papers of the user's library
//! (2026-10-19; Elsevier, IEEE, Springer, Nature, Frontiers, APS):
//!
//! - [`lines`]: download stamps ("Authorized licensed use limited to…") and running heads
//!   ("T.I. Arabadzhiev et al. / Journal of … 223–232") repeated in the text;
//! - [`front`]: the front matter (journal header, authors, affiliations, article history,
//!   corresponding author, copyright and ISSN lines) mixed into the opening and the first page's
//!   footnotes, moved into a `<!-- front -->` … `<!-- /front -->` section the reader shows apart;
//! - [`headings`]: section headings run into their paragraph (`1. Introduction Failure to…`,
//!   `**2. Methods** This work…`, `I. INTRODUCTION`) made headings;
//! - [`captions`]: figure captions inside other blocks, in tables built from a figure's labels, or
//!   with a bold label and no colon (`**Fig. 1** Adaptive mouse…`) made `Fig. 1. …` paragraphs;
//! - [`paragraphs`]: paragraphs cut by a page or column break joined, words hyphenated at a line
//!   end mended, Elsevier's math font characters (`ð1Þ` = `(1)`) read as what they show.
//!
//! The output is the reader's Markdown ([`crate::paper_markdown`]); each step is a pure function on lines or blocks, tested on
//! made-up text in the shape of the real cases.

mod captions;
mod front;
mod headings;
mod lines;
mod paragraphs;

use crate::paper_markdown::{FRONT_END, FRONT_START};

/// `markdown` from `pdf-inspector`, cleaned.
pub(crate) fn tidy(markdown: &str) -> String {
    let text = lines::remove_stamps_and_running_heads(markdown);
    let (text, footnotes) = front::take_front_lines(&text);
    let blocks = split_blocks(&text);
    let blocks = captions::separate(blocks);
    let blocks = headings::split(blocks);
    let (mut front, body) = front::split_opening(blocks);
    front.extend(footnotes);
    let body = paragraphs::repair(body);
    let mut out = Vec::new();
    if !front.is_empty() {
        out.push(FRONT_START.to_string());
        out.extend(front);
        out.push(FRONT_END.to_string());
    }
    out.extend(body);
    out.join("\n\n")
}

fn split_blocks(text: &str) -> Vec<String> {
    text.split("\n\n").map(str::trim).filter(|b| !b.is_empty()).map(str::to_string).collect()
}

/// A block's text without Markdown emphasis or heading marks, for matching.
fn plain(block: &str) -> String {
    block.trim_start_matches('#').replace(['*', '_'], "").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_heading(block: &str) -> bool {
    block.starts_with('#')
}

fn is_table(block: &str) -> bool {
    block.starts_with('|')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elsevier_opening() {
        let md = "Journal of Electromyography and Kinesiology 20 (2010) 223–232\n\n\
            Contents lists available at ScienceDirect\n\n# Journal of Electromyography and Kinesiology\n\n\
            journal homepage: www.elsevier.com/locate/jelekin\n\n\
            Interpretation of EMG integral or RMS\n\nTodor I. Arabadzhiev <sup>a,*</sup>, Vladimir G. Dimitrov <sup>b</sup>\n\n\
            ### article info abstract\n\nIn occupational and sports physiology, reduction of efficiency.\n\n\
            1. Introduction Failure to maintain the required force is referred to as muscle fatigue. For many\n\
            * Corresponding author. Tel.: +359 2 979 36 10. E-mail address: someone@example.org (T.I. A.).\n\
            1050-6411/$ - see front matter 2009 Elsevier Ltd. All rights reserved. doi:10.1016/j.jelekin.2009.01.008\n\n\
            years there has been an interest in the deter- mination of fatigue.\n\n\
            T.I. Arabadzhiev et al. / Journal of Electromyography and Kinesiology 20 (2010) 223–232\n\
            2. Methods NME is usually assessed during contractions.\n\n\
            T.I. Arabadzhiev et al. / Journal of Electromyography and Kinesiology 20 (2010) 223–232\n\n\
            MFPVðm=sÞ¼0:7 v þ 0:01 ð2Þ\n\n\
            T.I. Arabadzhiev et al. / Journal of Electromyography and Kinesiology 20 (2010) 223–232";
        let out = tidy(md);
        let (front, body) = out.split_once(FRONT_END).unwrap();
        assert!(front.starts_with(FRONT_START));
        for part in
            ["Contents lists available", "Arabadzhiev <sup>a,*</sup>", "Corresponding author", "see front matter"]
        {
            assert!(front.contains(part), "{part} in the front matter");
        }
        assert!(!body.contains("et al. /"), "running heads gone: {body}");
        assert!(body.trim_start().starts_with("## 1. Introduction\n\nFailure to maintain"), "{body}");
        assert!(body.contains("For many years there has been an interest in the determination of fatigue."), "{body}");
        assert!(body.contains("## 2. Methods\n\nNME is usually"), "{body}");
        assert!(body.contains("MFPV(m=s)=0:7 v + 0:01 (2)"), "{body}");
    }

    #[test]
    fn nothing_to_do_stays_the_same() {
        let md = "## 1 Introduction\n\nA paragraph that ends well.\n\n## 2 Methods\n\nAnother one.";
        assert_eq!(tidy(md), md);
    }
}
