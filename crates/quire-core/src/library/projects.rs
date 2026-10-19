//! Projects: created by name under an optional parent, found by slug path (`thesis/chapter-2`),
//! listed as a tree, and used to collect references for export.

use quire_model::{Project, ProjectId, Reference, SlugPath};

use crate::error::{CoreError, Result};
use crate::library::references::find;
use crate::ports::{Library, ReferenceFilter};

/// Creates a project named `name` under the project at `parent` (top level if `None`).
pub fn create(library: &mut impl Library, name: &str, parent: Option<&SlugPath>) -> Result<(SlugPath, Project)> {
    let parent_project = parent.map(|path| resolve(library, path)).transpose()?;
    let project = Project::new(name, parent_project.as_ref().map(|p| p.id), library.device())?;
    let path: SlugPath = match parent {
        Some(parent) => format!("{parent}/{}", project.slug).parse()?,
        None => project.slug.to_string().parse()?,
    };
    if library.child_project(project.parent, &project.slug)?.is_some() {
        return Err(CoreError::ProjectExists(path.to_string()));
    }
    library.insert_project(&project)?;
    Ok((path, project))
}

/// The project at `path`.
pub fn resolve(library: &impl Library, path: &SlugPath) -> Result<Project> {
    let mut parent: Option<Project> = None;
    for slug in path.segments() {
        let found = library.child_project(parent.as_ref().map(|p| p.id), slug)?;
        parent = Some(found.ok_or_else(|| CoreError::ProjectNotFound(path.to_string()))?);
    }
    parent.ok_or_else(|| CoreError::ProjectNotFound(path.to_string()))
}

/// The path of the project `id` (walking up its parents).
pub fn path_of(library: &impl Library, id: ProjectId) -> Result<SlugPath> {
    let mut slugs = Vec::new();
    let mut next = Some(id);
    while let Some(id) = next {
        let project = library.project(id)?.ok_or_else(|| CoreError::ProjectNotFound(id.to_string()))?;
        slugs.push(project.slug.to_string());
        next = project.parent;
    }
    slugs.reverse();
    Ok(slugs.join("/").parse()?)
}

/// Every project with its path, depth first, siblings by slug.
pub fn tree(library: &impl Library) -> Result<Vec<(SlugPath, Project)>> {
    let mut out = Vec::new();
    collect(library, None, "", &mut out)?;
    Ok(out)
}

fn collect(library: &impl Library, parent: Option<ProjectId>, prefix: &str, out: &mut Vec<(SlugPath, Project)>) -> Result<()> {
    for project in library.child_projects(parent)? {
        let path = format!("{prefix}{}", project.slug);
        let id = project.id;
        out.push((path.parse()?, project));
        collect(library, Some(id), &format!("{path}/"), out)?;
    }
    Ok(())
}

/// `project` and, if `recursive`, all projects below it.
pub fn with_descendants(library: &impl Library, project: ProjectId, recursive: bool) -> Result<Vec<ProjectId>> {
    let mut ids = vec![project];
    let mut next = 0;
    while recursive && next < ids.len() {
        ids.extend(library.child_projects(Some(ids[next]))?.into_iter().map(|p| p.id));
        next += 1;
    }
    Ok(ids)
}

/// The references in the project at `path` (and below it if `recursive`), ordered by cite key.
pub fn references(library: &impl Library, path: &SlugPath, recursive: bool) -> Result<Vec<Reference>> {
    let project = resolve(library, path)?;
    let filter = ReferenceFilter { projects: Some(with_descendants(library, project.id, recursive)?), include_deleted: false };
    library.references(&filter)
}

/// Links the reference `query` (ID, cite key or identifier) to the project at `path`; `false`
/// if it already was.
pub fn link(library: &mut impl Library, query: &str, path: &SlugPath) -> Result<(Reference, bool)> {
    let reference = find(library, query)?;
    let project = resolve(library, path)?;
    let linked = library.link(reference.id, project.id)?;
    Ok((reference, linked))
}

/// Unlinks them; the reference stays in the library. `false` if they were not linked.
pub fn unlink(library: &mut impl Library, query: &str, path: &SlugPath) -> Result<(Reference, bool)> {
    let reference = find(library, query)?;
    let project = resolve(library, path)?;
    let unlinked = library.unlink(reference.id, project.id)?;
    Ok((reference, unlinked))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::references::{add, AddRequest};
    use crate::memory::MemoryLibrary;
    use quire_model::{EntryKind, Identifier, Origin, ReferenceData};

    fn path(text: &str) -> SlugPath {
        text.parse().unwrap()
    }

    fn add_doi(lib: &mut MemoryLibrary, doi: &str, project: Option<ProjectId>) -> Reference {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.identifiers.push(Identifier::parse(doi).unwrap());
        let mut request = AddRequest::new(data, Origin::User);
        request.project = project;
        add(lib, request).unwrap().reference().clone()
    }

    #[test]
    fn nested_projects() {
        let mut lib = MemoryLibrary::default();
        create(&mut lib, "Thesis", None).unwrap();
        let (p, chapter) = create(&mut lib, "Chapter 2: Methods", Some(&path("thesis"))).unwrap();
        assert_eq!(p.to_string(), "thesis/chapter-2-methods");
        assert_eq!(path_of(&lib, chapter.id).unwrap(), p);
        assert!(matches!(create(&mut lib, "thesis", None), Err(CoreError::ProjectExists(_))));
        assert!(matches!(create(&mut lib, "X", Some(&path("nope"))), Err(CoreError::ProjectNotFound(_))));
        let paths: Vec<String> = tree(&lib).unwrap().iter().map(|(p, _)| p.to_string()).collect();
        assert_eq!(paths, ["thesis", "thesis/chapter-2-methods"]);
    }

    #[test]
    fn references_with_and_without_subprojects() {
        let mut lib = MemoryLibrary::default();
        let (_, thesis) = create(&mut lib, "thesis", None).unwrap();
        let (_, chapter) = create(&mut lib, "ch1", Some(&path("thesis"))).unwrap();
        add_doi(&mut lib, "10.1/top", Some(thesis.id));
        add_doi(&mut lib, "10.1/sub", Some(chapter.id));
        assert_eq!(references(&lib, &path("thesis"), false).unwrap().len(), 1);
        assert_eq!(references(&lib, &path("thesis"), true).unwrap().len(), 2);
    }

    #[test]
    fn one_reference_in_two_projects() {
        let mut lib = MemoryLibrary::default();
        create(&mut lib, "a", None).unwrap();
        create(&mut lib, "b", None).unwrap();
        let r = add_doi(&mut lib, "10.1/x", None);
        assert!(link(&mut lib, r.cite_key.as_str(), &path("a")).unwrap().1);
        assert!(link(&mut lib, "doi:10.1/x", &path("b")).unwrap().1);
        assert!(!link(&mut lib, "doi:10.1/x", &path("b")).unwrap().1);
        assert!(unlink(&mut lib, "doi:10.1/x", &path("a")).unwrap().1);
        assert_eq!(references(&lib, &path("a"), false).unwrap().len(), 0);
        assert_eq!(references(&lib, &path("b"), false).unwrap().len(), 1);
    }
}
