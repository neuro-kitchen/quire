//! Text fields: CSL variables ↔ BibLaTeX fields. Several CSL variables share one BibLaTeX field
//! depending on the entry type (`container-title` is `journaltitle` for articles, `booktitle`
//! otherwise; `issue`, `number` and `collection-number` all compete for `number`), following
//! citation-js `mapping/biblatex.js` (0.9.0).

use quire_model::{EntryKind, ReferenceData, TextField};

use super::types::{is_article, number_is_series_number, writes_series_number};

/// The BibLaTeX field for `field` in an entry of `kind`, or `None` if it is not written.
pub fn to_biblatex(kind: EntryKind, field: TextField, value: &str) -> Option<&'static str> {
    use TextField::*;
    Some(match field {
        Title => "title",
        TitleShort => "shorttitle",
        ContainerTitle if is_article(kind) => "journaltitle",
        ContainerTitle => "booktitle",
        ContainerTitleShort if is_article(kind) => "shortjournal",
        CollectionTitle => "series",
        CollectionNumber if writes_series_number(kind) => "number",
        Volume => "volume",
        NumberOfVolumes => "volumes",
        Issue if takes_issue_number(kind) && value.bytes().any(|b| b.is_ascii_digit()) => "number",
        Issue if is_article(kind) || kind == EntryKind::Periodical => "issue",
        Number if matches!(kind, EntryKind::Patent | EntryKind::Report | EntryKind::Legislation) => "number",
        Number if kind == EntryKind::ArticleJournal => "eid",
        Page => "pages",
        NumberOfPages => "pagetotal",
        Edition => "edition",
        Publisher if matches!(kind, EntryKind::PaperConference | EntryKind::Webpage) => "organization",
        Publisher if matches!(kind, EntryKind::Report | EntryKind::Thesis) => "institution",
        Publisher => "publisher",
        PublisherPlace => "location",
        EventTitle => "eventtitle",
        EventPlace => "venue",
        Genre => "type",
        Version => "version",
        Status => "pubstate",
        Abstract => "abstract",
        Language => "language",
        Url => "url",
        Issn => "issn",
        Isbn => "isbn",
        Note => "note",
        _ => return None,
    })
}

/// Types whose numeric issue is written as `number`.
fn takes_issue_number(kind: EntryKind) -> bool {
    is_article(kind) || matches!(kind, EntryKind::PaperConference | EntryKind::Periodical)
}

/// Value as written to BibLaTeX: page ranges use `--`.
pub fn to_biblatex_value(field: TextField, value: &str) -> String {
    match field {
        TextField::Page => value.replace('–', "--").split("--").map(|part| part.replace('-', "--")).collect::<Vec<_>>().join("--"),
        _ => value.into(),
    }
}

/// Reads BibLaTeX field `name` (lowercase) of an entry of BibLaTeX type `bib_type` into `data`.
/// Returns `false` if the field is not a text field quire maps.
pub fn from_biblatex(bib_type: &str, name: &str, value: &str, data: &mut ReferenceData) -> bool {
    use TextField::*;
    let field = match name {
        "title" => Title,
        "shorttitle" => TitleShort,
        "journaltitle" | "journal" | "booktitle" | "maintitle" => ContainerTitle,
        "shortjournal" => ContainerTitleShort,
        "series" => CollectionTitle,
        "number" if matches!(bib_type, "article" | "periodical" | "inproceedings") => Issue,
        "number" if number_is_series_number(bib_type) => CollectionNumber,
        "number" | "eid" => Number,
        "issue" => Issue,
        "volume" => Volume,
        "volumes" => NumberOfVolumes,
        "pages" => Page,
        "pagetotal" => NumberOfPages,
        "edition" => Edition,
        "publisher" | "organization" | "institution" | "school" => Publisher,
        "location" | "address" => PublisherPlace,
        "eventtitle" => EventTitle,
        "venue" => EventPlace,
        "type" => Genre,
        "version" => Version,
        "pubstate" => Status,
        "abstract" => Abstract,
        "language" | "langid" => Language,
        "url" => Url,
        "issn" => Issn,
        "isbn" => Isbn,
        "note" | "addendum" => Note,
        _ => return false,
    };
    // The first source field wins when several map to one variable (`journaltitle` over
    // `booktitle`, `publisher` over `institution`, …): fields are visited in a fixed order.
    if data.get(field).is_none() {
        let value = if field == Page { value.replace("--", "-").replace('–', "-") } else { value.into() };
        data.set(field, value);
    }
    true
}

/// Order in which BibLaTeX fields are read, so that preferred sources come first.
pub const READ_ORDER: &[&str] = &["journaltitle", "journal", "booktitle", "maintitle", "publisher", "organization", "institution", "school", "location", "address", "note", "addendum", "language", "langid", "number", "issue"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_title_depends_on_type() {
        assert_eq!(to_biblatex(EntryKind::ArticleJournal, TextField::ContainerTitle, "Nature"), Some("journaltitle"));
        assert_eq!(to_biblatex(EntryKind::Chapter, TextField::ContainerTitle, "A Book"), Some("booktitle"));
    }

    #[test]
    fn issue_number_and_series_number() {
        assert_eq!(to_biblatex(EntryKind::ArticleJournal, TextField::Issue, "7460"), Some("number"));
        assert_eq!(to_biblatex(EntryKind::ArticleJournal, TextField::Issue, "Spring"), Some("issue"));
        assert_eq!(to_biblatex(EntryKind::Book, TextField::CollectionNumber, "12"), Some("number"));
        assert_eq!(to_biblatex(EntryKind::Report, TextField::Number, "TR-1"), Some("number"));
        assert_eq!(to_biblatex(EntryKind::Book, TextField::Issue, "3"), None);
    }

    #[test]
    fn page_ranges() {
        assert_eq!(to_biblatex_value(TextField::Page, "54-58"), "54--58");
        assert_eq!(to_biblatex_value(TextField::Page, "54–58"), "54--58");
        assert_eq!(to_biblatex_value(TextField::Page, "54--58"), "54--58");
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        from_biblatex("article", "pages", "54--58", &mut data);
        assert_eq!(data.get(TextField::Page), Some("54-58"));
    }

    #[test]
    fn reading_number_depends_on_type() {
        let read = |bib_type: &str| {
            let mut data = ReferenceData::new(EntryKind::Document);
            from_biblatex(bib_type, "number", "7", &mut data);
            data.text.keys().next().copied()
        };
        assert_eq!(read("article"), Some(TextField::Issue));
        assert_eq!(read("book"), Some(TextField::CollectionNumber));
        assert_eq!(read("report"), Some(TextField::Number));
    }
}
