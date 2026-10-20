//! Sync with another person's device (`.knowledge/sync.md`, several users): what they may see
//! ([`SqliteLibrary::visible_changes`]) and which of their changes are taken
//! ([`SqliteLibrary::authorize`]). Checks are by the sender's user: a peer can change only what
//! its user may change, whoever wrote the row first.
//!
//! Seen by a member: the projects shared with them and their subprojects, those projects'
//! memberships and members, the references in them and their attachments. Never: tags,
//! taggings, notes, highlights, relations, "not duplicates", reading status, saved searches.

use std::collections::HashSet;

use quire_core::Result;
use quire_core::library::sharing;
use quire_core::prelude::*;
use quire_core::sync::Row;
use quire_model::{ProjectId, ReferenceId, UserId};

use super::merge::alias;
use crate::repo::SqliteLibrary;
use crate::repo::codec::id_blob;
use crate::repo::{attachments, links, projects, references};

impl SqliteLibrary<'_> {
    /// `rows` (changes since a peer's last sync) narrowed to what `user` may see, with what
    /// becomes visible through them: a reference linked into a shared project brings its row and
    /// attachments; a grant to `user` brings the whole project.
    pub fn visible_changes(&self, rows: Vec<Row>, user: &UserId) -> Result<Vec<Row>> {
        let visible = sharing::visible_projects(self, user)?;
        let shown_reference = |id: ReferenceId| -> Result<bool> {
            Ok(links::projects_of(self.conn, id)?.iter().any(|p| visible.contains(p)))
        };
        let mut out = Vec::new();
        let mut keys: HashSet<String> = HashSet::new();
        let mut push = |row: Row, out: &mut Vec<Row>| {
            if keys.insert(key(&row)) {
                out.push(row);
            }
        };
        let mut granted: Vec<ProjectId> = Vec::new();
        let mut linked: Vec<ReferenceId> = Vec::new();
        for row in rows {
            let keep = match &row {
                Row::Project(p) => visible.contains(&p.id),
                Row::Member(m) => {
                    if &m.user == user && !m.meta.is_deleted() {
                        granted.push(m.project);
                    }
                    visible.contains(&m.project) || &m.user == user
                }
                Row::Membership(m) => {
                    if visible.contains(&m.project) && m.stamp.deleted_at.is_none() {
                        linked.push(m.reference);
                    }
                    visible.contains(&m.project)
                }
                Row::Reference(r) => shown_reference(r.id)?,
                Row::Attachment(a) => shown_reference(a.reference)?,
                _ => false,
            };
            if keep {
                push(row, &mut out);
            }
        }
        for project in granted {
            for id in quire_core::library::projects::with_descendants(self, project, true)? {
                if let Some(found) = projects::by_id(self.conn, id)? {
                    push(Row::Project(found), &mut out);
                }
                for member in self.members_of(id)? {
                    push(Row::Member(member), &mut out);
                }
                for reference in
                    self.references(&quire_core::ReferenceFilter { projects: Some(vec![id]), ..Default::default() })?
                {
                    linked.push(reference.id);
                    if let Some(membership) =
                        super::rows::load(self.conn, "project_reference", &id_blob(id), &id_blob(reference.id))?
                    {
                        push(membership, &mut out);
                    }
                }
            }
        }
        for id in linked {
            if let Some(reference) = references::by_id(self.conn, id)? {
                push(Row::Reference(reference), &mut out);
            }
            for attachment in attachments::of(self.conn, id)? {
                push(Row::Attachment(attachment), &mut out);
            }
        }
        // Parents before the rows that need them, as `merge` expects anyway.
        Ok(out)
    }

    /// Of `rows` from a device of `sender` (another person), those `sender` may write here, and
    /// how many were refused. A collaborator cannot put a reference in the trash (they unlink it
    /// from the shared project instead): the trash state of references stays this library's.
    pub fn authorize(&self, rows: Vec<Row>, sender: &UserId) -> Result<(Vec<Row>, usize)> {
        let conn = self.conn;
        // Projects the sender may edit: from what is here, and from the batch (a project they
        // share for the first time, or a subproject of an editable one).
        let mut editable: HashSet<ProjectId> = HashSet::new();
        let mut owned: HashSet<ProjectId> = HashSet::new();
        for (_, project) in quire_core::library::projects::tree(self)? {
            match sharing::role_of(self, sender, project.id)? {
                Some(role) if role.can_manage() => {
                    owned.insert(project.id);
                    editable.insert(project.id);
                }
                Some(role) if role.can_edit() => {
                    editable.insert(project.id);
                }
                _ => {}
            }
        }
        let mut accepted_projects = HashSet::new();
        let mut incoming_projects: Vec<_> = rows
            .iter()
            .filter_map(|r| match r {
                Row::Project(p) => Some(p.clone()),
                _ => None,
            })
            .collect();
        loop {
            let before = accepted_projects.len();
            incoming_projects.retain(|p| {
                let id = alias(conn, p.id).unwrap_or(p.id);
                let local = projects::by_id(conn, id).ok().flatten();
                let ok = match &local {
                    // The owner never changes; removing a project takes an owner.
                    Some(local) => {
                        local.owner == p.owner
                            && editable.contains(&id)
                            && (p.meta.deleted_at.is_none() || owned.contains(&id))
                    }
                    None => {
                        p.owner.as_ref() == Some(sender)
                            || p.parent.is_some_and(|parent| editable.contains(&alias(conn, parent).unwrap_or(parent)))
                    }
                };
                if ok {
                    accepted_projects.insert(p.id);
                    editable.insert(id);
                    if p.owner.as_ref() == Some(sender) {
                        owned.insert(id);
                    }
                }
                !ok
            });
            if accepted_projects.len() == before {
                break;
            }
        }
        let resolve = |id: ProjectId| alias(conn, id).unwrap_or(id);
        // References the batch links into an editable project, by the sender's IDs.
        let linked: HashSet<ReferenceId> = rows
            .iter()
            .filter_map(|r| match r {
                Row::Membership(m) if editable.contains(&resolve(m.project)) => Some(m.reference),
                _ => None,
            })
            .collect();
        let reference_ok = |id: ReferenceId| -> Result<bool> {
            if linked.contains(&id) {
                return Ok(true);
            }
            let local = alias(conn, id)?;
            Ok(links::projects_of(conn, local)?.iter().any(|p| editable.contains(p)))
        };
        let mut kept = Vec::with_capacity(rows.len());
        let mut refused = 0;
        for row in rows {
            let ok = match &row {
                Row::Project(p) => accepted_projects.contains(&p.id),
                Row::Member(m) => owned.contains(&resolve(m.project)) && &m.granted_by == sender,
                Row::Membership(m) => editable.contains(&resolve(m.project)),
                Row::Reference(r) => reference_ok(r.id)?,
                Row::Attachment(a) => reference_ok(a.reference)?,
                _ => false,
            };
            if !ok {
                refused += 1;
                continue;
            }
            kept.push(match row {
                Row::Reference(mut r) => {
                    let local = references::by_id(conn, alias(conn, r.id)?)?;
                    r.meta.deleted_at = local.and_then(|l| l.meta.deleted_at);
                    r.clocks.remove(quire_model::reference::parts::TRASH);
                    Row::Reference(r)
                }
                other => other,
            });
        }
        Ok((kept, refused))
    }

    /// Hashes of the files of attachments `user` may see (what a collaborator may ask for).
    pub fn visible_hashes(&self, user: &UserId) -> Result<HashSet<String>> {
        let visible: Vec<ProjectId> = sharing::visible_projects(self, user)?.into_iter().collect();
        if visible.is_empty() {
            return Ok(HashSet::new());
        }
        let filter = quire_core::ReferenceFilter { projects: Some(visible), ..Default::default() };
        let mut hashes = HashSet::new();
        for reference in self.references(&filter)? {
            hashes.extend(attachments::of(self.conn, reference.id)?.into_iter().map(|a| a.hash));
        }
        Ok(hashes)
    }
}

/// A row's identity, to send it once.
fn key(row: &Row) -> String {
    match row {
        Row::Project(p) => format!("p{}", p.id),
        Row::Member(m) => format!("m{}{}", m.project, m.user),
        Row::Membership(m) => format!("l{}{}", m.project, m.reference),
        Row::Reference(r) => format!("r{}", r.id),
        Row::Attachment(a) => format!("a{}", a.id),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use quire_core::library::{projects, references, sharing, tags};
    use quire_core::sync::MergeSummary;
    use quire_core::{AddRequest, ReferenceFilter};
    use quire_model::{EntryKind, Identifier, Origin, ReferenceData, Role, TagKind, TextField};
    use rusqlite::Connection;

    use super::*;

    fn library(conn: &Connection) -> SqliteLibrary<'_> {
        SqliteLibrary::new(conn, crate::device_id(conn).unwrap())
    }

    fn user(c: char) -> UserId {
        UserId::parse(&c.to_string().repeat(64)).unwrap()
    }

    fn paper(title: &str, doi: &str) -> AddRequest {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.set(TextField::Title, title);
        data.identifiers.push(Identifier::parse(doi).unwrap());
        AddRequest::new(data, Origin::User)
    }

    /// One direction of a session between two people's libraries.
    fn send(
        from: &SqliteLibrary,
        from_user: &UserId,
        to: &mut SqliteLibrary,
        to_user: &UserId,
    ) -> (MergeSummary, usize) {
        let peer = from.device.to_string();
        let since = to.received_from(&peer).unwrap();
        let (rows, last) = from.changes_since(since, &to.device.to_string()).unwrap();
        let visible = from.visible_changes(rows, to_user).unwrap();
        let (allowed, refused) = to.authorize(visible, from_user).unwrap();
        let merged = to.merge(allowed, &peer).unwrap();
        to.record_received(&peer, last).unwrap();
        (merged, refused)
    }

    #[test]
    fn a_shared_project_and_nothing_else() {
        let (bob_conn, alice_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut bob, mut alice) = (library(&bob_conn), library(&alice_conn));
        let (b, a) = (user('b'), user('a'));
        let lab: quire_model::SlugPath = "lab".parse().unwrap();
        // Bob: a shared project with one paper, a private project, a tag and a note.
        projects::create(&mut bob, "Lab", None).unwrap();
        projects::create(&mut bob, "Private", None).unwrap();
        let shared =
            references::add(&mut bob, AddRequest { project: None, ..paper("shared paper", "10.1/shared") }).unwrap();
        let shared_key = shared.reference().cite_key.to_string();
        projects::link(&mut bob, &shared_key, &lab).unwrap();
        let private = references::add(&mut bob, paper("private paper", "10.1/private")).unwrap();
        projects::link(&mut bob, &private.reference().cite_key.to_string(), &"private".parse().unwrap()).unwrap();
        tags::add(&mut bob, &shared_key, "secret-tag", TagKind::Manual).unwrap();
        tags::add_note(&mut bob, Some(&shared_key), "bob's private note").unwrap();
        sharing::share(&mut bob, &b, &lab, &a, "Alice", Role::Editor).unwrap();

        let (_, refused) = send(&bob, &b, &mut alice, &a);
        assert_eq!(refused, 0);
        let all = alice.references(&ReferenceFilter::default()).unwrap();
        assert_eq!(all.len(), 1, "only the shared paper: {all:?}");
        assert_eq!(projects::tree(&alice).unwrap().len(), 1, "only the shared project");
        assert!(alice.tags().unwrap().is_empty() && alice.notes_of(Some(all[0].id)).unwrap().is_empty());
        let on_alice = projects::resolve(&alice, &lab).unwrap();
        assert_eq!(on_alice.owner.as_ref(), Some(&b));
        assert_eq!(sharing::role_of(&alice, &a, on_alice.id).unwrap(), Some(Role::Editor));

        // Alice fixes the title, adds a paper to the project, writes a private note, and tries
        // what an editor may not: trash Bob's paper, add a member.
        let mut fixed = all[0].clone();
        fixed.data.set(TextField::Title, "Shared Paper");
        fixed.meta.touch(alice.device);
        references::save(&mut alice, &mut fixed).unwrap();
        let added =
            references::add(&mut alice, AddRequest { project: Some(on_alice.id), ..paper("alice's find", "10.1/new") })
                .unwrap();
        tags::add_note(&mut alice, Some(&added.reference().cite_key.to_string()), "alice's private note").unwrap();
        references::trash(&mut alice, &fixed.cite_key.to_string()).unwrap();
        let mut sneaky =
            quire_model::ProjectMember::new(on_alice.id, user('c'), "Carol", Role::Owner, a.clone(), alice.device);
        sneaky.meta.touch(alice.device);
        alice.set_member(&sneaky).unwrap();

        let (merged, refused) = send(&alice, &a, &mut bob, &b);
        assert!(refused >= 1, "the member Alice added is refused: {merged:?}");
        let on_bob = bob.reference(shared.reference().id).unwrap().unwrap();
        assert_eq!(on_bob.data.title(), Some("Shared Paper"), "the editor's edit");
        assert!(!on_bob.meta.is_deleted(), "a collaborator cannot trash it");
        let in_lab = projects::references(&bob, &lab, false).unwrap();
        assert_eq!(in_lab.len(), 2, "Alice's find is in the project");
        let found = in_lab.iter().find(|r| r.data.title() == Some("alice's find")).unwrap();
        assert!(bob.notes_of(Some(found.id)).unwrap().is_empty(), "her note stays hers");
        assert!(bob.members_of(projects::resolve(&bob, &lab).unwrap().id).unwrap().iter().all(|m| m.user != user('c')));
    }

    #[test]
    fn readers_cannot_edit() {
        let (bob_conn, rita_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut bob, mut rita) = (library(&bob_conn), library(&rita_conn));
        let (b, r) = (user('b'), user('d'));
        let lab: quire_model::SlugPath = "lab".parse().unwrap();
        projects::create(&mut bob, "Lab", None).unwrap();
        let shared = references::add(&mut bob, paper("shared paper", "10.1/shared")).unwrap();
        projects::link(&mut bob, &shared.reference().cite_key.to_string(), &lab).unwrap();
        sharing::share(&mut bob, &b, &lab, &r, "Rita", Role::Reader).unwrap();
        send(&bob, &b, &mut rita, &r);
        let mut edited = rita.reference(shared.reference().id).unwrap().unwrap();
        edited.data.set(TextField::Title, "Rita was here");
        edited.meta.touch(rita.device);
        references::save(&mut rita, &mut edited).unwrap();
        let (_, refused) = send(&rita, &r, &mut bob, &b);
        assert_eq!(refused, 1);
        assert_eq!(bob.reference(shared.reference().id).unwrap().unwrap().data.title(), Some("shared paper"));
    }

    #[test]
    fn a_dishonest_peer_is_held_to_its_role() {
        let (bob_conn, eve_conn) = (crate::open_in_memory().unwrap(), crate::open_in_memory().unwrap());
        let (mut bob, mut eve) = (library(&bob_conn), library(&eve_conn));
        let (b, e) = (user('b'), user('e'));
        let lab: quire_model::SlugPath = "lab".parse().unwrap();
        projects::create(&mut bob, "Lab", None).unwrap();
        let shared = references::add(&mut bob, paper("shared paper", "10.1/shared")).unwrap();
        let key = shared.reference().cite_key.to_string();
        projects::link(&mut bob, &key, &lab).unwrap();
        sharing::share(&mut bob, &b, &lab, &e, "Eve", Role::Editor).unwrap();
        send(&bob, &b, &mut eve, &e);
        // Eve's own changes: a tag and a note on the shared paper, a project of hers, a grant.
        tags::add(&mut eve, &key, "eve-tag", TagKind::Manual).unwrap();
        tags::add_note(&mut eve, Some(&key), "eve's note").unwrap();
        projects::create(&mut eve, "Eve's own", None).unwrap();
        let on_eve = projects::resolve(&eve, &lab).unwrap();
        let mut grant =
            quire_model::ProjectMember::new(on_eve.id, user('c'), "Carol", Role::Owner, e.clone(), eve.device);
        grant.meta.touch(eve.device);
        eve.set_member(&grant).unwrap();
        // Sent without the visibility filter, as a modified client could.
        let (rows, _) = eve.changes_since(0, &bob.device.to_string()).unwrap();
        let total = rows.len();
        let (allowed, refused) = bob.authorize(rows, &e).unwrap();
        assert!(refused >= 4, "{refused} of {total} refused");
        assert!(
            allowed
                .iter()
                .all(|r| matches!(r, Row::Project(_) | Row::Membership(_) | Row::Reference(_) | Row::Attachment(_)))
        );
        bob.merge(allowed, &eve.device.to_string()).unwrap();
        assert!(bob.tags().unwrap().is_empty());
        assert!(bob.notes_of(Some(shared.reference().id)).unwrap().is_empty());
        assert_eq!(projects::tree(&bob).unwrap().len(), 1, "Eve's own project is not taken");
        assert!(bob.members_of(projects::resolve(&bob, &lab).unwrap().id).unwrap().iter().all(|m| m.user != user('c')));
    }
}
