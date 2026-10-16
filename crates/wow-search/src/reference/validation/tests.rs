use super::*;
use wow_core::{
    ContentDigest, ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId, SchemaVersionEntry,
    SourceKind,
};
use wow_reference::{
    native::{ingest_document, source_digest},
    native_view::{NativeCallableCorpus, project_callables},
};
use wow_store::WriteBatch;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn native_validation_distinguishes_verdict_failure_and_checked_closure() -> TestResult {
    let stop = AtomicBool::new(false);
    let revision = "1111111111111111111111111111111111111111";
    let input = r#"APIDocumentation:AddDocumentationTable({Name="Inspect",Type="System",Namespace="C_Inspect",Environment="All",Functions={{Name="Read",Type="Function"}}})"#;
    let document = ingest_document(
        revision,
        "Inspect.lua",
        input,
        &source_digest(input.as_bytes()),
        &stop,
    )?;
    let native = project_callables(
        &[document],
        "Mainline",
        ReferenceGenerationId::derive(&"search-validation-fixture")?,
        NativeCallableCorpus::explicit_partial(),
        &stop,
    )?;
    let bounds: SearchBounds = serde_json::from_str(
        r#"{"max_documents":8,"max_document_id_bytes":256,"max_owner_metadata_label_bytes":512,"max_literal_query_bytes":64,"max_candidate_results":8,"max_owner_evidence_records":128,"max_owner_manifest_bytes":16384,"max_database_bytes":256000,"max_store_object_bytes":800000}"#,
    )?;
    let search_profile = ReferenceSearchProfile::new(bounds)?;
    let limits = StoreLimits::new(1000000, 8, 8, 128, 8)?;
    let mut store =
        Store::open_in_memory(wow_store::StoreConfiguration::new_with_instance_identity(
            "reference-validation-fixture",
            limits,
        )?)?;
    let profile = ProfileIdentityBuilder::new(
        "profile:fixture:search-validation".parse()?,
        ProfileKind::Fixture,
        "retail",
        1,
        SourceKind::SyntheticFixture,
        revision,
        ContentDigest::from_bytes([9; 32]),
    )
    .fixture_scope("native search validation fixture")
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:native-validation-fixture".parse()?,
        "1.0.0".parse()?,
    )])
    .build()?;
    let reference_id = ObjectId::new(format!("store-object:sha256:{}", "a".repeat(64)))?;
    let binding = ReferenceSearchBinding::new(
        profile,
        &store,
        reference_id.clone(),
        true,
        revision,
        Vec::new(),
    )?;
    let prepared = prepare_reference_callable_shard(
        &native,
        binding,
        &search_profile,
        LocalMetadataPermission::authorize_local_metadata_only(),
        limits,
        &stop,
    )?;
    let mut bad_manifest = prepared.manifest.clone();
    let logical = prepared.logical_id().to_owned();
    let object = prepared.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(prepared.into_object())?;
    store.commit(batch)?;
    let lease = |name| {
        SearchStoreLeaseRequest::new(
            LeaseId::new(name)?,
            "native-validation-test",
            LogicalEpoch::new(100),
        )
    };
    let result = validate_reference_callable_shard(
        &mut store,
        &logical,
        &object,
        &search_profile,
        &reference_id,
        &native.view,
        LocalMetadataPermission::authorize_local_metadata_only(),
        lease("validation-positive")?,
        &stop,
    )?;
    let report = serde_json::to_value(result)?;
    assert_eq!(report["status"], "valid");
    assert_eq!(report["shard"]["document_count"], 1);
    assert_eq!(report["shard"]["full_e4_validation"], "not_evaluated");
    assert_eq!(report["shard"]["physical_read_only_validated"], true);
    // A missing physical object and cancellation are not completed Invalid.
    assert!(
        validate_reference_callable_shard(
            &mut store,
            &logical,
            &reference_id,
            &search_profile,
            &reference_id,
            &native.view,
            LocalMetadataPermission::authorize_local_metadata_only(),
            lease("validation-missing")?,
            &stop
        )
        .is_err()
    );
    assert!(matches!(
        validate_reference_callable_shard(
            &mut store,
            &logical,
            &object,
            &search_profile,
            &reference_id,
            &native.view,
            LocalMetadataPermission::authorize_local_metadata_only(),
            lease("validation-cancelled")?,
            &AtomicBool::new(true)
        ),
        Err(SearchError::Cancelled)
    ));
    // The physical artifact remains valid. Only the domain owner's exact
    // privacy binding is contradicted, so its validator can attest Invalid.
    bad_manifest.privacy_scope = "wrong_scope".into();
    let bytes = wow_core::canonical_json_bytes(&bad_manifest)?;
    let invalid_logical = format!("search-shard:sha256:{}", hex(&Sha256::digest(&bytes)));
    let rows = bad_manifest
        .documents
        .iter()
        .map(|doc| {
            Ok(SearchStoreDocument::new(
                doc.entity_key.as_str(),
                documentation::row_text(doc, bounds)?,
            ))
        })
        .collect::<SearchResult<Vec<_>>>()?;
    let physical_profile =
        SearchStoreProfile::new(search_profile.digest.to_string(), bounds.physical()?)?;
    let prepared = SearchStore::prepare(
        physical_profile.clone(),
        invalid_logical.as_str(),
        &rows,
        &bytes,
        limits,
        &stop,
    )?;
    let invalid_object = prepared.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(prepared.into_object())?;
    store.commit(batch)?;
    assert!(matches!(
        validate_reference_callable_shard(
            &mut store,
            &invalid_logical,
            &invalid_object,
            &search_profile,
            &reference_id,
            &native.view,
            LocalMetadataPermission::authorize_local_metadata_only(),
            lease("validation-invalid")?,
            &stop
        )?,
        ReferenceShardValidation::Invalid { .. }
    ));
    // Query admission uses the same validator and preserves its old error.
    assert!(matches!(
        open_reference_callable_shard(
            &mut store,
            &invalid_logical,
            &invalid_object,
            &search_profile,
            &reference_id,
            &native.view,
            LocalMetadataPermission::authorize_local_metadata_only(),
            lease("validation-query-invalid")?,
            &stop
        ),
        Err(SearchError::OwnerEvidenceInvalid)
    ));
    let physical = SearchStore::open_exact(
        &mut store,
        physical_profile,
        invalid_logical.as_str(),
        &invalid_object,
        lease("validation-cleanup-unknown")?,
        &stop,
    )?;
    let actual_lease = physical.lease().lease_id().clone();
    store.release_lease(&actual_lease, physical.lease().holder())?;
    assert!(
        matches!(ValidationFailure::Invalid("owner_manifest_binding").after_release(physical, &mut store), ValidationFailure::Failed(SearchError::ShardLeaseCleanupOutcomeUnknown { lease_id, .. }) if lease_id == actual_lease)
    );
    let gc = store.collect_garbage(LogicalEpoch::new(1), 8)?;
    assert_eq!(gc.expired_leases_deleted(), 0);
    assert!(gc.deleted().contains(&object));
    assert!(gc.deleted().contains(&invalid_object));
    Ok(())
}
