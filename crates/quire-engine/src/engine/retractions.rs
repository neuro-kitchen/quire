//! Retraction check: asks Crossref which notices update each reference with a DOI (Crossref
//! carries Retraction Watch's data), and records `retracted` / `expression-of-concern` issues,
//! which `ref check`, `list` and the TUI then show. A notice that is gone clears its issue.

use quire_core::check;
use quire_fetch::crossref::{Update, UpdateKind};
use quire_model::{Identifier, Issue, IssueCode, Reference, SlugPath};

use super::Engine;
use crate::error::Result;

/// One reference's notices, or why they could not be looked up.
#[derive(Debug, Clone)]
pub struct RetractionCheck {
    pub reference: Reference,
    pub updates: Vec<Update>,
    pub error: Option<String>,
}

impl Engine {
    /// Checks every reference with a DOI in `project` (and subprojects if `recursive`), or the
    /// whole library. Returns those with notices or a failed lookup.
    pub async fn check_retractions(&self, project: Option<&SlugPath>, recursive: bool) -> Result<Vec<RetractionCheck>> {
        let references = self.list(project, recursive, false).await?;
        let mut found = Vec::new();
        for reference in references {
            let Some(doi) = reference.data.identifiers.iter().find_map(|id| match id {
                Identifier::Doi(doi) => Some(doi.clone()),
                _ => None,
            }) else {
                continue;
            };
            let updates = match self.fetcher.updates(&doi).await {
                Ok(updates) => updates,
                Err(error) => {
                    found.push(RetractionCheck { reference, updates: Vec::new(), error: Some(error.to_string()) });
                    continue;
                }
            };
            let id = reference.id;
            let issues = issues_for(&updates);
            self.run(move |lib| {
                for code in [IssueCode::Retracted, IssueCode::Concern] {
                    check::clear(lib, id, code)?;
                }
                for issue in issues {
                    check::record(lib, id, issue)?;
                }
                Ok(())
            })
            .await?;
            if !updates.is_empty() {
                found.push(RetractionCheck { reference, updates, error: None });
            }
        }
        self.mark_changed();
        Ok(found)
    }
}

/// The issues a work's notices call for: one per kind, naming the latest notice.
fn issues_for(updates: &[Update]) -> Vec<Issue> {
    let describe = |update: &Update, what: &str| {
        let mut text = what.to_string();
        if let Some(date) = &update.date {
            text.push_str(&format!(" {date}"));
        }
        if let Some(notice) = &update.notice {
            text.push_str(&format!(", notice doi:{notice}"));
        }
        if update.source.as_deref() == Some("retraction-watch") {
            text.push_str(" (Retraction Watch)");
        }
        text
    };
    let latest =
        |pick: fn(&UpdateKind) -> bool| updates.iter().filter(|u| pick(&u.kind)).max_by_key(|u| u.date.clone());
    let mut issues = Vec::new();
    if let Some(update) = latest(|k| matches!(k, UpdateKind::Retracted(_))) {
        let UpdateKind::Retracted(kind) = &update.kind else { unreachable!() };
        issues.push(Issue::recorded(IssueCode::Retracted, describe(update, &kind.replace('_', " "))));
    }
    if let Some(update) = latest(|k| matches!(k, UpdateKind::Concern)) {
        issues.push(Issue::recorded(IssueCode::Concern, describe(update, "expression of concern")));
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retraction_becomes_one_issue() {
        let update = |kind, date: &str| Update {
            kind,
            notice: Some("10.1/notice".into()),
            date: Some(date.into()),
            source: Some("retraction-watch".into()),
        };
        let issues = issues_for(&[
            update(UpdateKind::Other("correction".into()), "2004-03-06"),
            update(UpdateKind::Retracted("retraction".into()), "2010-02-06"),
        ]);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, IssueCode::Retracted);
        assert_eq!(
            issues[0].detail.as_deref(),
            Some("retraction 2010-02-06, notice doi:10.1/notice (Retraction Watch)")
        );
        assert!(issues_for(&[update(UpdateKind::Other("correction".into()), "2004-03-06")]).is_empty());
    }
}
