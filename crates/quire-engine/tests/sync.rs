//! Sync between two devices on real libraries, over an in-memory pipe (the QUIC transport only
//! carries the same session): a new copy joins, gets references, a note, a highlight and the PDF
//! file; nothing is sent twice; a later change on either side reaches the other.

use quire_engine::{Engine, LibraryChoice, OpenOptions, SyncRole};

const BIB: &str = "@article{rivera2025,
  title = {Diaphragm pacing restores breathing after injury},
  author = {Rivera, Ana},
  journal = {Journal of Tests},
  year = {2025},
}";

/// A one-page PDF showing `text`.
fn tiny_pdf(text: &str) -> Vec<u8> {
    let stream = format!("BT /F1 12 Tf 72 720 Td ({text}) Tj ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
        format!("<< /Length {} >>\nstream\n{stream}\nendstream", stream.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{body}\nendobj\n", i + 1));
    }
    let xref = pdf.len();
    pdf.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1));
    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1));
    pdf.into_bytes()
}

async fn engine(dir: &std::path::Path) -> Engine {
    let options = OpenOptions {
        library: LibraryChoice::Path(dir.join("library")),
        config_file: Some(dir.join("config.toml")),
        endpoints: None,
    };
    Engine::open(options).await.unwrap()
}

/// One session between `a` (connecting) and `b`.
async fn sync(a: &Engine, b: &Engine) -> (quire_engine::SyncReport, quire_engine::SyncReport) {
    let (left, right) = tokio::io::duplex(64 * 1024);
    let (mut left_read, mut left_write) = tokio::io::split(left);
    let (mut right_read, mut right_write) = tokio::io::split(right);
    let (client, server) = tokio::join!(
        a.sync_session(SyncRole::Client, &mut left_read, &mut left_write),
        b.sync_session(SyncRole::Server, &mut right_read, &mut right_write),
    );
    (client.unwrap(), server.unwrap())
}

#[tokio::test]
async fn a_new_device_joins_and_changes_flow_both_ways() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (engine(dir_a.path()).await, engine(dir_b.path()).await);
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

#[tokio::test]
async fn two_libraries_with_data_do_not_mix() {
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (a, b) = (engine(dir_a.path()).await, engine(dir_b.path()).await);
    a.create_project("Thesis", None).await.unwrap();
    b.create_project("Lab", None).await.unwrap();
    let (left, right) = tokio::io::duplex(64 * 1024);
    let (mut lr, mut lw) = tokio::io::split(left);
    let (mut rr, mut rw) = tokio::io::split(right);
    let (client, server) = tokio::join!(
        a.sync_session(SyncRole::Client, &mut lr, &mut lw),
        b.sync_session(SyncRole::Server, &mut rr, &mut rw),
    );
    assert!(client.is_err() && server.is_err(), "different libraries are refused");
}
