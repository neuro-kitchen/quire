//! Changing the config file with `toml_edit`, so the user's comments and layout are kept.

use std::path::Path;

use toml_edit::{DocumentMut, Item, Table};

use super::keys;
use crate::error::{EngineError, Result};
use crate::{paths, private};

/// Sets `key` to `text` (checked against its [`keys::KEYS`] entry) in the file at `path`, created
/// with its directory if missing.
pub fn set(path: &Path, key: &str, text: &str) -> Result<()> {
    let spec = keys::spec(key)?;
    let value = keys::to_toml(spec.kind, text)?;
    edit(path, |document| {
        let (tables, field) = split(key);
        table_at(document, &tables)?.insert(field, Item::Value(value));
        Ok(())
    })
}

/// Removes `key`; `false` if it was not set.
pub fn unset(path: &Path, key: &str) -> Result<bool> {
    keys::spec(key)?;
    let mut removed = false;
    edit(path, |document| {
        let (tables, field) = split(key);
        let mut table = document.as_table_mut();
        for name in tables {
            match table.get_mut(name).and_then(Item::as_table_mut) {
                Some(inner) => table = inner,
                None => return Ok(()),
            }
        }
        removed = table.remove(field).is_some();
        Ok(())
    })?;
    Ok(removed)
}

/// Renames the table `[<parent>.<old>]` to `[<parent>.<new>]` (e.g. a library), keeping its contents.
pub fn rename_table(path: &Path, parent: &str, old: &str, new: &str) -> Result<()> {
    edit(path, |document| {
        let table = table_at(document, &[parent])?;
        let item = table.remove(old).ok_or_else(|| EngineError::Config(format!("no `{parent}.{old}`")))?;
        table.insert(new, item);
        Ok(())
    })
}

/// Removes the table `[<parent>.<name>]`; `false` if it did not exist.
pub fn remove_table(path: &Path, parent: &str, name: &str) -> Result<bool> {
    let mut removed = false;
    edit(path, |document| {
        if let Some(table) = document.get_mut(parent).and_then(Item::as_table_mut) {
            removed = table.remove(name).is_some();
        }
        Ok(())
    })?;
    Ok(removed)
}

/// Reads the file (empty if missing), applies `change`, writes it back.
pub(super) fn edit(path: &Path, change: impl FnOnce(&mut DocumentMut) -> Result<()>) -> Result<()> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(EngineError::file(path)(e)),
    };
    let mut document: DocumentMut =
        text.parse().map_err(|e| EngineError::Config(format!("{}: {e}", path.display())))?;
    change(&mut document)?;
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        // quire's own config folder is made private; a folder the user chose is left as it is.
        if paths::config_dir().is_ok_and(|ours| ours == dir) {
            private::dir(dir)?;
        } else {
            std::fs::create_dir_all(dir).map_err(EngineError::file(dir))?;
        }
    }
    std::fs::write(path, document.to_string()).map_err(EngineError::file(path))?;
    private::file(path)
}

/// `a.b.c` → (`[a, b]`, `c`).
fn split(key: &str) -> (Vec<&str>, &str) {
    let mut parts: Vec<&str> = key.split('.').collect();
    let field = parts.pop().expect("split yields at least one part");
    (parts, field)
}

/// The table at `names`, creating missing ones. Intermediate tables are implicit, so
/// `libraries.lab.path` is written as `[libraries.lab]` without an empty `[libraries]`.
pub(super) fn table_at<'d>(document: &'d mut DocumentMut, names: &[&str]) -> Result<&'d mut Table> {
    let mut table = document.as_table_mut();
    for (depth, name) in names.iter().enumerate() {
        let item = table.entry(name).or_insert_with(|| {
            let mut new = Table::new();
            new.set_implicit(depth + 1 < names.len());
            Item::Table(new)
        });
        table = item.as_table_mut().ok_or_else(|| EngineError::Config(format!("`{name}` is not a table")))?;
    }
    Ok(table)
}
