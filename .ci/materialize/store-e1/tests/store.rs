use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Barrier};
use std::thread;

use tempfile::TempDir;
use wow_store::{
    BlobId, RefName, RefUpdate, SnapshotEntryInput, Store, StoreErrorCode, StoreLimits,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn store() -> TestResult<(TempDir, Store)> {
    let root = TempDir::new()?;
    let store = Store::open(root.path(), StoreLimits::default())?;
    Ok((root, store))
}

fn snapshot(store: &Store, path: &str, bytes: &[u8]) -> TestResult<wow_store::StoreSnapshot> {
    let blob = store.put_blob(bytes)?;
    Ok(store.commit_snapshot([SnapshotEntryInput::new(path, blob)?])?)
}

#[test]
fn blobs_and_snapshots_are_content_addressed_and_reopen_cleanly() -> TestResult {
    let (root, store) = store()?;
    let first = store.put_blob(b"same bytes")?;
    let second = store.put_blob(b"same bytes")?;
    assert_eq!(first, second);
    assert_eq!(store.read_blob(&first)?, b"same bytes");

    let left = store.commit_snapshot([
        SnapshotEntryInput::new("b.lua", store.put_blob(b"b")?)?,
        SnapshotEntryInput::new("a.lua", store.put_blob(b"a")?)?,
    ])?;
    let right = store.commit_snapshot([
        SnapshotEntryInput::new("a.lua", store.put_blob(b"a")?)?,
        SnapshotEntryInput::new("b.lua", store.put_blob(b"b")?)?,
    ])?;
    assert_eq!(left, right);
    assert_eq!(left.entries()[0].path(), "a.lua");

    drop(store);
    let reopened = Store::open(root.path(), StoreLimits::default())?;
    assert_eq!(reopened.load_snapshot(left.snapshot_id())?, left);
    Ok(())
}

#[test]
fn invalid_paths_duplicates_missing_objects_and_limits_fail_closed() -> TestResult {
    let (_root, store) = store()?;
    let blob = store.put_blob(b"value")?;
    for path in ["", "/absolute", "../escape", "a//b", "a\\b", "C:drive"] {
        assert!(SnapshotEntryInput::new(path, blob.clone()).is_err(), "{path}");
    }
    let duplicate = store.commit_snapshot([
        SnapshotEntryInput::new("a.lua", blob.clone())?,
        SnapshotEntryInput::new("a.lua", blob.clone())?,
    ]);
    assert_eq!(
        duplicate.err().map(|error| error.code()),
        Some(StoreErrorCode::InvalidPath)
    );

    let missing = BlobId::parse(format!("sha256:{}", "f".repeat(64)))?;
    let result = store.commit_snapshot([SnapshotEntryInput::new("missing.lua", missing)?]);
    assert_eq!(
        result.err().map(|error| error.code()),
        Some(StoreErrorCode::ObjectMissing)
    );

    let root = TempDir::new()?;
    let limited = Store::open(root.path(), StoreLimits::new(4, 2, 8, 1024)?)?;
    assert_eq!(
        limited.put_blob(b"12345").err().map(|error| error.code()),
        Some(StoreErrorCode::InputLimitExceeded)
    );
    Ok(())
}

#[test]
fn ref_journal_is_compare_and_swap_and_persistent() -> TestResult {
    let (root, store) = store()?;
    let name = RefName::parse("mainline.current")?;
    let first = snapshot(&store, "one.lua", b"one")?;
    let second = snapshot(&store, "two.lua", b"two")?;

    assert_eq!(store.read_ref(&name)?, None);
    assert_eq!(
        store.update_ref(&name, None, first.snapshot_id())?,
        RefUpdate::Created
    );
    assert_eq!(store.read_ref(&name)?.as_ref(), Some(first.snapshot_id()));
    assert_eq!(
        store.update_ref(&name, Some(first.snapshot_id()), first.snapshot_id())?,
        RefUpdate::NoChange
    );
    assert_eq!(
        store.update_ref(&name, Some(first.snapshot_id()), second.snapshot_id())?,
        RefUpdate::Advanced
    );
    assert_eq!(
        store
            .update_ref(&name, Some(first.snapshot_id()), first.snapshot_id())
            .err()
            .map(|error| error.code()),
        Some(StoreErrorCode::RefConflict)
    );

    drop(store);
    let reopened = Store::open(root.path(), StoreLimits::default())?;
    assert_eq!(
        reopened.read_ref(&name)?.as_ref(),
        Some(second.snapshot_id())
    );
    Ok(())
}

#[test]
fn incomplete_trailing_ref_record_is_ignored_but_complete_corruption_rejects() -> TestResult {
    let (root, store) = store()?;
    let name = RefName::parse("stable")?;
    let first = snapshot(&store, "one.lua", b"one")?;
    store.update_ref(&name, None, first.snapshot_id())?;
    let journal = root.path().join("refs/stable.jsonl");
    OpenOptions::new()
        .append(true)
        .open(&journal)?
        .write_all(b"{partial")?;
    assert_eq!(store.read_ref(&name)?.as_ref(), Some(first.snapshot_id()));
    OpenOptions::new()
        .append(true)
        .open(&journal)?
        .write_all(b"}\n")?;
    assert_eq!(
        store.read_ref(&name).err().map(|error| error.code()),
        Some(StoreErrorCode::RefCorrupt)
    );
    Ok(())
}

#[test]
fn concurrent_compare_and_swap_allows_only_one_advance() -> TestResult {
    let (root, store) = store()?;
    let name = RefName::parse("concurrent")?;
    let base = snapshot(&store, "base.lua", b"base")?;
    let left = snapshot(&store, "left.lua", b"left")?;
    let right = snapshot(&store, "right.lua", b"right")?;
    store.update_ref(&name, None, base.snapshot_id())?;
    drop(store);

    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for next in [left.snapshot_id().clone(), right.snapshot_id().clone()] {
        let root = root.path().to_path_buf();
        let name = name.clone();
        let base = base.snapshot_id().clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            let store = Store::open(root, StoreLimits::default()).expect("open concurrent store");
            barrier.wait();
            store.update_ref(&name, Some(&base), &next)
        }));
    }
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().expect("writer thread"))
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| {
                result
                    .as_ref()
                    .err()
                    .is_some_and(|error| error.code() == StoreErrorCode::RefConflict)
            })
            .count(),
        1
    );
    Ok(())
}

#[test]
fn deserialization_rejects_malformed_ids_before_path_sharding() -> TestResult {
    assert!(serde_json::from_str::<BlobId>("\"bad\"").is_err());
    assert!(
        serde_json::from_str::<wow_store::SnapshotId>("\"store-snapshot:bad\"").is_err()
    );
    assert!(serde_json::from_str::<RefName>("\"../escape\"").is_err());
    Ok(())
}
