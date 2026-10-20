//! Finding works: search (and saved searches), alerts for new work, the citation graph.

use std::str::FromStr;

use quire_engine::{Engine, GraphKind, SearchHit};
use quire_model::AlertSource;

use crate::records::ReferenceRecord;
use crate::references::{AddResult, ReferenceFields, add_result, data_of, fields_of};
use crate::{QuireError, QuireLibrary, Result, query};

/// A reference a search found, best first; `snippet` shows where (`in_text`: in the paper's text).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SearchHitRecord {
    pub reference: ReferenceRecord,
    pub score: f64,
    pub snippet: String,
    pub in_text: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SavedSearchRecord {
    pub name: String,
    pub query: String,
}

/// A search run against a source for new work: `source` `arxiv`, `pubmed` or `crossref`.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AlertRecord {
    pub name: String,
    pub source: String,
    pub query: String,
    /// When it was last checked (`YYYY-MM-DD`).
    pub checked: Option<String>,
}

/// What checking an alert found: works not in the library yet (add them with `add_found`).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AlertCheckRecord {
    pub alert: AlertRecord,
    pub found: Vec<ReferenceFields>,
    pub error: Option<String>,
}

/// A work next to a reference in the citation graph; `in_library` its reference ID if it is here.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GraphWorkRecord {
    pub id: String,
    pub doi: Option<String>,
    pub title: Option<String>,
    pub year: Option<i64>,
    pub first_author: Option<String>,
    pub cited_by: u64,
    pub in_library: Option<String>,
}

#[uniffi::export]
impl QuireLibrary {
    /// The references `query` finds (the engine's search syntax), best first.
    pub async fn search(&self, query: String, limit: u32) -> Result<Vec<SearchHitRecord>> {
        self.with(|engine| async move { hits(&engine, engine.search(&query, limit as usize).await?).await }).await
    }

    /// Saves `query` as `name` (replacing that search's query).
    pub async fn save_search(&self, name: String, query: String) -> Result<SavedSearchRecord> {
        self.with(|engine| async move { Ok(saved(engine.save_search(&name, &query).await?)) }).await
    }

    pub async fn saved_searches(&self) -> Result<Vec<SavedSearchRecord>> {
        self.with(|engine| async move { Ok(engine.saved_searches().await?.into_iter().map(saved).collect()) }).await
    }

    pub async fn run_saved(&self, name: String, limit: u32) -> Result<Vec<SearchHitRecord>> {
        self.with(|engine| async move { hits(&engine, engine.run_saved(&name, limit as usize).await?.1).await }).await
    }

    pub async fn rename_saved(&self, name: String, new_name: String) -> Result<SavedSearchRecord> {
        self.with(|engine| async move { Ok(saved(engine.rename_saved(&name, &new_name).await?)) }).await
    }

    pub async fn remove_saved(&self, name: String) -> Result<()> {
        self.with(|engine| async move {
            engine.remove_saved(&name).await?;
            Ok(())
        })
        .await
    }

    pub async fn add_alert(&self, name: String, source: String, query: String) -> Result<AlertRecord> {
        self.with(|engine| async move { Ok(alert(engine.add_alert(&name, alert_source(&source)?, &query).await?)) })
            .await
    }

    pub async fn alerts(&self) -> Result<Vec<AlertRecord>> {
        self.with(|engine| async move { Ok(engine.alerts().await?.into_iter().map(alert).collect()) }).await
    }

    pub async fn remove_alert(&self, name: String) -> Result<()> {
        self.with(|engine| async move {
            engine.remove_alert(&name).await?;
            Ok(())
        })
        .await
    }

    /// Checks the alert `name` (or every alert) for new work, `days` back if given.
    pub async fn check_alerts(&self, name: Option<String>, days: Option<i64>) -> Result<Vec<AlertCheckRecord>> {
        self.with(|engine| async move {
            Ok(engine
                .check_alerts(name.as_deref(), days)
                .await?
                .into_iter()
                .map(|check| AlertCheckRecord {
                    alert: alert(check.alert),
                    found: check.new.iter().map(fields_of).collect(),
                    error: check.error,
                })
                .collect())
        })
        .await
    }

    /// Adds a work an alert found, as the source described it (no second lookup).
    pub async fn add_found(
        &self,
        fields: ReferenceFields,
        source: String,
        project: Option<String>,
    ) -> Result<AddResult> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let added = engine.add_found(data_of(&fields)?, alert_source(&source)?, project.as_ref()).await?;
            add_result(&engine, added).await
        })
        .await
    }

    /// The works next to a reference: `kind` `references` (it cites), `citing` or `related`.
    pub async fn graph(&self, id: String, kind: String, limit: u32) -> Result<Vec<GraphWorkRecord>> {
        self.with(|engine| async move {
            let kind = match kind.as_str() {
                "references" => GraphKind::References,
                "citing" => GraphKind::Citing,
                "related" => GraphKind::Related,
                other => return Err(QuireError::Invalid(format!("`{other}`: references, citing or related"))),
            };
            Ok(engine
                .graph(&id, kind, limit as usize)
                .await?
                .1
                .into_iter()
                .map(|entry| GraphWorkRecord {
                    id: entry.work.id,
                    doi: entry.work.doi,
                    title: entry.work.title,
                    year: entry.work.year,
                    first_author: entry.work.first_author,
                    cited_by: entry.work.cited_by,
                    in_library: entry.in_library.map(|r| r.id.to_string()),
                })
                .collect())
        })
        .await
    }
}

async fn hits(engine: &Engine, found: Vec<(quire_model::Reference, SearchHit)>) -> Result<Vec<SearchHitRecord>> {
    let mut records = Vec::new();
    for (reference, hit) in found {
        records.push(SearchHitRecord {
            reference: query::reference(engine, &reference.id.to_string()).await?,
            score: hit.score,
            snippet: hit.snippet,
            in_text: hit.in_text,
        });
    }
    Ok(records)
}

fn saved(search: quire_model::SavedSearch) -> SavedSearchRecord {
    SavedSearchRecord { name: search.name, query: search.query }
}

fn alert(alert: quire_model::Alert) -> AlertRecord {
    AlertRecord {
        name: alert.name,
        source: alert.source.as_str().to_string(),
        query: alert.query,
        checked: alert.checked_at.map(|t| t.date().to_string()),
    }
}

fn alert_source(text: &str) -> Result<AlertSource> {
    AlertSource::from_str(text).map_err(|_| QuireError::Invalid(format!("`{text}`: arxiv, pubmed or crossref")))
}
