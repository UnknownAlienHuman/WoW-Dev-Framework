//! Four-root captured federation sessions, authenticated by prior completion.
use super::source_slot::SourceRetainedBinding;
use super::*;
mod explanation;
pub use explanation::FederationExplainRequest;
pub(in crate::search) use explanation::explain_result;
use wow_search::federation::{
    FederationPage, FederationPageCursor, FederationQueryResult, page_retained_source_federation,
};
const SESSION_SCHEMA: &str = "wow-service/federated-search-session/1";
const PLATFORM_SESSION_SCHEMA: &str = "wow-service/platform-federated-search-session/1";
const OBSERVATION_SESSION_SCHEMA: &str = "wow-service/observed-source-federated-search-session/1";
const SESSION_KIND: &str = "wow.search.federated-result-session";
const ROOT_CATALOG: &str = "wow-search.federated-result-sessions";
const TOKEN_SCHEMA: &str = "wow-service/federated-search-continuation/1";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Continuation {
    schema: String,
    session_object_id: wow_store::ObjectId,
    receipt_operation_id: String,
    receipt_request_digest: String,
    cursor: FederationPageCursor,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    schema: String,
    original_request_digest: String,
    start_operation_id: String,
    primary: SourceRetainedBinding,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_binding"
    )]
    blizzard: Option<SourceRetainedBinding>,
    reference: reference::Handle,
    query: FederationQueryRequest,
    policy: FederationPagePolicy,
    manifest: serde_json::Value,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    privacy_scope: String,
}
pub struct FederationContinueRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    continuation: Continuation,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    permission: LocalMetadataPermission,
}
impl FederationContinueRequest {
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        continuation_json: &[u8],
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || continuation_json.len() > 16384 {
            return Err(invalid());
        }
        let continuation: Continuation =
            serde_json::from_slice(continuation_json).map_err(|_| invalid())?;
        if continuation.schema != TOKEN_SCHEMA {
            return Err(invalid());
        }
        StoreOperationId::new(continuation.receipt_operation_id.clone())
            .map_err(map_store_error)?;
        RequestDigest::new(continuation.receipt_request_digest.clone()).map_err(map_store_error)?;
        Ok(Self {
            operation_id,
            search_store,
            continuation,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}
pub struct FederationResultReleaseRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    session_object_id: wow_store::ObjectId,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}
impl FederationResultReleaseRequest {
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        session_object_id: &str,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        _permission: LocalMetadataPermission,
    ) -> ServiceResult<Self> {
        if !search_store.is_absolute() {
            return Err(invalid());
        }
        Ok(Self {
            operation_id,
            search_store,
            session_object_id: wow_store::ObjectId::new(session_object_id.to_owned())
                .map_err(map_store_error)?,
            store_limits,
            max_output_bytes,
        })
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn start(
    request: &FederationRequest,
    policy: &FederationPagePolicy,
    result: &FederationQueryResult,
    digest: &str,
    store: &mut Store,
    batch: &mut WriteBatch,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let page = result.page(policy, None, stop).map_err(map_search_error)?;
    let session = Session {
        schema: session_schema(&request.query, request.blizzard.is_some()).into(),
        original_request_digest: digest.into(),
        start_operation_id: request.operation_id.as_str().into(),
        primary: request.primary.retained(),
        blizzard: request.blizzard.as_ref().map(|source| source.retained()),
        reference: reference::retained_handle(&request.reference),
        query: request.query.clone(),
        policy: policy.clone(),
        manifest: serde_json::from_slice(result.canonical_bytes()).map_err(|_| invalid())?,
        store_limits: request.store_limits,
        max_output_bytes: request.max_output_bytes,
        lease_expires_after: request.lease_expires_after,
        privacy_scope: "local_metadata_only".into(),
    };
    let object =
        PendingObject::from_json(SESSION_KIND, 1, &session, store.configuration().limits())
            .map_err(map_store_error)?;
    let id = object.object_id().clone();
    batch.add_object(object).map_err(map_store_error)?;
    for (root, target) in roots(&id, &session) {
        let (catalog, path) = root_key(&id, root)?;
        batch
            .add_catalog_mutation(CatalogMutation::set(
                catalog,
                path,
                CatalogExpectation::Absent,
                target,
            ))
            .map_err(map_store_error)?;
    }
    response(
        "search_federated_page",
        &request.operation_id,
        digest,
        id,
        page,
        result.status(),
        request.max_output_bytes.get(),
    )
}
pub(in crate::search) fn continue_result(
    request: &FederationContinueRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/federated-search-continue/1",
        &request.operation_id,
        &(
            &request.continuation,
            request.store_limits,
            request.max_output_bytes,
            "local_metadata_only",
        ),
    )?;
    let mut store = open_durable_search_store(&request.search_store, request.store_limits)?;
    match start_durable_operation(
        &mut store,
        &operation_id,
        &digest,
        request.max_output_bytes.get(),
    )? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }
    let mut batch = WriteBatch::new();
    let outcome = (|| {
        let prior = store
            .operation(
                &StoreOperationId::new(request.continuation.receipt_operation_id.clone())
                    .map_err(map_store_error)?,
            )
            .map_err(map_store_error)?
            .ok_or_else(unavailable)?;
        if prior.state() != StoreOperationState::Completed
            || prior.request_digest().as_str() != request.continuation.receipt_request_digest
        {
            return Err(mismatch());
        }
        let bytes = read_completed_response(&store, &prior, request.max_output_bytes.get())?;
        let prior: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if prior
            .get("payload")
            .and_then(|payload| payload.get("continuation"))
            != Some(&serde_json::to_value(&request.continuation).map_err(|_| invalid())?)
        {
            return Err(mismatch());
        }
        let session = load_session(&store, &request.continuation.session_object_id)?;
        if session.store_limits != request.store_limits
            || session.max_output_bytes != request.max_output_bytes
        {
            return Err(mismatch());
        }
        let retained = canonical_json_bytes(&session.manifest).map_err(|_| invalid())?;
        request
            .continuation
            .cursor
            .check_replay_budget(&session.policy, retained.len() as u64)
            .map_err(map_search_error)?;
        for (root, target) in roots(&request.continuation.session_object_id, &session) {
            let (catalog, path) = root_key(&request.continuation.session_object_id, root)?;
            batch
                .add_catalog_mutation(CatalogMutation::set(
                    catalog,
                    path,
                    CatalogExpectation::Exact(target.clone()),
                    target,
                ))
                .map_err(map_store_error)?;
        }
        let primary = session.primary.open(
            &mut store,
            &public_digest,
            "federated-page-primary",
            session.lease_expires_after,
            request.permission,
            stop,
        )?;
        // Always release primary, including any later Reference open/validate/
        // release error. The inner result cannot return out of the outer scope.
        let page_result = (|| {
            let blizzard = session
                .blizzard
                .as_ref()
                .map(|binding| {
                    binding.open(
                        &mut store,
                        &public_digest,
                        "federated-page-blizzard",
                        session.lease_expires_after,
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
                    session.lease_expires_after,
                    request.permission,
                    stop,
                )?;
                let page = page_retained_source_federation(
                    primary.view(),
                    blizzard.as_ref().map(|b| b.view()),
                    &reference,
                    &retained,
                    &session.query,
                    &session.policy,
                    Some(&request.continuation.cursor),
                    stop,
                )
                .map_err(map_search_error);
                reference.release(&mut store).map_err(map_search_error)?;
                page
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
        let primary_close = primary.release(&mut store).map_err(map_search_error);
        let page = preserve_closure_errors(page_result, primary_close)?;
        let status = session
            .manifest
            .get("payload")
            .and_then(|payload| payload.get("status"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(invalid)?;
        response(
            "search_federated_continue",
            &request.operation_id,
            &public_digest,
            request.continuation.session_object_id.clone(),
            page,
            status,
            request.max_output_bytes.get(),
        )
    })();
    finish(
        &mut store,
        &operation_id,
        &digest,
        &request.operation_id,
        outcome,
        batch,
        request.max_output_bytes.get(),
        stop,
    )
}
pub(in crate::search) fn release_result(
    request: &FederationResultReleaseRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/federated-search-release/1",
        &request.operation_id,
        &(
            &request.session_object_id,
            request.store_limits,
            request.max_output_bytes,
        ),
    )?;
    let mut store = open_durable_search_store(&request.search_store, request.store_limits)?;
    match start_durable_operation(
        &mut store,
        &operation_id,
        &digest,
        request.max_output_bytes.get(),
    )? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }
    let mut batch = WriteBatch::new();
    let outcome = (|| {
        let session = load_session(&store, &request.session_object_id)?;
        for (root, target) in roots(&request.session_object_id, &session) {
            let (catalog, path) = root_key(&request.session_object_id, root)?;
            batch
                .add_catalog_mutation(CatalogMutation::delete(catalog, path, target))
                .map_err(map_store_error)?;
        }
        durable_response(
            "search_federated_results_release",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":"released","resource_closure":"closed","session_object_id":request.session_object_id,"completed_response_replay":"retained"}),
            request.max_output_bytes.get(),
        )
    })();
    finish(
        &mut store,
        &operation_id,
        &digest,
        &request.operation_id,
        outcome,
        batch,
        request.max_output_bytes.get(),
        stop,
    )
}
fn roots(id: &wow_store::ObjectId, session: &Session) -> Vec<(&'static str, wow_store::ObjectId)> {
    let (reference, view) = reference::retained_objects(&session.reference);
    let mut roots = vec![
        ("result", id.clone()),
        ("primary", session.primary.object_id().clone()),
        ("reference", reference),
        ("reference_view", view),
    ];
    if let Some(blizzard) = &session.blizzard {
        roots.push(("blizzard", blizzard.object_id().clone()));
    }
    roots
}
fn root_key(id: &wow_store::ObjectId, root: &str) -> ServiceResult<(CatalogName, CatalogPath)> {
    let key = canonical_digest("sha256:", id)?;
    Ok((
        CatalogName::new(ROOT_CATALOG).map_err(map_store_error)?,
        CatalogPath::new(format!("{}/{root}", key.trim_start_matches("sha256:")))
            .map_err(map_store_error)?,
    ))
}
fn load_session(store: &Store, id: &wow_store::ObjectId) -> ServiceResult<Session> {
    let (catalog, path) = root_key(id, "result")?;
    if store
        .catalog_entry(&catalog, &path)
        .map_err(map_store_error)?
        .is_none_or(|entry| entry.object_id() != id)
    {
        return Err(unavailable());
    }
    let object = store
        .object(id)
        .map_err(map_store_error)?
        .ok_or_else(unavailable)?;
    if object.kind() != SESSION_KIND || object.schema_version() != 1 {
        return Err(mismatch());
    }
    let session: Session =
        serde_json::from_slice(object.canonical_json()).map_err(|_| invalid())?;
    if session.schema != session_schema(&session.query, session.blizzard.is_some())
        || session.query.requires_blizzard() != session.blizzard.is_some()
        || session.primary.uses_observations() != session.query.uses_observations()
        || session
            .blizzard
            .as_ref()
            .is_some_and(|b| b.uses_observations() != session.query.uses_observations())
        || session.privacy_scope != "local_metadata_only"
    {
        return Err(mismatch());
    }
    for (root, target) in roots(id, &session) {
        let (catalog, path) = root_key(id, root)?;
        if store
            .catalog_entry(&catalog, &path)
            .map_err(map_store_error)?
            .is_none_or(|entry| entry.object_id() != &target)
        {
            return Err(unavailable());
        }
    }
    Ok(session)
}
#[derive(Serialize)]
struct PageOutcome<'a> {
    status: &'a str,
    resource_closure: &'static str,
    session_object_id: wow_store::ObjectId,
    retention: &'static str,
    page: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation: Option<Continuation>,
}
#[allow(clippy::too_many_arguments)]
fn response(
    operation: &'static str,
    operation_id: &OperationId,
    digest: &str,
    id: wow_store::ObjectId,
    page: FederationPage,
    status: &str,
    max_bytes: u64,
) -> ServiceResult<Box<[u8]>> {
    let continuation = page.next_cursor().map(|cursor| Continuation {
        schema: TOKEN_SCHEMA.into(),
        session_object_id: id.clone(),
        receipt_operation_id: operation_id.as_str().into(),
        receipt_request_digest: digest.into(),
        cursor: cursor.clone(),
    });
    let page_value: serde_json::Value =
        serde_json::from_slice(page.canonical_bytes()).map_err(|_| invalid())?;
    let has_blizzard = page_value
        .get("result_summary")
        .and_then(|summary| summary.get("blizzard_ui"))
        .and_then(|state| state.get("state"))
        .and_then(serde_json::Value::as_str)
        == Some("required_exact");
    let payload = PageOutcome {
        status,
        resource_closure: "closed",
        session_object_id: id,
        retention: if has_blizzard {
            "exact_result_primary_blizzard_reference_shards_and_reference_view_roots_until_explicit_release"
        } else {
            "exact_result_primary_reference_shard_and_reference_view_roots_until_explicit_release"
        },
        page: page_value,
        continuation,
    };
    durable_response(operation, operation_id, digest, &payload, max_bytes)
}
fn mismatch() -> ServiceError {
    failure(
        ServiceErrorCode::IdentityMismatch,
        "captured federation session differs from exact receipt/owners",
    )
}
fn unavailable() -> ServiceError {
    failure(
        ServiceErrorCode::ExactGenerationUnavailable,
        "exact captured federation result/shards/ReferenceView unavailable",
    )
}
fn preserve_closure_errors<T>(
    result: ServiceResult<T>,
    closed: ServiceResult<()>,
) -> ServiceResult<T> {
    preserve_named_closure_errors(result, closed, "primary")
}
fn preserve_named_closure_errors<T>(
    result: ServiceResult<T>,
    closed: ServiceResult<()>,
    owner: &str,
) -> ServiceResult<T> {
    match (result, closed) {
        (Ok(page), Ok(())) => Ok(page),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(prior), Err(close)) => Err(ServiceError::new(
            ServiceErrorCode::StoreOutcomeUnknown,
            format!(
                "federated page prior failure: {}; {owner} close failure: {}",
                prior.message(),
                close.message()
            ),
        )),
    }
}
fn present_binding<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SourceRetainedBinding>, D::Error> {
    SourceRetainedBinding::deserialize(deserializer).map(Some)
}
fn session_schema(query: &FederationQueryRequest, blizzard: bool) -> &'static str {
    if query.uses_observations() {
        OBSERVATION_SESSION_SCHEMA
    } else if blizzard {
        PLATFORM_SESSION_SCHEMA
    } else {
        SESSION_SCHEMA
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dual_close_failure_keeps_both_reconciliation_identities() -> ServiceResult<()> {
        let result: ServiceResult<FederationPage> = preserve_closure_errors(
            Err(failure(
                ServiceErrorCode::StoreOutcomeUnknown,
                "reference lease=reference-123 object=reference-456",
            )),
            Err(failure(
                ServiceErrorCode::StoreOutcomeUnknown,
                "primary lease=primary-123 object=primary-456",
            )),
        );
        let error = result.err().ok_or_else(invalid)?;
        assert_eq!(error.code(), ServiceErrorCode::StoreOutcomeUnknown);
        assert!(
            error.message().contains("reference-123") && error.message().contains("reference-456")
        );
        assert!(error.message().contains("primary-123") && error.message().contains("primary-456"));
        Ok(())
    }
    #[test]
    fn three_close_failures_keep_every_owner_identity() -> ServiceResult<()> {
        let reference: ServiceResult<FederationPage> = Err(failure(
            ServiceErrorCode::StoreOutcomeUnknown,
            "reference lease=ref-third object=ref-object",
        ));
        let both = preserve_named_closure_errors(
            reference,
            Err(failure(
                ServiceErrorCode::StoreOutcomeUnknown,
                "blizzard lease=bliz-third object=bliz-object",
            )),
            "blizzard",
        );
        let all = preserve_closure_errors(
            both,
            Err(failure(
                ServiceErrorCode::StoreOutcomeUnknown,
                "primary lease=primary-third object=primary-object",
            )),
        );
        let error = all.err().ok_or_else(invalid)?;
        assert_eq!(error.code(), ServiceErrorCode::StoreOutcomeUnknown);
        for identity in [
            "ref-third",
            "ref-object",
            "bliz-third",
            "bliz-object",
            "primary-third",
            "primary-object",
        ] {
            assert!(error.message().contains(identity));
        }
        Ok(())
    }
}
