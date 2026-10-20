//! Zotero's data (`zotero.sqlite`, read by the engine) as quire's model: item types and fields
//! to CSL, creators, dates (`2017-06-12 June 12, 2017`), identifiers in `DOI`, `archiveID` and
//! the `extra` field, and notes (HTML) as text. Pure: the reader fills [`ZoteroItem`]s.
//!
//! Field names are Zotero's (`publicationTitle`, `bookTitle`, …); the type-specific ones are
//! mapped like their base field (Zotero's `baseFieldMappings`), so the reader can pass them as
//! stored.

use std::collections::BTreeMap;

use quire_model::{Creator, CreatorRole, Date, EntryKind, Identifier, ReferenceData, TextField};

use crate::text::decode;

/// What is imported from a Zotero library (its "My Library"; items in the trash left out).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ZoteroLibrary {
    pub items: Vec<ZoteroItem>,
    /// Parents before their subcollections.
    pub collections: Vec<ZoteroCollection>,
    pub notes: Vec<ZoteroNote>,
    pub attachments: Vec<ZoteroAttachment>,
    pub annotations: Vec<ZoteroAnnotation>,
}

/// One regular item (not a note, attachment or annotation).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ZoteroItem {
    /// Zotero's 8-character key (`N7SMB24A`).
    pub key: String,
    /// Item type name (`journalArticle`, `book`, …).
    pub item_type: String,
    /// Field name → value, as stored.
    pub fields: BTreeMap<String, String>,
    /// In order.
    pub creators: Vec<ZoteroCreator>,
    /// Tag names; `true` for automatic tags (from imported data).
    pub tags: Vec<(String, bool)>,
    /// Keys of related items.
    pub related: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoteroCollection {
    pub key: String,
    pub name: String,
    /// The parent collection's key.
    pub parent: Option<String>,
    /// Keys of its items.
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoteroNote {
    /// The item it belongs to (`None`: a standalone note).
    pub parent: Option<String>,
    pub html: String,
}

/// A file attached to an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoteroAttachment {
    pub key: String,
    /// The item it belongs to (`None`: a standalone file).
    pub parent: Option<String>,
    pub content_type: Option<String>,
    /// Where the file is: `storage:<name>` (in `storage/<key>/`), an absolute path (linked file),
    /// `attachments:<relative>` (under the linked-files base folder), or `None` (a link).
    pub path: Option<String>,
}

/// A mark made in Zotero's PDF reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoteroAnnotation {
    /// The attachment (PDF) it is on.
    pub attachment: String,
    pub kind: AnnotationKind,
    /// The marked text (highlights, underlines).
    pub text: Option<String>,
    pub comment: Option<String>,
    pub page_label: Option<String>,
}

/// Zotero's annotation types (`itemAnnotations.type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationKind {
    Highlight,
    Note,
    Image,
    Ink,
    Underline,
    Text,
    Other,
}

impl AnnotationKind {
    pub fn from_code(code: i64) -> Self {
        match code {
            1 => Self::Highlight,
            2 => Self::Note,
            3 => Self::Image,
            4 => Self::Ink,
            5 => Self::Underline,
            6 => Self::Text,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoteroCreator {
    /// Creator type name (`author`, `editor`, `bookAuthor`, …).
    pub role: String,
    pub first: String,
    pub last: String,
    /// Stored as one field (an organisation): `last` holds the whole name.
    pub single_field: bool,
}

/// A Zotero item as quire data, with the citation key it carries (Zotero 7's `citationKey`
/// field, or Better BibTeX's `Citation Key:` line in `extra`).
#[derive(Debug, Clone, PartialEq)]
pub struct ZoteroRecord {
    pub data: ReferenceData,
    pub cite_key: Option<String>,
}

/// Maps `item` to quire's model.
pub fn to_record(item: &ZoteroItem) -> ZoteroRecord {
    let mut data = ReferenceData::new(kind(&item.item_type));
    let mut cite_key = None;
    for (name, value) in &item.fields {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match name.as_str() {
            "date" => data.issued = date(value),
            "DOI" => add_identifier(&mut data, value),
            // Preprints: `arXiv:2101.00001`; other archives' IDs are not identifiers we know.
            "archiveID" => {
                if value.to_ascii_lowercase().starts_with("arxiv:") {
                    add_identifier(&mut data, value);
                }
            }
            "citationKey" => cite_key = Some(value.to_string()),
            "extra" => {
                let rest = read_extra(value, &mut data, &mut cite_key);
                if !rest.is_empty() {
                    data.set(TextField::Note, rest);
                }
            }
            other => {
                if let Some(field) = text_field(other) {
                    data.set(field, value);
                }
            }
        }
    }
    data.creators = item.creators.iter().map(creator).collect();
    ZoteroRecord { data, cite_key }
}

/// CSL type of a Zotero item type (as Zotero's own CSL export maps them).
fn kind(item_type: &str) -> EntryKind {
    match item_type {
        "journalArticle" => EntryKind::ArticleJournal,
        "magazineArticle" => EntryKind::ArticleMagazine,
        "newspaperArticle" => EntryKind::ArticleNewspaper,
        "preprint" => EntryKind::Article,
        "book" => EntryKind::Book,
        "bookSection" => EntryKind::Chapter,
        "conferencePaper" => EntryKind::PaperConference,
        "thesis" => EntryKind::Thesis,
        "report" => EntryKind::Report,
        "webpage" => EntryKind::Webpage,
        "blogPost" => EntryKind::PostWeblog,
        "forumPost" => EntryKind::Post,
        "dataset" => EntryKind::Dataset,
        "computerProgram" => EntryKind::Software,
        "patent" => EntryKind::Patent,
        "manuscript" => EntryKind::Manuscript,
        "presentation" => EntryKind::Speech,
        "encyclopediaArticle" => EntryKind::EntryEncyclopedia,
        "dictionaryEntry" => EntryKind::EntryDictionary,
        "letter" | "email" | "instantMessage" => EntryKind::PersonalCommunication,
        "interview" => EntryKind::Interview,
        "film" | "videoRecording" => EntryKind::MotionPicture,
        "audioRecording" | "podcast" => EntryKind::Song,
        "radioBroadcast" | "tvBroadcast" => EntryKind::Broadcast,
        "artwork" => EntryKind::Graphic,
        "map" => EntryKind::Map,
        "case" => EntryKind::LegalCase,
        "statute" => EntryKind::Legislation,
        "bill" => EntryKind::Bill,
        "hearing" => EntryKind::Hearing,
        "standard" => EntryKind::Standard,
        _ => EntryKind::Document,
    }
}

/// The CSL variable a Zotero field (or a type-specific field of the same base) is.
fn text_field(name: &str) -> Option<TextField> {
    Some(match name {
        "title" | "caseName" | "nameOfAct" | "subject" => TextField::Title,
        "shortTitle" => TextField::TitleShort,
        "publicationTitle" | "bookTitle" | "proceedingsTitle" | "websiteTitle" | "blogTitle" | "forumTitle"
        | "encyclopediaTitle" | "dictionaryTitle" | "programTitle" | "reporter" | "code" => TextField::ContainerTitle,
        "journalAbbreviation" => TextField::ContainerTitleShort,
        "series" => TextField::CollectionTitle,
        "seriesNumber" => TextField::CollectionNumber,
        "volume" | "codeVolume" | "reporterVolume" => TextField::Volume,
        "numberOfVolumes" => TextField::NumberOfVolumes,
        "issue" => TextField::Issue,
        "pages" | "firstPage" | "codePages" => TextField::Page,
        "numPages" => TextField::NumberOfPages,
        "number" | "reportNumber" | "billNumber" | "patentNumber" | "episodeNumber" | "docketNumber"
        | "publicLawNumber" => TextField::Number,
        "edition" => TextField::Edition,
        "publisher" | "university" | "institution" | "label" | "company" | "repository" | "distributor" | "studio"
        | "network" | "court" | "legislativeBody" | "organization" => TextField::Publisher,
        "place" => TextField::PublisherPlace,
        "conferenceName" | "meetingName" => TextField::EventTitle,
        "type" | "thesisType" | "reportType" | "manuscriptType" | "letterType" | "presentationType" | "websiteType"
        | "postType" | "mapType" | "genre" => TextField::Genre,
        "medium"
        | "artworkMedium"
        | "audioRecordingFormat"
        | "videoRecordingFormat"
        | "interviewMedium"
        | "audioFileType" => TextField::Medium,
        "versionNumber" => TextField::Version,
        "abstractNote" => TextField::Abstract,
        "language" => TextField::Language,
        "url" => TextField::Url,
        "ISSN" => TextField::Issn,
        "ISBN" => TextField::Isbn,
        _ => return None,
    })
}

fn creator(c: &ZoteroCreator) -> Creator {
    let role = match c.role.as_str() {
        "editor" => CreatorRole::Editor,
        "seriesEditor" => CreatorRole::CollectionEditor,
        "bookAuthor" => CreatorRole::ContainerAuthor,
        "reviewedAuthor" => CreatorRole::ReviewedAuthor,
        "translator" => CreatorRole::Translator,
        "contributor" => CreatorRole::Contributor,
        "director" => CreatorRole::Director,
        "interviewer" => CreatorRole::Interviewer,
        "recipient" => CreatorRole::Recipient,
        "producer" => CreatorRole::Producer,
        "scriptwriter" => CreatorRole::ScriptWriter,
        "guest" => CreatorRole::Guest,
        "host" | "podcaster" => CreatorRole::Host,
        "performer" | "castMember" => CreatorRole::Performer,
        "composer" => CreatorRole::Composer,
        _ => CreatorRole::Author,
    };
    let given = Some(c.first.trim()).filter(|g| !g.is_empty()).map(str::to_string);
    if c.single_field || given.is_none() {
        Creator::literal(role, c.last.trim())
    } else {
        Creator::person(role, c.last.trim(), given)
    }
}

/// Zotero's stored date: `YYYY-MM-DD` (with `00` for unknown parts), a space, then the date as
/// typed. Falls back to a four-digit year anywhere in the text.
fn date(value: &str) -> Option<Date> {
    let sql = value.split_whitespace().next()?;
    let mut parts = sql.split('-').map(|p| p.parse::<u16>().ok());
    if let (Some(Some(year)), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next())
        && year > 0
    {
        let month = month.filter(|m| (1..=12).contains(m)).map(|m| m as u8);
        let day = day.filter(|d| *d > 0 && month.is_some()).map(|d| d as u8);
        if let Ok(date) = Date::new(year, month, day) {
            return Some(date);
        }
    }
    value.split(|c: char| !c.is_ascii_digit()).find(|w| w.len() == 4).and_then(|y| y.parse().ok()).map(Date::year)
}

fn add_identifier(data: &mut ReferenceData, text: &str) {
    if let Ok(identifier) = Identifier::parse(text)
        && !data.identifiers.iter().any(|i| i.kind() == identifier.kind() && i.key() == identifier.key())
    {
        data.identifiers.push(identifier);
    }
}

/// Reads the `Label: value` lines of `extra` that are identifiers or a citation key (as Zotero
/// and Better BibTeX write them); returns the other lines.
fn read_extra(extra: &str, data: &mut ReferenceData, cite_key: &mut Option<String>) -> String {
    let mut rest = Vec::new();
    for line in extra.lines() {
        let Some((label, value)) = line.split_once(':') else {
            rest.push(line);
            continue;
        };
        let value = value.trim();
        match label.trim().to_ascii_lowercase().as_str() {
            "doi" => add_identifier(data, value),
            // Bare digits read as a PMID only when labelled so.
            "pmid" => add_identifier(data, &format!("pmid:{value}")),
            "pmcid" => add_identifier(data, value),
            "arxiv" => add_identifier(data, &format!("arXiv:{value}")),
            "citation key" if cite_key.is_none() => *cite_key = Some(value.to_string()),
            _ => rest.push(line),
        }
    }
    rest.join("\n").trim().to_string()
}

/// A Zotero note (HTML) as plain text: paragraphs and line breaks kept, tags dropped, entities
/// decoded.
pub fn note_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&decode(&rest[..start]));
        let Some(end) = rest[start..].find('>') else {
            rest = &rest[start..];
            break;
        };
        let tag = rest[start + 1..start + end].trim_start_matches('/').to_ascii_lowercase();
        let name = tag.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("");
        match name {
            "br" => out.push('\n'),
            "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "blockquote" | "pre" | "tr"
                if !out.ends_with('\n') && !out.is_empty() =>
            {
                out.push('\n')
            }
            _ => {}
        }
        rest = &rest[start + end + 1..];
    }
    out.push_str(&decode(rest));
    // At most one empty line between paragraphs.
    let lines: Vec<&str> = out.lines().map(str::trim_end).collect();
    let mut text = String::new();
    for line in lines {
        if line.is_empty() && (text.is_empty() || text.ends_with("\n\n")) {
            continue;
        }
        text.push_str(line);
        text.push('\n');
    }
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use quire_model::Name;

    use super::*;

    fn item(item_type: &str, fields: &[(&str, &str)]) -> ZoteroItem {
        ZoteroItem {
            key: "ABCD2345".into(),
            item_type: item_type.into(),
            fields: fields.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            ..ZoteroItem::default()
        }
    }

    #[test]
    fn journal_article() {
        let mut article = item(
            "journalArticle",
            &[
                ("title", "Nanometre-scale thermometry in a living cell"),
                ("publicationTitle", "Nature"),
                ("journalAbbreviation", "Nature"),
                ("volume", "500"),
                ("issue", "7460"),
                ("pages", "54-58"),
                ("date", "2013-07-31 2013-07-31"),
                ("DOI", "10.1038/nature12373"),
                ("extra", "PMID: 23903748\nCitation Key: kucsko2013\nSome other note"),
                ("accessDate", "2024-01-01 10:00:00"),
            ],
        );
        article.creators = vec![
            ZoteroCreator { role: "author".into(), first: "G.".into(), last: "Kucsko".into(), single_field: false },
            ZoteroCreator { role: "author".into(), first: String::new(), last: "LIGO".into(), single_field: true },
        ];
        let record = to_record(&article);
        let data = &record.data;
        assert_eq!(data.kind, EntryKind::ArticleJournal);
        assert_eq!(data.title(), Some("Nanometre-scale thermometry in a living cell"));
        assert_eq!(data.get(TextField::ContainerTitle), Some("Nature"));
        assert_eq!(data.get(TextField::Page), Some("54-58"));
        assert_eq!(data.issued, Some(Date::new(2013, Some(7), Some(31)).unwrap()));
        let ids: Vec<String> = data.identifiers.iter().map(ToString::to_string).collect();
        assert_eq!(ids, ["doi:10.1038/nature12373", "pmid:23903748"]);
        assert_eq!(record.cite_key.as_deref(), Some("kucsko2013"));
        assert_eq!(data.get(TextField::Note), Some("Some other note"));
        assert!(matches!(&data.creators[0].name, Name::Person(p) if p.family == "Kucsko"));
        assert!(matches!(&data.creators[1].name, Name::Literal { literal } if literal == "LIGO"));
    }

    #[test]
    fn type_specific_fields_and_partial_dates() {
        let chapter = item(
            "bookSection",
            &[("bookTitle", "Handbook"), ("date", "2019-00-00 2019"), ("citationKey", "smith2019")],
        );
        let record = to_record(&chapter);
        assert_eq!(record.data.kind, EntryKind::Chapter);
        assert_eq!(record.data.get(TextField::ContainerTitle), Some("Handbook"));
        assert_eq!(record.data.issued, Some(Date::year(2019)));
        assert_eq!(record.cite_key.as_deref(), Some("smith2019"));

        let preprint = item("preprint", &[("archiveID", "arXiv:1706.03762"), ("date", "June 2017")]);
        let record = to_record(&preprint);
        assert_eq!(record.data.identifiers[0].to_string(), "arXiv:1706.03762");
        assert_eq!(record.data.issued, Some(Date::year(2017)));
        assert_eq!(to_record(&item("mystery", &[])).data.kind, EntryKind::Document);
    }

    #[test]
    fn notes_as_text() {
        let html = r#"<div data-schema-version="9"><h1>Key idea</h1><p>Spins &amp; <b>heat</b>&nbsp;work.</p><p>Line<br/>two &#8212; &#x2014;</p></div>"#;
        assert_eq!(note_text(html), "Key idea\nSpins & heat work.\nLine\ntwo — —");
        assert_eq!(note_text("plain & simple"), "plain & simple");
        assert_eq!(note_text("a < b"), "a < b");
    }
}
