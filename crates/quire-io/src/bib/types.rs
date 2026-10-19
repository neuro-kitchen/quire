//! Entry types: CSL ↔ BibLaTeX, from citation-js `mapping/biblatexTypes.json` (0.9.0).
//! `map` and `pamphlet` have no target there and fall back to `misc`, as citation-js does.
//! One deviation: CSL `article` (a preprint, e.g. arXiv) is written as `@online`, not `@article`,
//! because Biber warns about an `@article` without `journaltitle`.

use quire_model::EntryKind;

/// BibLaTeX entry type for a CSL type (`target` table).
pub fn to_biblatex(kind: EntryKind) -> &'static str {
    use EntryKind::*;
    match kind {
        Article => "online",
        ArticleJournal | ArticleMagazine | ArticleNewspaper => "article",
        Bill | Legislation => "legislation",
        Book => "book",
        Broadcast | Interview | Speech => "audio",
        Chapter => "inbook",
        Classic | Manuscript => "unpublished",
        Collection | Document | Event | Map | Pamphlet => "misc",
        Dataset => "dataset",
        Entry | EntryDictionary | EntryEncyclopedia => "inreference",
        Figure | Graphic => "artwork",
        Hearing | Regulation | Treaty => "legal",
        LegalCase => "jurisdiction",
        MotionPicture => "movie",
        MusicalScore | Song => "music",
        PaperConference => "inproceedings",
        Patent => "patent",
        Performance => "performance",
        Periodical => "periodical",
        PersonalCommunication => "letter",
        Post | PostWeblog | Webpage => "online",
        Report => "report",
        Review | ReviewBook => "review",
        Software => "software",
        Standard => "standard",
        Thesis => "thesis",
    }
}

/// CSL type for a BibLaTeX (or BibTeX) entry type name (`source` table); unknown → `document`.
pub fn from_biblatex(name: &str) -> EntryKind {
    use EntryKind::*;
    match name.to_ascii_lowercase().as_str() {
        "article" => ArticleJournal,
        "book" | "mvbook" | "bookinbook" | "booklet" | "collection" | "mvcollection" | "proceedings"
        | "mvproceedings" | "reference" | "mvreference" => Book,
        "inbook" | "incollection" => Chapter,
        "dataset" => Dataset,
        "manual" | "report" | "techreport" => Report,
        "online" | "electronic" | "www" => Webpage,
        "patent" => Patent,
        "periodical" => Periodical,
        "inproceedings" | "conference" => PaperConference,
        "inreference" => Entry,
        "software" => Software,
        "thesis" | "mastersthesis" | "phdthesis" => Thesis,
        "unpublished" => Manuscript,
        "artwork" => Graphic,
        "audio" => Song,
        "image" => Figure,
        "jurisdiction" => LegalCase,
        "legislation" => Legislation,
        "legal" => Treaty,
        "letter" => PersonalCommunication,
        "movie" | "video" => MotionPicture,
        "music" => MusicalScore,
        "performance" => Performance,
        "review" => Review,
        "standard" => Standard,
        _ => Document,
    }
}

/// Journal-like types: container title is `journaltitle`, issue goes to `number`.
pub fn is_article(kind: EntryKind) -> bool {
    matches!(
        kind,
        EntryKind::Article | EntryKind::ArticleJournal | EntryKind::ArticleMagazine | EntryKind::ArticleNewspaper
    )
}

/// BibLaTeX source types whose `number` is the series number (citation-js `collection-number` rule).
pub fn number_is_series_number(biblatex_type: &str) -> bool {
    matches!(
        biblatex_type,
        "book"
            | "mvbook"
            | "inbook"
            | "bookinbook"
            | "suppbook"
            | "collection"
            | "mvcollection"
            | "incollection"
            | "suppcollection"
            | "manual"
            | "suppperiodical"
            | "proceedings"
            | "mvproceedings"
            | "reference"
    )
}

/// CSL types whose `collection-number` is written as BibLaTeX `number` (citation-js target list).
pub fn writes_series_number(kind: EntryKind) -> bool {
    use EntryKind::*;
    matches!(
        kind,
        Bill | Book
            | Broadcast
            | Chapter
            | Dataset
            | Entry
            | EntryDictionary
            | EntryEncyclopedia
            | Figure
            | Graphic
            | Interview
            | Legislation
            | LegalCase
            | Manuscript
            | Map
            | MotionPicture
            | MusicalScore
            | Pamphlet
            | Post
            | PostWeblog
            | PersonalCommunication
            | Review
            | ReviewBook
            | Song
            | Speech
            | Thesis
            | Treaty
            | Webpage
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_csl_type_has_a_biblatex_type() {
        for kind in EntryKind::ALL {
            assert!(!to_biblatex(*kind).is_empty());
        }
    }

    #[test]
    fn common_types_round_trip() {
        use EntryKind::*;
        for kind in [ArticleJournal, Book, Chapter, PaperConference, Report, Thesis, Webpage, Dataset, Software, Patent]
        {
            assert_eq!(from_biblatex(to_biblatex(kind)), kind, "{kind}");
        }
        assert_eq!(from_biblatex("PhDThesis"), Thesis);
        assert_eq!(from_biblatex("nonsense"), Document);
    }
}
