//! Keeping the user's data private on a shared computer: quire's own folders are readable by the
//! user only (`700`), its config and secret files too (`600`). Like `~/.ssh` and gh's
//! `hosts.yml`. Files written for sharing (exports, `quire.toml`) keep normal permissions.

use std::path::Path;

use crate::error::{EngineError, Result};

/// Creates `dir` if needed and makes it the user's only.
pub fn dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(EngineError::file(dir))?;
    restrict(dir, 0o700)
}

/// Makes the existing file `path` readable and writable by the user only.
pub fn file(path: &Path) -> Result<()> {
    if path.is_file() { restrict(path, 0o600) } else { Ok(()) }
}

#[cfg(unix)]
fn restrict(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let current = std::fs::metadata(path).map_err(EngineError::file(path))?.permissions().mode() & 0o777;
    if current != mode {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(EngineError::file(path))?;
    }
    Ok(())
}

/// Elsewhere (Windows) a user's profile folders are private by default.
#[cfg(not(unix))]
fn restrict(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn tightens_dirs_and_files() {
        let root = tempfile::tempdir().unwrap();
        let library = root.path().join("quire/library");
        dir(&library).unwrap();
        let config = root.path().join("config.toml");
        std::fs::write(&config, "x").unwrap();
        std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o644)).unwrap();
        file(&config).unwrap();
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!((mode(&library), mode(&config)), (0o700, 0o600));
    }
}
