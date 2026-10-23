//! Answering a [`Query`] with engine calls: the source's references, narrowed by tags and search
//! words, then each one's tags, projects, PDF and reading status, sorted.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use quire_engine::{Engine, ReadingStatus, Reference, SlugPath};
use quire_model::ReferenceId;

use crate::records::{ProjectRecord, Query, ReferenceRecord, SortColumn, Source, tag_names};
use crate::{QuireError, Result};

/// Search results considered; more than any screen shows.
const SEARCH_LIMIT: usize = 10_000;

pub(crate) async fn references(engine: &Engine, query: &Query) -> Result<Vec<ReferenceRecord>> {
    let memberships = memberships(engine).await?;
    let mut found = match &query.source {
        Source::All | Source::Unfiled => engine.list_tagged(None, false, false, &query.tags).await?,
        Source::Project { path } => engine.list_tagged(Some(&slug_path(path)?), false, false, &query.tags).await?,
        Source::Trash => engine.list_tagged(None, false, true, &query.tags).await?,
    };
    if query.source == Source::Unfiled {
        found.retain(|r| !memberships.contains_key(&r.id));
    }
    if !query.text.trim().is_empty() {
        let hits: HashSet<ReferenceId> =
            engine.search(&query.text, SEARCH_LIMIT).await?.into_iter().map(|(r, _)| r.id).collect();
        found.retain(|r| hits.contains(&r.id));
    }
    let mut records = records(engine, &found, &memberships).await?;
    sort(&mut records, query.sort, query.ascending);
    Ok(records)
}

pub(crate) async fn reference(engine: &Engine, id: &str) -> Result<ReferenceRecord> {
    let (reference, paths) = engine.show(id).await?;
    let tags = engine.tags_of(vec![reference.id]).await?;
    let has_pdf = engine.references_with_pdfs().await?.contains(&reference.id);
    let reading = engine.reading_statuses().await?.get(&reference.id).map_or(ReadingStatus::Unread, |r| r.status);
    Ok(ReferenceRecord::new(
        &reference,
        tags.get(&reference.id).map(|t| tag_names(t)).unwrap_or_default(),
        paths.iter().map(ToString::to_string).collect(),
        has_pdf,
        reading,
    ))
}

pub(crate) async fn projects(engine: &Engine) -> Result<Vec<ProjectRecord>> {
    let mut summaries = engine.project_summaries().await?;
    summaries.sort_by_key(|s| s.path.to_string());
    Ok(summaries
        .into_iter()
        .map(|s| ProjectRecord {
            path: s.path.to_string(),
            name: s.project.name,
            parent: s.path.parent().map(|p| p.to_string()),
            references: s.references as u64,
        })
        .collect())
}

/// The projects each reference is in (references in none are left out), from one store query.
async fn memberships(engine: &Engine) -> Result<HashMap<ReferenceId, Vec<String>>> {
    Ok(engine
        .memberships()
        .await?
        .into_iter()
        .map(|(id, paths)| {
            let mut paths: Vec<String> = paths.iter().map(ToString::to_string).collect();
            paths.sort();
            (id, paths)
        })
        .collect())
}

async fn records(
    engine: &Engine,
    references: &[Reference],
    memberships: &HashMap<ReferenceId, Vec<String>>,
) -> Result<Vec<ReferenceRecord>> {
    let tags = engine.tags_of(references.iter().map(|r| r.id).collect()).await?;
    let pdfs = engine.references_with_pdfs().await?;
    let readings = engine.reading_statuses().await?;
    Ok(references
        .iter()
        .map(|r| {
            ReferenceRecord::new(
                r,
                tags.get(&r.id).map(|t| tag_names(t)).unwrap_or_default(),
                memberships.get(&r.id).cloned().unwrap_or_default(),
                pdfs.contains(&r.id),
                readings.get(&r.id).map_or(ReadingStatus::Unread, |reading| reading.status),
            )
        })
        .collect())
}

fn sort(records: &mut [ReferenceRecord], column: SortColumn, ascending: bool) {
    let key = |r: &ReferenceRecord| r.title.to_lowercase();
    records.sort_by(|a, b| {
        let order = match column {
            SortColumn::Title => key(a).cmp(&key(b)),
            SortColumn::Creator => first_creator(a).cmp(&first_creator(b)),
            // Undated last when ascending, like Zotero.
            SortColumn::Year => match (a.year, b.year) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            },
            SortColumn::DateAdded => a.date_added.cmp(&b.date_added),
        };
        let order = order.then_with(|| key(a).cmp(&key(b)));
        if ascending { order } else { order.reverse() }
    });
}

fn first_creator(record: &ReferenceRecord) -> String {
    record.creators.first().map(|c| c.sort_name.to_lowercase()).unwrap_or_default()
}

pub(crate) fn slug_path(path: &str) -> Result<SlugPath> {
    path.parse().map_err(|e| QuireError::Invalid(format!("project {path}: {e}")))
}
