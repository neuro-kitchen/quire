//! Checking references: lints computed from the fields (no title, no date, title in capitals, …)
//! plus the issues recorded when they were fetched (`.knowledge/roadmap.md`, M3), and possible
//! duplicates ([`crate::library::duplicates`]).

use quire_model::{EntryKind, Issue, IssueCode, Reference, ReferenceData, ReferenceId, TextField};

use std::collections::{HashMap, HashSet};

use crate::error::Result;
use crate::library::duplicates;
use crate::ports::{Library, ReferenceFilter};

/// A reference and everything wrong with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    pub reference: Reference,
    /// Recorded issues first, then lints; worst first within each.
    pub issues: Vec<Issue>,
}

/// Issues found in `data`'s fields.
pub(crate) fn lint(data: &ReferenceData) -> Vec<Issue> {
    let mut found = Vec::new();
    let mut flag = |code| found.push(Issue::new(code));
    match data.title() {
        None => flag(IssueCode::NoTitle),
        Some(title) if is_all_caps(title) => flag(IssueCode::TitleAllCaps),
        Some(_) => {}
    }
    if data.creators.is_empty() {
        flag(IssueCode::NoCreators);
    }
    if data.issued.is_none() {
        flag(IssueCode::NoDate);
    }
    if needs_container(data.kind) && data.get(TextField::ContainerTitle).is_none() {
        flag(IssueCode::NoContainer);
    }
    if data.kind == EntryKind::Document {
        flag(IssueCode::UnknownType);
    }
    if data.identifiers.is_empty()
        && matches!(data.kind, EntryKind::ArticleJournal | EntryKind::Article | EntryKind::PaperConference)
    {
        flag(IssueCode::NoIdentifier);
    }
    found
}

/// Types that appear inside something else (a journal, a book, proceedings).
fn needs_container(kind: EntryKind) -> bool {
    use EntryKind::*;
    matches!(
        kind,
        ArticleJournal
            | ArticleMagazine
            | ArticleNewspaper
            | Chapter
            | PaperConference
            | EntryDictionary
            | EntryEncyclopedia
    )
}

/// At least four letters and no lowercase ones.
fn is_all_caps(text: &str) -> bool {
    text.chars().filter(|c| c.is_alphabetic()).count() >= 4 && !text.chars().any(char::is_lowercase)
}

/// Records `issue` on `reference`, replacing an earlier one with the same code.
pub fn record(library: &mut impl Library, reference: ReferenceId, issue: Issue) -> Result<()> {
    let mut issues = library.recorded_issues(reference)?;
    issues.retain(|known| known.code != issue.code);
    issues.push(issue);
    library.set_recorded_issues(reference, &issues)
}

/// Removes the recorded issue `code` from `reference`.
pub fn clear(library: &mut impl Library, reference: ReferenceId, code: IssueCode) -> Result<()> {
    let mut issues = library.recorded_issues(reference)?;
    let before = issues.len();
    issues.retain(|known| known.code != code);
    if issues.len() < before {
        library.set_recorded_issues(reference, &issues)?;
    }
    Ok(())
}

/// Recorded issues and lints of `reference`, worst first.
pub fn issues(library: &impl Library, reference: &Reference) -> Result<Vec<Issue>> {
    let mut all = library.recorded_issues(reference.id)?;
    all.extend(lint(&reference.data));
    all.sort_by_key(|issue| std::cmp::Reverse(issue.severity()));
    Ok(all)
}

/// Every reference of `references` with its issues (none for a clean one), possible duplicates
/// included (found against the whole library).
pub fn with_issues(library: &impl Library, references: Vec<Reference>) -> Result<Vec<Checked>> {
    let all = library.references(&ReferenceFilter::default())?;
    let dismissed: HashSet<_> =
        library.dismissed_duplicates()?.into_iter().map(|(a, b)| duplicates::ordered(a, b)).collect();
    let found = duplicates::index(&all, &dismissed);
    let keys: HashMap<ReferenceId, &str> = all.iter().map(|r| (r.id, r.cite_key.as_str())).collect();
    references
        .into_iter()
        .map(|reference| {
            let mut issues = issues(library, &reference)?;
            for m in found.get(&reference.id).into_iter().flatten() {
                let other = keys.get(&m.other).copied().unwrap_or("?");
                issues.push(Issue {
                    detail: Some(format!("{other} ({})", m.reason.describe())),
                    ..Issue::new(IssueCode::PossibleDuplicate)
                });
            }
            Ok(Checked { issues, reference })
        })
        .collect()
}

/// The references among `references` that have issues.
pub fn check(library: &impl Library, references: Vec<Reference>) -> Result<Vec<Checked>> {
    Ok(with_issues(library, references)?.into_iter().filter(|c| !c.issues.is_empty()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_model::{Creator, CreatorRole, Date, Identifier};

    fn codes(data: &ReferenceData) -> Vec<IssueCode> {
        lint(data).into_iter().map(|i| i.code).collect()
    }

    #[test]
    fn complete_article_has_no_lints() {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, "Nanometre-scale thermometry");
        data.set(TextField::ContainerTitle, "Nature");
        data.creators.push(Creator::person(CreatorRole::Author, "Kucsko", None));
        data.issued = Some(Date::year(2013));
        data.identifiers.push(Identifier::parse("10.1038/nature12373").unwrap());
        assert!(lint(&data).is_empty());
    }

    #[test]
    fn empty_document_has_every_lint() {
        let data = ReferenceData::new(EntryKind::Document);
        assert_eq!(
            codes(&data),
            [IssueCode::NoTitle, IssueCode::NoCreators, IssueCode::NoDate, IssueCode::UnknownType]
        );
        let mut article = ReferenceData::new(EntryKind::ArticleJournal);
        article.set(TextField::Title, "A STUDY OF THINGS");
        let found = codes(&article);
        assert!(
            found.contains(&IssueCode::TitleAllCaps)
                && found.contains(&IssueCode::NoContainer)
                && found.contains(&IssueCode::NoIdentifier)
        );
    }

    #[test]
    fn short_acronym_titles_are_not_caps() {
        assert!(!is_all_caps("BERT"[..3].as_ref()));
        assert!(is_all_caps("THE DATA"));
        assert!(!is_all_caps("The DATA"));
    }
}
