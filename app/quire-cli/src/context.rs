//! What every command gets: the global options (which library, which config file), the folder's
//! `quire.toml` if there is one, and the engine opened once on them.

use std::path::PathBuf;

use quire_engine::{Engine, FolderFile, LibraryChoice, OpenOptions, SlugPath, paths};
use tokio::sync::OnceCell;

pub struct Context {
    /// `--lib <name>`.
    pub lib: Option<String>,
    /// `--library <path>`.
    pub library: Option<PathBuf>,
    /// `--config <file>`.
    pub config: Option<PathBuf>,
    /// The nearest `quire.toml`, from the current folder up.
    pub folder: Option<FolderFile>,
    engine: OnceCell<Engine>,
}

impl Context {
    pub fn new(lib: Option<String>, library: Option<PathBuf>, config: Option<PathBuf>) -> anyhow::Result<Self> {
        let folder = FolderFile::find(&std::env::current_dir()?)?;
        Ok(Self { lib, library, config, folder, engine: OnceCell::new() })
    }

    pub fn config_file(&self) -> anyhow::Result<PathBuf> {
        Ok(paths::config_file(self.config.as_deref())?)
    }

    /// The library to use: `--library`, `--lib`, `$QUIRE_LIBRARY`, `$QUIRE_LIB`, the folder's
    /// `quire.toml`, else the default. Flags win over the environment.
    pub fn choice(&self) -> LibraryChoice {
        let env = |name| std::env::var_os(name).filter(|v| !v.is_empty());
        if let Some(path) = &self.library {
            LibraryChoice::Path(path.clone())
        } else if let Some(name) = &self.lib {
            LibraryChoice::Named(name.clone())
        } else if let Some(path) = env(paths::LIBRARY_ENV) {
            LibraryChoice::Path(path.into())
        } else if let Some(name) = env(paths::LIB_ENV) {
            LibraryChoice::Named(name.to_string_lossy().into_owned())
        } else if let Some(name) = self.folder.as_ref().and_then(|f| f.library.clone()) {
            LibraryChoice::Named(name)
        } else {
            LibraryChoice::Default
        }
    }

    /// The engine on the chosen library, opened (and created) on first use.
    pub async fn engine(&self) -> anyhow::Result<&Engine> {
        self.engine.get_or_try_init(|| self.open(self.choice())).await
    }

    /// The engine, if a command opened it.
    pub fn opened(&self) -> Option<&Engine> {
        self.engine.get()
    }

    /// The engine on another library, e.g. the target of `ref copy`.
    pub async fn open(&self, library: LibraryChoice) -> anyhow::Result<Engine> {
        let options = OpenOptions { library, config_file: self.config.clone(), endpoints: None };
        Ok(Engine::open(options).await?)
    }

    /// The project a command acts on: `-p`, else the folder's (unless `all`), with whether to
    /// include subprojects (`-r`, or the folder's `recursive` for its project).
    pub fn project(&self, flag: Option<&SlugPath>, recursive: bool, all: bool) -> (Option<SlugPath>, bool) {
        match (flag, &self.folder) {
            (Some(project), _) => (Some(project.clone()), recursive),
            (None, Some(folder)) if !all && folder.project.is_some() => {
                (folder.project.clone(), recursive || folder.recursive)
            }
            _ => (None, recursive),
        }
    }
}
