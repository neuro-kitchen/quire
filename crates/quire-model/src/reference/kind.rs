//! [`EntryKind`]: what sort of work a reference is, named as in CSL 1.0.2 (Appendix III, 45 types).
//! BibLaTeX entry types are mapped to and from these in `quire-io`.

use crate::error::ModelError;

name_enum! {
    /// A CSL 1.0.2 item type.
    pub enum EntryKind (unknown: ModelError::UnknownEntryKind) {
        Article => "article",
        ArticleJournal => "article-journal",
        ArticleMagazine => "article-magazine",
        ArticleNewspaper => "article-newspaper",
        Bill => "bill",
        Book => "book",
        Broadcast => "broadcast",
        Chapter => "chapter",
        Classic => "classic",
        Collection => "collection",
        Dataset => "dataset",
        Document => "document",
        Entry => "entry",
        EntryDictionary => "entry-dictionary",
        EntryEncyclopedia => "entry-encyclopedia",
        Event => "event",
        Figure => "figure",
        Graphic => "graphic",
        Hearing => "hearing",
        Interview => "interview",
        LegalCase => "legal_case",
        Legislation => "legislation",
        Manuscript => "manuscript",
        Map => "map",
        MotionPicture => "motion_picture",
        MusicalScore => "musical_score",
        Pamphlet => "pamphlet",
        PaperConference => "paper-conference",
        Patent => "patent",
        Performance => "performance",
        Periodical => "periodical",
        PersonalCommunication => "personal_communication",
        Post => "post",
        PostWeblog => "post-weblog",
        Regulation => "regulation",
        Report => "report",
        Review => "review",
        ReviewBook => "review-book",
        Software => "software",
        Song => "song",
        Speech => "speech",
        Standard => "standard",
        Thesis => "thesis",
        Treaty => "treaty",
        Webpage => "webpage",
    }
}

#[cfg(test)]
mod tests {
    use super::EntryKind;

    #[test]
    fn all_45_csl_types_round_trip() {
        assert_eq!(EntryKind::ALL.len(), 45);
        for kind in EntryKind::ALL {
            assert_eq!(kind.as_str().parse::<EntryKind>().unwrap(), *kind);
        }
        assert!("journal-article".parse::<EntryKind>().is_err());
    }
}
