//! Backup and restore of a whole library as one `.tar.gz`: a consistent copy of the database
//! (SQLite `VACUUM INTO`, taken while the library may be in use), the stored PDFs (`files/`), the
//! papers' texts and figures (`text/`) and the other files of the folder, with a manifest
//! (`quire-backup.json`). A restored library gets a new device ID, so it is never taken for the
//! copy it came from when both sync.

use std::fs::File;
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};

use super::Engine;
use crate::error::{EngineError, Result};
use crate::paths::DATABASE_FILE;
use crate::private;

/// The manifest's name in the archive.
const MANIFEST: &str = "quire-backup.json";
/// Format of the archive; raised if its layout changes.
const FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub library_id: String,
    /// Milliseconds since the epoch.
    pub created_at: i64,
    pub quire: String,
}

/// What a backup holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupReport {
    pub path: PathBuf,
    /// Files in the archive (the database included).
    pub files: usize,
    /// Size of the archive.
    pub bytes: u64,
}

impl Engine {
    /// Writes the library to `output` (`.tar.gz`, readable by the user only).
    pub async fn backup(&self, output: &Path) -> Result<BackupReport> {
        let snapshot_dir = tempfile::tempdir().map_err(EngineError::file(output))?;
        let snapshot = snapshot_dir.path().join(DATABASE_FILE);
        let target = snapshot.display().to_string();
        // `VACUUM INTO` cannot run in a transaction: on the connection itself.
        self.conn.call_raw(move |conn| conn.execute("VACUUM INTO ?1", [target])).await??;
        let library_id = self.run(|lib| lib.library_id()).await?;
        let manifest = Manifest {
            format: FORMAT,
            library_id,
            created_at: quire_model::Timestamp::now().as_millis(),
            quire: env!("CARGO_PKG_VERSION").into(),
        };
        let dir = self.library_dir().to_path_buf();
        let output = output.to_path_buf();
        tokio::task::spawn_blocking(move || write_archive(&dir, &snapshot, &manifest, &output))
            .await
            .map_err(|e| EngineError::Config(format!("backup: {e}")))?
    }
}

fn write_archive(dir: &Path, snapshot: &Path, manifest: &Manifest, output: &Path) -> Result<BackupReport> {
    let partial = output.with_extension("partial");
    let file = File::create(&partial).map_err(EngineError::file(&partial))?;
    private::file(&partial)?;
    let mut archive = tar::Builder::new(GzEncoder::new(file, Compression::default()));
    let io = |path: &Path| EngineError::file(path.to_path_buf());
    let json = serde_json::to_vec_pretty(manifest).map_err(|e| EngineError::Config(e.to_string()))?;
    let mut header = tar::Header::new_gnu();
    header.set_size(json.len() as u64);
    header.set_mode(0o600);
    header.set_mtime((manifest.created_at / 1000) as u64);
    header.set_cksum();
    archive.append_data(&mut header, MANIFEST, json.as_slice()).map_err(io(output))?;
    archive.append_path_with_name(snapshot, DATABASE_FILE).map_err(io(snapshot))?;
    let mut files = 1;
    for path in walk(dir)? {
        let relative = path.strip_prefix(dir).expect("under the library folder");
        let name = relative.to_string_lossy();
        // The live database is in the snapshot; half-written files are not files yet.
        if name.starts_with(DATABASE_FILE) || name.ends_with(".partial") {
            continue;
        }
        archive.append_path_with_name(&path, relative).map_err(io(&path))?;
        files += 1;
    }
    let encoder = archive.into_inner().map_err(io(output))?;
    encoder.finish().map_err(io(output))?;
    std::fs::rename(&partial, output).map_err(io(output))?;
    let bytes = std::fs::metadata(output).map_err(io(output))?.len();
    Ok(BackupReport { path: output.to_path_buf(), files, bytes })
}

/// Every regular file under `dir`, in name order.
fn walk(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in std::fs::read_dir(&current).map_err(EngineError::file(&current))? {
            let entry = entry.map_err(EngineError::file(&current))?;
            let kind = entry.file_type().map_err(EngineError::file(entry.path()))?;
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                out.push(entry.path());
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Restores the backup `archive` into the folder `target` (which must not exist or be empty);
/// the restored library gets a new device ID and no files kept updated. Returns its manifest.
pub fn restore(archive: &Path, target: &Path) -> Result<Manifest> {
    if target.exists() && std::fs::read_dir(target).map_err(EngineError::file(target))?.next().is_some() {
        return Err(EngineError::Library(format!("{} is not empty; restore into a new folder", target.display())));
    }
    std::fs::create_dir_all(target).map_err(EngineError::file(target))?;
    private::dir(target)?;
    let file = File::open(archive).map_err(EngineError::file(archive))?;
    // `unpack` keeps every entry inside `target` (no `..`, no absolute paths).
    tar::Archive::new(GzDecoder::new(file)).unpack(target).map_err(EngineError::file(archive))?;
    let manifest_path = target.join(MANIFEST);
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|_| EngineError::Library(format!("{} is not a quire backup (no {MANIFEST})", archive.display())))?;
    let manifest: Manifest =
        serde_json::from_str(&text).map_err(|e| EngineError::Library(format!("unreadable backup manifest: {e}")))?;
    if manifest.format > FORMAT {
        return Err(EngineError::Library(format!(
            "this backup is format {}; this quire reads up to {FORMAT}: update quire",
            manifest.format
        )));
    }
    std::fs::remove_file(&manifest_path).map_err(EngineError::file(&manifest_path))?;
    let database = target.join(DATABASE_FILE);
    let conn = rusqlite::Connection::open(&database)?;
    // A new device: this copy and the one backed up may both sync.
    conn.execute("DELETE FROM library_meta WHERE key = 'device_id'", [])?;
    // Files kept updated are paths on the machine the backup came from: rewriting them from here
    // would overwrite whatever is there (and two libraries would write one file).
    conn.execute("DELETE FROM auto_export", [])?;
    Ok(manifest)
}
