//! Explicit file-set variant of the existing one-shot exact-generation operation.
use super::*;
use wow_search::{
    ProjectFileSetSelection, SearchFileSetBuildProfile, SearchFileSetOpenRequest,
    SearchFileSetPrepareRequest, SearchFileSetShard, open_project_file_set_shard,
    prepare_project_file_set_shard,
};

#[derive(Serialize)]
struct RequestIdentity<'a> {
    operation_id: &'a str,
    store_generation_id: &'a str,
    file_ids: &'a [ProjectFileId],
    query_file_id: &'a str,
    literal_query: &'a str,
    search_profile_digest: &'a str,
    store_limits: SearchStoreLimits,
    max_output_bytes: u64,
    lease_expires_after: u64,
    privacy_scope: &'static str,
}

pub(super) fn request_digest(
    request: &SearchBuildQueryRequest,
    selection: &ProjectFileSetSelection,
    profile: &SearchFileSetBuildProfile,
) -> ServiceResult<Box<str>> {
    canonical_digest(
        "wow-service/search-file-set-build-query-request/1:",
        &RequestIdentity {
            operation_id: request.operation_id.as_str(),
            store_generation_id: &request.store_generation_id,
            file_ids: selection.ids(),
            query_file_id: &request.file_id,
            literal_query: &request.literal_query,
            search_profile_digest: profile.digest(),
            store_limits: request.store_limits,
            max_output_bytes: request.max_output_bytes.get(),
            lease_expires_after: request.lease_expires_after,
            privacy_scope: "local_metadata_only",
        },
    )
}

#[derive(Serialize)]
struct DocumentRecord<'a> {
    entity_id: &'a str,
    owner_metadata_label: &'a str,
}

#[derive(Serialize)]
struct Payload<'a> {
    status: SearchBuildQueryStatus,
    resource_closure: &'static str,
    source_store_generation_id: &'a str,
    logical_shard_id: &'a str,
    search_profile_digest: &'a str,
    documents: Vec<DocumentRecord<'a>>,
    query_file_id: &'a str,
    exact_id_match: bool,
    exact_owner_metadata_label_matches: usize,
    literal_query_digest: Box<str>,
    literal_fts_candidates: Vec<SearchCandidateRecord>,
    privacy_scope: &'static str,
    license_state: &'static str,
}

#[derive(Serialize)]
struct ResultIdentity<'a, 'b> {
    request_digest: &'a str,
    payload: &'a Payload<'b>,
}

#[derive(Serialize)]
struct Envelope<'a, 'b> {
    schema: &'static str,
    operation: &'static str,
    operation_id: &'a str,
    request_digest: &'a str,
    result_id: &'a str,
    payload: &'a Payload<'b>,
}

pub(super) fn execute(
    request: &SearchBuildQueryRequest,
    selection: &ProjectFileSetSelection,
    profile: &SearchFileSetBuildProfile,
    request_digest: &str,
    stop: &AtomicBool,
) -> ServiceResult<SearchBuildQueryResult> {
    check_cancelled(stop)?;
    let generation = StoreGenerationId::parse(request.store_generation_id.to_string())
        .map_err(map_store_error)?;
    let query_id = ProjectFileId::parse(&request.file_id).map_err(map_project_error)?;
    let mut store = Store::open_in_memory(
        StoreConfiguration::new(SEARCH_STORE_PROFILE_ID, request.store_limits)
            .map_err(map_store_error)?,
    )
    .map_err(map_store_error)?;
    let project_store = crate::live_project::LiveProjectStore::open_retained_owner_store(
        &request.project_store_root,
    )?;
    let read = match project_store.read(&ReadSelector::Exact(generation.clone()), stop) {
        Ok(read) => read,
        Err(error) => {
            if project_store.close().is_err() {
                return Err(outcome_unknown());
            }
            return Err(map_store_error(error));
        }
    };
    let result = (|| {
        let prepared = prepare_project_file_set_shard(
            &read,
            SearchFileSetPrepareRequest::new(
                generation.clone(),
                selection.clone(),
                profile.clone(),
                request.permission,
            ),
            request.store_limits,
            stop,
        )
        .map_err(map_search_error)?;
        let logical_id: Box<str> = prepared.logical_id().into();
        let object_id = prepared.object_id().clone();
        let mut batch = WriteBatch::new();
        batch
            .add_object(prepared.into_object())
            .map_err(map_store_error)?;
        store.commit(batch).map_err(map_store_error)?;
        let lease = SearchStoreLeaseRequest::new(
            LeaseId::new(format!("search-file-set-{}", request.operation_id.as_str()))
                .map_err(map_store_error)?,
            "wow-service-search",
            LogicalEpoch::new(request.lease_expires_after),
        )
        .map_err(map_store_error)?;
        let shard = open_project_file_set_shard(
            &mut store,
            SearchFileSetOpenRequest::new(
                logical_id,
                object_id,
                generation.clone(),
                profile.clone(),
                request.permission,
                lease,
            ),
            stop,
        )
        .map_err(map_search_error)?;
        let result = query(&shard, &query_id, request, request_digest, stop);
        // A query/cancellation failure cannot hide a failed lease release.
        shard.release(&mut store).map_err(map_search_error)?;
        result
    })();
    drop(store);
    let read_close = read.close();
    let store_close = project_store.close();
    if read_close.is_err() || store_close.is_err() {
        return Err(failure(
            ServiceErrorCode::InternalContractViolation,
            "search owner resource close failed",
        ));
    }
    check_cancelled(stop)?;
    result
}

fn query(
    shard: &SearchFileSetShard,
    query_id: &ProjectFileId,
    request: &SearchBuildQueryRequest,
    request_digest: &str,
    stop: &AtomicBool,
) -> ServiceResult<SearchBuildQueryResult> {
    check_cancelled(stop)?;
    let exact = shard
        .exact_id_lookup(query_id)
        .map_err(map_search_error)?
        .ok_or_else(|| {
            failure(
                ServiceErrorCode::IdentityMismatch,
                "query file is absent from the selected shard",
            )
        })?;
    let labels = shard
        .exact_label_lookup(exact.document().owner_metadata_label().as_str())
        .map_err(map_search_error)?;
    if !labels.iter().any(|hit| {
        hit.document().entity_id() == query_id
            && hit.evidence() == SearchEvidenceClass::ExactOwnerMetadataLabel
    }) {
        return Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "query label did not resolve the exact owner record",
        ));
    }
    let candidates = shard
        .literal_candidates(&request.literal_query)
        .map_err(map_search_error)?
        .into_iter()
        .map(|candidate| {
            check_cancelled(stop)?;
            if candidate.evidence() != SearchEvidenceClass::Candidate {
                return Err(failure(
                    ServiceErrorCode::IdentityMismatch,
                    "literal FTS returned noncandidate evidence",
                ));
            }
            Ok(SearchCandidateRecord {
                entity_id: candidate.entity_id().as_str().into(),
                owner_metadata_label: candidate.owner_metadata_label().as_str().into(),
                evidence: "candidate",
            })
        })
        .collect::<ServiceResult<Vec<_>>>()?;
    let documents = shard
        .documents()
        .iter()
        .map(|document| {
            check_cancelled(stop)?;
            Ok(DocumentRecord {
                entity_id: document.entity_id().as_str(),
                owner_metadata_label: document.owner_metadata_label().as_str(),
            })
        })
        .collect::<ServiceResult<Vec<_>>>()?;
    let payload = Payload {
        status: SearchBuildQueryStatus::ExactFound,
        resource_closure: "closed",
        source_store_generation_id: shard.source_generation(),
        logical_shard_id: shard.logical_id(),
        search_profile_digest: shard.profile_digest(),
        documents,
        query_file_id: query_id.as_str(),
        exact_id_match: exact.evidence() == SearchEvidenceClass::ExactEntityIdentity,
        exact_owner_metadata_label_matches: labels.len(),
        literal_query_digest: canonical_digest(
            "wow-service/search-literal-query/1:",
            request.literal_query.as_ref(),
        )?,
        literal_fts_candidates: candidates,
        privacy_scope: "local_metadata_only",
        license_state: "not_evaluated",
    };
    let result_id = canonical_digest(
        "wow-service/search-file-set-build-query-result/1:",
        &ResultIdentity {
            request_digest,
            payload: &payload,
        },
    )?;
    let bytes = canonical_json_bytes(&Envelope {
        schema: "wow-service/search-file-set-result-envelope/1",
        operation: "search_file_set_build_query",
        operation_id: request.operation_id.as_str(),
        request_digest,
        result_id: &result_id,
        payload: &payload,
    })
    .map_err(|_| {
        failure(
            ServiceErrorCode::CanonicalizationFailed,
            "search file-set result encoding failed",
        )
    })?;
    check_output_bound(&bytes, request.max_output_bytes.get())?;
    Ok(SearchBuildQueryResult {
        result_id,
        canonical_bytes: bytes.into_boxed_slice(),
    })
}
