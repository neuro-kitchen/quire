//! Sharing projects with other people. A member's role on a project holds for its subprojects;
//! the project's owner (set when it is first shared) is an owner of it everywhere. Only owners
//! add or remove members. What a person sees of the library is the projects shared with them,
//! their subprojects, the references in them and their PDFs (`.knowledge/sync.md`).

use std::collections::HashSet;

use quire_model::{Project, ProjectId, ProjectMember, Role, SlugPath, UserId};

use crate::error::{CoreError, Result};
use crate::library::projects;
use crate::ports::Library;

/// `user`'s role on `project`: the highest of their roles on it and its ancestors (an owner
/// recorded on any of them is an owner).
pub fn role_of(library: &impl Library, user: &UserId, project: ProjectId) -> Result<Option<Role>> {
    let mut best: Option<Role> = None;
    for found in projects::ancestors(library, project)? {
        if found.owner.as_ref() == Some(user) {
            return Ok(Some(Role::Owner));
        }
        for member in library.members_of(found.id)? {
            if &member.user == user && !member.meta.is_deleted() {
                best = best.max(Some(member.role));
            }
        }
    }
    Ok(best)
}

/// Gives `user` (named `name`) `role` on the project at `path`, as `me`. The first share of a
/// project nobody owns makes `me` its owner.
pub fn share(
    library: &mut impl Library,
    me: &UserId,
    path: &SlugPath,
    user: &UserId,
    name: &str,
    role: Role,
) -> Result<ProjectMember> {
    let mut project = projects::resolve(library, path)?;
    let device = library.device();
    if project.owner.is_none() && role_of(library, me, project.id)?.is_none() {
        project.owner = Some(me.clone());
        project.meta.touch(device);
        library.update_project(&project)?;
    }
    ensure_owner(library, me, &project)?;
    if user == me {
        return Err(CoreError::Sharing("you are already in your own project".into()));
    }
    let mut member = ProjectMember::new(project.id, user.clone(), name, role, me.clone(), device);
    if let Some(known) = library.members_of(project.id)?.into_iter().find(|m| &m.user == user) {
        member.meta = known.meta;
        member.meta.restore(device);
    }
    library.set_member(&member)?;
    Ok(member)
}

/// Removes `user` from the project at `path`, as `me` (an owner).
pub fn unshare(library: &mut impl Library, me: &UserId, path: &SlugPath, user: &UserId) -> Result<ProjectMember> {
    let project = projects::resolve(library, path)?;
    ensure_owner(library, me, &project)?;
    let mut member = library
        .members_of(project.id)?
        .into_iter()
        .find(|m| &m.user == user && !m.meta.is_deleted())
        .ok_or_else(|| CoreError::Sharing(format!("{} is not a member of {path}", user.short())))?;
    member.granted_by = me.clone();
    member.meta.delete(library.device());
    library.set_member(&member)?;
    Ok(member)
}

/// The members of the project at `path` (its own, not its ancestors'), and its owner.
pub fn members(library: &impl Library, path: &SlugPath) -> Result<(Project, Vec<ProjectMember>)> {
    let project = projects::resolve(library, path)?;
    let members = library.members_of(project.id)?.into_iter().filter(|m| !m.meta.is_deleted()).collect();
    Ok((project, members))
}

/// Every project `user` can see: those shared with them (or owned) and their subprojects.
pub fn visible_projects(library: &impl Library, user: &UserId) -> Result<HashSet<ProjectId>> {
    let mut roots: Vec<ProjectId> =
        library.memberships(user)?.into_iter().filter(|m| !m.meta.is_deleted()).map(|m| m.project).collect();
    for (_, project) in projects::tree(library)? {
        if project.owner.as_ref() == Some(user) {
            roots.push(project.id);
        }
    }
    let mut visible = HashSet::new();
    for root in roots {
        visible.extend(projects::with_descendants(library, root, true)?);
    }
    Ok(visible)
}

fn ensure_owner(library: &impl Library, me: &UserId, project: &Project) -> Result<()> {
    match role_of(library, me, project.id)? {
        Some(role) if role.can_manage() => Ok(()),
        _ => Err(CoreError::Sharing(format!("only an owner of `{}` can change who it is shared with", project.name))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryLibrary;

    fn user(c: char) -> UserId {
        UserId::parse(&c.to_string().repeat(64)).unwrap()
    }

    #[test]
    fn roles_inherit_and_only_owners_share() {
        let mut library = MemoryLibrary::default();
        projects::create(&mut library, "Lab", None).unwrap();
        projects::create(&mut library, "Paper", Some(&"lab".parse().unwrap())).unwrap();
        let (me, alice, bob) = (user('a'), user('b'), user('c'));
        let lab: SlugPath = "lab".parse().unwrap();
        share(&mut library, &me, &lab, &alice, "Alice", Role::Editor).unwrap();
        let paper = projects::resolve(&library, &"lab/paper".parse().unwrap()).unwrap();
        assert_eq!(role_of(&library, &me, paper.id).unwrap(), Some(Role::Owner), "the first share makes me owner");
        assert_eq!(role_of(&library, &alice, paper.id).unwrap(), Some(Role::Editor), "inherited by subprojects");
        assert_eq!(role_of(&library, &bob, paper.id).unwrap(), None);
        assert!(share(&mut library, &alice, &lab, &bob, "Bob", Role::Reader).is_err(), "editors do not share");
        assert_eq!(visible_projects(&library, &alice).unwrap().len(), 2);
        unshare(&mut library, &me, &lab, &alice).unwrap();
        assert_eq!(role_of(&library, &alice, paper.id).unwrap(), None);
        assert!(visible_projects(&library, &alice).unwrap().is_empty());
        assert_eq!(members(&library, &lab).unwrap().1.len(), 0);
    }
}
