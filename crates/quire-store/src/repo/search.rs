//! Full-text search over `search_index` (metadata, tags, notes, highlights; kept by triggers) and
//! `paper_text_index` (papers' text, written by the engine). Rows are keyed by the reference's
//! rowid. Ranking is FTS5's BM25 with the title and authors weighted above the rest, and the
//! paper's text below the metadata.

use quire_core::ports::SearchStore;
use quire_core::{Result, SearchHit};
use quire_model::ReferenceId;
use rusqlite::{Connection, params};

use super::SqliteLibrary;
use super::codec::{id_at, id_blob};
use super::core::all;
use crate::error::OrCore;

impl SearchStore for SqliteLibrary<'_> {
    fn search(&self, query: &str, in_text: bool, limit: usize) -> Result<Vec<SearchHit>> {
        search(self.conn, query, in_text, limit)
    }

    fn index_paper_text(&mut self, reference: ReferenceId, text: Option<&str>) -> Result<()> {
        index_paper_text(self.conn, reference, text)
    }
}

/// BM25 weights of `search_index`'s columns: cite key, title, authors, abstract, venue,
/// identifiers, tags, notes.
const WEIGHTS: &str = "2.0, 10.0, 6.0, 3.0, 2.0, 4.0, 4.0, 3.0";
/// A match only in the paper's text counts this much of a metadata match.
const TEXT_WEIGHT: f64 = 0.4;

fn search(conn: &Connection, query: &str, in_text: bool, limit: usize) -> Result<Vec<SearchHit>> {
    let mut hits: Vec<SearchHit> = Vec::new();
    let sql = format!(
        "SELECT r.id AS id, bm25(search_index, {WEIGHTS}) AS rank,
                snippet(search_index, -1, '«', '»', '…', 12) AS snippet
         FROM search_index JOIN reference r ON r.rowid = search_index.rowid
         WHERE search_index MATCH ?1 AND r.deleted_at IS NULL
         ORDER BY rank LIMIT ?2"
    );
    hits.extend(all(conn, &sql, params![query, limit as i64], |row| {
        Ok(SearchHit {
            reference: id_at(row, "id")?,
            score: -row.get::<_, f64>("rank")?,
            snippet: row.get("snippet")?,
            in_text: false,
        })
    })?);
    if in_text {
        let sql = "SELECT r.id AS id, bm25(paper_text_index) AS rank,
                          snippet(paper_text_index, 0, '«', '»', '…', 12) AS snippet
                   FROM paper_text_index JOIN reference r ON r.rowid = paper_text_index.rowid
                   WHERE paper_text_index MATCH ?1 AND r.deleted_at IS NULL
                   ORDER BY rank LIMIT ?2";
        let text_hits = all(conn, sql, params![query, limit as i64], |row| {
            Ok(SearchHit {
                reference: id_at(row, "id")?,
                score: -row.get::<_, f64>("rank")? * TEXT_WEIGHT,
                snippet: row.get("snippet")?,
                in_text: true,
            })
        })?;
        for hit in text_hits {
            // A reference found in both: one hit, the scores added, the metadata snippet kept.
            match hits.iter_mut().find(|h| h.reference == hit.reference) {
                Some(known) => known.score += hit.score,
                None => hits.push(hit),
            }
        }
    }
    hits.sort_by(|a, b| b.score.total_cmp(&a.score));
    hits.truncate(limit);
    Ok(hits)
}

/// Replaces the indexed text of `reference` (`None`: removes it).
fn index_paper_text(conn: &Connection, reference: ReferenceId, text: Option<&str>) -> Result<()> {
    let rowid: i64 = conn
        .query_row("SELECT rowid FROM reference WHERE id = ?1", [id_blob(reference)], |row| row.get(0))
        .or_core()?;
    conn.execute("DELETE FROM paper_text_index WHERE rowid = ?1", [rowid]).or_core()?;
    if let Some(text) = text {
        conn.execute("INSERT INTO paper_text_index (rowid, body) VALUES (?1, ?2)", params![rowid, text]).or_core()?;
    }
    Ok(())
}
