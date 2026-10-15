use super::*;
#[cfg(unix)]
use std::path::PathBuf;
use std::{error::Error, time::SystemTime};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const IDENTITY_PROFILE: &str = "wow-search.durable-shard-store.v2";

fn identity_configuration() -> StoreResult<StoreConfiguration> {
    StoreConfiguration::new_with_instance_identity(IDENTITY_PROFILE, crate::StoreLimits::default())
}

fn root() -> TestResult<std::path::PathBuf> {
    let root = std::env::temp_dir().join(format!(
        "wow-store-instance-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    Ok(root)
}

#[test]
fn identity_is_owned_stable_and_copy_bound_without_widening_legacy_profiles() -> TestResult {
    let root = root()?;
    let store_a_path = root.join("a.sqlite");
    let store_b_path = root.join("b.sqlite");
    let store_copy_path = root.join("a-copy.sqlite");
    let legacy_v2_path = root.join("legacy-v2.sqlite");
    let legacy_v1_path = root.join("legacy-v1.sqlite");
    let identity_configuration = identity_configuration()?;

    let (store_a_id, store_b_id) = {
        let store_a = Store::open(&store_a_path, identity_configuration.clone())?;
        let store_b = Store::open(&store_b_path, identity_configuration.clone())?;
        let store_a_id = store_a.store_instance_id()?.clone();
        let store_b_id = store_b.store_instance_id()?.clone();
        assert_ne!(store_a_id, store_b_id);
        (store_a_id, store_b_id)
    };

    {
        let reopened = Store::open(&store_a_path, identity_configuration.clone())?;
        assert_eq!(reopened.store_instance_id()?, &store_a_id);
    }

    fs::copy(&store_a_path, &store_copy_path)?;
    let copied_id: String = rusqlite::Connection::open(&store_copy_path)?.query_row(
        "SELECT value FROM store_meta WHERE key='store_instance_id'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(copied_id, store_a_id.as_str());
    let copy_open = Store::open(&store_copy_path, identity_configuration.clone());
    let copy_error = match copy_open {
        Ok(_) => return Err("a byte copy was admitted as an independent store".into()),
        Err(error) => error,
    };
    assert_eq!(
        copy_error.code(),
        StoreErrorCode::StoreInstanceIdentityMismatch
    );

    let legacy_v2_configuration =
        StoreConfiguration::new(IDENTITY_PROFILE, crate::StoreLimits::default())?;
    {
        let legacy = Store::open(&legacy_v2_path, legacy_v2_configuration)?;
        let error = match legacy.store_instance_id() {
            Err(error) => error,
            Ok(_) => return Err("legacy identity was unexpectedly available".into()),
        };
        assert_eq!(
            error.code(),
            StoreErrorCode::StoreInstanceIdentityUnavailable
        );
    }
    let identity_open = Store::open(&legacy_v2_path, identity_configuration);
    let identity_error = match identity_open {
        Ok(_) => return Err("identity-aware profile adopted legacy bytes".into()),
        Err(error) => error,
    };
    assert_eq!(
        identity_error.code(),
        StoreErrorCode::StoreInstanceIdentityUnavailable
    );

    let legacy_v1_configuration = StoreConfiguration::new(
        "wow-search.durable-shard-store.v1",
        crate::StoreLimits::default(),
    )?;
    {
        drop(Store::open(
            &legacy_v1_path,
            legacy_v1_configuration.clone(),
        )?);
        let legacy_reopened = Store::open(&legacy_v1_path, legacy_v1_configuration)?;
        let error = match legacy_reopened.store_instance_id() {
            Err(error) => error,
            Ok(_) => return Err("legacy profile identity was unexpectedly available".into()),
        };
        assert_eq!(
            error.code(),
            StoreErrorCode::StoreInstanceIdentityUnavailable
        );
    }

    assert_ne!(store_a_id, store_b_id);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[cfg(unix)]
struct OwnedTestRoot(PathBuf);

#[cfg(unix)]
impl Drop for OwnedTestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
struct AbaPaths {
    store: PathBuf,
    copy: PathBuf,
    original: PathBuf,
    moved_copy: PathBuf,
    phase: u8,
}

#[cfg(unix)]
static ABA_PATHS: std::sync::Mutex<Option<AbaPaths>> = std::sync::Mutex::new(None);

#[cfg(unix)]
fn test_hook_error() -> crate::StoreError {
    crate::StoreError::new(
        StoreErrorCode::DatabaseUnavailable,
        "Unix admission ABA test hook failed",
    )
}

#[cfg(unix)]
fn swap_test_paths(paths: &AbaPaths, point: super::AdmissionTestPoint) -> StoreResult<()> {
    let result = match point {
        super::AdmissionTestPoint::AfterReadOnlyProbe if paths.phase == 0 => {
            fs::rename(&paths.store, &paths.original)
                .and_then(|()| fs::rename(&paths.copy, &paths.store))
        }
        super::AdmissionTestPoint::AfterWritableOpen if paths.phase == 1 => {
            fs::rename(&paths.store, &paths.moved_copy)
                .and_then(|()| fs::rename(&paths.original, &paths.store))
        }
        _ => return Err(test_hook_error()),
    };
    result.map_err(|_| test_hook_error())
}

#[cfg(unix)]
pub(super) fn admission_checkpoint(
    path: &std::path::Path,
    point: super::AdmissionTestPoint,
) -> StoreResult<()> {
    let mut active = ABA_PATHS.lock().map_err(|_| test_hook_error())?;
    let Some(paths) = active.as_mut() else {
        return Ok(());
    };
    if paths.store.as_path() != path {
        return Ok(());
    }
    swap_test_paths(paths, point)?;
    paths.phase = match point {
        super::AdmissionTestPoint::AfterReadOnlyProbe => 1,
        super::AdmissionTestPoint::AfterWritableOpen => 2,
    };
    Ok(())
}

#[cfg(unix)]
#[test]
fn path_aba_is_rejected_by_connected_sqlite_write_admission() -> TestResult {
    let root = root()?;
    let _owned_root = OwnedTestRoot(root.clone());
    let store_path = root.join("store.sqlite");
    let copy_path = root.join("store-copy.sqlite");
    let original_path = root.join("store-original.sqlite");
    let moved_copy_path = root.join("store-copy-moved.sqlite");
    let configuration = identity_configuration()?;

    let original_id = {
        let store = Store::open(&store_path, configuration.clone())?;
        store.store_instance_id()?.clone()
    };
    fs::copy(&store_path, &copy_path)?;

    let marker_id = OperationId::new("aba-copy-marker")?;
    let marker_digest = RequestDigest::new(format!("sha256:{}", "0".repeat(64)))?;
    {
        let copy = rusqlite::Connection::open(&copy_path)?;
        copy.execute(
            "INSERT INTO store_operations(
                 operation_id, request_digest, state, result_object_id
             ) VALUES (?1, ?2, 'prepared', NULL)",
            [marker_id.as_str(), marker_digest.as_str()],
        )?;
    }

    {
        let mut active = ABA_PATHS.lock().map_err(|_| "ABA hook lock poisoned")?;
        if active.is_some() {
            return Err("another ABA test hook is active".into());
        }
        *active = Some(AbaPaths {
            store: store_path.clone(),
            copy: copy_path.clone(),
            original: original_path,
            moved_copy: moved_copy_path.clone(),
            phase: 0,
        });
    }
    let opened = Store::open(&store_path, configuration.clone());
    let phase = {
        let mut active = ABA_PATHS.lock().map_err(|_| "ABA hook lock poisoned")?;
        let phase = active.as_ref().map(|paths| paths.phase);
        *active = None;
        phase
    };
    assert_eq!(phase, Some(2), "both deterministic path swaps must run");
    let open_error = match opened {
        Ok(_) => return Err("Store exposed an identity for the moved copied file".into()),
        Err(error) => error,
    };
    assert_eq!(
        open_error.code(),
        StoreErrorCode::StoreInstanceIdentityMismatch
    );
    assert_eq!(
        open_error.message(),
        "SQLite reports that the opened identity store moved during admission"
    );

    let write_id = OperationId::new("aba-copy-write")?;
    let copy = rusqlite::Connection::open(&moved_copy_path)?;
    let marker_count: i64 = copy.query_row(
        "SELECT COUNT(*) FROM store_operations WHERE operation_id = ?1",
        [marker_id.as_str()],
        |row| row.get(0),
    )?;
    let write_count: i64 = copy.query_row(
        "SELECT COUNT(*) FROM store_operations WHERE operation_id = ?1",
        [write_id.as_str()],
        |row| row.get(0),
    )?;
    assert_eq!(
        marker_count, 1,
        "the writable SQLite handle had opened copy B"
    );
    assert_eq!(
        write_count, 0,
        "admission must not begin a Search operation"
    );

    let original = Store::open(&store_path, configuration)?;
    assert_eq!(original.store_instance_id()?, &original_id);
    assert!(original.operation(&marker_id)?.is_none());

    Ok(())
}
