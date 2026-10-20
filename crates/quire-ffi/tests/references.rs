//! Adding and changing references through the bridge: typed in, fields, edits, trash, issues.

mod common;
use common::fixture;

use quire_ffi::{CreatorInput, ReferenceFields};

fn typed() -> ReferenceFields {
    ReferenceFields {
        kind: "article-journal".into(),
        title: "Grid cells in the entorhinal cortex".into(),
        creators: vec![
            CreatorInput {
                role: "author".into(),
                family: "Hafting".into(),
                given: Some("Torkel".into()),
                literal: false,
            },
            CreatorInput { role: "author".into(), family: "Kavli Institute".into(), given: None, literal: true },
        ],
        issued: Some("2005-08".into()),
        container_title: Some("Nature".into()),
        publisher: None,
        volume: Some("436".into()),
        issue: None,
        pages: Some("801-806".into()),
        url: None,
        abstract_text: None,
        identifiers: vec!["10.1038/nature03721".into()],
    }
}

#[tokio::test]
async fn a_typed_reference_is_added_and_reads_back() {
    let f = fixture().await;
    let added = f.library.add_manual(typed(), None).await.unwrap();
    assert!(added.new);
    assert_eq!(added.reference.title, "Grid cells in the entorhinal cortex");
    assert_eq!(added.reference.doi.as_deref(), Some("10.1038/nature03721"));
    assert_eq!(added.reference.date.as_deref(), Some("2005-08"));
    let fields = f.library.fields(added.reference.id.clone()).await.unwrap();
    // Identifiers read back labelled; they parse either way.
    assert_eq!(fields, ReferenceFields { identifiers: vec!["doi:10.1038/nature03721".into()], ..typed() });
    let again = f.library.add_manual(typed(), None).await.unwrap();
    assert!(!again.new, "same DOI: already in the library");
}

#[tokio::test]
async fn editing_reports_what_changed() {
    let f = fixture().await;
    let mut fields = f.library.fields("okeefe1971".into()).await.unwrap();
    fields.volume = Some("34".into());
    let preview = f.library.edit("okeefe1971".into(), fields.clone(), None, true).await.unwrap();
    assert_eq!(preview.len(), 1, "{preview:?}");
    assert_eq!((preview[0].field.as_str(), preview[0].new.as_deref()), ("volume", Some("34")));
    assert_eq!(f.library.fields("okeefe1971".into()).await.unwrap().volume, None, "a dry run writes nothing");
    f.library.edit("okeefe1971".into(), fields, None, false).await.unwrap();
    assert_eq!(f.library.reference("okeefe1971".into()).await.unwrap().volume.as_deref(), Some("34"));
}

#[tokio::test]
async fn trash_restore_and_issues() {
    let f = fixture().await;
    f.library.trash("buzsaki2006".into()).await.unwrap();
    let all = f.library.references(quire_ffi::Query {
        source: quire_ffi::Source::All,
        text: String::new(),
        tags: vec![],
        sort: quire_ffi::SortColumn::Title,
        ascending: true,
    });
    assert!(all.await.unwrap().iter().all(|r| r.cite_key != "buzsaki2006"));
    f.library.restore("buzsaki2006".into()).await.unwrap();
    // A book with no ISBN: a lint, at least.
    let issues = f.library.issues("buzsaki2006".into()).await.unwrap();
    assert!(issues.iter().all(|i| !i.code.is_empty() && !i.message.is_empty()), "{issues:?}");
}

#[tokio::test]
async fn bad_fields_are_refused_with_a_message() {
    let f = fixture().await;
    let mut fields = typed();
    fields.issued = Some("August 2005".into());
    let error = f.library.add_manual(fields, None).await.unwrap_err();
    assert!(error.to_string().contains("August 2005"), "{error}");
    assert!(f.library.failures().await.unwrap().is_empty());
}

#[tokio::test]
async fn pasted_bib_text_is_imported() {
    let f = fixture().await;
    let pasted = "@article{moser2008, title = {Place cells, grid cells, and the brain's spatial system}, author = {Moser, Edvard}, year = {2008}, doi = {10.1146/annurev.neuro.31.061307.090723}}";
    assert_eq!(f.library.import_bib_text(pasted.into(), None).await.unwrap(), 1);
    assert_eq!(f.library.reference("moser2008".into()).await.unwrap().year, Some(2008));
    assert_eq!(f.library.import_bib_text(pasted.into(), None).await.unwrap(), 0, "same DOI: already in the library");
    let error = f.library.import_bib_text("@article{broken, title = {no end".into(), None).await.unwrap_err();
    assert!(error.to_string().contains("end of file"), "{error}");
}
