//! Named libraries (`.knowledge/library-model.md`, multiple libraries): each is its own folder
//! (database, files, device ID), registered in the config as `[libraries.<name>] path = …`.
//!
//! Which library a command uses: an explicit folder, else a name, else `library.default`, else
//! `default`. The `default` library needs no registration: unregistered, it is the folder quire
//! used before named libraries existed.

use std::path::{Path, PathBuf};

use quire_model::Slug;

use crate::config::{self, Config, edit};
use crate::error::{EngineError, Result};
use crate::paths;

/// Name of the library used when nothing else is chosen or configured.
pub const DEFAULT_NAME: &str = "default";

/// Which library to open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LibraryChoice {
    /// `library.default`, else `default`.
    #[default]
    Default,
    /// A registered library (or `default`).
    Named(String),
    /// Any folder, registered or not.
    Path(PathBuf),
}

/// A library as resolved from the config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryInfo {
    /// `None` for an unregistered folder.
    pub name: Option<String>,
    pub path: PathBuf,
    /// Listed in `[libraries]` (the implicit `default` is not).
    pub registered: bool,
    /// The one used when none is chosen.
    pub is_default: bool,
}

impl LibraryInfo {
    /// Whether the folder holds a library database yet.
    pub fn exists(&self) -> bool {
        self.path.join(paths::DATABASE_FILE).is_file()
    }

    /// The name, or the folder for an unregistered one.
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.path.display().to_string())
    }
}

/// The name of the default library.
pub fn default_name(config: &Config) -> &str {
    config.library.default.as_deref().unwrap_or(DEFAULT_NAME)
}

/// The library `choice` names.
pub fn resolve(config: &Config, choice: &LibraryChoice) -> Result<LibraryInfo> {
    match choice {
        LibraryChoice::Default => named(config, default_name(config)),
        LibraryChoice::Named(name) => named(config, name),
        LibraryChoice::Path(path) => {
            let path = paths::expand_home(path);
            Ok(list(config)?.into_iter().find(|info| info.path == path).unwrap_or(LibraryInfo {
                name: None,
                path,
                registered: false,
                is_default: false,
            }))
        }
    }
}

fn named(config: &Config, name: &str) -> Result<LibraryInfo> {
    list(config)?.into_iter().find(|info| info.name.as_deref() == Some(name)).ok_or_else(|| {
        let known: Vec<String> = list(config).unwrap_or_default().iter().map(LibraryInfo::label).collect();
        EngineError::Library(format!("no library `{name}` (known: {})", known.join(", ")))
    })
}

/// Every registered library, plus `default` when it is not registered; sorted by name.
pub fn list(config: &Config) -> Result<Vec<LibraryInfo>> {
    let default = default_name(config);
    let mut out: Vec<LibraryInfo> = config
        .libraries
        .iter()
        .map(|(name, entry)| LibraryInfo {
            name: Some(name.clone()),
            path: paths::expand_home(&entry.path),
            registered: true,
            is_default: name == default,
        })
        .collect();
    if !config.libraries.contains_key(DEFAULT_NAME) {
        let path = match &config.library.path {
            Some(old) => paths::expand_home(old),
            None => paths::default_library_dir()?,
        };
        // Once its folder is registered under another name (renamed), it is that library.
        if !out.iter().any(|info| info.path == path) {
            out.push(LibraryInfo {
                name: Some(DEFAULT_NAME.into()),
                path,
                registered: false,
                is_default: default == DEFAULT_NAME,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Registers a library `name` at `path` (default: `~/.local/share/quire/libraries/<name>`).
/// The caller creates its database by opening it.
pub fn register(config_file: &Path, name: &str, path: Option<&Path>) -> Result<LibraryInfo> {
    check_name(name)?;
    let config = Config::load(config_file)?;
    if list(&config)?.iter().any(|info| info.name.as_deref() == Some(name)) {
        return Err(EngineError::Library(format!("library `{name}` already exists")));
    }
    let path = match path {
        Some(path) => std::path::absolute(paths::expand_home(path)).map_err(EngineError::file(path))?,
        None => paths::new_library_dir(name)?,
    };
    if let Some(other) = list(&config)?.into_iter().find(|info| info.path == path) {
        return Err(EngineError::Library(format!("{} is already the library `{}`", path.display(), other.label())));
    }
    config::set(config_file, &format!("libraries.{name}.path"), &path.display().to_string())?;
    resolve(&Config::load(config_file)?, &LibraryChoice::Named(name.into()))
}

/// Makes `name` the default library.
pub fn set_default(config_file: &Path, name: &str) -> Result<LibraryInfo> {
    let info = named(&Config::load(config_file)?, name)?;
    config::set(config_file, "library.default", name)?;
    Ok(LibraryInfo { is_default: true, ..info })
}

/// Renames a library (its folder stays where it is).
pub fn rename(config_file: &Path, old: &str, new: &str) -> Result<LibraryInfo> {
    check_name(new)?;
    let config = Config::load(config_file)?;
    let info = named(&config, old)?;
    if named(&config, new).is_ok() {
        return Err(EngineError::Library(format!("library `{new}` already exists")));
    }
    if info.registered {
        edit::rename_table(config_file, "libraries", old, new)?;
    } else {
        // The implicit `default`: register its folder under the new name.
        config::set(config_file, &format!("libraries.{new}.path"), &info.path.display().to_string())?;
    }
    if info.is_default {
        config::set(config_file, "library.default", new)?;
    }
    resolve(&Config::load(config_file)?, &LibraryChoice::Named(new.into()))
}

/// Forgets a registered library; its folder and files stay. If it was the default, `default`
/// becomes the default again.
pub fn unregister(config_file: &Path, name: &str) -> Result<LibraryInfo> {
    let info = named(&Config::load(config_file)?, name)?;
    if !info.registered {
        return Err(EngineError::Library(format!("`{name}` is not registered (it is the built-in default library)")));
    }
    edit::remove_table(config_file, "libraries", name)?;
    if info.is_default {
        config::unset(config_file, "library.default")?;
    }
    Ok(info)
}

fn check_name(name: &str) -> Result<()> {
    Slug::parse(name).map(|_| ()).map_err(|_| {
        EngineError::Library(format!("`{name}` is not a library name (lowercase letters, digits and `-`)"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.toml");
        (dir, file)
    }

    #[test]
    fn default_exists_without_registration() {
        let config = Config::default();
        let all = list(&config).unwrap();
        assert_eq!(all.len(), 1);
        assert!(all[0].is_default && !all[0].registered);
        assert_eq!(resolve(&config, &LibraryChoice::Default).unwrap().name.as_deref(), Some(DEFAULT_NAME));
    }

    #[test]
    fn register_use_rename_unregister() {
        let (dir, file) = setup();
        let lab = register(&file, "lab", Some(&dir.path().join("lab"))).unwrap();
        assert!(lab.registered && !lab.is_default);
        assert!(register(&file, "lab", None).is_err());
        assert!(register(&file, "other", Some(&dir.path().join("lab"))).is_err());
        assert!(register(&file, "Bad Name", None).is_err());

        set_default(&file, "lab").unwrap();
        let config = Config::load(&file).unwrap();
        assert_eq!(resolve(&config, &LibraryChoice::Default).unwrap().path, dir.path().join("lab"));
        let by_path = resolve(&config, &LibraryChoice::Path(dir.path().join("lab"))).unwrap();
        assert_eq!(by_path.name.as_deref(), Some("lab"));

        rename(&file, "lab", "group").unwrap();
        let config = Config::load(&file).unwrap();
        assert_eq!(default_name(&config), "group");
        assert!(resolve(&config, &LibraryChoice::Named("lab".into())).is_err());

        unregister(&file, "group").unwrap();
        let config = Config::load(&file).unwrap();
        assert_eq!(default_name(&config), DEFAULT_NAME);
        assert!(unregister(&file, DEFAULT_NAME).is_err());
    }

    #[test]
    fn renaming_the_implicit_default_registers_it() {
        let (_dir, file) = setup();
        let renamed = rename(&file, DEFAULT_NAME, "personal").unwrap();
        assert!(renamed.registered && renamed.is_default);
        assert_eq!(renamed.path, paths::default_library_dir().unwrap());
        let all = list(&Config::load(&file).unwrap()).unwrap();
        assert_eq!(all.len(), 1, "{all:?}");
    }
}
