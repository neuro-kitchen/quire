use super::*;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    (dir, store)
}

fn batch(author: &str, number: u64, sealed: &[u8]) -> Batch {
    Batch { author: author.into(), number, kind: BatchKind::Changes as i32, sealed: sealed.to_vec(), covers: 0, seq: 0 }
}

fn snapshot(author: &str, covers: u64) -> Batch {
    Batch {
        author: author.into(),
        number: covers,
        kind: BatchKind::Snapshot as i32,
        sealed: b"snap".to_vec(),
        covers,
        seq: 0,
    }
}

fn refused(result: Result<impl std::fmt::Debug>, code: ErrorCode) {
    match result {
        Err(HubError::Refused(c, _)) => assert_eq!(c, code),
        other => panic!("expected {code:?}, got {other:?}"),
    }
}

#[test]
fn the_setup_code_registers_one_owner_once() {
    let (_dir, s) = store();
    let code = s.owner_code().unwrap().unwrap();
    assert_eq!(s.owner_code().unwrap(), Some(code.clone()), "the same code until used");
    assert_eq!(code.len(), 14);
    let owner = s.register("alice", &code.to_lowercase(), 0).unwrap();
    assert_eq!(s.register("alice", "anything", 0).unwrap(), owner, "registered already: the code is not needed");
    refused(s.register("mallory", &code, 0), ErrorCode::BadCode);
    assert_eq!(s.owner_code().unwrap(), None, "no setup code once someone registered");
    let invite = s.new_code(Purpose::Invite, None, Some(Duration::from_secs(60))).unwrap();
    assert_ne!(s.register("bob", &invite, 1000).unwrap().id, owner.id);
    refused(s.register("carol", &invite, 0), ErrorCode::BadCode);
    let expired = s.new_code(Purpose::Invite, None, Some(Duration::ZERO)).unwrap();
    refused(s.register("carol", &expired, 0), ErrorCode::BadCode);
}

#[test]
fn a_link_code_binds_a_person_to_an_account_made_first() {
    let (_dir, s) = store();
    let (account, code) = s.new_account(5_000).unwrap();
    let linked = s.register("alice", &code, 0).unwrap();
    assert_eq!(linked, Account { id: account, quota: 5_000 }, "the account's quota, not the default");
}

#[test]
fn batches_are_numbered_repeats_skipped_and_own_ones_not_pulled() {
    let (_dir, s) = store();
    let a = s.register("alice", &s.owner_code().unwrap().unwrap(), 0).unwrap().id;
    assert_eq!(s.push(a, "lib", "phone", &batch("phone", 5, b"one")).unwrap(), 1);
    assert_eq!(s.push(a, "lib", "phone", &batch("phone", 5, b"one again")).unwrap(), 1, "a retried push");
    assert_eq!(s.push(a, "lib", "laptop", &batch("laptop", 3, b"two")).unwrap(), 2);
    assert_eq!(s.positions(a, "lib", "phone").unwrap(), (5, 2));
    let pulled = s.pull(a, "lib", "phone", 0, 100).unwrap();
    assert_eq!(pulled.batches.iter().map(|b| b.sealed.clone()).collect::<Vec<_>>(), vec![b"two".to_vec()]);
    assert_eq!((pulled.up_to, pulled.more), (2, false), "own batch skipped, still covered");
    let limited = s.pull(a, "lib", "laptop", 0, 1).unwrap();
    assert_eq!((limited.batches.len(), limited.up_to, limited.more), (1, 1, true));
    assert!(s.pull(a, "other", "phone", 0, 100).unwrap().batches.is_empty(), "libraries apart");
}

#[test]
fn a_snapshot_drops_what_it_covers_and_comes_first_for_a_device_behind_it() {
    let (_dir, s) = store();
    let a = s.register("alice", &s.owner_code().unwrap().unwrap(), 0).unwrap().id;
    for n in 1..=3 {
        s.push(a, "lib", "laptop", &batch("laptop", n, b"x")).unwrap();
    }
    refused(s.push(a, "lib", "laptop", &snapshot("laptop", 9)), ErrorCode::Protocol);
    s.push(a, "lib", "laptop", &snapshot("laptop", 2)).unwrap();
    let new_device = s.pull(a, "lib", "tablet", 0, 100).unwrap();
    assert_eq!(new_device.batches.iter().map(|b| (b.kind, b.seq)).collect::<Vec<_>>(), vec![(1, 2), (0, 3)]);
    assert_eq!(new_device.since_snapshot, 1, "batches 1 and 2 dropped");
    let current = s.pull(a, "lib", "tablet", 3, 100).unwrap();
    assert!(current.batches.is_empty(), "a device past the snapshot does not get it again");
    s.push(a, "lib", "laptop", &snapshot("laptop", 1)).unwrap();
    assert_eq!(s.pull(a, "lib", "tablet", 0, 100).unwrap().batches[0].covers, 2, "an older snapshot does not replace");
}

#[test]
fn files_quota_use_revocation_and_deletion() {
    let (dir, s) = store();
    let a = s.register("alice", &s.owner_code().unwrap().unwrap(), 0).unwrap().id;
    let id = "ab".repeat(32);
    refused(s.put_file(a, "../../etc/passwd", b"x"), ErrorCode::Protocol);
    s.put_file(a, &id, b"sealed bytes").unwrap();
    assert_eq!(s.has_files(a, &[id.clone(), "cd".repeat(32)]).unwrap(), vec![id.clone()]);
    assert_eq!(s.get_file(a, &id).unwrap().unwrap(), b"sealed bytes");
    s.push(a, "lib", "phone", &batch("phone", 1, b"1234")).unwrap();
    assert_eq!(s.used(a).unwrap(), 12 + 4);
    s.see_device(a, "dev-1").unwrap();
    assert!(s.revoke(a, "dev-1").unwrap());
    refused(s.see_device(a, "dev-1"), ErrorCode::Revoked);
    let b = s.new_code(Purpose::Invite, None, None).and_then(|c| s.register("bob", &c, 0)).unwrap().id;
    refused(s.see_device(b, "dev-1"), ErrorCode::Revoked);
    assert!(!s.revoke(b, "dev-1").unwrap(), "not Bob's device");
    assert!(s.get_file(b, &id).unwrap().is_none(), "Bob cannot read Alice's files");
    s.delete_account(a).unwrap();
    assert!(s.account_of("alice").unwrap().is_none());
    assert!(!dir.path().join("files").join(a.to_string()).exists());
}

#[test]
fn a_restart_keeps_everything() {
    let dir = tempfile::tempdir().unwrap();
    let (account, id) = {
        let s = Store::open(dir.path()).unwrap();
        let a = s.register("alice", &s.owner_code().unwrap().unwrap(), 0).unwrap().id;
        s.push(a, "lib", "phone", &batch("phone", 4, b"sealed")).unwrap();
        let id = "ef".repeat(32);
        s.put_file(a, &id, b"file").unwrap();
        (a, id)
    };
    let s = Store::open(dir.path()).unwrap();
    assert_eq!(s.account_of("alice").unwrap().unwrap().id, account);
    assert_eq!(s.positions(account, "lib", "phone").unwrap(), (4, 1));
    assert_eq!(s.get_file(account, &id).unwrap().unwrap(), b"file");
    assert_eq!(s.owner_code().unwrap(), None);
}

#[test]
fn an_account_made_first_is_deleted_with_everything_it_stored() {
    let (dir, s) = store();
    let (account, code) = s.new_account(0).unwrap();
    s.register("alice", &code, 0).unwrap();
    s.push(account, "lib", "phone", &batch("phone", 1, b"x")).unwrap();
    s.put_file(account, &"ab".repeat(32), b"f").unwrap();
    s.delete_account(account).unwrap();
    assert!(s.account_of("alice").unwrap().is_none());
    assert_eq!(s.used(account).unwrap(), 0);
    assert!(s.libraries(account).unwrap().is_empty());
    assert!(!dir.path().join("files").join(account.to_string()).exists());
    refused(s.register("alice", &code, 0), ErrorCode::BadCode);
}
