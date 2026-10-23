//! Sync between two devices on real libraries, over an in-memory pipe (the QUIC transport only
//! carries the same session): a new copy joins, gets references, a note, a highlight and the PDF
//! file; nothing is sent twice; a later change on either side reaches the other.

mod common;
use common::engine;
use quire_testkit::tiny_pdf;

use quire_engine::{Engine, SyncSide};

const BIB: &str = "@article{rivera2025,
  title = {Diaphragm pacing restores breathing after injury},
  author = {Rivera, Ana},
  journal = {Journal of Tests},
  year = {2025},
}";

/// One session between `a` (connecting) and `b`.
async fn sync(a: &Engine, b: &Engine) -> (quire_engine::SyncReport, quire_engine::SyncReport) {
    let (left, right) = tokio::io::duplex(64 * 1024);
    let (mut left_read, mut left_write) = tokio::io::split(left);
    let (mut right_read, mut right_write) = tokio::io::split(right);
    let (client, server) = tokio::join!(
        a.sync_session(SyncSide::Client, &mut left_read, &mut left_write),
        b.sync_session(SyncSide::Server, &mut right_read, &mut right_write),
    );
    (client.unwrap(), server.unwrap())
}

#[tokio::test]
async fn a_new_device_joins_and_changes_flow_both_ways() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (engine(dir_a.path(), None).await, engine(dir_b.path(), None).await);
    // Two devices of one person: the same user key.
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    let bib = dir_a.path().join("refs.bib");
    std::fs::write(&bib, BIB).unwrap();
    let key = a.import_bib(&bib, None).await.unwrap()[0].reference().cite_key.to_string();
    a.create_project("Thesis", None).await.unwrap();
    a.link(&key, &"thesis".parse().unwrap()).await.unwrap();
    a.add_note(Some(&key), "compare with the 2019 study").await.unwrap();
    a.add_highlight(&key, "restores breathing", "", Some("main claim")).await.unwrap();
    let (reference, _) = a.show(&key).await.unwrap();
    a.attach_bytes(&reference, &tiny_pdf("Diaphragm pacing restores breathing after injury"), "test", false)
        .await
        .unwrap();

    let (client, server) = sync(&b, &a).await;
    assert!(client.joined, "B was new: it joined A's library");
    assert!(client.received.applied >= 5, "{client:?}");
    assert_eq!((client.files_received, server.files_sent), (1, 1));

    let (on_b, projects) = b.show(&key).await.unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(b.notes(Some(&key)).await.unwrap().len(), 1);
    assert_eq!(b.highlights(&key).await.unwrap().1.len(), 1);
    assert_eq!(b.pdfs_of(&key).await.unwrap().1.len(), 1, "the PDF file arrived");
    assert_eq!(b.search("pacing", 5).await.unwrap().len(), 1, "searchable on B");

    // Again: nothing new either way.
    let (again, _) = sync(&b, &a).await;
    assert_eq!((again.received.applied, again.files_received), (0, 0), "{again:?}");

    // A change on B reaches A.
    b.tag(&key, &["to-read".to_string()]).await.unwrap();
    b.trash(&key).await.unwrap();
    sync(&a, &b).await;
    let (on_a, _) = a.show(&key).await.unwrap();
    assert!(on_a.meta.is_deleted(), "the trash travels");
    assert_eq!(a.tags_of(vec![on_b.id]).await.unwrap()[&on_b.id].len(), 1);
}

/// One session, keeping the errors.
async fn try_sync(
    a: &Engine,
    b: &Engine,
) -> (quire_engine::Result<quire_engine::SyncReport>, quire_engine::Result<quire_engine::SyncReport>) {
    let (left, right) = tokio::io::duplex(64 * 1024);
    let (mut lr, mut lw) = tokio::io::split(left);
    let (mut rr, mut rw) = tokio::io::split(right);
    tokio::join!(a.sync_session(SyncSide::Client, &mut lr, &mut lw), b.sync_session(SyncSide::Server, &mut rr, &mut rw),)
}

#[tokio::test]
async fn one_persons_two_libraries_do_not_mix() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (engine(dir_a.path(), None).await, engine(dir_b.path(), None).await);
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    a.create_project("Thesis", None).await.unwrap();
    b.create_project("Lab", None).await.unwrap();
    let (client, server) = try_sync(&a, &b).await;
    assert!(client.is_err() && server.is_err(), "different libraries are refused");
}

#[tokio::test]
async fn collaborators_see_only_what_is_shared() {
    let (dir_bob, dir_alice) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (bob, alice) = (engine(dir_bob.path(), None).await, engine(dir_alice.path(), None).await);
    let bib = dir_bob.path().join("refs.bib");
    std::fs::write(&bib, BIB).unwrap();
    let key = bob.import_bib(&bib, None).await.unwrap()[0].reference().cite_key.to_string();
    bob.create_project("Lab", None).await.unwrap();
    bob.create_project("Private", None).await.unwrap();
    let lab: quire_engine::SlugPath = "lab".parse().unwrap();
    bob.link(&key, &lab).await.unwrap();
    bob.add_note(Some(&key), "bob's thoughts").await.unwrap();
    let pdf = dir_bob.path().join("rivera.pdf");
    std::fs::write(&pdf, tiny_pdf("Diaphragm pacing restores breathing after injury")).unwrap();
    bob.attach_file(&key, &pdf, false).await.unwrap();
    alice.create_project("Mine", None).await.unwrap();

    // Not shared yet: a session works and moves nothing.
    let (client, _) = try_sync(&alice, &bob).await;
    let client = client.unwrap();
    assert!(client.collaborator.is_some());
    assert_eq!((client.received.applied, client.files_received), (0, 0), "{client:?}");

    let (alice_id, _) = alice.user_id().unwrap();
    bob.share_project(&lab, &alice_id, "Alice", quire_engine::MemberRole::Editor).await.unwrap();
    let (client, _) = sync(&alice, &bob).await;
    assert_eq!(client.files_received, 1, "the shared paper's PDF: {client:?}");
    let (on_alice, projects) = alice.show(&key).await.unwrap();
    assert_eq!(projects.len(), 1, "in Lab");
    assert!(alice.notes(Some(&key)).await.unwrap().is_empty(), "Bob's note stays his");
    assert_eq!(alice.projects().await.unwrap().len(), 2, "Mine and Lab, not Bob's Private");
    let (_, members) = alice.project_members(&lab).await.unwrap();
    assert_eq!(members.len(), 1);

    // Alice's tags and notes stay hers.
    alice.tag(&key, &["alice-only".to_string()]).await.unwrap();
    alice.add_note(Some(&key), "alice's thoughts").await.unwrap();
    sync(&alice, &bob).await;
    assert!(bob.notes(Some(&key)).await.unwrap().iter().all(|n| n.body != "alice's thoughts"));
    assert!(bob.tags().await.unwrap().is_empty(), "Alice's tag stays hers");
    assert_eq!(on_alice.id, bob.show(&key).await.unwrap().0.id);
}

/// The key sealing data for a hub: the same on one person's devices of one library (derived, never
/// carried), different for another person; the recovery kit holds what recovers it.
#[tokio::test]
async fn one_persons_devices_share_the_library_key() {
    let (dir_a, dir_b, dir_c) =
        (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b, c) =
        (engine(dir_a.path(), None).await, engine(dir_b.path(), None).await, engine(dir_c.path(), None).await);
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    let bib = dir_a.path().join("refs.bib");
    std::fs::write(&bib, BIB).unwrap();
    a.import_bib(&bib, None).await.unwrap();
    assert_ne!(
        a.library_key().await.unwrap().file_id("h"),
        b.library_key().await.unwrap().file_id("h"),
        "not joined yet"
    );
    sync(&b, &a).await;
    assert_eq!(a.library_key().await.unwrap().file_id("h"), b.library_key().await.unwrap().file_id("h"));
    assert_ne!(
        a.library_key().await.unwrap().file_id("h"),
        c.library_key().await.unwrap().file_id("h"),
        "another person"
    );
    assert!(a.recovery_kit().await.unwrap().contains(&a.export_user_key().unwrap()));
}

/// With PDFs loading as they are opened, a direct sync with another device brings no file; asking
/// for one brings that one only.
#[tokio::test]
async fn an_on_open_device_gets_one_file_from_another_device_when_asked() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let a = engine(dir_a.path(), None).await;
    std::fs::write(dir_b.path().join("config.toml"), "[files]\ndownload = \"on-open\"\n").unwrap();
    let b = engine(dir_b.path(), None).await;
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    let bib = dir_a.path().join("refs.bib");
    std::fs::write(&bib, "@article{one, title = {One}, year = {2020}}\n@article{two, title = {Two}, year = {2021}}")
        .unwrap();
    a.import_bib(&bib, None).await.unwrap();
    for key in ["one", "two"] {
        let (reference, _) = a.show(key).await.unwrap();
        a.attach_bytes(&reference, &tiny_pdf(key), "test", false).await.unwrap();
    }
    let (client, _) = sync(&b, &a).await;
    assert_eq!(client.files_received, 0, "{client:?}");

    let hash = b.pdfs_of("one").await.unwrap().1[0].0.hash.clone();
    let (left, right) = tokio::io::duplex(64 * 1024);
    let (mut left_read, mut left_write) = tokio::io::split(left);
    let (mut right_read, mut right_write) = tokio::io::split(right);
    let (got, served) = tokio::join!(
        b.fetch_session(&hash, SyncSide::Client, &mut left_read, &mut left_write),
        a.sync_session(SyncSide::Server, &mut right_read, &mut right_write),
    );
    assert!(got.unwrap());
    assert_eq!(served.unwrap().files_sent, 1, "only the file asked for");
    assert!(!b.pdfs_of("two").await.unwrap().1[0].1.is_file());
}
