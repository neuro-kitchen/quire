//! Exports, libraries, backup, manuscripts and sharing through the bridge (no network).

mod common;
use common::fixture;

use quire_ffi::{
    ExportOptionsRecord, ExportSelection, export_encodings, export_formats, list_libraries, register_library, rename_library, set_default_library,
    unregister_library,
};

fn whole(format: &str) -> ExportSelection {
    ExportSelection {
        project: None,
        recursive: true,
        references: vec![],
        profile: None,
        options: None,
        format: Some(format.into()),
    }
}

/// The app's export dialog: chosen references, options given directly, a file in its encoding.
#[tokio::test]
async fn a_selection_with_options_to_a_file() {
    let f = fixture().await;
    assert_eq!(export_encodings(), ["utf-8", "utf-8-bom", "windows-1252"]);
    let options = ExportOptionsRecord {
        format: Some("bibtex".into()),
        omit: vec!["abstract".into()],
        ascii: false,
        month_macros: false,
        preprints_as_articles: false,
        encoding: Some("utf-8-bom".into()),
    };
    let selection = ExportSelection { references: vec!["okeefe1971".into()], options: Some(options), ..whole("bibtex") };
    let preview = f.library.export(selection.clone()).await.unwrap();
    assert_eq!(preview.count, 1);
    assert!(preview.text.starts_with("@article{okeefe1971") && !preview.text.contains("abstract"), "{}", preview.text);

    let file = f.dir.path().join("one.bib");
    let written = f.library.export_to(selection, file.display().to_string()).await.unwrap();
    assert_eq!(written.count, 1);
    assert!(std::fs::read(&file).unwrap().starts_with(b"\xEF\xBB\xBF@article{okeefe1971"));
    assert!(f.library.export_profiles().await.unwrap().is_empty(), "none in this config");
}

#[tokio::test]
async fn exports_once_and_kept_up_to_date() {
    let f = fixture().await;
    assert_eq!(export_formats(), ["biblatex", "bibtex", "csl-json", "hayagriva"]);
    let exported = f.library.export(whole("biblatex")).await.unwrap();
    assert_eq!(exported.count, 3);
    assert!(exported.text.contains("okeefe1971"));
    assert!(f.library.export(whole("docx")).await.unwrap_err().to_string().contains("biblatex"));

    let file = f.dir.path().join("refs.yml");
    let kept = f.library.keep_updated(whole("hayagriva"), file.display().to_string()).await.unwrap();
    assert_eq!(kept.count, 3);
    assert!(std::fs::read_to_string(&file).unwrap().contains("okeefe1971"));
    let autos = f.library.auto_exports().await.unwrap();
    assert_eq!((autos.len(), autos[0].format.as_deref()), (1, Some("hayagriva")));
    assert_eq!(f.library.run_auto_exports(true).await.unwrap()[0].outcome, "unchanged", "same content");
    f.library.trash("buzsaki2006".into()).await.unwrap();
    let runs = f.library.run_auto_exports(false).await.unwrap();
    assert_eq!((runs[0].outcome.as_str(), runs[0].count), ("written", 2));
    assert!(f.library.stop_auto_export(file.display().to_string()).await.unwrap());
}

#[tokio::test]
async fn highlights_as_markdown_and_a_manuscript_check() {
    let f = fixture().await;
    f.library
        .add_highlight("okeefe1971".into(), "a spatial map".into(), "The hippocampus as ".into(), None)
        .await
        .unwrap();
    let notes = f.library.annotations_markdown(None, true).await.unwrap();
    assert_eq!(notes.len(), 1);
    assert!(notes[0].markdown.contains("a spatial map"), "{}", notes[0].markdown);

    let draft = f.dir.path().join("draft.md");
    std::fs::write(&draft, "Place cells [@okeefe1971] and rhythms [@buzsaki2006; @okeef1971].\n").unwrap();
    let check = f.library.check_manuscript(vec![draft.display().to_string()], None, true).await.unwrap();
    assert_eq!(check.cited, 3, "every cite key used, the unknown one too");
    assert_eq!(check.unknown.len(), 1);
    assert_eq!((check.unknown[0].key.as_str(), check.unknown[0].uses[0].line), ("okeef1971", 1));
    assert!(check.unknown[0].suggestions.contains(&"okeefe1971".to_string()), "{:?}", check.unknown[0].suggestions);
}

#[tokio::test]
async fn libraries_backup_and_copy() {
    let f = fixture().await;
    let config = Some(f.library.info().config_file);
    let other = f.dir.path().join("other");
    let entry = register_library(config.clone(), "other".into(), Some(other.display().to_string())).unwrap();
    assert!(entry.registered && !entry.is_default);
    rename_library(config.clone(), "other".into(), "second".into()).unwrap();
    assert!(set_default_library(config.clone(), "second".into()).unwrap().is_default);
    let names: Vec<Option<String>> = list_libraries(config.clone()).unwrap().into_iter().map(|l| l.name).collect();
    assert!(names.contains(&Some("second".into())), "{names:?}");

    let copied = f.library.copy_to_library("okeefe1971".into(), "second".into(), None).await.unwrap();
    assert!(copied.new);
    assert_eq!(copied.reference.cite_key, "okeefe1971");

    let backup = f.dir.path().join("backup.tar.gz");
    let report = f.library.backup(backup.display().to_string()).await.unwrap();
    assert!(report.files >= 1 && report.bytes > 0 && backup.exists());
    unregister_library(config, "second".into()).unwrap();
}

#[tokio::test]
async fn identity_and_sharing() {
    let f = fixture().await;
    assert!(!f.library.device_id().await.unwrap().is_empty());
    let me = f.library.user_id().await.unwrap();
    assert_eq!(me.len(), 64, "{me}");
    let key = f.library.export_user_key().await.unwrap();
    assert_eq!(f.library.import_user_key(key).await.unwrap(), me);

    f.library.create_project("Lab".into(), None).await.unwrap();
    let colleague = "b".repeat(64);
    let member = f.library.share_project("lab".into(), colleague.clone(), "Ana".into(), "editor".into()).await.unwrap();
    assert_eq!((member.name.as_str(), member.role.as_str()), ("Ana", "editor"));
    assert!(f.library.project_members("lab".into()).await.unwrap().iter().any(|m| m.user == colleague));
    f.library.unshare_project("lab".into(), colleague.clone()).await.unwrap();
    assert!(f.library.project_members("lab".into()).await.unwrap().iter().all(|m| m.user != colleague));
    assert!(f.library.share_project("lab".into(), "abc".into(), "X".into(), "editor".into()).await.is_err());
}
