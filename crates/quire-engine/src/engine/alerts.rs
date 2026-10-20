//! Alerts through the engine: searches of arXiv, PubMed or Crossref checked for papers published
//! since the last check (the first check looks back [`FIRST_CHECK_DAYS`]).

use quire_core::library::{alerts, references};
use quire_core::prelude::*;
use quire_core::{AddRequest, Added};
use quire_model::{Alert, AlertSource, Origin, ProjectId, ReferenceData, SlugPath, Timestamp};

use super::Engine;
use crate::error::Result;

/// How far back an alert's first check looks.
pub(super) const FIRST_CHECK_DAYS: i64 = 30;
/// Works asked for per alert and check.
pub(super) const PER_CHECK: usize = 50;

/// One alert's check: the new works, or why the source could not be asked.
#[derive(Debug, Clone)]
pub struct AlertCheck {
    pub alert: Alert,
    pub new: Vec<ReferenceData>,
    pub error: Option<String>,
}

impl Engine {
    pub async fn add_alert(&self, name: &str, source: AlertSource, query: &str) -> Result<Alert> {
        let (name, query) = (name.to_string(), query.to_string());
        self.run(move |lib| alerts::create(lib, &name, source, &query)).await
    }

    pub async fn alerts(&self) -> Result<Vec<Alert>> {
        self.run(|lib| lib.alerts()).await
    }

    pub async fn remove_alert(&self, name: &str) -> Result<Alert> {
        let name = name.to_string();
        self.run(move |lib| alerts::remove(lib, &name)).await
    }

    /// Checks the alert `name`, or every alert. `days` overrides how far back to look.
    pub async fn check_alerts(&self, name: Option<&str>, days: Option<i64>) -> Result<Vec<AlertCheck>> {
        let chosen = match name {
            Some(name) => {
                let name = name.to_string();
                vec![self.run(move |lib| alerts::find(lib, &name)).await?]
            }
            None => self.alerts().await?,
        };
        let mut checks = Vec::with_capacity(chosen.len());
        for alert in chosen {
            let now = Timestamp::now();
            let since = match (days, alert.checked_at) {
                (Some(days), _) => now.days_before(days),
                (None, Some(last)) => last,
                (None, None) => now.days_before(FIRST_CHECK_DAYS),
            };
            match self.fetcher.new_papers(alert.source, &alert.query, since.date(), PER_CHECK).await {
                Ok(found) => {
                    let checked = alert.clone();
                    let new = self.run(move |lib| alerts::record_check(lib, &checked, found, now)).await?;
                    checks.push(AlertCheck { alert, new, error: None });
                }
                Err(error) => checks.push(AlertCheck { alert, new: Vec::new(), error: Some(error.to_string()) }),
            }
        }
        Ok(checks)
    }

    /// Adds a work an alert found, as the source described it (no second lookup).
    pub async fn add_found(
        &self,
        data: ReferenceData,
        source: AlertSource,
        project: Option<&SlugPath>,
    ) -> Result<Added> {
        let project: Option<ProjectId> = self.project_id(project).await?;
        let origin = Origin::Fetched { source: source.as_str().into() };
        let added =
            self.run(move |lib| references::add(lib, AddRequest { project, ..AddRequest::new(data, origin) })).await?;
        self.mark_changed();
        Ok(added)
    }
}
