//! This device's knowledge of its PDF files for lazy loading (migration 017; never synced): which
//! the hub confirmed it holds, when each was last opened, and the papers and projects kept offline.

use std::collections::HashSet;

use quire_core::Result;
use quire_model::{ProjectId, ReferenceId};
use rusqlite::params;

use super::SqliteLibrary;
use super::codec::id_blob;
use crate::error::OrCore;

/// A live attachment's file as this device sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileState {
    pub hash: String,
    pub size: u64,
    /// The hub confirmed it holds this file (so evicting it here loses nothing).
    pub on_hub: bool,
    /// Milliseconds since 1970; `None`: never opened here.
    pub last_opened: Option<i64>,
}

/// What a keep-offline mark names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeepTarget {
    Reference(ReferenceId),
    /// The project and its subprojects.
    Project(ProjectId),
}

impl KeepTarget {
    fn parts(self) -> (&'static str, [u8; 16]) {
        match self {
            KeepTarget::Reference(id) => ("reference", id_blob(id)),
            KeepTarget::Project(id) => ("project", id_blob(id)),
        }
    }
}

impl SqliteLibrary<'_> {
    /// Records that the hub holds these files.
    pub fn mark_on_hub(&self, hashes: &[String]) -> Result<()> {
        let mut statement = self
            .conn
            .prepare("INSERT INTO file_local (hash, on_hub) VALUES (?1, 1) ON CONFLICT (hash) DO UPDATE SET on_hub = 1")
            .or_core()?;
        for hash in hashes {
            statement.execute([hash]).or_core()?;
        }
        Ok(())
    }

    /// Records that the file `hash` was opened now (eviction keeps the most recently opened).
    pub fn touch_file(&self, hash: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO file_local (hash, last_opened) VALUES (?1, ?2)
                 ON CONFLICT (hash) DO UPDATE SET last_opened = excluded.last_opened",
                params![hash, quire_model::Timestamp::now().as_millis()],
            )
            .or_core()?;
        Ok(())
    }

    /// Marks (or unmarks) a paper or project to keep downloaded; `false` if nothing changed.
    pub fn set_keep_offline(&self, target: KeepTarget, keep: bool) -> Result<bool> {
        let (kind, id) = target.parts();
        let sql = if keep {
            "INSERT OR IGNORE INTO keep_offline (kind, target) VALUES (?1, ?2)"
        } else {
            "DELETE FROM keep_offline WHERE kind = ?1 AND target = ?2"
        };
        Ok(self.conn.execute(sql, params![kind, id.as_slice()]).or_core()? > 0)
    }

    /// The files of papers kept offline: marked themselves, or in a marked project or one of its
    /// subprojects.
    pub fn kept_hashes(&self) -> Result<HashSet<String>> {
        let sql = "
            WITH RECURSIVE kept_project(id) AS (
                SELECT target FROM keep_offline WHERE kind = 'project'
                UNION
                SELECT p.id FROM project p JOIN kept_project k ON p.parent_id = k.id WHERE p.deleted_at IS NULL
            ),
            kept_reference(id) AS (
                SELECT target FROM keep_offline WHERE kind = 'reference'
                UNION
                SELECT pr.reference_id FROM project_reference pr JOIN kept_project k ON pr.project_id = k.id
                WHERE pr.deleted_at IS NULL
            )
            SELECT DISTINCT a.hash FROM attachment a JOIN kept_reference r ON a.reference_id = r.id
            WHERE a.deleted_at IS NULL";
        let mut statement = self.conn.prepare(sql).or_core()?;
        let rows = statement.query_map([], |row| row.get(0)).or_core()?;
        rows.collect::<rusqlite::Result<_>>().or_core()
    }

    /// Every live attachment's file with what this device knows of it.
    pub fn file_states(&self) -> Result<Vec<FileState>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT a.hash, MAX(a.size), COALESCE(f.on_hub, 0), f.last_opened
                 FROM attachment a LEFT JOIN file_local f ON f.hash = a.hash
                 WHERE a.deleted_at IS NULL GROUP BY a.hash",
            )
            .or_core()?;
        let rows = statement
            .query_map([], |row| {
                Ok(FileState {
                    hash: row.get(0)?,
                    size: row.get::<_, i64>(1)? as u64,
                    on_hub: row.get(2)?,
                    last_opened: row.get(3)?,
                })
            })
            .or_core()?;
        rows.collect::<rusqlite::Result<_>>().or_core()
    }
}
