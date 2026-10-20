//! Checking a manuscript against the library (`quire folder check`): the cite keys its files use
//! (`quire_io::manuscript`) that the library lacks, that are outside the document's project (so
//! missing from its bibliography file), and the project's references it never cites.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire_core::ReferenceFilter;
use quire_core::library::projects;
use quire_core::prelude::*;
use quire_io::manuscript::{self, Markup};
use quire_model::{Reference, SlugPath};

use super::Engine;
use crate::error::{EngineError, Result};

/// Folders never searched for manuscripts.
const SKIPPED_FOLDERS: &[&str] = &["node_modules", "target", "_build", "build", "dist"];

/// Where a key is cited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

/// A cite key the library does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKey {
    pub key: String,
    pub uses: Vec<Use>,
    /// Library keys it may be a typo of, or that cite the same author and year.
    pub suggestions: Vec<String>,
    /// It names a reference in the trash.
    pub trashed: bool,
}

/// What a manuscript cites, against the library and its project.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManuscriptCheck {
    pub files: Vec<PathBuf>,
    /// Distinct keys cited.
    pub cited: usize,
    pub unknown: Vec<UnknownKey>,
    /// In the library but not in the project: missing from the project's bibliography file.
    pub outside: Vec<(Reference, Vec<Use>)>,
    /// In the project, never cited (only with a project).
    pub uncited: Vec<Reference>,
}

impl ManuscriptCheck {
    pub fn is_clean(&self) -> bool {
        self.unknown.is_empty() && self.outside.is_empty()
    }
}

impl Engine {
    /// Checks the cite keys of `files` (`.typ`, `.tex`, `.md`; folders are searched for them)
    /// against the library and, if given, `project`.
    pub async fn check_manuscript(
        &self,
        files: &[PathBuf],
        project: Option<SlugPath>,
        recursive: bool,
    ) -> Result<ManuscriptCheck> {
        let files = manuscripts(files)?;
        let mut uses: BTreeMap<String, Vec<Use>> = BTreeMap::new();
        for file in &files {
            let markup = Markup::of(file)
                .ok_or_else(|| EngineError::Config(format!("{}: not a .typ, .tex or .md file", file.display())))?;
            let text = std::fs::read_to_string(file).map_err(EngineError::file(file))?;
            for citation in manuscript::citations(&text, markup) {
                let place = Use { file: file.clone(), line: citation.line, column: citation.column };
                uses.entry(citation.key).or_default().push(place);
            }
        }
        let (library, trash, in_project) = self
            .run(move |lib| {
                let all = lib.references(&ReferenceFilter { include_deleted: true, ..ReferenceFilter::default() })?;
                let (trash, library): (Vec<Reference>, Vec<Reference>) =
                    all.into_iter().partition(|r| r.meta.is_deleted());
                let in_project = match &project {
                    Some(path) => Some(projects::references(lib, path, recursive)?),
                    None => None,
                };
                Ok((library, trash, in_project))
            })
            .await?;

        let mut check = ManuscriptCheck { cited: uses.len(), files, ..ManuscriptCheck::default() };
        for (key, places) in &uses {
            match library.iter().find(|r| r.cite_key.as_str() == key) {
                Some(reference) => {
                    let outside = in_project.as_ref().is_some_and(|refs| !refs.iter().any(|r| r.id == reference.id));
                    if outside {
                        check.outside.push((reference.clone(), places.clone()));
                    }
                }
                None => check.unknown.push(UnknownKey {
                    key: key.clone(),
                    uses: places.clone(),
                    suggestions: suggest_keys(key, &library),
                    trashed: trash.iter().any(|r| r.cite_key.as_str() == key),
                }),
            }
        }
        if let Some(references) = in_project {
            check.uncited = references.into_iter().filter(|r| !uses.contains_key(r.cite_key.as_str())).collect();
        }
        Ok(check)
    }
}

/// `paths` with folders replaced by the manuscripts in them (not in hidden or build folders).
fn manuscripts(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            collect(path, &mut files)?;
        } else {
            files.push(path.clone());
        }
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        return Err(EngineError::Config("no .typ, .tex or .md files to check here".into()));
    }
    Ok(files)
}

fn collect(folder: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(folder).map_err(EngineError::file(folder))? {
        let path = entry.map_err(EngineError::file(folder))?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            if !SKIPPED_FOLDERS.contains(&name) {
                collect(&path, files)?;
            }
        } else if Markup::of(&path).is_some() {
            files.push(path);
        }
    }
    Ok(())
}

/// Library keys `key` may mean: the same ignoring case, a small typo, or the same leading name
/// and trailing year (`kucsko2013` → `kucskoNanometreScaleThermometry2013`). At most three.
pub fn suggest_keys(key: &str, library: &[Reference]) -> Vec<String> {
    let lower = key.to_lowercase();
    let name: String = lower.chars().take_while(|c| c.is_alphabetic()).collect();
    let year: String =
        lower.chars().rev().take_while(char::is_ascii_digit).collect::<Vec<_>>().into_iter().rev().collect();
    let mut found: Vec<(usize, String)> = library
        .iter()
        .filter_map(|reference| {
            let candidate = reference.cite_key.as_str();
            let candidate_lower = candidate.to_lowercase();
            let distance = edit_distance(&lower, &candidate_lower);
            let same_work = name.len() >= 3
                && year.len() == 4
                && candidate_lower.starts_with(&name)
                && candidate_lower.ends_with(&year);
            (distance <= 2 || same_work).then(|| (distance, candidate.to_string()))
        })
        .collect();
    found.sort();
    found.into_iter().take(3).map(|(_, key)| key).collect()
}

/// Levenshtein distance, in characters.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let next = (diagonal + usize::from(ca != *cb)).min(row[j] + 1).min(row[j + 1] + 1);
            diagonal = row[j + 1];
            row[j + 1] = next;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("smith2020", "smith2020"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
