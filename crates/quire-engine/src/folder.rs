//! `quire.toml` in a document's folder (`.knowledge/cli-tui.md`, project linked to a folder):
//! which library and project the document uses and where its bibliography goes. Inside the
//! folder (or below it, found like git finds `.git`) commands need no `--lib`, `-p` or `-o`.
//!
//! ```toml
//! library = "lab"          # optional: else the default library
//! project = "thesis"
//! recursive = true         # include subprojects
//! output = "refs.bib"      # relative to this file
//! profile = "thesis"       # optional export profile
//! format = "hayagriva"     # optional, overrides the profile's
//! ```

use std::path::{Path, PathBuf};

use quire_model::SlugPath;
use serde::Deserialize;

use crate::error::{EngineError, Result};

/// The file's name.
pub const FILE_NAME: &str = "quire.toml";

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FolderFile {
    pub library: Option<String>,
    #[serde(default, deserialize_with = "slug_path")]
    pub project: Option<SlugPath>,
    #[serde(default)]
    pub recursive: bool,
    pub output: Option<PathBuf>,
    pub profile: Option<String>,
    pub format: Option<String>,
    /// The folder holding the file (set when found).
    #[serde(skip)]
    pub folder: PathBuf,
}

fn slug_path<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Option<SlugPath>, D::Error> {
    let text = Option::<String>::deserialize(deserializer)?;
    text.map(|t| t.parse().map_err(serde::de::Error::custom)).transpose()
}

impl FolderFile {
    /// The nearest `quire.toml` in `start` or a folder above it.
    pub fn find(start: &Path) -> Result<Option<Self>> {
        for folder in start.ancestors() {
            let path = folder.join(FILE_NAME);
            if path.is_file() {
                let text = std::fs::read_to_string(&path).map_err(EngineError::file(&path))?;
                let mut file: Self =
                    toml::from_str(&text).map_err(|e| EngineError::Config(format!("{}: {e}", path.display())))?;
                file.folder = folder.to_path_buf();
                return Ok(Some(file));
            }
        }
        Ok(None)
    }

    /// The output file as an absolute path.
    pub fn output_path(&self) -> Option<PathBuf> {
        self.output.as_ref().map(|output| self.folder.join(output))
    }

    /// Writes `quire.toml` into `folder` (refusing to overwrite one).
    pub fn create(folder: &Path, file: &FolderFile) -> Result<PathBuf> {
        let path = folder.join(FILE_NAME);
        if path.exists() {
            return Err(EngineError::Config(format!("{} already exists", path.display())));
        }
        let mut document = toml_edit::DocumentMut::new();
        if let Some(library) = &file.library {
            document["library"] = toml_edit::value(library.as_str());
        }
        if let Some(project) = &file.project {
            document["project"] = toml_edit::value(project.to_string());
        }
        if file.recursive {
            document["recursive"] = toml_edit::value(true);
        }
        if let Some(output) = &file.output {
            document["output"] = toml_edit::value(output.display().to_string());
        }
        if let Some(profile) = &file.profile {
            document["profile"] = toml_edit::value(profile.as_str());
        }
        if let Some(format) = &file.format {
            document["format"] = toml_edit::value(format.as_str());
        }
        let text = format!("# quire: this folder's library, project and bibliography file.\n{document}");
        std::fs::write(&path, text).map_err(EngineError::file(&path))?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_find_from_a_subfolder() {
        let dir = tempfile::tempdir().unwrap();
        let file = FolderFile {
            project: Some("thesis/ch-1".parse().unwrap()),
            recursive: true,
            output: Some("refs.bib".into()),
            ..FolderFile::default()
        };
        FolderFile::create(dir.path(), &file).unwrap();
        assert!(FolderFile::create(dir.path(), &file).is_err(), "no overwrite");
        let sub = dir.path().join("chapters/one");
        std::fs::create_dir_all(&sub).unwrap();
        let found = FolderFile::find(&sub).unwrap().unwrap();
        assert_eq!(found.project, file.project);
        assert!(found.recursive);
        assert_eq!(found.output_path().unwrap(), dir.path().join("refs.bib"));
    }

    #[test]
    fn unknown_keys_are_errors() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(FILE_NAME), "projekt = \"x\"").unwrap();
        assert!(FolderFile::find(dir.path()).is_err());
    }
}
