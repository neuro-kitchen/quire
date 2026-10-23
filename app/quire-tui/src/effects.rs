//! Runs [`Effect`]s on the engine in background tasks; each answers with an [`Action`] on the
//! channel, so the interface never waits for the database or the network.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use quire_engine::{Added, Engine, ExportFormat, ExportRequest, IssueCode, Outcome, PdfOutcome};
use quire_model::ReferenceId;
use tokio::sync::mpsc::UnboundedSender;

use crate::reader::layout::{MAX_WIDTH, MIN_WIDTH};
use crate::reader::{Settings, Theme};

use crate::state::{Details, Row, Scope};
use crate::update::{Action, Effect};

pub fn spawn(engine: &Arc<Engine>, effect: Effect, send: &UnboundedSender<Action>) {
    let (engine, send) = (engine.clone(), send.clone());
    tokio::spawn(async move {
        // A job: its result, then the news that it ended (the jobs panel drops it).
        let (effect, job) = match effect {
            Effect::Job { id, effect } => (*effect, Some(id)),
            other => (other, None),
        };
        let action =
            run(&engine, effect).await.unwrap_or_else(|error| Action::Done { text: format!("{error:#}"), ok: false });
        // The interface may have quit meanwhile.
        let _ = send.send(action);
        if let Some(id) = job {
            let _ = send.send(Action::JobFinished(id));
        }
    });
}

async fn run(engine: &Engine, effect: Effect) -> anyhow::Result<Action> {
    Ok(match effect {
        Effect::LoadSidebar => {
            let projects = engine.project_summaries().await?;
            let colored = engine.tags().await?.into_iter().map(|(t, _)| t).filter(|t| t.position.is_some()).collect();
            let duplicate_pairs = engine.duplicates().await?.iter().map(|g| g.duplicates.len()).sum();
            let saved = engine.saved_searches().await?;
            Action::Loaded { projects, colored, duplicate_pairs, saved }
        }
        Effect::LoadRows(scope) => Action::Rows(rows(engine, &scope).await?),
        Effect::LoadDetails(id) => {
            let query = id.to_string();
            let (_, projects) = engine.show(&query).await?;
            let notes = engine.notes(Some(&query)).await?;
            let related = engine.related(&query).await?;
            Action::DetailsLoaded(Details { id: Some(id), projects, notes, related })
        }
        Effect::Add { input, project } => match engine.add(&input, project.as_ref()).await? {
            Added::New(r) => Action::Done { text: format!("added {}", r.cite_key), ok: true },
            Added::Existing(r) => Action::Done { text: format!("already in the library: {}", r.cite_key), ok: true },
        },
        Effect::ToggleTag { reference, tag } => {
            let query = reference.to_string();
            let current = engine.tags_of(vec![reference]).await?.remove(&reference).unwrap_or_default();
            let has = current.iter().any(|(t, _)| t.name.to_lowercase() == tag.trim().to_lowercase());
            if has {
                engine.untag(&query, std::slice::from_ref(&tag)).await?;
                Action::Done { text: format!("removed tag {tag}"), ok: true }
            } else {
                engine.tag(&query, std::slice::from_ref(&tag)).await?;
                Action::Done { text: format!("tagged {tag}"), ok: true }
            }
        }
        Effect::Merge { keep, other } => {
            let merged = engine.merge(&keep.to_string(), &other).await?;
            Action::Done { text: format!("merged {} into {}", merged.removed.cite_key, merged.kept.cite_key), ok: true }
        }
        Effect::Distinct { a, b } => {
            engine.dismiss_duplicate(&a.to_string(), &b).await?;
            Action::Done { text: format!("marked as not duplicates of {b}"), ok: true }
        }
        Effect::Trash(id) => {
            let reference = engine.trash(&id.to_string()).await?;
            Action::Done { text: format!("moved {} to the trash", reference.cite_key), ok: true }
        }
        Effect::OpenPdf(id) => {
            let (reference, pdfs) = engine.pdfs_of(&id.to_string()).await?;
            let Some((_, path)) = pdfs.first() else {
                return Ok(Action::Done { text: format!("{} has no PDF (p gets one)", reference.cite_key), ok: false });
            };
            quire_engine::tools::open(&path.display().to_string())?;
            Action::Done { text: format!("opened {}", reference.cite_key), ok: true }
        }
        Effect::GetPdf(id) => match engine.get_pdf(&id.to_string()).await? {
            PdfOutcome::Present(reference, _) => {
                Action::Done { text: format!("{} already has a PDF", reference.cite_key), ok: true }
            }
            PdfOutcome::Downloaded { attached, source, .. } => {
                Action::Done { text: format!("PDF for {} from {source}", attached.reference.cite_key), ok: true }
            }
        },
        Effect::ExtractText(id) => {
            let query = id.to_string();
            engine.forget_text(&query).await?;
            let (reference, text) = engine.reader_text(&query).await?;
            let words = text.markdown.split_whitespace().count();
            Action::Done {
                text: format!("re-extracted {}: {} text, {words} words", reference.cite_key, text.source.label()),
                ok: true,
            }
        }
        Effect::RefreshText(id) => {
            let query = id.to_string();
            let text = match engine.forget_text(&query).await {
                Ok(_) => engine.reader_text(&query).await.map(|(_, text)| text),
                Err(error) => Err(error),
            };
            Action::TextLoaded { id, text: text.map_err(|e| format!("{e:#}")) }
        }
        Effect::LoadAnnotations(id) => annotations(engine, id).await?,
        Effect::LoadFigure(url) => {
            let result = engine.figure(&url).await.map_err(|e| e.to_string());
            Action::FigureLoaded { url, result }
        }
        Effect::Refetch(id) => {
            let updated = engine.refetch(&id.to_string(), false).await?;
            let changed = updated.changes.iter().filter(|c| c.outcome == Outcome::Updated).count();
            Action::Done {
                text: format!("refetched {}: {changed} field(s) updated", updated.reference.cite_key),
                ok: true,
            }
        }
        Effect::Link { reference, path } => {
            let (reference, linked) = engine.link(&reference.to_string(), &path).await?;
            let text = if linked { "linked" } else { "already in" };
            Action::Done { text: format!("{text} {path}: {}", reference.cite_key), ok: true }
        }
        Effect::Unlink { reference, path } => {
            let (reference, unlinked) = engine.unlink(&reference.to_string(), &path).await?;
            let text = if unlinked { "took out of" } else { "was not in" };
            Action::Done { text: format!("{text} {path}: {}", reference.cite_key), ok: true }
        }
        Effect::Export { scope, path } => export(engine, &scope, &path).await?,
        Effect::SetReading { reference, status } => {
            let (reference, _, _) = engine.set_reading(&reference.to_string(), status).await?;
            Action::Done { text: format!("{} {status}", reference.cite_key), ok: true }
        }
        Effect::Opened(reference) => {
            engine.opened(reference).await?;
            Action::Saved
        }
        Effect::SaveSearch { name, query } => {
            let search = engine.save_search(&name, &query).await?;
            Action::Done { text: format!("saved search `{}`", search.name), ok: true }
        }
        Effect::Job { .. } | Effect::Edit(_) | Effect::Clipboard { .. } | Effect::DetectGraphics => {
            unreachable!(
                "jobs are unwrapped in `spawn`; edits, the clipboard and the graphics query run in the event loop"
            )
        }
        Effect::AddHighlight { reference, quote, prefix, comment } => {
            engine.add_highlight(&reference.to_string(), &quote, &prefix, comment.as_deref()).await?;
            annotations(engine, reference).await?
        }
        Effect::RemoveHighlight { reference, highlight } => {
            engine.remove_highlight(highlight).await?;
            annotations(engine, reference).await?
        }
        Effect::AddNote { reference, body } => {
            engine.add_note(Some(&reference.to_string()), &body).await?;
            annotations(engine, reference).await?
        }
        Effect::LoadText(id) => {
            let text = engine.reader_text(&id.to_string()).await.map(|(_, text)| text);
            Action::TextLoaded { id, text: text.map_err(|e| format!("{e:#}")) }
        }
        Effect::SaveReaderSettings(settings) => {
            let file = engine.config_file();
            for (key, value) in [
                ("reader.width", settings.width.to_string()),
                ("reader.justify", settings.justify.to_string()),
                ("reader.spacing", settings.spacing.to_string()),
                ("reader.theme", settings.theme.as_str().to_string()),
            ] {
                quire_engine::config::set(file, key, &value)?;
            }
            Action::Saved
        }
        Effect::SavePositions(positions) => {
            save_positions(&engine.library_dir().join(POSITIONS), &positions)?;
            Action::Saved
        }
    })
}

/// Writes the references of `scope` to `path` in the format its extension names.
async fn export(engine: &Engine, scope: &Scope, path: &str) -> anyhow::Result<Action> {
    let format = ExportFormat::from_path(Path::new(path))
        .ok_or_else(|| anyhow::anyhow!("`{path}`: use .bib (BibLaTeX), .yml (Hayagriva) or .json (CSL-JSON)"))?;
    let project = match scope {
        Scope::Project(path) => Some(path.clone()),
        _ => None,
    };
    let request = ExportRequest { project, recursive: true, format: Some(format), ..ExportRequest::default() };
    let (text, count) = engine.export(&request).await?;
    let target = quire_engine::paths::expand_home(Path::new(path));
    std::fs::write(&target, text).map_err(|e| anyhow::anyhow!("{}: {e}", target.display()))?;
    Ok(Action::Done { text: format!("exported {count} reference(s) to {}", target.display()), ok: true })
}

/// A paper's highlights and notes, for the reader.
async fn annotations(engine: &Engine, id: ReferenceId) -> anyhow::Result<Action> {
    let query = id.to_string();
    let (_, highlights) = engine.highlights(&query).await?;
    let notes = engine.notes(Some(&query)).await?;
    Ok(Action::Annotations { id, highlights, notes })
}

/// Where each paper was left in the reader: `<reference>\t<block>` lines in the library folder.
pub(crate) const POSITIONS: &str = "reader-positions.tsv";

pub(crate) fn load_positions(path: &Path) -> HashMap<ReferenceId, usize> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    text.lines()
        .filter_map(|line| {
            let (id, block) = line.split_once('\t')?;
            Some((id.parse().ok()?, block.parse().ok()?))
        })
        .collect()
}

fn save_positions(path: &Path, positions: &[(ReferenceId, usize)]) -> anyhow::Result<()> {
    let text: String = positions.iter().map(|(id, block)| format!("{id}\t{block}\n")).collect();
    std::fs::write(path, text)?;
    quire_engine::private::file(path)?;
    Ok(())
}

/// The reader settings in the config, defaults for what is unset or invalid.
pub(crate) fn reader_settings(config: &quire_engine::Config) -> Settings {
    let saved = &config.reader;
    let default = Settings::default();
    Settings {
        width: saved.width.unwrap_or(default.width).clamp(MIN_WIDTH, MAX_WIDTH),
        justify: saved.justify.unwrap_or(default.justify),
        spacing: saved
            .spacing
            .as_deref()
            .and_then(|s| s.parse().ok())
            .filter(|s| matches!(s, 1 | 2))
            .unwrap_or(default.spacing),
        theme: saved.theme.as_deref().and_then(Theme::parse).unwrap_or(default.theme),
    }
}

/// The rows of `scope`: references with issues, PDF status and tags.
async fn rows(engine: &Engine, scope: &Scope) -> anyhow::Result<Vec<Row>> {
    if let Scope::Search(query) = scope {
        return search_rows(engine, query).await;
    }
    if let Scope::Saved(name) = scope {
        let query = engine.saved_searches().await?.into_iter().find(|s| &s.name == name).map(|s| s.query);
        return match query {
            Some(query) => search_rows(engine, &query).await,
            None => Ok(Vec::new()),
        };
    }
    let checked = match scope {
        Scope::All => engine.list_with_issues(None, false, false, &[]).await?,
        Scope::Project(path) => engine.list_with_issues(Some(path), false, false, &[]).await?,
        Scope::Trash => engine.list_with_issues(None, false, true, &[]).await?,
        Scope::Search(_) | Scope::Saved(_) => unreachable!("answered above"),
        Scope::Duplicates => {
            let all = engine.list_with_issues(None, false, false, &[]).await?;
            all.into_iter().filter(|c| c.issues.iter().any(|i| i.code == IssueCode::PossibleDuplicate)).collect()
        }
    };
    let pdfs = engine.pdf_statuses().await?;
    let readings = engine.reading_statuses().await?;
    let mut tags = engine.tags_of(checked.iter().map(|c| c.reference.id).collect()).await?;
    Ok(checked
        .into_iter()
        .map(|c| Row {
            pdf: pdfs.get(&c.reference.id).copied(),
            tags: tags.remove(&c.reference.id).unwrap_or_default(),
            reading: readings.get(&c.reference.id).map(|r| r.status),
            reference: c.reference,
            issues: c.issues,
            hit: None,
        })
        .collect())
}

/// Full-text search results as rows, best first, each with its matching words.
async fn search_rows(engine: &Engine, query: &str) -> anyhow::Result<Vec<Row>> {
    let hits = engine.search(query, SEARCH_ROWS).await?;
    let pdfs = engine.pdf_statuses().await?;
    let readings = engine.reading_statuses().await?;
    let mut tags = engine.tags_of(hits.iter().map(|(r, _)| r.id).collect()).await?;
    let mut rows = Vec::with_capacity(hits.len());
    for (reference, hit) in hits {
        rows.push(Row {
            pdf: pdfs.get(&reference.id).copied(),
            tags: tags.remove(&reference.id).unwrap_or_default(),
            issues: engine.issues(&reference).await?,
            hit: Some((hit.snippet, hit.in_text)),
            reading: readings.get(&reference.id).map(|r| r.status),
            reference,
        });
    }
    Ok(rows)
}

/// Search results shown at most.
const SEARCH_ROWS: usize = 200;
