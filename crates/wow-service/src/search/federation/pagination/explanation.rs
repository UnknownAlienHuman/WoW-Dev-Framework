//! Exact captured candidate read; it does not resolve owners or select intent.
use super::*;
use wow_store::ObjectId;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    session_object_id: ObjectId,
    result_set_id: String,
    candidate_id: String,
}
pub struct FederationExplainRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    input: Input,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
}
impl FederationExplainRequest {
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
        if input.schema != "wow-service/federated-explain-input/1" {
            return Err(invalid());
        }
        for (value, prefix) in [
            (&input.result_set_id, "federated-search-result-set:"),
            (&input.candidate_id, "federated-search-candidate:"),
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
    request: &FederationExplainRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (id, digest, public_digest) = durable_identity(
        "search_federated_explain",
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
        for (root, target) in roots(&request.input.session_object_id, &session) {
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
        let primary = session.primary.open(
            &mut store,
            &public_digest,
            "federated-explain-primary",
            request.lease_expires_after,
            request.permission,
            stop,
        )?;
        let explained = (|| {
            let blizzard = session
                .blizzard
                .as_ref()
                .map(|binding| {
                    binding.open(
                        &mut store,
                        &public_digest,
                        "federated-explain-blizzard",
                        request.lease_expires_after,
                        request.permission,
                        stop,
                    )
                })
                .transpose()?;
            let reference_result = (|| {
                let reference = reference::open_retained_shard(
                    &session.reference,
                    &mut store,
                    &public_digest,
                    request.lease_expires_after,
                    request.permission,
                    stop,
                )?;
                let result = wow_search::federation::explain_retained_source_federation(
                    primary.view(),
                    blizzard.as_ref().map(|owner| owner.view()),
                    &reference,
                    &retained,
                    &session.query,
                    &request.input.result_set_id,
                    &request.input.candidate_id,
                    request.max_output_bytes.get(),
                    stop,
                )
                .map_err(map_search_error);
                reference.release(&mut store).map_err(map_search_error)?;
                result
            })();
            if let Some(blizzard) = blizzard {
                preserve_named_closure_errors(
                    reference_result,
                    blizzard.release(&mut store).map_err(map_search_error),
                    "blizzard",
                )
            } else {
                reference_result
            }
        })();
        let closed = primary.release(&mut store).map_err(map_search_error);
        let explained = preserve_closure_errors(explained, closed)?;
        let value: serde_json::Value =
            serde_json::from_slice(explained.canonical_bytes()).map_err(|_| invalid())?;
        durable_response(
            "search_federated_explain",
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
