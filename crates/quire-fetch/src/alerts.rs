//! New papers for an alert: one search of arXiv, PubMed or Crossref, newest first, since a day.

use quire_model::{AlertSource, Date, ReferenceData};

use crate::error::Result;
use crate::{Fetcher, arxiv, crossref, pubmed};

impl Fetcher {
    /// Up to `limit` works `source` finds for `query` since `since`. arXiv cannot filter by date,
    /// so its newest results are filtered here.
    pub async fn new_papers(
        &self,
        source: AlertSource,
        query: &str,
        since: Date,
        limit: usize,
    ) -> Result<Vec<ReferenceData>> {
        let day = since.to_string();
        Ok(match source {
            AlertSource::Arxiv => arxiv::search(&self.http, &self.endpoints.arxiv, query, limit)
                .await?
                .into_iter()
                .filter(|data| data.issued.is_none_or(|issued| issued >= since))
                .collect(),
            AlertSource::Pubmed => {
                pubmed::search(&self.http, &self.endpoints.eutils, self.email.as_deref(), query, &day, limit).await?
            }
            AlertSource::Crossref => crossref::search(&self.http, &self.endpoints.crossref, query, &day, limit).await?,
        })
    }
}
