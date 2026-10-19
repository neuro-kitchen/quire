//! The `quire-core` use cases on a real (in-memory) SQLite library.

use quire_core::library::{projects, references};
use quire_core::{AddRequest, Added, Library, RawRecord, ReferenceFilter};
use quire_model::{Creator, CreatorRole, Date, EntryKind, Identifier, Origin, ReferenceData, SlugPath, TextField, Timestamp};
use quire_store::{device_id, open_in_memory, SqliteLibrary};
use rusqlite::Connection;

fn article(doi: &str) -> ReferenceData {
    let mut data = ReferenceData::new(EntryKind::ArticleJournal);
    data.creators.push(Creator::person(CreatorRole::Author, "Vaswani", Some("Ashish".into())));
    data.set(TextField::Title, "Attention Is All You Need");
    data.issued = Some(Date::year(2017));
    data.identifiers.push(Identifier::parse(doi).unwrap());
    data
}

fn path(text: &str) -> SlugPath {
    text.parse().unwrap()
}

/// Runs `f` on a library over a transaction and commits.
fn with_library<T>(conn: &mut Connection, f: impl FnOnce(&mut SqliteLibrary) -> T) -> T {
    let device = device_id(conn).unwrap();
    let tx = conn.transaction().unwrap();
    let out = f(&mut SqliteLibrary::new(&tx, device));
    tx.commit().unwrap();
    out
}

#[test]
fn add_dedupe_and_read_back() {
    let mut conn = open_in_memory().unwrap();
    let raw = RawRecord { source: "doi.org".into(), format: "csl-json".into(), body: "{}".into(), received_at: Timestamp::now() };
    let first = with_library(&mut conn, |lib| {
        let mut request = AddRequest::new(article("10.1/a"), Origin::Fetched { source: "doi.org".into() });
        request.raw = Some(raw);
        references::add(lib, request).unwrap()
    });
    let Added::New(stored) = &first else { panic!("expected a new reference") };

    with_library(&mut conn, |lib| {
        let again = references::add(lib, AddRequest::new(article("doi:10.1/A"), Origin::User)).unwrap();
        assert!(matches!(&again, Added::Existing(r) if r.id == stored.id));
        let read = references::find(lib, "vaswaniAttentionAllYou2017").unwrap();
        assert_eq!(&read, stored, "a reference reads back exactly as stored");
    });
    let raw_rows: i64 = conn.query_row("SELECT count(*) FROM raw_record", [], |r| r.get(0)).unwrap();
    assert_eq!(raw_rows, 1);
}

#[test]
fn cite_key_postfix_counts_the_trash() {
    let mut conn = open_in_memory().unwrap();
    with_library(&mut conn, |lib| {
        references::add(lib, AddRequest::new(article("10.1/a"), Origin::User)).unwrap();
        references::trash(lib, "doi:10.1/a").unwrap();
        let second = references::add(lib, AddRequest::new(article("10.1/b"), Origin::User)).unwrap();
        assert_eq!(second.reference().cite_key.as_str(), "vaswaniAttentionAllYou2017a");
        let live = lib.references(&ReferenceFilter::default()).unwrap();
        assert_eq!(live.len(), 1);
    });
}

#[test]
fn projects_links_and_export_selection() {
    let mut conn = open_in_memory().unwrap();
    with_library(&mut conn, |lib| {
        let (_, thesis) = projects::create(lib, "Thesis", None).unwrap();
        let (_, chapter) = projects::create(lib, "Chapter 1", Some(&path("thesis"))).unwrap();
        let mut request = AddRequest::new(article("10.1/a"), Origin::User);
        request.project = Some(thesis.id);
        references::add(lib, request).unwrap();
        let mut request = AddRequest::new(article("10.1/b"), Origin::User);
        request.project = Some(chapter.id);
        references::add(lib, request).unwrap();

        assert_eq!(projects::references(lib, &path("thesis"), false).unwrap().len(), 1);
        assert_eq!(projects::references(lib, &path("thesis"), true).unwrap().len(), 2);
        assert_eq!(projects::references(lib, &path("thesis/chapter-1"), false).unwrap().len(), 1);

        assert!(projects::link(lib, "doi:10.1/a", &path("thesis/chapter-1")).unwrap().1);
        assert!(!projects::link(lib, "doi:10.1/a", &path("thesis/chapter-1")).unwrap().1);
        assert!(projects::unlink(lib, "doi:10.1/a", &path("thesis/chapter-1")).unwrap().1);
        assert!(projects::link(lib, "doi:10.1/a", &path("thesis/chapter-1")).unwrap().1, "re-link after unlink");

        let tree: Vec<String> = projects::tree(lib).unwrap().iter().map(|(p, _)| p.to_string()).collect();
        assert_eq!(tree, ["thesis", "thesis/chapter-1"]);
    });
}

#[test]
fn duplicate_sibling_slug_is_rejected_by_the_database_too() {
    let mut conn = open_in_memory().unwrap();
    with_library(&mut conn, |lib| {
        projects::create(lib, "a", None).unwrap();
        assert!(projects::create(lib, "a", None).is_err());
        projects::create(lib, "a", Some(&path("a"))).unwrap();
    });
}
