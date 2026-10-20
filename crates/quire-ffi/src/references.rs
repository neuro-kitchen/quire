//! Adding and changing references: by identifier or URL, from a page, typed in; their fields,
//! trash, refetch, issues and failed lookups. Fields cross as [`ReferenceFields`] (CSL names,
//! dates as `YYYY[-MM[-DD]]`); the engine decides what changes.

use std::str::FromStr;

use quire_engine::Engine;
use quire_engine::{Added, FieldChange, Issue, Updated};
use quire_model::{Creator, CreatorRole, Date, EntryKind, Identifier, Name, ReferenceData, TextField};

use crate::records::ReferenceRecord;
use crate::{QuireError, QuireLibrary, Result, query};

/// A reference's editable fields.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ReferenceFields {
    /// CSL type: `article-journal`, `book`, …
    pub kind: String,
    pub title: String,
    pub creators: Vec<CreatorInput>,
    /// `YYYY`, `YYYY-MM` or `YYYY-MM-DD`.
    pub issued: Option<String>,
    pub container_title: Option<String>,
    pub publisher: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub abstract_text: Option<String>,
    /// Identifiers as typed or labelled: `10.1038/…`, `doi:…`, `arXiv:…`, `pmid:…`.
    pub identifiers: Vec<String>,
}

/// A creator: a person (`family`, `given`) or, with `literal`, an organisation named `family`.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct CreatorInput {
    /// CSL role: `author`, `editor`, …
    pub role: String,
    pub family: String,
    pub given: Option<String>,
    pub literal: bool,
}

/// What adding did: `new` false when the work was already in the library (then only linked).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AddResult {
    pub new: bool,
    pub reference: ReferenceRecord,
}

/// A field an edit or a refetch changed (or would change); `kept` when the user's value stays.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FieldChangeRecord {
    pub field: String,
    pub old: Option<String>,
    pub new: Option<String>,
    pub kept: bool,
}

/// A problem with a reference (a missing field, a failed lookup, a retraction…).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct IssueRecord {
    pub code: String,
    pub message: String,
}

/// An input whose lookup failed, to retry or forget.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FailureRecord {
    pub input: String,
    pub error: String,
    pub attempts: u32,
    /// The project it was to go into.
    pub project: Option<String>,
}

#[uniffi::export]
impl QuireLibrary {
    /// What `input` (DOI, arXiv ID, PMID, PMCID, URL) names, looked up but not added: a preview.
    pub async fn lookup(&self, input: String) -> Result<ReferenceFields> {
        self.with(|engine| async move { Ok(fields_of(&engine.lookup(&input).await?)) }).await
    }

    /// Adds what `input` names, into `project` (a path) if given.
    pub async fn add(&self, input: String, project: Option<String>) -> Result<AddResult> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let added = engine.add(&input, project.as_ref()).await?;
            add_result(&engine, added).await
        })
        .await
    }

    /// Adds a reference typed in by hand.
    pub async fn add_manual(&self, fields: ReferenceFields, project: Option<String>) -> Result<AddResult> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let added = engine.add_manual(data_of(&fields)?, project.as_ref()).await?;
            add_result(&engine, added).await
        })
        .await
    }

    /// Adds the work the page at `url` describes, from its `html` as the browser loaded it.
    pub async fn add_from_page(&self, url: String, html: String, project: Option<String>) -> Result<AddResult> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let added = engine.add_from_page(&url, &html, project.as_ref()).await?;
            add_result(&engine, added).await
        })
        .await
    }

    /// Imports `.bib` text (pasted), into `project` if given; returns how many references were new.
    pub async fn import_bib_text(&self, text: String, project: Option<String>) -> Result<u32> {
        self.with(|engine| async move {
            let project = project.as_deref().map(query::slug_path).transpose()?;
            let added = engine.import_bib_text(&text, "pasted", project.as_ref()).await?;
            Ok(added.iter().filter(|a| matches!(a, Added::New(_))).count() as u32)
        })
        .await
    }

    /// The editable fields of a reference (ID, cite key or identifier).
    pub async fn fields(&self, id: String) -> Result<ReferenceFields> {
        self.with(|engine| async move { Ok(fields_of(&engine.show(&id).await?.0.data)) }).await
    }

    /// Replaces a reference's fields (and its cite key, if given); with `dry_run`, only reports.
    pub async fn edit(
        &self,
        id: String,
        fields: ReferenceFields,
        cite_key: Option<String>,
        dry_run: bool,
    ) -> Result<Vec<FieldChangeRecord>> {
        self.with(|engine| async move {
            let cite_key = cite_key
                .map(|k| k.parse().map_err(|_| QuireError::Invalid(format!("`{k}` is not a cite key"))))
                .transpose()?;
            Ok(changes(engine.edit_fields(&id, data_of(&fields)?, cite_key, dry_run).await?))
        })
        .await
    }

    /// Looks a reference up again at its source; fields the user edited are kept.
    pub async fn refetch(&self, id: String, dry_run: bool) -> Result<Vec<FieldChangeRecord>> {
        self.with(|engine| async move { Ok(changes(engine.refetch(&id, dry_run).await?)) }).await
    }

    pub async fn trash(&self, id: String) -> Result<()> {
        self.with(|engine| async move {
            engine.trash(&id).await?;
            Ok(())
        })
        .await
    }

    pub async fn restore(&self, id: String) -> Result<()> {
        self.with(|engine| async move {
            engine.restore(&id).await?;
            Ok(())
        })
        .await
    }

    /// A reference's recorded issues and lints.
    pub async fn issues(&self, id: String) -> Result<Vec<IssueRecord>> {
        self.with(|engine| async move {
            let (reference, _) = engine.show(&id).await?;
            Ok(engine.issues(&reference).await?.iter().map(issue).collect())
        })
        .await
    }

    /// Inputs whose lookup failed.
    pub async fn failures(&self) -> Result<Vec<FailureRecord>> {
        self.with(|engine| async move {
            Ok(engine
                .failures()
                .await?
                .into_iter()
                .map(|(failed, project)| FailureRecord {
                    input: failed.input,
                    error: failed.error,
                    attempts: failed.attempts,
                    project: project.map(|p| p.to_string()),
                })
                .collect())
        })
        .await
    }

    /// Forgets a failed input; `false` if it was not recorded.
    pub async fn forget_failure(&self, input: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.forget_failure(&input).await?) }).await
    }
}

pub(crate) async fn add_result(engine: &Engine, added: Added) -> Result<AddResult> {
    let new = matches!(added, Added::New(_));
    let reference = query::reference(engine, &added.reference().id.to_string()).await?;
    Ok(AddResult { new, reference })
}

pub(crate) fn issue(issue: &Issue) -> IssueRecord {
    IssueRecord { code: issue.code.as_str().to_string(), message: issue.message() }
}

fn changes(updated: Updated) -> Vec<FieldChangeRecord> {
    updated
        .changes
        .into_iter()
        .map(|FieldChange { field, old, new, outcome }| FieldChangeRecord {
            field: field.to_string(),
            old,
            new,
            kept: outcome == quire_engine::Outcome::KeptUserValue,
        })
        .collect()
}

pub(crate) fn fields_of(data: &ReferenceData) -> ReferenceFields {
    let text = |field: TextField| data.get(field).map(str::to_string);
    ReferenceFields {
        kind: data.kind.as_str().to_string(),
        title: data.title().unwrap_or_default().to_string(),
        creators: data
            .creators
            .iter()
            .map(|c| match &c.name {
                Name::Person(p) => CreatorInput {
                    role: c.role.as_str().to_string(),
                    family: p.family.clone(),
                    given: p.given.clone(),
                    literal: false,
                },
                Name::Literal { literal } => CreatorInput {
                    role: c.role.as_str().to_string(),
                    family: literal.clone(),
                    given: None,
                    literal: true,
                },
            })
            .collect(),
        issued: data.issued.map(|d| d.to_string()),
        container_title: text(TextField::ContainerTitle),
        publisher: text(TextField::Publisher),
        volume: text(TextField::Volume),
        issue: text(TextField::Issue),
        pages: text(TextField::Page),
        url: text(TextField::Url),
        abstract_text: text(TextField::Abstract),
        identifiers: data.identifiers.iter().map(ToString::to_string).collect(),
    }
}

pub(crate) fn data_of(fields: &ReferenceFields) -> Result<ReferenceData> {
    let invalid = |what: &str, value: &str| QuireError::Invalid(format!("`{value}` is not a valid {what}"));
    let mut data = ReferenceData::new(EntryKind::from_str(&fields.kind).map_err(|_| invalid("type", &fields.kind))?);
    data.set(TextField::Title, fields.title.trim());
    for creator in &fields.creators {
        let role = CreatorRole::from_str(&creator.role).map_err(|_| invalid("role", &creator.role))?;
        let family = creator.family.trim();
        if family.is_empty() {
            continue;
        }
        data.creators.push(if creator.literal {
            Creator::literal(role, family)
        } else {
            Creator::person(role, family, creator.given.clone().filter(|g| !g.trim().is_empty()))
        });
    }
    data.issued = fields
        .issued
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(|d| Date::from_str(d).map_err(|_| invalid("date", d)))
        .transpose()?;
    for (field, value) in [
        (TextField::ContainerTitle, &fields.container_title),
        (TextField::Publisher, &fields.publisher),
        (TextField::Volume, &fields.volume),
        (TextField::Issue, &fields.issue),
        (TextField::Page, &fields.pages),
        (TextField::Url, &fields.url),
        (TextField::Abstract, &fields.abstract_text),
    ] {
        if let Some(value) = value.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            data.set(field, value);
        }
    }
    for text in fields.identifiers.iter().map(|i| i.trim()).filter(|i| !i.is_empty()) {
        data.identifiers.push(Identifier::parse(text).map_err(|_| invalid("identifier", text))?);
    }
    Ok(data)
}
