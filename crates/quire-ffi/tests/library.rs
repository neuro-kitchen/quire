//! The bridge as the app uses it: open a library, import, query by source, tags, words and sort.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use quire_ffi::{ChangeListener, Query, QuireLibrary, SortColumn, Source, open_library_at};

const BIB: &str = r#"
@article{hodgkin1952,
  title = {A quantitative description of membrane current and its application to conduction and excitation in nerve},
  author = {Hodgkin, A. L. and Huxley, A. F.},
  journal = {The Journal of Physiology},
  year = {1952}, volume = {117}, number = {4}, pages = {500--544},
  doi = {10.1113/jphysiol.1952.sp004764},
}
@article{okeefe1971,
  title = {The hippocampus as a spatial map},
  author = {O'Keefe, J. and Dostrovsky, J.},
  journal = {Brain Research},
  year = {1971},
  doi = {10.1016/0006-8993(71)90358-1},
}
@book{buzsaki2006,
  title = {Rhythms of the Brain},
  author = {Buzsáki, György},
  publisher = {Oxford University Press},
  year = {2006},
}
"#;

struct Fixture {
    _dir: tempfile::TempDir,
    library: Arc<QuireLibrary>,
}

async fn fixture() -> Fixture {
    // SAFETY: tests in this file set the same value; no other thread reads the environment yet.
    unsafe { std::env::set_var("QUIRE_NO_KEYRING", "1") };
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let library = open_library_at(dir.path().join("lib").display().to_string(), Some(config.display().to_string()))
        .await
        .unwrap();
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, BIB).unwrap();
    assert_eq!(library.import_bib(bib.display().to_string(), None).await.unwrap(), 3);
    Fixture { _dir: dir, library }
}

fn query(source: Source) -> Query {
    Query { source, text: String::new(), tags: vec![], sort: SortColumn::Title, ascending: true }
}

fn keys(records: &[quire_ffi::ReferenceRecord]) -> Vec<&str> {
    records.iter().map(|r| r.cite_key.as_str()).collect()
}

#[tokio::test]
async fn lists_sorts_and_describes_references() {
    let f = fixture().await;
    let all = f.library.references(query(Source::All)).await.unwrap();
    assert_eq!(keys(&all), ["hodgkin1952", "buzsaki2006", "okeefe1971"]);
    let hodgkin = &all[0];
    assert_eq!(hodgkin.kind, "article-journal");
    assert_eq!(hodgkin.creators[1].display_name, "Huxley, A. F.");
    assert_eq!(hodgkin.year, Some(1952));
    assert_eq!(hodgkin.container_title.as_deref(), Some("The Journal of Physiology"));
    assert_eq!(hodgkin.doi.as_deref(), Some("10.1113/jphysiol.1952.sp004764"));
    assert_eq!(hodgkin.reading, "unread");

    let by_year = Query { sort: SortColumn::Year, ascending: false, ..query(Source::All) };
    assert_eq!(keys(&f.library.references(by_year).await.unwrap()), ["buzsaki2006", "okeefe1971", "hodgkin1952"]);
}

#[tokio::test]
async fn projects_unfiled_tags_and_search_narrow_the_list() {
    let f = fixture().await;
    let thesis = f.library.create_project("Thesis".into(), None).await.unwrap();
    let chapter = f.library.create_project("Chapter 2".into(), Some(thesis.path.clone())).await.unwrap();
    assert_eq!(chapter.parent.as_deref(), Some(thesis.path.as_str()));
    let path = &chapter.path;

    // Linking goes through the import, which takes a project.
    let bib = f._dir.path().join("one.bib");
    std::fs::write(&bib, BIB.split("@article{okeefe1971").next().unwrap()).unwrap();
    f.library.import_bib(bib.display().to_string(), Some(path.clone())).await.unwrap();

    let in_chapter = f.library.references(query(Source::Project { path: path.clone() })).await.unwrap();
    assert_eq!(keys(&in_chapter), ["hodgkin1952"]);
    assert_eq!(in_chapter[0].projects, std::slice::from_ref(path));
    assert_eq!(keys(&f.library.references(query(Source::Unfiled)).await.unwrap()), ["buzsaki2006", "okeefe1971"]);

    let projects = f.library.projects().await.unwrap();
    assert_eq!(projects.iter().map(|p| p.path.as_str()).collect::<Vec<_>>(), [thesis.path.as_str(), path.as_str()]);

    f.library.tag("okeefe1971".into(), vec!["hippocampus".into()]).await.unwrap();
    let tagged = Query { tags: vec!["hippocampus".into()], ..query(Source::All) };
    assert_eq!(keys(&f.library.references(tagged).await.unwrap()), ["okeefe1971"]);

    let words = Query { text: "rhythms".into(), ..query(Source::All) };
    assert_eq!(keys(&f.library.references(words).await.unwrap()), ["buzsaki2006"]);
}

struct Counter(AtomicUsize);

impl ChangeListener for Counter {
    fn on_change(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn changes_reach_subscribers() {
    let f = fixture().await;
    let counter = Arc::new(Counter(AtomicUsize::new(0)));
    let subscription = f.library.subscribe(counter.clone());
    f.library.tag("buzsaki2006".into(), vec!["oscillations".into()]).await.unwrap();
    for _ in 0..50 {
        if counter.0.load(Ordering::SeqCst) > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(counter.0.load(Ordering::SeqCst) > 0, "the listener heard the change");
    subscription.cancel();
}
