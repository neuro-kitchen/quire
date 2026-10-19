//! The `quire-core` use cases on a real (in-memory) SQLite library.

use quire_core::library::{projects, references};
use quire_core::{AddRequest, Added, Library, RawRecord, ReferenceFilter};
use quire_model::{
    Creator, CreatorRole, Date, EntryKind, Identifier, Origin, ReferenceData, SlugPath, TextField, Timestamp,
};
use quire_store::{SqliteLibrary, device_id, open_in_memory};
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
    let raw = RawRecord {
        source: "doi.org".into(),
        format: "csl-json".into(),
        body: "{}".into(),
        received_at: Timestamp::now(),
    };
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

#[test]
fn recorded_issues_and_failed_inputs() {
    use quire_model::{Issue, IssueCode};
    let mut conn = open_in_memory().unwrap();
    with_library(&mut conn, |lib| {
        let mut request = AddRequest::new(article("10.1/a"), Origin::Fetched { source: "pubmed".into() });
        request.issues = vec![Issue::recorded(IssueCode::Fallback, "doi.org: HTTP 503")];
        let added = references::add(lib, request).unwrap();
        let id = added.reference().id;
        let issues = lib.recorded_issues(id).unwrap();
        assert_eq!((issues.len(), issues[0].code), (1, IssueCode::Fallback));
        lib.set_recorded_issues(id, &[]).unwrap();
        assert!(lib.recorded_issues(id).unwrap().is_empty());

        lib.record_failure("10.1/nope", None, "not found").unwrap();
        lib.record_failure("10.1/nope", None, "still not found").unwrap();
        let failures = lib.failures().unwrap();
        assert_eq!((failures.len(), failures[0].attempts, failures[0].error.as_str()), (1, 2, "still not found"));
        assert!(lib.clear_failure("10.1/nope").unwrap());
        assert!(lib.failures().unwrap().is_empty());
    });
}

#[test]
fn tags_notes_and_relations_on_sqlite() {
    use quire_core::library::tags;
    use quire_model::TagKind;
    let mut conn = open_in_memory().unwrap();
    with_library(&mut conn, |lib| {
        let a = references::add(lib, AddRequest::new(article("10.1/a"), Origin::User)).unwrap().reference().clone();
        references::add(lib, AddRequest::new(article("10.1/b"), Origin::User)).unwrap();
        tags::add(lib, "doi:10.1/a", "Méthodes", TagKind::Manual).unwrap();
        assert!(!tags::add(lib, "doi:10.1/a", "MÉTHODES", TagKind::Manual).unwrap().1, "same tag, any case");
        tags::add(lib, "doi:10.1/b", "méthodes", TagKind::Automatic).unwrap();
        tags::color(lib, "méthodes", Some("blue"), None).unwrap();
        let tag = tags::find_tag(lib, "methodes").ok();
        assert!(tag.is_none(), "accents matter");
        let tag = tags::find_tag(lib, "méthodes").unwrap();
        assert_eq!((tag.color.as_deref(), tag.position), (Some("#0090ff"), Some(1)));
        let filter = ReferenceFilter { tags: vec![tag.id], ..ReferenceFilter::default() };
        assert_eq!(lib.references(&filter).unwrap().len(), 2);
        assert!(tags::remove(lib, "doi:10.1/b", "méthodes").unwrap().1);
        assert!(tags::add(lib, "doi:10.1/b", "méthodes", TagKind::Manual).unwrap().1, "re-added after removal");
        assert_eq!(lib.tags_of(a.id).unwrap()[0].1, TagKind::Manual);

        let note = tags::add_note(lib, Some("doi:10.1/a"), "first note").unwrap();
        assert_eq!(lib.note_counts().unwrap(), [(a.id, 1)]);
        tags::edit_note(lib, note.id, "edited").unwrap();
        assert_eq!(lib.note(note.id).unwrap().unwrap().body, "edited");

        assert!(tags::relate(lib, "doi:10.1/b", "doi:10.1/a").unwrap().2);
        assert_eq!(lib.related(a.id).unwrap().len(), 1);
        assert!(tags::unrelate(lib, "doi:10.1/a", "doi:10.1/b").unwrap().2);
        assert!(tags::relate(lib, "doi:10.1/a", "doi:10.1/b").unwrap().2, "related again after removal");
    });
}
