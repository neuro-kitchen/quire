//! Projects, tags, related references and standalone notes through the bridge.

mod common;
use common::fixture;

#[tokio::test]
async fn projects_hold_rename_and_go() {
    let f = fixture().await;
    f.library.create_project("Thesis".into(), None).await.unwrap();
    assert!(f.library.link("okeefe1971".into(), "thesis".into()).await.unwrap());
    assert!(!f.library.link("okeefe1971".into(), "thesis".into()).await.unwrap(), "already in it");
    let renamed = f.library.rename_project("thesis".into(), "PhD Thesis".into()).await.unwrap();
    assert_eq!((renamed.path.as_str(), renamed.name.as_str(), renamed.references), ("phd-thesis", "PhD Thesis", 1));
    assert_eq!(f.library.reference("okeefe1971".into()).await.unwrap().projects, ["phd-thesis"]);
    assert!(f.library.unlink("okeefe1971".into(), "phd-thesis".into()).await.unwrap());
    assert_eq!(f.library.remove_project("phd-thesis".into(), true).await.unwrap(), 1);
    assert!(f.library.projects().await.unwrap().is_empty());
    assert_eq!(f.library.references(common_query()).await.unwrap().len(), 3, "references stay");
}

#[tokio::test]
async fn tags_untag_rename_color_and_delete() {
    let f = fixture().await;
    f.library.tag("okeefe1971".into(), vec!["maps".into(), "rats".into()]).await.unwrap();
    f.library.tag("hodgkin1952".into(), vec!["rats".into()]).await.unwrap();
    f.library.untag("okeefe1971".into(), vec!["maps".into()]).await.unwrap();
    assert_eq!(f.library.reference("okeefe1971".into()).await.unwrap().tags, ["rats"]);
    assert!(!f.library.rename_tag("rats".into(), "Rodents".into()).await.unwrap());
    let colored = f.library.color_tag("Rodents".into(), Some("#ff0000".into()), None).await.unwrap();
    assert_eq!((colored.name.as_str(), colored.color.as_deref(), colored.count), ("Rodents", Some("#ff0000"), 2));
    assert_eq!(f.library.delete_tag("Rodents".into()).await.unwrap(), 2);
}

#[tokio::test]
async fn related_references_and_standalone_notes() {
    let f = fixture().await;
    assert!(f.library.relate("okeefe1971".into(), "buzsaki2006".into()).await.unwrap());
    let related: Vec<String> =
        f.library.related("buzsaki2006".into()).await.unwrap().into_iter().map(|r| r.cite_key).collect();
    assert_eq!(related, ["okeefe1971"], "both ways");
    assert!(f.library.unrelate("buzsaki2006".into(), "okeefe1971".into()).await.unwrap());
    f.library.add_standalone_note("Ideas for chapter 2".into()).await.unwrap();
    assert_eq!(
        f.library.standalone_notes().await.unwrap().into_iter().map(|n| n.body).collect::<Vec<_>>(),
        ["Ideas for chapter 2"]
    );
}

fn common_query() -> quire_ffi::Query {
    quire_ffi::Query {
        source: quire_ffi::Source::All,
        text: String::new(),
        tags: vec![],
        sort: quire_ffi::SortColumn::Title,
        ascending: true,
    }
}
