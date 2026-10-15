//! Durable exact-result continuation. Catalog roots protect the result and shard;
//! prior completed operation bytes, not an editable cursor digest, authorize progress.
use super::*;
use serde::Deserialize;
use wow_search::{RetrievalCursor, RetrievalPage, RetrievalPagePolicy, RetrievalResult};

const SESSION_KIND: &str = "wow.search.result-session";
const SESSION_SCHEMA: &str = "wow-service/search-result-session/1";
const ROOT_CATALOG: &str = "wow-search.result-sessions";
const TOKEN_SCHEMA: &str = "wow-service/search-continuation/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Continuation {
    schema: String,
    session_object_id: ObjectId,
    receipt_operation_id: String,
    receipt_request_digest: String,
    cursor: RetrievalCursor,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    schema: String,
    original_request_digest: String,
    generation: StoreGenerationId,
    shard_id: String,
    shard_object_id: ObjectId,
    bounds: SearchBounds,
    query: RetrievalRequest,
    page_policy: RetrievalPagePolicy,
    manifest: serde_json::Value,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    privacy_scope: String,
}

pub struct SearchContinueRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    continuation: Continuation,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    permission: LocalMetadataPermission,
}

impl SearchContinueRequest {
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        continuation_json: &[u8],
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store_path.is_absolute() || continuation_json.len() > 16 * 1024 {
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
            search_store_path,
            continuation,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}

pub struct SearchResultReleaseRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    session_object_id: ObjectId,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}

impl SearchResultReleaseRequest {
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        session_object_id: &str,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        _permission: LocalMetadataPermission,
    ) -> ServiceResult<Self> {
        if !search_store_path.is_absolute() {
            return Err(invalid());
        }
        Ok(Self {
            operation_id,
            search_store_path,
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
    request: &SearchRetrievalRequest,
    policy: &RetrievalPagePolicy,
    result: &RetrievalResult,
    request_digest: &str,
    store: &mut Store,
    batch: &mut WriteBatch,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let page = result.page(policy, None, stop).map_err(map_search_error)?;
    let session = Session {
        schema: SESSION_SCHEMA.into(),
        original_request_digest: request_digest.into(),
        generation: request.generation.clone(),
        shard_id: request.logical_id.to_string(),
        shard_object_id: request.object_id.clone(),
        bounds: request.profile.bounds(),
        query: request.query.clone(),
        page_policy: policy.clone(),
        manifest: serde_json::from_slice(result.canonical_bytes()).map_err(|_| invalid())?,
        store_limits: request.store_limits,
        max_output_bytes: request.max_output_bytes,
        lease_expires_after: request.lease_expires_after,
        privacy_scope: "local_metadata_only".into(),
    };
    let object =
        PendingObject::from_json(SESSION_KIND, 1, &session, store.configuration().limits())
            .map_err(map_store_error)?;
    let session_id = object.object_id().clone();
    batch.add_object(object).map_err(map_store_error)?;
    for (root, target) in [
        ("result", session_id.clone()),
        ("shard", request.object_id.clone()),
    ] {
        let (catalog, path) = root_key(&session_id, root)?;
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
        "search_retrieve_page",
        &request.operation_id,
        request_digest,
        session_id,
        page,
        result.state(),
        request.max_output_bytes.get(),
    )
}

fn response(
    operation: &'static str,
    operation_id: &OperationId,
    request_digest: &str,
    session_id: ObjectId,
    page: RetrievalPage,
    state: RetrievalState,
    max_output_bytes: u64,
) -> ServiceResult<Box<[u8]>> {
    let continuation = page.next_cursor().map(|cursor| Continuation {
        schema: TOKEN_SCHEMA.into(),
        session_object_id: session_id.clone(),
        receipt_operation_id: operation_id.as_str().into(),
        receipt_request_digest: request_digest.into(),
        cursor: cursor.clone(),
    });
    let payload = PageOutcome {
        status: status(state),
        resource_closure: "closed",
        session_object_id: session_id,
        retention: "exact_result_and_shard_catalog_roots_until_explicit_release",
        page: serde_json::from_slice(page.canonical_bytes()).map_err(|_| invalid())?,
        continuation,
    };
    durable_response(
        operation,
        operation_id,
        request_digest,
        &payload,
        max_output_bytes,
    )
}

pub(in crate::search) fn continue_result(
    request: &SearchContinueRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, request_digest, digest) = durable_identity(
        "wow-service/search-continue-request/1",
        &request.operation_id,
        &(
            &request.continuation,
            request.store_limits,
            request.max_output_bytes,
            "local_metadata_only",
        ),
    )?;
    let mut store = open_durable_search_store(&request.search_store_path, request.store_limits)?;
    match start_durable_operation(
        &mut store,
        &operation_id,
        &request_digest,
        request.max_output_bytes.get(),
    )? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }
    let mut batch = WriteBatch::new();
    let outcome = (|| {
        // Read the exact preceding completed result before trusting any budget or index.
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
        let prior_bytes = read_completed_response(&store, &prior, request.max_output_bytes.get())?;
        let prior_value: serde_json::Value =
            serde_json::from_slice(&prior_bytes).map_err(|_| invalid())?;
        let expected = prior_value
            .get("payload")
            .and_then(|v| v.get("continuation"))
            .ok_or_else(mismatch)?;
        if expected != &serde_json::to_value(&request.continuation).map_err(|_| invalid())? {
            return Err(mismatch());
        }
        let session = load_session(&store, &request.continuation.session_object_id)?;
        if session.max_output_bytes != request.max_output_bytes
            || session.store_limits != request.store_limits
        {
            return Err(mismatch());
        }
        let retained = canonical_json_bytes(&session.manifest).map_err(|_| invalid())?;
        request
            .continuation
            .cursor
            .check_replay_budget(&session.page_policy, retained.len() as u64)
            .map_err(|error| owner_error("page replay admission", error))?;
        // CAS both roots again in the completion transaction. Concurrent release
        // must not let this operation advertise a cursor without retention.
        for (root, target) in [
            ("result", request.continuation.session_object_id.clone()),
            ("shard", session.shard_object_id.clone()),
        ] {
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
        let profile = SearchFileSetBuildProfile::new(session.bounds).map_err(map_search_error)?;
        let lease = SearchStoreLeaseRequest::new(
            lease_id_for("search-page", &digest)?,
            "wow-service-search",
            LogicalEpoch::new(session.lease_expires_after),
        )
        .map_err(map_store_error)?;
        let shard = open_project_file_set_shard(
            &mut store,
            SearchFileSetOpenRequest::new(
                session.shard_id.into_boxed_str(),
                session.shard_object_id,
                session.generation,
                profile,
                request.permission,
                lease,
            ),
            stop,
        )
        .map_err(|error| owner_error("page exact shard open", error))?;
        let result = shard
            .retrieve(&session.query, stop)
            .map_err(|error| owner_error("page exact query replay", error));
        shard.release(&mut store).map_err(map_search_error)?;
        let result = result?;
        if result.canonical_bytes() != retained {
            return Err(mismatch());
        }
        let page = result
            .page(
                &session.page_policy,
                Some(&request.continuation.cursor),
                stop,
            )
            .map_err(|error| owner_error("page whole-candidate rendering", error))?;
        response(
            "search_continue",
            &request.operation_id,
            &digest,
            request.continuation.session_object_id.clone(),
            page,
            result.state(),
            request.max_output_bytes.get(),
        )
    })();
    finish(
        &mut store,
        &operation_id,
        &request_digest,
        &request.operation_id,
        outcome,
        batch,
        request.max_output_bytes.get(),
        stop,
    )
}

pub(in crate::search) fn release_result(
    request: &SearchResultReleaseRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, request_digest, digest) = durable_identity(
        "wow-service/search-result-release-request/1",
        &request.operation_id,
        &(
            &request.session_object_id,
            request.store_limits,
            request.max_output_bytes,
        ),
    )?;
    let mut store = open_durable_search_store(&request.search_store_path, request.store_limits)?;
    match start_durable_operation(
        &mut store,
        &operation_id,
        &request_digest,
        request.max_output_bytes.get(),
    )? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }
    let mut batch = WriteBatch::new();
    let outcome = (|| {
        let session = load_session(&store, &request.session_object_id)?;
        for (root, target) in [
            ("result", request.session_object_id.clone()),
            ("shard", session.shard_object_id),
        ] {
            let (catalog, path) = root_key(&request.session_object_id, root)?;
            batch
                .add_catalog_mutation(CatalogMutation::delete(catalog, path, target))
                .map_err(map_store_error)?;
        }
        durable_response(
            "search_result_release",
            &request.operation_id,
            &digest,
            &serde_json::json!({"status":"released", "resource_closure":"closed",
                "session_object_id":request.session_object_id,
                "completed_response_replay":"retained"}),
            request.max_output_bytes.get(),
        )
    })();
    finish(
        &mut store,
        &operation_id,
        &request_digest,
        &request.operation_id,
        outcome,
        batch,
        request.max_output_bytes.get(),
        stop,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    public_id: &OperationId,
    outcome: ServiceResult<Box<[u8]>>,
    batch: WriteBatch,
    max_output_bytes: u64,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let bytes = match outcome.and_then(|bytes| {
        check_cancelled(stop)?;
        Ok(bytes)
    }) {
        Ok(bytes) => bytes,
        Err(error) => {
            let error = if error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                let _ = store.mark_outcome_unknown(operation_id, request_digest);
                error
            } else {
                record_no_effect(store, operation_id, request_digest, error)
            };
            return Err(operation_failure(error, public_id));
        }
    };
    complete_durable_operation(
        store,
        operation_id,
        request_digest,
        batch,
        &bytes,
        max_output_bytes,
    )
    .map_err(|error| operation_failure(error, public_id))
}

fn root_key(session_id: &ObjectId, root: &str) -> ServiceResult<(CatalogName, CatalogPath)> {
    let key = canonical_digest("sha256:", session_id)?;
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
    if session.schema != SESSION_SCHEMA || session.privacy_scope != "local_metadata_only" {
        return Err(mismatch());
    }
    let (catalog, path) = root_key(id, "shard")?;
    if store
        .catalog_entry(&catalog, &path)
        .map_err(map_store_error)?
        .is_none_or(|entry| entry.object_id() != &session.shard_object_id)
    {
        return Err(unavailable());
    }
    Ok(session)
}

fn invalid() -> ServiceError {
    failure(
        ServiceErrorCode::InvalidRequest,
        "invalid search continuation data",
    )
}
fn owner_error(stage: &str, error: SearchError) -> ServiceError {
    let error = map_search_error(error);
    ServiceError::new(error.code(), format!("{stage}: {}", error.message()))
}
fn mismatch() -> ServiceError {
    failure(
        ServiceErrorCode::IdentityMismatch,
        "search continuation differs from its exact retained receipt or manifest",
    )
}
fn unavailable() -> ServiceError {
    failure(
        ServiceErrorCode::ExactGenerationUnavailable,
        "exact search result session or retention root is unavailable",
    )
}
