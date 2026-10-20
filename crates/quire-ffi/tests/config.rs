//! The settings screen's calls on a throwaway config file.

use quire_ffi::{config_check, config_entries, config_keys, config_read, config_set, config_unset, config_write};

fn file() -> (tempfile::TempDir, Option<String>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("quire").join("config.toml").display().to_string();
    (dir, Some(path))
}

#[test]
fn keys_describe_their_values() {
    let keys = config_keys();
    let width = keys.iter().find(|k| k.pattern == "reader.width").unwrap();
    assert_eq!((width.kind.as_str(), width.min, width.max), ("number", Some(40), Some(120)));
    let theme = keys.iter().find(|k| k.pattern == "reader.theme").unwrap();
    assert!(theme.choices.contains(&"sepia".to_string()));
}

#[test]
fn values_are_checked_set_and_removed_keeping_comments() {
    let (_dir, path) = file();
    config_write(path.clone(), "# my settings\n[user]\nname = \"Ada\"\n".into()).unwrap();
    config_set(path.clone(), "reader.width".into(), "80".into()).unwrap();
    assert!(config_set(path.clone(), "reader.width".into(), "500".into()).is_err(), "out of range");
    assert!(config_set(path.clone(), "user.email".into(), "not-an-email".into()).is_err());
    assert!(config_set(path.clone(), "no.such.key".into(), "x".into()).is_err());

    let entries = config_entries(path.clone()).unwrap();
    let get = |key: &str| entries.iter().find(|e| e.key == key).map(|e| e.value.clone());
    assert_eq!(get("user.name").as_deref(), Some("Ada"));
    assert_eq!(get("reader.width").as_deref(), Some("80"));
    assert!(config_read(path.clone()).unwrap().starts_with("# my settings"), "comments kept");

    assert!(config_unset(path.clone(), "reader.width".into()).unwrap());
    assert!(!config_unset(path.clone(), "reader.width".into()).unwrap());
}

#[test]
fn the_raw_file_is_checked_before_it_is_written() {
    let (_dir, path) = file();
    assert_eq!(config_read(path.clone()).unwrap(), "", "missing file reads empty");
    assert!(config_check("[user]\nname = \"Ada\"\n".into()).is_empty());
    assert_eq!(config_check("[colour]\nbad = 1\n".into()), ["unknown key `colour.bad`"]);
    let broken = "[user\nname = ";
    assert_eq!(config_check(broken.into()).len(), 1);
    assert!(config_write(path.clone(), broken.into()).is_err());
    assert_eq!(config_read(path).unwrap(), "", "nothing written");
}
