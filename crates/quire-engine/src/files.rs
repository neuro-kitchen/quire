//! The library's file store (`.knowledge/storage.md`): each file once, named by the BLAKE3 hash
//! of its content, in `files/<first two hex digits>/<hash>.<extension>` inside the library
//! folder. Adding the same file twice stores it once; a file is never changed in place.

use std::path::{Path, PathBuf};

use crate::error::{EngineError, Result};

/// The folder inside a library holding stored files.
pub(crate) const FILES_DIR: &str = "files";

pub struct FileStore {
    root: PathBuf,
}

impl FileStore {
    pub fn new(library_dir: &Path) -> Self {
        Self { root: library_dir.join(FILES_DIR) }
    }

    /// The BLAKE3 hash of `bytes`, hex.
    pub fn hash(bytes: &[u8]) -> String {
        blake3::hash(bytes).to_hex().to_string()
    }

    /// Where the file with `hash` and `extension` is (or would be) stored.
    pub fn path(&self, hash: &str, extension: &str) -> PathBuf {
        self.root.join(&hash[..2]).join(format!("{hash}.{extension}"))
    }

    /// Stores `bytes` (unless already there); returns the hash. Written to a temporary name and
    /// renamed, so a crash never leaves a partial file under a valid hash.
    pub fn put(&self, bytes: &[u8], extension: &str) -> Result<String> {
        let hash = Self::hash(bytes);
        let path = self.path(&hash, extension);
        if path.is_file() {
            return Ok(hash);
        }
        let dir = path.parent().expect("stored files are in a subfolder");
        std::fs::create_dir_all(dir).map_err(EngineError::file(dir))?;
        let temporary = dir.join(format!(".{hash}.partial"));
        std::fs::write(&temporary, bytes).map_err(EngineError::file(&temporary))?;
        std::fs::rename(&temporary, &path).map_err(EngineError::file(&path))?;
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_content_is_stored_once() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileStore::new(dir.path());
        let first = store.put(b"%PDF-1.4 a", "pdf").unwrap();
        assert_eq!(store.put(b"%PDF-1.4 a", "pdf").unwrap(), first);
        assert_ne!(store.put(b"%PDF-1.4 b", "pdf").unwrap(), first);
        let path = store.path(&first, "pdf");
        assert!(path.starts_with(dir.path().join("files").join(&first[..2])));
        assert_eq!(std::fs::read(path).unwrap(), b"%PDF-1.4 a");
    }
}
