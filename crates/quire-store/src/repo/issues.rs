//! Queries on `reference_issue` and `failed_input`.

use quire_core::ports::IssueStore;
use quire_core::{FailedInput, Result};
use quire_model::{Issue, ProjectId, ReferenceId, Timestamp};
use rusqlite::{Connection, params};

use super::SqliteLibrary;
use super::codec::{id_blob, optional_id_at, parsed_at};
use super::core::all;
use crate::error::OrCore;

impl IssueStore for SqliteLibrary<'_> {
    fn recorded_issues(&self, reference: ReferenceId) -> Result<Vec<Issue>> {
        recorded(self.conn, reference)
    }

    fn set_recorded_issues(&mut self, reference: ReferenceId, issues: &[Issue]) -> Result<()> {
        set_recorded(self.conn, reference, issues)
    }

    fn record_failure(&mut self, input: &str, project: Option<ProjectId>, error: &str) -> Result<()> {
        record_failure(self.conn, input, project, error)
    }

    fn clear_failure(&mut self, input: &str) -> Result<bool> {
        clear_failure(self.conn, input)
    }

    fn failures(&self) -> Result<Vec<FailedInput>> {
        failures(self.conn)
    }
}

fn recorded(conn: &Connection, reference: ReferenceId) -> Result<Vec<Issue>> {
    let sql = "SELECT code, detail, recorded_at FROM reference_issue WHERE reference_id = ?1 ORDER BY code";
    all(conn, sql, [id_blob(reference)], |row| {
        Ok(Issue {
            code: parsed_at(row, "code")?,
            detail: row.get("detail")?,
            at: Some(Timestamp::from_millis(row.get("recorded_at")?)),
        })
    })
}

fn set_recorded(conn: &Connection, reference: ReferenceId, issues: &[Issue]) -> Result<()> {
    conn.execute("DELETE FROM reference_issue WHERE reference_id = ?1", [id_blob(reference)]).or_core()?;
    let sql =
        "INSERT OR REPLACE INTO reference_issue (reference_id, code, detail, recorded_at) VALUES (?1, ?2, ?3, ?4)";
    for issue in issues {
        let at = issue.at.unwrap_or_else(Timestamp::now).as_millis();
        conn.execute(sql, params![id_blob(reference), issue.code.as_str(), issue.detail, at]).or_core()?;
    }
    Ok(())
}

fn record_failure(conn: &Connection, input: &str, project: Option<ProjectId>, error: &str) -> Result<()> {
    let now = Timestamp::now().as_millis();
    let sql = "INSERT INTO failed_input (input, project_id, error, attempts, first_at, last_at) VALUES (?1, ?2, ?3, 1, ?4, ?4)
               ON CONFLICT (input) DO UPDATE SET project_id = ?2, error = ?3, attempts = attempts + 1, last_at = ?4";
    conn.execute(sql, params![input, project.map(id_blob), error, now]).or_core()?;
    Ok(())
}

fn clear_failure(conn: &Connection, input: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM failed_input WHERE input = ?1", [input]).or_core()? > 0)
}

fn failures(conn: &Connection) -> Result<Vec<FailedInput>> {
    let sql = "SELECT input, project_id, error, attempts, first_at, last_at FROM failed_input ORDER BY first_at, input";
    all(conn, sql, [], |row| {
        Ok(FailedInput {
            input: row.get("input")?,
            project: optional_id_at(row, "project_id")?,
            error: row.get("error")?,
            attempts: row.get("attempts")?,
            first_at: Timestamp::from_millis(row.get("first_at")?),
            last_at: Timestamp::from_millis(row.get("last_at")?),
        })
    })
}
