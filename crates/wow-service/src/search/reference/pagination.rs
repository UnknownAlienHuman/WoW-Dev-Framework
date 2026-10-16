//! Native Reference continuation retains result, Search shard AND source view.
use super::*;
use wow_search::reference::{ReferencePage, ReferencePageCursor, ReferenceQueryResult};
mod explanation;
pub use explanation::ReferenceExplainRequest;
pub(in crate::search) use explanation::explain_result;
const SESSION_SCHEMA: &str = "wow-service/native-reference-search-session/1";
const SESSION_KIND: &str = "wow.search.reference-result-session";
const ROOT_CATALOG: &str = "wow-search.reference-result-sessions";
const TOKEN_SCHEMA: &str = "wow-service/native-reference-search-continuation/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Continuation {
    schema: String,
    session_object_id: ObjectId,
    receipt_operation_id: String,
    receipt_request_digest: String,
    cursor: ReferencePageCursor,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    schema: String,
    original_request_digest: String,
    handle: Handle,
    query: ReferenceQueryRequest,
    policy: ReferencePagePolicy,
    manifest: serde_json::Value,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    privacy_scope: String,
}
pub struct ReferenceContinueRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    continuation: Continuation,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    permission: LocalMetadataPermission,
}
impl ReferenceContinueRequest {
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
pub struct ReferenceResultReleaseRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    session_object_id: ObjectId,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}
impl ReferenceResultReleaseRequest {
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
#[derive(Serialize)]
struct PageOutcome {
    status: &'static str,
    resource_closure: &'static str,
    session_object_id: ObjectId,
    retention: &'static str,
    page: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation: Option<Continuation>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn start(
    request: &ReferenceShardRequest,
    policy: &ReferencePagePolicy,
    result: &ReferenceQueryResult,
    digest: &str,
    store: &mut Store,
    batch: &mut WriteBatch,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let page = result.page(policy, None, stop).map_err(map_search_error)?;
    let session = Session {
        schema: SESSION_SCHEMA.into(),
        original_request_digest: digest.into(),
        handle: request.handle.clone(),
        query: request.query.clone().ok_or_else(invalid)?,
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
    for (root, target) in roots(&id, &session.handle) {
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
        "search_reference_query_page",
        &request.operation_id,
        digest,
        id,
        page,
        result.status(),
        request.max_output_bytes.get(),
    )
}

pub(in crate::search) fn continue_result(
    request: &ReferenceContinueRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/reference-search-continue/1",
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
        // Roots must still exist in the completion transaction, including the
        // original ReferenceView. Never recreate a concurrently released root.
        for (root, target) in roots(&request.continuation.session_object_id, &session.handle) {
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
        let h = &session.handle;
        let owner = PersistentReferenceStore::new(&mut store)
            .read_exact(&h.reference_view_object_id)
            .map_err(reference_error)?
            .ok_or_else(unavailable)?;
        if owner.generation_id() != h.reference_generation
            || owner.self_digest() != h.reference_view_digest
        {
            return Err(mismatch());
        }
        let profile = handle_profile(h)?;
        let lease = SearchStoreLeaseRequest::new(
            lease_id_for("reference-page", &public_digest)?,
            "wow-service-search",
            LogicalEpoch::new(session.lease_expires_after),
        )
        .map_err(map_store_error)?;
        let shard = open_reference_callable_shard(
            &mut store,
            &h.logical_shard_id,
            &h.object_id,
            &profile,
            &h.reference_view_object_id,
            &owner,
            request.permission,
            lease,
            stop,
        )
        .map_err(map_search_error)?;
        let result = shard
            .retrieve(&session.query, &owner, stop)
            .map_err(map_search_error);
        shard.release(&mut store).map_err(map_search_error)?;
        let result = result?;
        if result.canonical_bytes() != retained {
            return Err(mismatch());
        }
        let page = result
            .page(&session.policy, Some(&request.continuation.cursor), stop)
            .map_err(map_search_error)?;
        response(
            "search_reference_continue",
            &request.operation_id,
            &public_digest,
            request.continuation.session_object_id.clone(),
            page,
            result.status(),
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
    request: &ReferenceResultReleaseRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/reference-search-release/1",
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
        for (root, target) in roots(&request.session_object_id, &session.handle) {
            let (catalog, path) = root_key(&request.session_object_id, root)?;
            batch
                .add_catalog_mutation(CatalogMutation::delete(catalog, path, target))
                .map_err(map_store_error)?;
        }
        durable_response(
            "search_reference_results_release",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":"released","resource_closure":"closed", "session_object_id":request.session_object_id,
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

fn roots(id: &ObjectId, handle: &Handle) -> [(&'static str, ObjectId); 3] {
    [
        ("result", id.clone()),
        ("shard", handle.object_id.clone()),
        ("reference", handle.reference_view_object_id.clone()),
    ]
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
    for (root, target) in roots(id, &session.handle) {
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
#[allow(clippy::too_many_arguments)]
fn response(
    operation: &'static str,
    operation_id: &OperationId,
    digest: &str,
    id: ObjectId,
    page: ReferencePage,
    status: &'static str,
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
        retention: "exact_result_shard_and_reference_view_roots_until_explicit_release",
        page: serde_json::from_slice(page.canonical_bytes()).map_err(|_| invalid())?,
        continuation,
    };
    durable_response(operation, operation_id, digest, &payload, max_bytes)
}
