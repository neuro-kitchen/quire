//! Libraries: which exist (the config's `[libraries]`), registering, renaming, choosing the
//! default; backing one up; copying a reference into another.

use std::path::Path;

use quire_engine::config::Config;
use quire_engine::{Engine, LibraryChoice, OpenOptions, libraries};

use crate::config;
use crate::references::{AddResult, add_result};
use crate::{QuireLibrary, Result, query};

/// A library: `name` `None` for an unregistered folder.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct LibraryEntryRecord {
    pub name: Option<String>,
    pub path: String,
    pub registered: bool,
    pub is_default: bool,
}

/// A backup written: the archive, how many files and bytes it holds.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct BackupRecord {
    pub path: String,
    pub files: u32,
    pub bytes: u64,
}

/// The libraries the config file knows (`config_file` `None`: the usual one), the default first.
#[uniffi::export]
pub fn list_libraries(config_file: Option<String>) -> Result<Vec<LibraryEntryRecord>> {
    let config = Config::load(&config::path(config_file)?)?;
    Ok(libraries::list(&config)?.into_iter().map(entry).collect())
}

/// Registers a library named `name` in `path` (`None`: the usual place for libraries).
#[uniffi::export]
pub fn register_library(config_file: Option<String>, name: String, path: Option<String>) -> Result<LibraryEntryRecord> {
    Ok(entry(libraries::register(&config::path(config_file)?, &name, path.as_deref().map(Path::new))?))
}

#[uniffi::export]
pub fn set_default_library(config_file: Option<String>, name: String) -> Result<LibraryEntryRecord> {
    Ok(entry(libraries::set_default(&config::path(config_file)?, &name)?))
}

#[uniffi::export]
pub fn rename_library(config_file: Option<String>, name: String, new_name: String) -> Result<LibraryEntryRecord> {
    Ok(entry(libraries::rename(&config::path(config_file)?, &name, &new_name)?))
}

/// Forgets a registered library (its folder stays).
#[uniffi::export]
pub fn unregister_library(config_file: Option<String>, name: String) -> Result<LibraryEntryRecord> {
    Ok(entry(libraries::unregister(&config::path(config_file)?, &name)?))
}

#[uniffi::export]
impl QuireLibrary {
    /// Writes the library to `output` (`.tar.gz`, readable by the user only).
    pub async fn backup(&self, output: String) -> Result<BackupRecord> {
        self.with(|engine| async move {
            let report = engine.backup(Path::new(&output)).await?;
            Ok(BackupRecord {
                path: report.path.display().to_string(),
                files: report.files as u32,
                bytes: report.bytes,
            })
        })
        .await
    }

    /// Copies a reference into the library named `library` (keeping its ID), into `project` there.
    pub async fn copy_to_library(&self, id: String, library: String, project: Option<String>) -> Result<AddResult> {
        let config_file = self.engine.config_file().to_path_buf();
        self.with(|engine| async move {
            let target = Engine::open(OpenOptions {
                library: LibraryChoice::Named(library),
                config_file: Some(config_file),
                endpoints: None,
            })
            .await?;
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let added = engine.copy_to(&id, &target, project.as_ref()).await?;
            add_result(&target, added).await
        })
        .await
    }
}

fn entry(info: libraries::LibraryInfo) -> LibraryEntryRecord {
    LibraryEntryRecord {
        name: info.name,
        path: info.path.display().to_string(),
        registered: info.registered,
        is_default: info.is_default,
    }
}
