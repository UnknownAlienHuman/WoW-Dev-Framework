//! Continue an immutable Search capture; never rehydrate or recompute Graph truth.
use super::*;
use serde::Deserialize;
use wow_search::entities::{EntityPage, EntityPageCursor, EntityQueryResult};
const SESSION_SCHEMA: &str = "wow-service/project-entity-search-session/1";
const SESSION_KIND: &str = "wow.search.entity-result-session";
const ROOT_CATALOG: &str = "wow-search.entity-result-sessions";
const TOKEN_SCHEMA: &str = "wow-service/project-entity-search-continuation/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Continuation {
    schema: String,
    session_object_id: ObjectId,
    receipt_operation_id: String,
    receipt_request_digest: String,
    cursor: EntityPageCursor,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    schema: String,
    original_request_digest: String,
    generation: StoreGenerationId,
    logical_id: Box<str>,
    object_id: ObjectId,
    bounds: SearchBounds,
    query: EntityQueryRequest,
    policy: EntityPagePolicy,
    manifest: serde_json::Value,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    privacy_scope: String,
}
pub struct EntityContinueRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    continuation: Continuation,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    permission: LocalMetadataPermission,
}
impl EntityContinueRequest {
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
pub struct EntityResultReleaseRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    session_object_id: ObjectId,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}
impl EntityResultReleaseRequest {
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
            session_object_id: ObjectId::new(session_object_id.to_owned())
                .map_err(map_store_error)?,
            store_limits,
            max_output_bytes,
        })
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn start(
    request: &EntityRetrievalRequest,
    policy: &EntityPagePolicy,
    result: &EntityQueryResult,
    digest: &str,
    store: &mut Store,
    batch: &mut WriteBatch,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let page = result.page(policy, None, stop).map_err(map_search_error)?;
    let session = Session {
        schema: SESSION_SCHEMA.into(),
        original_request_digest: digest.into(),
        generation: request.generation.clone(),
        logical_id: request.logical_id.clone(),
        object_id: request.object_id.clone(),
        bounds: request.profile.bounds(),
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
    for (root, target) in roots(&id, &session.object_id) {
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
        "search_project_entities_page",
        &request.operation_id,
        digest,
        id,
        page,
        result.status(),
        request.max_output_bytes.get(),
    )
}

pub(in crate::search) fn continue_result(
    request: &EntityContinueRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/entity-search-continue/1",
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
        let prior_id = StoreOperationId::new(request.continuation.receipt_operation_id.clone())
            .map_err(map_store_error)?;
        let prior = store
            .operation(&prior_id)
            .map_err(map_store_error)?
            .ok_or_else(unavailable)?;
        if prior.state() != StoreOperationState::Completed
            || prior.request_digest().as_str() != request.continuation.receipt_request_digest
        {
            return Err(mismatch());
        }
        let bytes = read_completed_response(&store, &prior, request.max_output_bytes.get())?;
        let prior: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if prior.get("payload").and_then(|p| p.get("continuation"))
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
        // Both exact roots are checked again in the completion transaction.
        // A concurrent release cannot be undone by page completion.
        for (root, target) in roots(&request.continuation.session_object_id, &session.object_id) {
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
        let profile = EntitySearchProfile::new(session.bounds).map_err(map_search_error)?;
        let lease = SearchStoreLeaseRequest::new(
            lease_id_for("entity-page", &public_digest)?,
            "wow-service-search",
            LogicalEpoch::new(session.lease_expires_after),
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
        // Search validates its own captured representation. No native Graph
        // decoder, source acquisition or query replay occurs on continuation.
        let page = shard
            .page_retained_result(
                &retained,
                &session.query,
                &session.policy,
                Some(&request.continuation.cursor),
                stop,
            )
            .map_err(map_search_error);
        shard.release(&mut store).map_err(map_search_error)?;
        let page = page?;
        let status = session
            .manifest
            .get("payload")
            .and_then(|p| p.get("status"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(invalid)?;
        response(
            "search_project_entities_continue",
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
    request: &EntityResultReleaseRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/entity-search-release/1",
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
        for (root, target) in roots(&request.session_object_id, &session.object_id) {
            let (catalog, path) = root_key(&request.session_object_id, root)?;
            batch
                .add_catalog_mutation(CatalogMutation::delete(catalog, path, target))
                .map_err(map_store_error)?;
        }
        durable_response(
            "search_project_entities_results_release",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":"released","resource_closure":"closed","session_object_id":request.session_object_id,
                "completed_response_replay":"retained"}),
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

fn roots(id: &ObjectId, shard: &ObjectId) -> [(&'static str, ObjectId); 2] {
    [("result", id.clone()), ("shard", shard.clone())]
}
fn root_key(id: &ObjectId, root: &str) -> ServiceResult<(CatalogName, CatalogPath)> {
    let key = canonical_digest("sha256:", id)?;
    Ok((
        CatalogName::new(ROOT_CATALOG).map_err(map_store_error)?,
        CatalogPath::new(format!("{}/{root}", key.trim_start_matches("sha256:")))
            .map_err(map_store_error)?,
    ))
}
fn load_session(store: &Store, id: &ObjectId) -> ServiceResult<Session> {
    let (catalog, path) = root_key(id, "result")?;
    if store
        .catalog_entry(&catalog, &path)
        .map_err(map_store_error)?
        .is_none_or(|e| e.object_id() != id)
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
    if session.schema != SESSION_SCHEMA || session.privacy_scope != "local_metadata_only" {
        return Err(mismatch());
    }
    for (root, target) in roots(id, &session.object_id) {
        let (catalog, path) = root_key(id, root)?;
        if store
            .catalog_entry(&catalog, &path)
            .map_err(map_store_error)?
            .is_none_or(|e| e.object_id() != &target)
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
    session_object_id: ObjectId,
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
    id: ObjectId,
    page: EntityPage,
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
    let payload = PageOutcome {
        status,
        resource_closure: "closed",
        session_object_id: id,
        retention: "exact_captured_search_result_and_shard_roots_until_explicit_release",
        page: serde_json::from_slice(page.canonical_bytes()).map_err(|_| invalid())?,
        continuation,
    };
    durable_response(operation, operation_id, digest, &payload, max_bytes)
}
fn mismatch() -> ServiceError {
    failure(
        ServiceErrorCode::IdentityMismatch,
        "ProjectEntity captured result identity differs from exact session",
    )
}
fn unavailable() -> ServiceError {
    failure(
        ServiceErrorCode::ExactGenerationUnavailable,
        "exact captured ProjectEntity result or shard is unavailable",
    )
}
