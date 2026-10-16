//! Exact retained catalog observation and owner validation, without retrieval.
use super::*;
use wow_search::reference::validate_reference_callable_shard;

pub(super) fn execute(
    request: &ReferenceShardRequest,
    validate: bool,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    if request.query.is_some() || request.page_policy.is_some() {
        return Err(invalid());
    }
    let operation = if validate {
        "search_reference_validate"
    } else {
        "search_reference_status"
    };
    let (id, digest, public_digest) = durable_identity(
        operation,
        &request.operation_id,
        &ShardIdentity {
            handle: &request.handle,
            query: None,
            store_limits: request.store_limits,
            max_output_bytes: request.max_output_bytes,
            lease_expires_after: request.lease_expires_after,
            privacy_scope: "local_metadata_only",
            page_policy: None,
        },
    )?;
    let configuration = StoreConfiguration::new_with_instance_identity(
        DURABLE_SEARCH_STORE_PROFILE_ID,
        request.store_limits,
    )
    .map_err(map_store_error)?;
    let mut store =
        Store::open_existing(&request.search_store, configuration).map_err(map_store_error)?;
    match start_durable_operation(&mut store, &id, &digest, request.max_output_bytes.get())? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }
    let mut batch = WriteBatch::new();
    let outcome = (|| {
        let handle = &request.handle;
        let shard_root = observe_root(
            &store,
            SEARCH_SHARD_CATALOG,
            &handle.logical_shard_id,
            &handle.object_id,
        )?;
        let source_root = observe_root(
            &store,
            SOURCE_ROOTS,
            &handle.logical_shard_id,
            &handle.reference_view_object_id,
        )?;
        let shard_present = store
            .object(&handle.object_id)
            .map_err(map_store_error)?
            .is_some();
        let reference_present = store
            .object(&handle.reference_view_object_id)
            .map_err(map_store_error)?
            .is_some();
        let exact_roots = shard_root == "exact" && source_root == "exact";
        let validation = if !exact_roots {
            serde_json::json!({"status":"not_evaluated", "reason":"exact_catalog_roots_not_admitted"})
        } else if !shard_present || !reference_present {
            serde_json::json!({"status":"not_evaluated", "reason":"exact_object_missing"})
        } else {
            let owner = PersistentReferenceStore::new(&mut store)
                .read_exact(&handle.reference_view_object_id)
                .map_err(reference_error)?
                .ok_or_else(unavailable)?;
            if owner.generation_id() != handle.reference_generation
                || owner.self_digest() != handle.reference_view_digest
            {
                return Err(mismatch());
            }
            let profile = handle_profile(handle)?;
            let lease = SearchStoreLeaseRequest::new(
                lease_id_for("reference-inspection", &public_digest)?,
                "wow-service-search",
                LogicalEpoch::new(request.lease_expires_after),
            )
            .map_err(map_store_error)?;
            let verdict = validate_reference_callable_shard(
                &mut store,
                &handle.logical_shard_id,
                &handle.object_id,
                &profile,
                &handle.reference_view_object_id,
                &owner,
                request.permission,
                lease,
                stop,
            )
            .map_err(map_search_error)?;
            serde_json::to_value(verdict).map_err(|_| invalid())?
        };
        if exact_roots {
            // Admission cannot change before the exact observation is recorded.
            fence_result(request, &mut batch)?;
        }
        durable_response(
            operation,
            &request.operation_id,
            &public_digest,
            &serde_json::json!({
                "status":validation["status"],
                "resource_closure":"closed",
                "handle":handle,
                "catalog":{
                    "shard_root":shard_root,
                    "reference_root":source_root,
                    "shard_object_present":shard_present,
                    "reference_object_present":reference_present,
                    "observation":"exact_handle_independent_reads",
                    "absent_root_reason":"retirement_or_never_admitted_not_distinguished"
                },
                "validation":validation,
                "required_missing_gates":["full_e4_golden_query_validation"],
                "full_e4_validation":"not_evaluated"
            }),
            request.max_output_bytes.get(),
        )
    })();
    finish(
        &mut store,
        &id,
        &digest,
        &request.operation_id,
        outcome,
        batch,
        request.max_output_bytes.get(),
        stop,
    )
}

fn observe_root(
    store: &Store,
    catalog: &str,
    path: &str,
    expected: &ObjectId,
) -> ServiceResult<&'static str> {
    let entry = store
        .catalog_entry(
            &CatalogName::new(catalog).map_err(map_store_error)?,
            &CatalogPath::new(path.to_owned()).map_err(map_store_error)?,
        )
        .map_err(map_store_error)?;
    Ok(match entry {
        Some(entry) if entry.object_id() == expected => "exact",
        Some(_) => "different_object",
        None => "absent",
    })
}
