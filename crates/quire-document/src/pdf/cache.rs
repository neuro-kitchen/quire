//! Thread-safe document and text layer cache: the last [`MAX_DOCUMENTS`] documents used stay
//! parsed, with the text layers computed so far, until their file changes (size or mtime).
//!
//! Files are read and parsed outside the lock, so a large PDF never holds up another one.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::SystemTime;

use hayro_syntax::Pdf;

use super::layer::TextCollector;
use super::types::PageText;
use crate::error::DocumentError;

/// Documents kept parsed: the open papers and a few recent ones (a parsed PDF holds its bytes).
const MAX_DOCUMENTS: usize = 8;

struct CachedDocument {
    mtime: SystemTime,
    size: u64,
    pdf: Arc<Pdf>,
    texts: HashMap<u32, Arc<PageText>>,
    /// When it was last used, on [`Documents::clock`].
    used: u64,
}

#[derive(Default)]
struct Documents {
    map: HashMap<PathBuf, CachedDocument>,
    clock: u64,
}

impl Documents {
    /// The cached document for this version of `path`, marked as just used.
    fn fresh(&mut self, path: &Path, mtime: SystemTime, size: u64) -> Option<&mut CachedDocument> {
        self.clock += 1;
        let clock = self.clock;
        let cached = self.map.get_mut(path).filter(|c| c.mtime == mtime && c.size == size)?;
        cached.used = clock;
        Some(cached)
    }

    /// The cached entry of `path` if it still holds `pdf` (not replaced by a newer version).
    fn holding(&mut self, path: &Path, pdf: &Arc<Pdf>) -> Option<&mut CachedDocument> {
        self.map.get_mut(path).filter(|c| Arc::ptr_eq(&c.pdf, pdf))
    }

    fn insert(&mut self, path: &Path, document: CachedDocument) {
        self.map.insert(path.to_path_buf(), document);
        while self.map.len() > MAX_DOCUMENTS {
            let Some(oldest) = self.map.iter().min_by_key(|(_, c)| c.used).map(|(p, _)| p.clone()) else { break };
            self.map.remove(&oldest);
        }
    }
}

#[derive(Default)]
pub struct DocumentCache {
    documents: Mutex<Documents>,
}

static CACHE: OnceLock<DocumentCache> = OnceLock::new();

pub fn global_cache() -> &'static DocumentCache {
    CACHE.get_or_init(DocumentCache::default)
}

/// Locks `mutex`, recovering it if a thread panicked while holding it: the cache is only changed
/// by single `insert`/`remove`/`clear` calls, so it is never left half-updated.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl DocumentCache {
    /// Loads or returns a cached `Pdf` document, validating file metadata (mtime and size).
    pub fn get_pdf(&self, path: &Path) -> Result<Arc<Pdf>, DocumentError> {
        let meta = fs::metadata(path)?;
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let size = meta.len();
        if let Some(cached) = lock(&self.documents).fresh(path, mtime, size) {
            return Ok(cached.pdf.clone());
        }

        let bytes = fs::read(path)?;
        let pdf = Arc::new(Pdf::new(bytes).map_err(|e| DocumentError::Load(format!("{path:?}: {e:?}")))?);

        let mut documents = lock(&self.documents);
        // Another call may have parsed the same version meanwhile: keep the first.
        if let Some(cached) = documents.fresh(path, mtime, size) {
            return Ok(cached.pdf.clone());
        }
        let used = documents.clock;
        documents.insert(path, CachedDocument { mtime, size, pdf: pdf.clone(), texts: HashMap::new(), used });
        Ok(pdf)
    }

    /// Retrieves or computes the text layer for `page_index`.
    pub fn get_page_text(&self, path: &Path, page_index: u32) -> Result<Arc<PageText>, DocumentError> {
        let pdf = self.get_pdf(path)?;
        let pages = pdf.pages();
        let page = pages.get(page_index as usize).ok_or(DocumentError::PageNotFound(page_index))?;
        if let Some(text) = lock(&self.documents).holding(path, &pdf).and_then(|c| c.texts.get(&page_index).cloned()) {
            return Ok(text);
        }

        let extracted = Arc::new(TextCollector::extract(page)?);
        // Kept only while the entry still holds this version of the file.
        if let Some(cached) = lock(&self.documents).holding(path, &pdf) {
            cached.texts.insert(page_index, extracted.clone());
        }
        Ok(extracted)
    }

    /// Clears all cached documents and text layers.
    pub fn clear(&self) {
        lock(&self.documents).map.clear();
    }
}

#[cfg(test)]
mod tests {
    use quire_testkit::tiny_pdf;

    use super::*;

    /// A panic while the cache's lock is held must not make every later call panic.
    #[test]
    fn a_panic_while_locked_does_not_break_the_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("paper.pdf");
        std::fs::write(&path, tiny_pdf("Place cells")).unwrap();
        let cache = DocumentCache::default();

        let poisoned = std::thread::scope(|s| {
            s.spawn(|| {
                let _guard = cache.documents.lock();
                panic!("a panic with the lock held");
            })
            .join()
        });
        assert!(poisoned.is_err());

        assert_eq!(cache.get_pdf(&path).unwrap().pages().len(), 1);
        assert_eq!(cache.get_page_text(&path, 0).unwrap().words.len(), 2);
    }

    /// The least recently used document goes first, not the oldest loaded.
    #[test]
    fn the_least_recently_used_document_is_evicted() {
        let tmp = tempfile::tempdir().unwrap();
        let paths: Vec<PathBuf> = (0..=MAX_DOCUMENTS)
            .map(|i| {
                let path = tmp.path().join(format!("paper-{i}.pdf"));
                std::fs::write(&path, tiny_pdf(&format!("Paper {i}"))).unwrap();
                path
            })
            .collect();
        let cache = DocumentCache::default();
        for path in &paths[..MAX_DOCUMENTS] {
            cache.get_pdf(path).unwrap();
        }
        cache.get_pdf(&paths[0]).unwrap(); // paper 0 used again: paper 1 is now the least recent
        cache.get_pdf(&paths[MAX_DOCUMENTS]).unwrap();

        let documents = lock(&cache.documents);
        assert_eq!(documents.map.len(), MAX_DOCUMENTS);
        assert!(documents.map.contains_key(&paths[0]));
        assert!(!documents.map.contains_key(&paths[1]));
    }

    /// A rewritten file is parsed again, and its old text layer is not served.
    #[test]
    fn a_changed_file_is_read_again() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("paper.pdf");
        let cache = DocumentCache::default();
        std::fs::write(&path, tiny_pdf("Place cells")).unwrap();
        assert_eq!(cache.get_page_text(&path, 0).unwrap().words[0].text, "Place");
        std::fs::write(&path, tiny_pdf("Grid cells fire")).unwrap(); // another size
        assert_eq!(cache.get_page_text(&path, 0).unwrap().words[0].text, "Grid");
    }
}
