//! Sync through the hub on real libraries (`.knowledge/sync-hub.md`): the hub's own store and
//! session over in-memory pipes (the QUIC transport only carries the same session). Devices
//! converge through it; PDFs come only when asked for; the hub's disk holds no plain text.

mod common;
use std::path::Path;
use std::sync::Arc;

use common::engine;
use quire_engine::{Engine, EngineError, HubOptions, HubReport};
use quire_hub::{Settings, Store, hub_id, hub_key};
use quire_testkit::tiny_pdf;

const BIB: &str = "@article{rivera2025,
  title = {Diaphragm pacing restores breathing after injury},
  author = {Rivera, Ana},
  journal = {Journal of Tests},
  year = {2025},
}";
const NOTE: &str = "compare with the 2019 study";
const PDF_TEXT: &str = "Electrode placement near the phrenic nerve";

/// A hub in a temporary folder, reached over pipes.
struct Hub {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    settings: Settings,
    id: String,
}

impl Hub {
    fn new(quota: u64) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path()).unwrap());
        let id = hub_id(&hub_key(dir.path()).unwrap());
        Self { dir, store, settings: Settings { quota }, id }
    }

    fn setup_code(&self) -> String {
        self.store.owner_code().unwrap().expect("nobody registered yet")
    }

    async fn sync(&self, device: &Engine, options: HubOptions) -> Result<HubReport, EngineError> {
        let (left, right) = tokio::io::duplex(256 * 1024);
        let (mut left_read, mut left_write) = tokio::io::split(left);
        let (mut right_read, mut right_write) = tokio::io::split(right);
        let (client, server) = tokio::join!(
            device.hub_session(&self.id, &options, &mut left_read, &mut left_write),
            quire_hub::session::serve(&self.store, &self.settings, None, &mut right_read, &mut right_write),
        );
        server.unwrap();
        client
    }

    async fn fetch(&self, device: &Engine, hash: &str) -> bool {
        let (left, right) = tokio::io::duplex(256 * 1024);
        let (mut left_read, mut left_write) = tokio::io::split(left);
        let (mut right_read, mut right_write) = tokio::io::split(right);
        let (client, server) = tokio::join!(
            device.hub_fetch_session(&self.id, hash, &mut left_read, &mut left_write),
            quire_hub::session::serve(&self.store, &self.settings, None, &mut right_read, &mut right_write),
        );
        server.unwrap();
        client.unwrap()
    }

    /// Every byte the hub keeps on disk (database, its journal, sealed files).
    fn bytes_on_disk(&self) -> Vec<u8> {
        fn walk(dir: &Path, out: &mut Vec<u8>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    out.extend(std::fs::read(&path).unwrap());
                }
            }
        }
        let mut out = Vec::new();
        walk(self.dir.path(), &mut out);
        out
    }
}

/// An engine whose PDFs load as they are opened (`files.download = "on-open"`), with a cache limit.
async fn lazy_engine(dir: &Path, cache_mb: u32) -> Engine {
    std::fs::write(dir.join("config.toml"), format!("[files]\ndownload = \"on-open\"\ncache_mb = {cache_mb}\n"))
        .unwrap();
    engine(dir, None).await
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle.as_bytes())
}

fn options(code: Option<&str>) -> HubOptions {
    HubOptions { code: code.map(str::to_string), ..HubOptions::default() }
}

/// Device A with a reference, a note, a highlight and a PDF; returns its cite key and PDF hash.
async fn filled(a: &Engine, dir: &Path) -> (String, String) {
    let bib = dir.join("refs.bib");
    std::fs::write(&bib, BIB).unwrap();
    let key = a.import_bib(&bib, None).await.unwrap()[0].reference().cite_key.to_string();
    a.add_note(Some(&key), NOTE).await.unwrap();
    a.add_highlight(&key, "restores breathing", "", Some("main claim")).await.unwrap();
    let (reference, _) = a.show(&key).await.unwrap();
    a.attach_bytes(&reference, &tiny_pdf(PDF_TEXT), "test", false).await.unwrap();
    let hash = a.pdfs_of(&key).await.unwrap().1[0].0.hash.clone();
    (key, hash)
}

#[tokio::test]
async fn devices_converge_through_a_hub_that_reads_nothing() {
    let hub = Hub::new(0);
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (engine(dir_a.path(), None).await, lazy_engine(dir_b.path(), 0).await);
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    let (key, hash) = filled(&a, dir_a.path()).await;

    let first = hub.sync(&a, options(Some(&hub.setup_code()))).await.unwrap();
    assert!(first.registered && first.pushed >= 4, "{first:?}");
    assert_eq!(first.files_uploaded, 1);

    // The second device needs no code (its certificate is the same person's) and joins.
    let joined = hub.sync(&b, options(None)).await.unwrap();
    assert!(joined.joined && !joined.registered, "{joined:?}");
    assert!(joined.pulled.applied >= 4, "{joined:?}");
    assert_eq!(b.notes(Some(&key)).await.unwrap().len(), 1);
    assert_eq!(b.highlights(&key).await.unwrap().1.len(), 1);
    let (_, pdfs) = b.pdfs_of(&key).await.unwrap();
    assert_eq!(pdfs.len(), 1, "the attachment is known");
    assert!(!pdfs[0].1.is_file(), "the PDF itself is not downloaded until asked for");
    assert!(hub.fetch(&b, &hash).await);
    assert!(b.pdfs_of(&key).await.unwrap().1[0].1.is_file(), "now it is, opened and checked");

    // Nothing new: nothing moves.
    let again = hub.sync(&a, options(None)).await.unwrap();
    assert_eq!((again.pushed, again.pulled.applied, again.files_uploaded), (0, 0, 0), "{again:?}");

    // A change on B reaches A.
    b.tag(&key, &["to-read".to_string()]).await.unwrap();
    hub.sync(&b, options(None)).await.unwrap();
    hub.sync(&a, options(None)).await.unwrap();
    let (reference, _) = a.show(&key).await.unwrap();
    assert_eq!(a.tags_of(vec![reference.id]).await.unwrap()[&reference.id].len(), 1);
    // A merged B's tag from the hub: its next sync must not send it back.
    let echo = hub.sync(&a, options(None)).await.unwrap();
    assert_eq!(echo.pushed, 0, "rows pulled from the hub are not pushed back: {echo:?}");

    // The hub's disk: no title, note, cite key, PDF text or PDF header anywhere.
    let disk = hub.bytes_on_disk();
    assert!(disk.len() > 1000, "the hub did store something");
    for plain in ["Diaphragm pacing", NOTE, "rivera2025", "Rivera", PDF_TEXT, "%PDF", "to-read", "main claim"] {
        assert!(!contains(&disk, plain), "the hub's disk holds `{plain}`");
    }
}

#[tokio::test]
async fn people_need_a_code_and_stay_apart() {
    let hub = Hub::new(0);
    let (dir_a, dir_m) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (alice, mallory) = (engine(dir_a.path(), None).await, engine(dir_m.path(), None).await);
    filled(&alice, dir_a.path()).await;
    let code = hub.setup_code();

    let unknown = hub.sync(&alice, options(None)).await.unwrap_err();
    assert!(unknown.to_string().contains("does not know you"), "{unknown}");
    let wrong = hub.sync(&alice, options(Some("AAAA-BBBB-CCCC"))).await.unwrap_err();
    assert!(wrong.to_string().contains("wrong, used or expired"), "{wrong}");
    hub.sync(&alice, options(Some(&code))).await.unwrap();
    let reused = hub.sync(&mallory, options(Some(&code))).await.unwrap_err();
    assert!(reused.to_string().contains("wrong, used or expired"), "the setup code works once: {reused}");

    // Mallory, invited, has her own account: she gets nothing of Alice's.
    let invite = hub.store.new_code(quire_hub::Purpose::Invite, None, None).unwrap();
    let report = hub.sync(&mallory, options(Some(&invite))).await.unwrap();
    assert!(report.registered && !report.joined, "{report:?}");
    assert_eq!(report.pulled.applied, 0);
    assert!(mallory.search("pacing", 5).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_snapshot_lets_a_new_device_start_from_it() {
    let hub = Hub::new(0);
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let a = engine(dirs[0].path(), None).await;
    let b = engine(dirs[1].path(), None).await;
    let c = engine(dirs[2].path(), None).await;
    for other in [&b, &c] {
        other.import_user_key(&a.export_user_key().unwrap()).unwrap();
    }
    let (key, _) = filled(&a, dirs[0].path()).await;
    hub.sync(&a, options(Some(&hub.setup_code()))).await.unwrap();
    hub.sync(&b, options(None)).await.unwrap();
    b.tag(&key, &["later".to_string()]).await.unwrap();
    hub.sync(&b, options(None)).await.unwrap();
    // A pulls B's change, then (one batch since a snapshot is "many" here) uploads a snapshot.
    let snapshotting = hub.sync(&a, HubOptions { code: None, snapshot_after: 1 }).await.unwrap();
    assert!(snapshotting.snapshot, "{snapshotting:?}");
    let disk_before = hub.bytes_on_disk().len();
    assert!(disk_before > 0);

    // C starts from the snapshot: everything, including B's tag that came in a dropped batch.
    let joined = hub.sync(&c, options(None)).await.unwrap();
    assert!(joined.joined, "{joined:?}");
    let (reference, _) = c.show(&key).await.unwrap();
    assert_eq!(c.notes(Some(&key)).await.unwrap().len(), 1);
    assert_eq!(c.tags_of(vec![reference.id]).await.unwrap()[&reference.id].len(), 1, "B's tag, via the snapshot");
}

#[tokio::test]
async fn over_quota_files_wait_and_metadata_still_syncs() {
    let hub = Hub::new(4 * 1024);
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (engine(dir_a.path(), None).await, engine(dir_b.path(), None).await);
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    let (key, hash) = filled(&a, dir_a.path()).await;
    let (reference, _) = a.show(&key).await.unwrap();
    // A larger PDF than the quota leaves room for.
    let big: String = (0..400).map(|i| format!("Line {i} of a long paper about pacing. ")).collect();
    a.attach_bytes(&reference, &tiny_pdf(&big), "test", false).await.unwrap();
    let report = hub.sync(&a, options(Some(&hub.setup_code()))).await.unwrap();
    assert!(report.over_quota >= 1, "{report:?}");
    assert!(report.quota == 4 * 1024 && report.used <= report.quota, "{report:?}");
    hub.sync(&b, options(None)).await.unwrap();
    assert_eq!(b.notes(Some(&key)).await.unwrap().len(), 1, "metadata syncs anyway");
    let _ = hash;
}

#[tokio::test]
async fn a_revoked_device_is_refused() {
    let hub = Hub::new(0);
    let dir = tempfile::tempdir().unwrap();
    let phone = engine(dir.path(), None).await;
    filled(&phone, dir.path()).await;
    hub.sync(&phone, options(Some(&hub.setup_code()))).await.unwrap();
    let (user, _) = phone.user_id().unwrap();
    let account = hub.store.account_of(&user.to_string()).unwrap().unwrap();
    let (device, _) = phone.sync_id().unwrap();
    assert!(hub.store.revoke(account.id, &device).unwrap());
    let refused = hub.sync(&phone, options(None)).await.unwrap_err();
    assert!(refused.to_string().contains("removed from your account"), "{refused}");
}

/// A PDF big enough to count in megabytes.
fn big_pdf(label: &str) -> Vec<u8> {
    let text: String = (0..16000).map(|i| format!("{label} line {i} about diaphragm pacing and breathing. ")).collect();
    tiny_pdf(&text)
}

#[tokio::test]
async fn by_default_a_device_downloads_every_pdf_at_sync() {
    let hub = Hub::new(0);
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (engine(dir_a.path(), None).await, engine(dir_b.path(), None).await);
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    let (key, _) = filled(&a, dir_a.path()).await;
    hub.sync(&a, options(Some(&hub.setup_code()))).await.unwrap();
    let report = hub.sync(&b, options(None)).await.unwrap();
    assert_eq!(report.files_downloaded, 1, "{report:?}");
    assert!(b.pdfs_of(&key).await.unwrap().1[0].1.is_file());
    assert_eq!(b.open_pdf(&key).await.unwrap().source, quire_engine::PdfSource::Here);
}

#[tokio::test]
async fn kept_offline_papers_come_at_sync_and_eviction_keeps_what_matters() {
    let hub = Hub::new(0);
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let a = engine(dir_a.path(), None).await;
    let b = lazy_engine(dir_b.path(), 2).await;
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    // Three papers with ~0.9 MB PDFs (2.6 MB, over the 2 MB limit); one in the project "Thesis".
    let bib = dir_a.path().join("refs.bib");
    std::fs::write(&bib, "@article{one, title = {One}, year = {2020}}\n@article{two, title = {Two}, year = {2021}}\n@article{three, title = {Three}, year = {2022}}").unwrap();
    a.import_bib(&bib, None).await.unwrap();
    for key in ["one", "two", "three"] {
        let (reference, _) = a.show(key).await.unwrap();
        a.attach_bytes(&reference, &big_pdf(key), "test", false).await.unwrap();
    }
    a.create_project("Thesis", None).await.unwrap();
    a.link("three", &"thesis".parse().unwrap()).await.unwrap();
    hub.sync(&a, options(Some(&hub.setup_code()))).await.unwrap();

    let first = hub.sync(&b, options(None)).await.unwrap();
    assert_eq!(first.files_downloaded, 0, "nothing kept offline yet: {first:?}");
    b.keep_project_offline(&"thesis".parse().unwrap(), true).await.unwrap();
    let kept = hub.sync(&b, options(None)).await.unwrap();
    assert_eq!(kept.files_downloaded, 1, "the project's paper: {kept:?}");
    let on_disk = |key: &'static str| {
        let b = &b;
        async move { b.pdfs_of(key).await.unwrap().1[0].1.is_file() }
    };
    assert!(on_disk("three").await);

    // Two more downloaded (as when opened); past 2 MB the least recently opened one the hub holds
    // goes ("three" is kept offline, "one" was opened last): exactly one.
    for key in ["one", "two"] {
        let hash = b.pdfs_of(key).await.unwrap().1[0].0.hash.clone();
        assert!(hub.fetch(&b, &hash).await);
        b.open_pdf(key).await.unwrap();
    }
    b.open_pdf("one").await.unwrap(); // "one" is now the most recently opened
    let eviction = b.evict_pdfs().await.unwrap();
    assert!(eviction.removed == 1 && eviction.kept <= 2 * 1024 * 1024, "{eviction:?}");
    assert!(on_disk("three").await, "kept offline");
    assert!(!on_disk("two").await, "least recently opened");
    assert!(on_disk("one").await, "opened last");
}

#[tokio::test]
async fn without_a_hub_copy_nothing_is_evicted() {
    let dir = tempfile::tempdir().unwrap();
    let b = lazy_engine(dir.path(), 1).await;
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, "@article{one, title = {One}, year = {2020}}\n@article{two, title = {Two}, year = {2021}}")
        .unwrap();
    b.import_bib(&bib, None).await.unwrap();
    for key in ["one", "two"] {
        let (reference, _) = b.show(key).await.unwrap();
        b.attach_bytes(&reference, &big_pdf(key), "test", false).await.unwrap();
    }
    let eviction = b.evict_pdfs().await.unwrap();
    assert_eq!(eviction.removed, 0, "this device may hold the only copies: {eviction:?}");
    assert!(eviction.kept > 1024 * 1024);
}
