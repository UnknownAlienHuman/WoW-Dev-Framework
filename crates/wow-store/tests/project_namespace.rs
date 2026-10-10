//! One core native namespace-epoch check: stable logical identity survives host
//! roots and reopen, and tampered descriptors refuse without adopting a legacy
//! epoch.
use std::error::Error;
use wow_store::StoreErrorCode;
use wow_store::project::{
    ProjectStore, ProjectStoreNamespace, ProjectStoreNamespaceRequest, RecordCatalog,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn catalog() -> TestResult<RecordCatalog> {
    Ok(RecordCatalog::new(
        &["fixture.partition.v1"],
        &["fixture.owner.v1"],
    )?)
}

fn namespace(logical: &str, project: &str) -> TestResult<ProjectStoreNamespace> {
    Ok(ProjectStoreNamespace::new(ProjectStoreNamespaceRequest {
        logical_namespace: logical.into(),
        owner_project_id: project.into(),
    })?)
}

fn scratch(name: &str) -> TestResult<std::path::PathBuf> {
    let root = std::env::temp_dir().join(format!(
        "wow-store-namespace-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    Ok(root)
}

/// Descriptor IDs distinguish namespace from project, and the same compiled catalog
/// plus request yields one stable epoch identity across different host roots.
#[test]
fn namespace_identity_is_stable_across_host_roots_and_distinct_per_project() -> TestResult {
    let first = namespace("fixture.namespace", "fixture.project.one")?;
    let second = namespace("fixture.namespace", "fixture.project.two")?;
    // Distinct project identities keep distinct namespace IDs even under one
    // logical namespace.
    assert_ne!(first.id(), second.id());
    assert_eq!(first.logical_namespace(), second.logical_namespace());
    assert_ne!(first.owner_project_id(), second.owner_project_id());
    first.validate()?;
    second.validate()?;

    // Creation is deterministic per request, independent of the host root.
    let root = scratch("stable")?;
    let left = ProjectStore::create_with_namespace(&root, &first, catalog()?)?;
    let other = scratch("stable-other")?;
    let right = ProjectStore::create_with_namespace(&other, &first, catalog()?)?;
    assert_eq!(left.epoch().epoch_id(), right.epoch().epoch_id());
    assert_eq!(
        left.epoch()
            .namespace()
            .map(|ns| ns.id())
            .ok_or("missing admitted namespace")?,
        first.id()
    );
    assert_eq!(
        left.epoch().namespace().map(|ns| ns.logical_namespace()),
        Some(first.logical_namespace())
    );
    assert_eq!(
        left.epoch().namespace().map(|ns| ns.owner_project_id()),
        Some(first.owner_project_id())
    );
    // The epoch owner identity is the namespace ID, not the raw project label.
    assert_eq!(left.epoch().owner(), first.id().as_str());

    // Reopen retains the namespace, epoch and owner exactly.
    drop(left);
    let reopened = ProjectStore::open(&root, &catalog()?)?;
    assert_eq!(reopened.epoch().epoch_id(), right.epoch().epoch_id());
    assert_eq!(
        reopened.epoch().namespace().map(|ns| ns.id()),
        Some(first.id())
    );
    assert_eq!(reopened.epoch().owner(), first.id().as_str());
    drop(reopened);
    drop(right);
    std::fs::remove_dir_all(&root)?;
    std::fs::remove_dir_all(&other)?;
    Ok(())
}

/// Ordinary legacy create and open keep the legacy epoch, and a tampered namespace
/// descriptor, ID, schema or compiled policy manifest refuses on reopen.
#[test]
fn tampered_namespace_descriptors_refuse_while_legacy_epochs_remain_unchanged() -> TestResult {
    let held = namespace("fixture.namespace", "fixture.project.one")?;
    let root = scratch("tamper")?;
    let store = ProjectStore::create_with_namespace(&root, &held, catalog()?)?;
    let epoch_id = store.epoch().epoch_id().clone();
    let namespace_id = held.id().clone();
    let owner = store.epoch().owner().to_owned();
    drop(store);
    let reopened = ProjectStore::open(&root, &catalog()?)?;
    assert_eq!(reopened.epoch().epoch_id(), &epoch_id);
    drop(reopened);

    // A plain legacy epoch, created without a namespace, keeps its own recipe.
    let legacy_root = scratch("legacy")?;
    let legacy = ProjectStore::create_with_gc(&legacy_root, "fixture.legacy.owner", catalog()?)?;
    assert!(legacy.epoch().namespace().is_none());
    let legacy_epoch = legacy.epoch().epoch_id().clone();
    let legacy_owner = legacy.epoch().owner().to_owned();
    drop(legacy);
    let legacy_reopened = ProjectStore::open(&legacy_root, &catalog()?)?;
    assert_eq!(legacy_reopened.epoch().epoch_id(), &legacy_epoch);
    assert_eq!(legacy_reopened.epoch().owner(), legacy_owner);
    drop(legacy_reopened);

    // Tamper with the admitted namespace manifest bytes and refuse on reopen.
    let manifest = root.join("project-store-registry.json");
    let original = std::fs::read(&manifest)?;
    let mut value: serde_json::Value = serde_json::from_slice(&original)?;
    let object = value
        .as_object_mut()
        .ok_or("namespace epoch manifest is not an object")?;
    object.insert(
        "namespace".into(),
        serde_json::json!({
            "schema": "wow-store/project-store-identity/1",
            "request": {
                "logical_namespace": held.logical_namespace(),
                "owner_project_id": "fixture.substituted.project"
            },
            "id": namespace_id.as_str()
        }),
    );
    let tampered = wow_core::canonical_json_bytes(&value)?;
    std::fs::write(&manifest, &tampered)?;
    let refused = ProjectStore::open(&root, &catalog()?);
    match refused {
        Ok(_) => return Err("substituted namespace was admitted".into()),
        Err(error) => assert_eq!(
            error.code(),
            StoreErrorCode::IntegrityViolation,
            "substituted namespace must be an integrity violation"
        ),
    }
    std::fs::write(&manifest, &original)?;
    let restored = ProjectStore::open(&root, &catalog()?)?;
    assert_eq!(restored.epoch().epoch_id(), &epoch_id);
    assert_eq!(restored.epoch().owner(), owner);
    drop(restored);

    // A substituted catalog policy manifest refuses rather than admitting a
    // different compiled catalog into the same epoch.
    let policy = root.join("project-store-registry.json");
    let original = std::fs::read(&policy)?;
    let mut value: serde_json::Value = serde_json::from_slice(&original)?;
    let object = value
        .as_object_mut()
        .ok_or("namespace epoch manifest is not an object")?;
    object.insert(
        "catalog".into(),
        serde_json::json!({
            "checks": ["fixture.owner.v2"],
            "schemas": ["fixture.partition.v1"]
        }),
    );
    let tampered = wow_core::canonical_json_bytes(&value)?;
    std::fs::write(&policy, &tampered)?;
    let refused = ProjectStore::open(&root, &catalog()?);
    assert!(refused.is_err(), "substituted catalog policy must refuse");
    std::fs::write(&policy, &original)?;
    // The canonicalization/security profiles and exact identity mode are compiled
    // admission inputs. Editing the authoritative registry cannot override them.
    for field in [
        "canonicalization_version",
        "security_limit_digest",
        "schema",
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(&original)?;
        value[field] = serde_json::Value::String("fixture.substituted.policy".into());
        std::fs::write(&policy, wow_core::canonical_json_bytes(&value)?)?;
        assert!(ProjectStore::open(&root, &catalog()?).is_err());
    }
    std::fs::write(&policy, &original)?;
    let final_reopened = ProjectStore::open(&root, &catalog()?)?;
    assert_eq!(final_reopened.epoch().epoch_id(), &epoch_id);
    drop(final_reopened);

    std::fs::remove_dir_all(&root)?;
    std::fs::remove_dir_all(&legacy_root)?;
    Ok(())
}
