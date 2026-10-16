//! Exact captured candidate read; it does not resolve owners or select intent.
use super::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    session_object_id: ObjectId,
    result_set_id: String,
    candidate_id: String,
}
pub struct EntityExplainRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    input: Input,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
}
impl EntityExplainRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        input_json: &[u8],
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || input_json.len() > 4096 || lease_expires_after == 0 {
            return Err(invalid());
        }
        let input: Input = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/project-entity-explain-input/1" {
            return Err(invalid());
        }
        for (value, prefix) in [
            (&input.result_set_id, "project-entity-result-set:"),
            (&input.candidate_id, "project-entity-candidate:"),
        ] {
            RequestDigest::new(value.strip_prefix(prefix).ok_or_else(invalid)?.to_owned())
                .map_err(|_| invalid())?;
        }
        Ok(Self {
            operation_id,
            search_store,
            input,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
        })
    }
}

pub(in crate::search) fn explain_result(
    request: &EntityExplainRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (id, digest, public_digest) = durable_identity(
        "search_project_entities_explain",
        &request.operation_id,
        &(
            &request.input,
            request.store_limits,
            request.max_output_bytes,
            request.lease_expires_after,
            "local_metadata_only",
        ),
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
        let session = load_session(&store, &request.input.session_object_id)?;
        if session.store_limits != request.store_limits {
            return Err(mismatch());
        }
        for (root, target) in roots(&request.input.session_object_id, &session.object_id) {
            let (catalog, path) = root_key(&request.input.session_object_id, root)?;
            batch
                .add_catalog_mutation(CatalogMutation::set(
                    catalog,
                    path,
                    CatalogExpectation::Exact(target.clone()),
                    target,
                ))
                .map_err(map_store_error)?;
        }
        let retained = canonical_json_bytes(&session.manifest).map_err(|_| invalid())?;
        let profile = EntitySearchProfile::new(session.bounds).map_err(map_search_error)?;
        let lease = SearchStoreLeaseRequest::new(
            lease_id_for("entity-explain", &public_digest)?,
            "wow-service-search",
            LogicalEpoch::new(request.lease_expires_after),
        )
        .map_err(map_store_error)?;
        let shard = open_project_entity_shard(
            &mut store,
            &session.logical_id,
            &session.object_id,
            &session.generation,
            &profile,
            request.permission,
            lease,
            stop,
        )
        .map_err(map_search_error)?;
        let explained = shard
            .explain_retained_result(
                &retained,
                &session.query,
                &request.input.result_set_id,
                &request.input.candidate_id,
                request.max_output_bytes.get(),
                stop,
            )
            .map_err(map_search_error);
        // Close even when validation/refusal/cancellation failed. A cleanup
        // failure supersedes the read outcome with its reconciliation identity.
        shard.release(&mut store).map_err(map_search_error)?;
        let explained = explained?;
        let value: serde_json::Value =
            serde_json::from_slice(explained.canonical_bytes()).map_err(|_| invalid())?;
        durable_response(
            "search_project_entities_explain",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({
                "status":explained.status(), "resource_closure":"closed",
                "session_object_id":request.input.session_object_id,
                "explanation":value
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
