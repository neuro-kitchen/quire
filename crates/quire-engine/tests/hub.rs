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

    /// `by` (a device already on the hub) introduces `new` (linking does this before the keys).
    async fn introduce(&self, by: &Engine, new: &Engine) {
        let (device, _) = new.sync_id().unwrap();
        let (left, right) = tokio::io::duplex(256 * 1024);
        let (mut left_read, mut left_write) = tokio::io::split(left);
        let (mut right_read, mut right_write) = tokio::io::split(right);
        let (client, server) = tokio::join!(
            by.hub_introduce_session(&self.id, &device, &mut left_read, &mut left_write),
            quire_hub::session::serve(&self.store, &self.settings, None, &mut right_read, &mut right_write),
        );
        server.unwrap();
        client.unwrap();
    }

    async fn unlink(&self, by: &Engine, gone: &Engine) -> quire_sync::hub::Rotation {
        let (device, _) = gone.sync_id().unwrap();
        let (left, right) = tokio::io::duplex(256 * 1024);
        let (mut left_read, mut left_write) = tokio::io::split(left);
        let (mut right_read, mut right_write) = tokio::io::split(right);
        let (client, server) = tokio::join!(
            by.hub_unlink_session(&self.id, &device, &mut left_read, &mut left_write),
            quire_hub::session::serve(&self.store, &self.settings, None, &mut right_read, &mut right_write),
        );
        server.unwrap();
        client.unwrap()
    }

    async fn devices(&self, device: &Engine) -> Vec<quire_sync::hub::HubDevice> {
        let (left, right) = tokio::io::duplex(256 * 1024);
        let (mut left_read, mut left_write) = tokio::io::split(left);
        let (mut right_read, mut right_write) = tokio::io::split(right);
        let (client, server) = tokio::join!(
            device.hub_devices_session(&self.id, &mut left_read, &mut left_write),
            quire_hub::session::serve(&self.store, &self.settings, None, &mut right_read, &mut right_write),
        );
        server.unwrap();
        client.unwrap()
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

    // The second device needs no code once one of the person's devices introduced it (linking), and joins.
    hub.introduce(&a, &b).await;
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
    hub.introduce(&a, &b).await;
    hub.sync(&b, options(None)).await.unwrap();
    b.tag(&key, &["later".to_string()]).await.unwrap();
    hub.sync(&b, options(None)).await.unwrap();
    // A pulls B's change, then (one batch since a snapshot is "many" here) uploads a snapshot.
    let snapshotting = hub.sync(&a, HubOptions { code: None, snapshot_after: 1 }).await.unwrap();
    assert!(snapshotting.snapshot, "{snapshotting:?}");
    let disk_before = hub.bytes_on_disk().len();
    assert!(disk_before > 0);

    // C starts from the snapshot: everything, including B's tag that came in a dropped batch.
    hub.introduce(&a, &c).await;
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
    hub.introduce(&a, &b).await;
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
    hub.introduce(&a, &b).await;
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

    hub.introduce(&a, &b).await;
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

/// A file once confirmed on the hub stayed "on the hub" for good: after the hub lost it (a reset,
/// a restore from an older backup) and had no room to take it again, eviction removed what had
/// become the last copy. A session where the hub lacks a file here, and does not take it, clears
/// the mark.
#[tokio::test]
async fn a_file_the_hub_no_longer_holds_is_not_evicted() {
    let dir = tempfile::tempdir().unwrap();
    // Room for both while syncing (no eviction yet); reopened below with room for one.
    let b = lazy_engine(dir.path(), 4).await;
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, "@article{one, title = {One}, year = {2020}}\n@article{two, title = {Two}, year = {2021}}")
        .unwrap();
    b.import_bib(&bib, None).await.unwrap();
    for key in ["one", "two"] {
        let (reference, _) = b.show(key).await.unwrap();
        b.attach_bytes(&reference, &big_pdf(key), "test", false).await.unwrap();
    }
    // Confirmed on a hub, then that hub's files are gone and the one met now has no room for them.
    let first = Hub::new(0);
    let uploaded = first.sync(&b, options(Some(&first.setup_code()))).await.unwrap();
    assert_eq!(uploaded.files_uploaded, 2, "{uploaded:?}");
    let full = Hub::new(256 * 1024);
    let refused = full.sync(&b, options(Some(&full.setup_code()))).await.unwrap();
    assert!(refused.over_quota == 2 && refused.files_uploaded == 0, "{refused:?}");
    drop(b);
    let b = lazy_engine(dir.path(), 1).await;
    let eviction = b.evict_pdfs().await.unwrap();
    assert_eq!(eviction.removed, 0, "no hub holds them now: {eviction:?}");
}

/// Each device's name reaches the person's other devices through the hub, sealed: the hub's disk
/// never holds it, and the device list shows it with what each device last synced.
#[tokio::test]
async fn device_names_travel_sealed_and_the_list_shows_activity() {
    let hub = Hub::new(0);
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::write(dir_a.path().join("config.toml"), "[device]\nname = \"Ana's laptop\"\n").unwrap();
    std::fs::write(dir_b.path().join("config.toml"), "[device]\nname = \"Ana's phone\"\n").unwrap();
    let (a, b) = (engine(dir_a.path(), None).await, engine(dir_b.path(), None).await);
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    filled(&a, dir_a.path()).await;
    hub.sync(&a, options(Some(&hub.setup_code()))).await.unwrap();
    hub.introduce(&a, &b).await;
    hub.sync(&b, options(None)).await.unwrap();

    let devices = hub.devices(&b).await;
    let names: Vec<_> = devices.iter().map(|d| d.name.clone()).collect();
    assert_eq!(names, [Some("Ana's laptop".to_string()), Some("Ana's phone".to_string())]);
    let laptop = &devices[0];
    assert!(laptop.last_session > 0 && laptop.batches_in >= 1 && laptop.files_in == 1, "{laptop:?}");
    for plain in ["Ana's laptop", "Ana's phone"] {
        assert!(!contains(&hub.bytes_on_disk(), plain), "the hub's disk holds `{plain}`");
    }
}

/// Linking a device needs this device to sync through a hub first (the new one gets the library
/// from it, and the link uses its relay): refused before any network is touched.
#[tokio::test]
async fn linking_waits_for_a_hub() {
    let dir = tempfile::tempdir().unwrap();
    let a = engine(dir.path(), None).await;
    let refused = a.link_offer().await.err().expect("refused without a hub");
    assert!(refused.to_string().contains("no sync hub yet"), "{refused}");
}

/// The whole story of an unlinked device: a device with the user key alone is not let in until
/// introduced; unlinking rotates the library key, the remaining device takes the new key by
/// itself, the unlinked one (and any new ID it makes with the user key) is shut out, a device
/// linked after reads everything, and the recovery kit carries the new key.
#[tokio::test]
async fn unlinking_rotates_the_key_and_shuts_the_device_out() {
    let hub = Hub::new(0);
    let dirs: Vec<_> = (0..5).map(|_| tempfile::tempdir().unwrap()).collect();
    let mut devices = Vec::new();
    for dir in &dirs {
        devices.push(engine(dir.path(), None).await);
    }
    let [laptop, phone, tablet, thief, later] = &devices[..] else { unreachable!() };
    for other in [phone, tablet, thief, later] {
        other.import_user_key(&laptop.export_user_key().unwrap()).unwrap();
    }
    let (key, _) = filled(laptop, dirs[0].path()).await;
    hub.sync(laptop, options(Some(&hub.setup_code()))).await.unwrap();

    // The user key alone does not let a new device in.
    let refused = hub.sync(phone, options(None)).await.unwrap_err();
    assert!(refused.to_string().contains("link it from one of your devices"), "{refused}");
    hub.introduce(laptop, phone).await;
    hub.introduce(laptop, tablet).await;
    hub.sync(phone, options(None)).await.unwrap();
    hub.sync(tablet, options(None)).await.unwrap();

    // Unlink the tablet: the key rotates, sealed for the laptop and the phone.
    let rotation = hub.unlink(laptop, tablet).await;
    assert_eq!((rotation.epoch, rotation.sealed_for, rotation.left_out.len()), (1, 2, 0), "{rotation:?}");
    laptop.add_note(Some(&key), "after the unlink").await.unwrap();
    hub.sync(laptop, options(None)).await.unwrap();

    // The phone takes the new key by itself and reads the new note.
    hub.sync(phone, options(None)).await.unwrap();
    let notes: Vec<_> = phone.notes(Some(&key)).await.unwrap().into_iter().map(|n| n.body).collect();
    assert!(notes.iter().any(|n| n == "after the unlink"), "{notes:?}");
    assert_eq!(phone.library_keys().await.unwrap().current().0, 1);
    let told: Vec<_> = phone.activity(20).await.unwrap().into_iter().map(|a| (a.kind, a.detail)).collect();
    assert!(
        told.contains(&("key_changed".to_string(), "1".to_string())),
        "the phone is told the key changed: {told:?}"
    );
    let told: Vec<_> = laptop.activity(20).await.unwrap().into_iter().map(|a| (a.kind, a.detail)).collect();
    assert!(told.contains(&("unlinked".to_string(), "1".to_string())), "{told:?}");

    // The tablet is refused, and has no key of epoch 1; a new ID made with the user key is refused too.
    let refused = hub.sync(tablet, options(None)).await.unwrap_err();
    assert!(refused.to_string().contains("removed from your account"), "{refused}");
    assert_eq!(tablet.library_keys().await.unwrap().current().0, 0);
    let refused = hub.sync(thief, options(None)).await.unwrap_err();
    assert!(refused.to_string().contains("link it from one of your devices"), "{refused}");

    // A device set up after the unlink with the recovery kit's library keys (what linking gives it),
    // introduced: it reads everything, old and new.
    let kit = laptop.recovery_kit().await.unwrap();
    let keys_line = kit.lines().find_map(|l| l.strip_prefix("Library keys: ")).expect("the kit names the new key");
    later.import_library_keys(keys_line).unwrap();
    hub.introduce(laptop, later).await;
    hub.sync(later, options(None)).await.unwrap();
    let notes: Vec<_> = later.notes(Some(&key)).await.unwrap().into_iter().map(|n| n.body).collect();
    assert!(notes.iter().any(|n| n == "after the unlink") && notes.iter().any(|n| n == NOTE), "{notes:?}");

    // The recovery kit holds the new key; the hub's disk holds no plain text.
    assert!(kit.contains(&laptop.library_keys().await.unwrap().chain_text()));
    for plain in ["after the unlink", NOTE, "Diaphragm pacing"] {
        assert!(!contains(&hub.bytes_on_disk(), plain), "the hub's disk holds `{plain}`");
    }
}

/// Each device's notifications: what came from which of the person's devices (by name), a device
/// new to it, and an unlink. Nothing about the device's own changes.
#[tokio::test]
async fn notifications_say_what_came_from_whom() {
    let hub = Hub::new(0);
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::write(dir_a.path().join("config.toml"), "[device]\nname = \"Ana's laptop\"\n").unwrap();
    std::fs::write(dir_b.path().join("config.toml"), "[device]\nname = \"Ana's phone\"\n").unwrap();
    let (a, b) = (engine(dir_a.path(), None).await, engine(dir_b.path(), None).await);
    b.import_user_key(&a.export_user_key().unwrap()).unwrap();
    let (key, _) = filled(&a, dir_a.path()).await;
    hub.sync(&a, options(Some(&hub.setup_code()))).await.unwrap();
    assert!(a.activity(10).await.unwrap().is_empty(), "the first device list only teaches: {:?}", a.activity(10).await);

    hub.introduce(&a, &b).await;
    hub.sync(&b, options(None)).await.unwrap();
    let from_laptop = b.activity(10).await.unwrap().into_iter().find(|n| n.kind == "sync").expect("what came");
    assert_eq!(from_laptop.source.as_deref(), Some("Ana's laptop"));
    for part in ["reference=1", "note=1", "highlight=1", "attachment=1"] {
        assert!(from_laptop.detail.split(',').any(|p| p == part), "{part} in {}", from_laptop.detail);
    }

    // The laptop sees the phone for the first time, then what it changed.
    b.add_note(Some(&key), "from the phone").await.unwrap();
    hub.sync(&b, options(None)).await.unwrap();
    hub.sync(&a, options(None)).await.unwrap();
    let told = a.activity(10).await.unwrap();
    assert!(told.iter().any(|n| n.kind == "new_device" && n.source.as_deref() == Some("Ana's phone")), "{told:?}");
    let note = told.iter().find(|n| n.kind == "sync").expect("the phone's note");
    assert_eq!((note.source.as_deref(), note.detail.as_str()), (Some("Ana's phone"), "note=1"));
    assert_eq!(a.unread_activity().await.unwrap(), 2);
    a.mark_activity_read().await.unwrap();
    assert_eq!(a.unread_activity().await.unwrap(), 0);

    // Unlinking is noted, by name.
    hub.unlink(&a, &b).await;
    let unlinked = a.activity(1).await.unwrap().remove(0);
    assert_eq!((unlinked.kind.as_str(), unlinked.source.as_deref()), ("unlinked", Some("Ana's phone")));
}
