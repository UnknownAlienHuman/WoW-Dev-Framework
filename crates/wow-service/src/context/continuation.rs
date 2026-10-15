//! Persisted Primary progressive continuation through exact native replay.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use wow_core::{CanonicalResult, ContentDigest};
use wow_store::project::{
    ReadSelector, RetentionRoot, RetentionRootId, RetentionRootKind, StoreGenerationId,
};
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, ObjectId, OperationBegin,
    OperationId, OperationResultMutation, OperationState, PendingObject, RequestDigest, Store,
    StoreConfiguration, StoreErrorCode, StoreLimits, WriteBatch,
};

use super::operation::{with_checked_context_read, with_native_progressive};
use super::{
    CONTEXT_MAP_RESULT_MAX_BYTES, ContextBuildRequest, ContextInspectLevel, ContextMapHostInput,
    ContextMapPublication, ContextMapReferenceGuard, ContextMapResult, ContextMapRoot,
    ContextMapStatus, ContextRenderSelection, PRIMARY_L1_PROFILE_ID,
    PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID, PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
    checkpoint, context_error, failure,
};
use crate::{ServiceError, ServiceErrorCode, ServiceResult, live_project::LiveProjectStore};
use wow_context::ContextBudgetReceipt;

pub const CONTEXT_CONTINUE_REQUEST_SCHEMA: &str = "wow-service/context-continue-request/1";
pub const CONTEXT_CONTINUE_RESULT_SCHEMA: &str = "wow-service/context-continue-result/1";
pub const CONTEXT_CONTINUATION_HISTORY_PRUNE_REQUEST_SCHEMA: &str =
    "wow-service/context-continuation-history-prune-request/1";
pub const CONTEXT_CONTINUATION_HISTORY_PRUNE_RESULT_SCHEMA: &str =
    "wow-service/context-continuation-history-prune-result/1";

const HISTORY_PRUNE_OPERATION_BINDING_SCHEMA: &str =
    "wow-service/context-continuation-history-prune-operation/1";
const CONTINUATION_STORE_PROFILE: &str = "wow-service.context-continuation-store.2";
const CONTINUATION_CATALOG: &str = "wow-service.context-continuation";
const RECEIPT_KIND: &str = "wow-service.context-continuation-receipt";
const OUTCOME_KIND: &str = "wow-service.context-continuation-outcome";
const RESPONSE_KIND: &str = "wow-service.context-continuation-response";
const HISTORY_KIND: &str = "wow-service.context-continuation-history";
const HISTORY_PRUNE_RESULT_KIND: &str = "wow-service.context-continuation-history-prune-result";
const RECEIPT_SCHEMA: &str = "wow-service/context-continuation-receipt/2";
const OUTCOME_SCHEMA: &str = "wow-service/context-continuation-outcome/2";
const HISTORY_SCHEMA: &str = "wow-service/context-continuation-history/1";
const RESPONSE_SCHEMA_VERSION: u32 = 1;
const RETENTION_HOLDER: &str = "wow-service.context-continuation";

#[derive(Debug, Clone, Serialize)]
pub struct ContextContinueRequest {
    schema: &'static str,
    continuation_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_original_request_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_original_semantic_pack_id: Option<Box<str>>,
}

impl ContextContinueRequest {
    pub fn new(
        continuation_id: &str,
        expected_original_request_id: Option<&str>,
        expected_original_semantic_pack_id: Option<&str>,
    ) -> ServiceResult<Self> {
        if !valid_continuation_id(continuation_id)
            || expected_original_request_id.is_some_and(|value| !bounded_id(value))
            || expected_original_semantic_pack_id.is_some_and(|value| !bounded_id(value))
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            schema: CONTEXT_CONTINUE_REQUEST_SCHEMA,
            continuation_id: continuation_id.into(),
            expected_original_request_id: expected_original_request_id.map(Into::into),
            expected_original_semantic_pack_id: expected_original_semantic_pack_id.map(Into::into),
        })
    }

    pub fn continuation_id(&self) -> &str {
        &self.continuation_id
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextContinuationHistoryPruneRequest {
    schema: &'static str,
    original_request_id: Box<str>,
}

impl ContextContinuationHistoryPruneRequest {
    pub fn new(original_request_id: &str) -> ServiceResult<Self> {
        if !valid_original_request_id(original_request_id) {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            schema: CONTEXT_CONTINUATION_HISTORY_PRUNE_REQUEST_SCHEMA,
            original_request_id: original_request_id.into(),
        })
    }

    pub fn original_request_id(&self) -> &str {
        &self.original_request_id
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RenderMode {
    None,
    Json,
    Markdown,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReplaySelection {
    project_store_id: Box<str>,
    owner_binding_digest: Box<str>,
    store_generation_id: StoreGenerationId,
    root: ContextMapRoot,
    rounds: Vec<Vec<ContextMapRoot>>,
    render: RenderMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    l1_profile_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<ContextMapReferenceGuard>,
    original_request_id: Box<str>,
    resolved_request_id: Box<str>,
    publication_set_id: Box<str>,
    universe_set_id: Box<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContinuationRecord {
    schema: Box<str>,
    continuation_id: Box<str>,
    issuer_operation_id: OperationId,
    issuer_request_digest: RequestDigest,
    response_object_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_receipt_object_id: Option<ObjectId>,
    replay: ReplaySelection,
    completed_rounds: u8,
    semantic_pack_id: Box<str>,
    retention_root: RetentionRoot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ContinuationOperationOutcome {
    schema: Box<str>,
    owner_binding_digest: Box<str>,
    response_object_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_continuation_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_receipt_object_id: Option<ObjectId>,
    retention_root: RetentionRoot,
    release_retention_root: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ContinuationHistoryResponse {
    operation_id: OperationId,
    response_object_id: ObjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ContinuationHistoryReceipt {
    continuation_id: Box<str>,
    receipt_object_id: ObjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ContinuationHistoryRecord {
    schema: Box<str>,
    original_request_id: Box<str>,
    owner_binding_digest: Box<str>,
    retention_root: RetentionRoot,
    responses: Vec<ContinuationHistoryResponse>,
    receipts: Vec<ContinuationHistoryReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_continuation_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    terminal_operation_id: Option<OperationId>,
}

struct LoadedContinuationHistory {
    record: ContinuationHistoryRecord,
    object_id: Option<ObjectId>,
}

enum HistoryTransition {
    Initial {
        original_request_id: Box<str>,
    },
    Continue {
        prior: Box<LoadedContinuationHistory>,
    },
}

struct LoadedReceipt {
    record: ContinuationRecord,
    receipt_object_id: ObjectId,
}

struct ReceiptDraft {
    continuation_id: Box<str>,
    parent_receipt_object_id: Option<ObjectId>,
    replay: ReplaySelection,
    completed_rounds: u8,
    semantic_pack_id: Box<str>,
    retention_root: RetentionRoot,
}

struct ReplayOwnerContext<'r, 'u> {
    universe: &'r wow_context::ContextUniverseSet<'u>,
    read: &'r crate::live_project::LiveProjectRead,
    host: &'r ContextMapHostInput,
    replay: &'r ReplaySelection,
    store: &'r Store,
    stop: &'r AtomicBool,
}

struct OperationWrite<'a> {
    operation_id: OperationId,
    request_digest: RequestDigest,
    response_bytes: &'a [u8],
    receipt: Option<ReceiptDraft>,
    history: HistoryTransition,
    consumed_continuation: Option<(Box<str>, ObjectId)>,
    retention_root: RetentionRoot,
    release_retention_root: bool,
    owner_binding_digest: Box<str>,
}

pub(super) fn continuation_store_configuration() -> ServiceResult<StoreConfiguration> {
    let limits = StoreLimits::new(CONTEXT_MAP_RESULT_MAX_BYTES as u64, 8, 8, 100_000, 10_000)
        .map_err(|_| failure(ServiceErrorCode::InvalidConfiguration))?;
    StoreConfiguration::new(CONTINUATION_STORE_PROFILE, limits)
        .map_err(|_| failure(ServiceErrorCode::InvalidConfiguration))
}

fn open_continuation_store(host: &ContextMapHostInput) -> ServiceResult<Store> {
    Store::open(
        host.continuation_store_file()?,
        continuation_store_configuration()?,
    )
    .map_err(|_| failure(ServiceErrorCode::ComponentUnavailable))
}

fn request_digest<T: Serialize>(
    schema: &str,
    value: &T,
) -> ServiceResult<(RequestDigest, Box<str>)> {
    let bytes = wow_core::domain_separated_digest(schema, value)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let digest = format!("{}", ContentDigest::<CanonicalResult>::from_bytes(bytes));
    let typed = RequestDigest::new(digest.clone())
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    Ok((typed, digest.into_boxed_str()))
}

fn owner_binding_digest(host: &ContextMapHostInput) -> ServiceResult<Box<str>> {
    let digest = wow_core::domain_separated_digest(
        "wow-service/context-continuation-owner-binding/1",
        &(
            &host.project_store_id,
            &host.project_id,
            &host.limits,
            &host.reference_profile,
            &host.reference,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    Ok(format!("{}", ContentDigest::<CanonicalResult>::from_bytes(digest)).into_boxed_str())
}

fn verify_owner_binding(host: &ContextMapHostInput, expected: &str) -> ServiceResult<()> {
    if owner_binding_digest(host)?.as_ref() == expected {
        Ok(())
    } else {
        Err(failure(ServiceErrorCode::IdentityMismatch))
    }
}

fn continuation_id(
    request_id: &str,
    generation: &StoreGenerationId,
    completed_rounds: u8,
) -> ServiceResult<Box<str>> {
    let digest = wow_core::domain_separated_digest(
        "wow-service/context-continuation-id/1",
        &(request_id, generation, completed_rounds),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    Ok(format!(
        "context-continuation:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    )
    .into_boxed_str())
}

pub(super) fn retention_root(
    store: &LiveProjectStore,
    request_id: &str,
    generation: StoreGenerationId,
) -> ServiceResult<RetentionRoot> {
    let digest = wow_core::domain_separated_digest(
        "wow-service/context-continuation-retention/1",
        &(request_id, &generation),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let root_id = RetentionRootId::new(format!(
        "context-continuation-retention:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    ))
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    RetentionRoot::new(
        store.storage_epoch().epoch_id().clone(),
        root_id,
        RetentionRootKind::Policy,
        generation,
        RETENTION_HOLDER,
    )
    .map_err(|_| failure(ServiceErrorCode::InvalidConfiguration))
}

fn select_and_retain_generation(
    host: &ContextMapHostInput,
    request: &ContextBuildRequest,
    request_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<(
    StoreGenerationId,
    RetentionRoot,
    Option<wow_store::project::CurrentPublication>,
)> {
    let mut store = LiveProjectStore::open(&host.project_store_root)?;
    let selected = (|| {
        checkpoint(stop)?;
        let current = store.current()?;
        request.validate_current_guard(current.as_ref())?;
        let (generation, current_at_selection) = match &request.publication {
            ContextMapPublication::Current => {
                let selected = current
                    .as_ref()
                    .ok_or_else(|| failure(ServiceErrorCode::CurrentGenerationUnavailable))?;
                (selected.generation_id.clone(), current)
            }
            ContextMapPublication::StoreGeneration(generation) => (generation.clone(), None),
        };
        let root = retention_root(&store, request_id, generation.clone())?;
        store.put_retention_root(&root, stop)?;
        Ok((generation, root, current_at_selection))
    })();
    let closed = store.close();
    match (selected, closed) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok((_, root, _)), Err(_)) => match release_retention_root(host, &root) {
            Ok(()) => Err(failure(ServiceErrorCode::InternalContractViolation)),
            Err(_) => Err(failure(ServiceErrorCode::StoreOutcomeUnknown)),
        },
        (Err(_), Err(_)) => Err(failure(ServiceErrorCode::InternalContractViolation)),
    }
}

fn release_retention_root(host: &ContextMapHostInput, root: &RetentionRoot) -> ServiceResult<()> {
    let stop = AtomicBool::new(false);
    let mut store = LiveProjectStore::open(&host.project_store_root)?;
    let outcome = (|| {
        let existing = store
            .retention_roots(&stop)?
            .into_iter()
            .find(|candidate| candidate.root_id() == root.root_id());
        match existing {
            None => Ok(()),
            Some(existing) if existing == *root => {
                store.remove_retention_root(root.root_id(), root.pin_digest(), &stop)?;
                Ok(())
            }
            Some(_) => Err(failure(ServiceErrorCode::IdentityMismatch)),
        }
    })();
    let closed = store.close();
    match (outcome, closed) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(_)) => Err(failure(ServiceErrorCode::InternalContractViolation)),
    }
}

/// Remove only a hold from this prepared issuance when the atomic batch is
/// durably known not to have completed. Any terminal or unreadable journal
/// state keeps the hold so a receipt can never outlive its generation.
fn release_preissuance_retention_if_prepared(
    host: &ContextMapHostInput,
    continuation_store: &Store,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    root: &RetentionRoot,
) -> ServiceResult<bool> {
    let operation = continuation_store
        .operation(operation_id)
        .map_err(|_| failure(ServiceErrorCode::StoreOutcomeUnknown))?
        .ok_or_else(|| failure(ServiceErrorCode::StoreOutcomeUnknown))?;
    if operation.request_digest() != request_digest {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    if operation.state() != OperationState::Prepared {
        return Ok(false);
    }
    release_retention_root(host, root)?;
    Ok(true)
}

fn verify_retention_root(host: &ContextMapHostInput, root: &RetentionRoot) -> ServiceResult<()> {
    let stop = AtomicBool::new(false);
    let store = LiveProjectStore::open(&host.project_store_root)?;
    let found = store
        .retention_roots(&stop)?
        .into_iter()
        .any(|candidate| candidate == *root);
    store.close()?;
    if found {
        Ok(())
    } else {
        Err(failure(ServiceErrorCode::IdentityMismatch))
    }
}

fn ensure_retention_for_operation_outcome(
    host: &ContextMapHostInput,
    store: &Store,
    initial: &ContinuationOperationOutcome,
) -> ServiceResult<()> {
    let root = &initial.retention_root;
    let mut outcome = initial.clone();
    for _ in 0..4 {
        if outcome.release_retention_root {
            return release_retention_root(host, root);
        }
        let Some(token) = outcome.next_continuation_id.as_deref() else {
            return Err(failure(ServiceErrorCode::InternalContractViolation));
        };
        let operation_id = continuation_operation_id(&format!("context-continue:{token}"))?;
        let Some(operation) = store
            .operation(&operation_id)
            .map_err(|_| store_failure())?
        else {
            return verify_retention_root(host, root);
        };
        if operation.state() != OperationState::Completed {
            return verify_retention_root(host, root);
        }
        let result_id = operation.result_object_id().ok_or_else(store_failure)?;
        outcome = outcome_object(store, result_id)?;
        if &outcome.retention_root != root {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
    }
    Err(failure(ServiceErrorCode::IdentityMismatch))
}

pub(super) fn build_operation_identity(
    request: &ContextBuildRequest,
) -> ServiceResult<(Box<str>, OperationId, RequestDigest)> {
    let (digest, digest_text) = request_digest(request.schema, request)?;
    let request_id = format!("context-build-request:{digest_text}").into_boxed_str();
    let operation_id =
        continuation_operation_id(&format!("context-build-continuation:{digest_text}"))?;
    Ok((request_id, operation_id, digest))
}

pub(super) fn continue_operation_identity(
    request: &ContextContinueRequest,
) -> ServiceResult<(Box<str>, OperationId, RequestDigest)> {
    let (digest, digest_text) = request_digest(CONTEXT_CONTINUE_REQUEST_SCHEMA, request)?;
    let operation_id =
        continuation_operation_id(&format!("context-continue:{}", request.continuation_id))?;
    let request_id = format!("context-continue-request:{digest_text}").into_boxed_str();
    Ok((request_id, operation_id, digest))
}

fn history_prune_operation_identity(
    host: &ContextMapHostInput,
    request: &ContextContinuationHistoryPruneRequest,
) -> ServiceResult<(Box<str>, OperationId, RequestDigest)> {
    let (_, digest_text) =
        request_digest(CONTEXT_CONTINUATION_HISTORY_PRUNE_REQUEST_SCHEMA, request)?;
    let operation_id = continuation_operation_id(&format!("context-history-prune:{digest_text}"))?;
    let request_id = format!("context-history-prune-request:{digest_text}").into_boxed_str();
    // Preserve one journal slot per public prune request, while binding that
    // slot's durable request digest to the exact service-selected host. The
    // owner binding is recomputed from native host fields, never supplied by
    // the caller, and remains checkable after terminal history is collected.
    let owner_binding = owner_binding_digest(host)?;
    let (bound_digest, _) = request_digest(
        HISTORY_PRUNE_OPERATION_BINDING_SCHEMA,
        &(request, owner_binding),
    )?;
    Ok((request_id, operation_id, bound_digest))
}

fn replay_completed_result(
    store: &mut Store,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    host: &ContextMapHostInput,
    stop: &AtomicBool,
) -> ServiceResult<Option<ContextMapResult>> {
    let Some(operation) = store.operation(operation_id).map_err(|_| store_failure())? else {
        return Ok(None);
    };
    if operation.request_digest() != request_digest {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    if operation.state() != OperationState::Completed {
        return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
    }
    let result_id = operation.result_object_id().ok_or_else(store_failure)?;
    let outcome = outcome_object(store, result_id)?;
    if outcome.schema.as_ref() != OUTCOME_SCHEMA {
        return Err(store_failure());
    }
    verify_owner_binding(host, &outcome.owner_binding_digest)?;
    let bytes = response_for_operation(store, operation_id, &outcome.response_object_id)?;
    if let Some(receipt_id) = &outcome.next_receipt_object_id {
        let receipt = receipt_from_object(store, receipt_id)?;
        if outcome.next_continuation_id.as_deref() != Some(receipt.record.continuation_id.as_ref())
            || receipt.record.response_object_id != outcome.response_object_id
        {
            return Err(store_failure());
        }
        if let Some(history) = load_history(store, &receipt.record.replay.original_request_id)? {
            if history.record.owner_binding_digest != outcome.owner_binding_digest
                || !history.record.responses.iter().any(|response| {
                    &response.operation_id == operation_id
                        && response.response_object_id == outcome.response_object_id
                })
                || !history.record.receipts.iter().any(|root| {
                    root.continuation_id == receipt.record.continuation_id
                        && &root.receipt_object_id == receipt_id
                })
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            verify_history_roots(store, &history.record)?;
        }
    }
    // Completed response replay either confirms an active exact hold or finishes
    // idempotent release when a later round already completed.
    ensure_retention_for_operation_outcome(host, store, &outcome)?;
    checkpoint(stop)?;
    Ok(Some(response_result(bytes)?))
}

fn prepare_initial_operation(
    store: &mut Store,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    host: &ContextMapHostInput,
    stop: &AtomicBool,
) -> ServiceResult<Option<ContextMapResult>> {
    match store
        .begin_operation(operation_id.clone(), request_digest.clone())
        .map_err(|error| {
            if error.code() == StoreErrorCode::OperationConflict {
                failure(ServiceErrorCode::OperationConflict)
            } else {
                store_failure()
            }
        })? {
        OperationBegin::Started(_) => Ok(None),
        OperationBegin::Replay(record) if record.state() == OperationState::Completed => {
            replay_completed_result(store, operation_id, request_digest, host, stop)?
                .map(Some)
                .ok_or_else(store_failure)
        }
        OperationBegin::Replay(_) => Err(failure(ServiceErrorCode::StoreOutcomeUnknown)),
    }
}

pub(super) fn execute_context_build_with_continuation(
    host: &ContextMapHostInput,
    request: &ContextBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    checkpoint(stop)?;
    let (request_id, operation_id, request_digest) = build_operation_identity(request)?;
    if let Err(error) = host.validate(stop) {
        return super::operation::failed_operation(
            request,
            &request_id,
            error.code(),
            "not_acquired",
            super::build::CONTEXT_BUILD_CONTINUATION_RESULT_SCHEMA,
            "context_build",
        );
    }
    if request.has_source_excerpt()
        || request.continuation_retention_profile_id
            != Some(super::CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)
        || request
            .progressive
            .as_ref()
            .is_none_or(|progressive| !(2..=4).contains(&progressive.rounds.len()))
        || super::l0_profile_from_id(request.l0_profile_id).is_err()
        || super::l1_profile_from_id(request.l1_profile_id).is_err()
    {
        return super::operation::failed_operation(
            request,
            &request_id,
            ServiceErrorCode::InvalidRequest,
            "not_acquired",
            super::build::CONTEXT_BUILD_CONTINUATION_RESULT_SCHEMA,
            "context_build",
        );
    }
    let mut continuation_store = match open_continuation_store(host) {
        Ok(store) => store,
        Err(error) => {
            return super::operation::failed_operation(
                request,
                &request_id,
                error.code(),
                "not_acquired",
                super::build::CONTEXT_BUILD_CONTINUATION_RESULT_SCHEMA,
                "context_build",
            );
        }
    };
    match prepare_initial_operation(
        &mut continuation_store,
        &operation_id,
        &request_digest,
        host,
        stop,
    ) {
        Ok(Some(result)) => return Ok(result),
        Ok(None) => {}
        Err(error) => {
            return super::operation::failed_operation(
                request,
                &request_id,
                error.code(),
                "not_acquired",
                super::build::CONTEXT_BUILD_CONTINUATION_RESULT_SCHEMA,
                "context_build",
            );
        }
    }

    let (generation, retention, current_at_selection) =
        match select_and_retain_generation(host, request, &request_id, stop) {
            Ok(value) => value,
            Err(error) => {
                return super::operation::failed_operation(
                    request,
                    &request_id,
                    error.code(),
                    "closed",
                    super::build::CONTEXT_BUILD_CONTINUATION_RESULT_SCHEMA,
                    "context_build",
                );
            }
        };
    let token = match continuation_id(&request_id, &generation, 1) {
        Ok(token) => token,
        Err(error) => {
            if !release_preissuance_retention_if_prepared(
                host,
                &continuation_store,
                &operation_id,
                &request_digest,
                &retention,
            )? {
                return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
            }
            return Err(error);
        }
    };
    let selector = ReadSelector::Exact(generation.clone());
    let (built, closure) =
        with_checked_context_read(host, &request.project_store_id, &selector, stop, |read| {
            super::build::build_continuation_prefix(
                read,
                host,
                request,
                &request_id,
                &token,
                current_at_selection.as_ref(),
                stop,
            )
        });
    let (response_bytes, _) = match built {
        Ok(value) if closure == "closed" => value,
        Ok(_) => {
            let code = if release_preissuance_retention_if_prepared(
                host,
                &continuation_store,
                &operation_id,
                &request_digest,
                &retention,
            )? {
                ServiceErrorCode::InternalContractViolation
            } else {
                ServiceErrorCode::StoreOutcomeUnknown
            };
            return super::operation::failed_operation(
                request,
                &request_id,
                code,
                closure,
                super::build::CONTEXT_BUILD_CONTINUATION_RESULT_SCHEMA,
                "context_build",
            );
        }
        Err(error) => {
            let code = if release_preissuance_retention_if_prepared(
                host,
                &continuation_store,
                &operation_id,
                &request_digest,
                &retention,
            )? {
                error.code()
            } else {
                ServiceErrorCode::StoreOutcomeUnknown
            };
            return super::operation::failed_operation(
                request,
                &request_id,
                code,
                closure,
                super::build::CONTEXT_BUILD_CONTINUATION_RESULT_SCHEMA,
                "context_build",
            );
        }
    };
    let receipt_data = (|| {
        let response = response_value(&response_bytes)?;
        let replay =
            ReplaySelection::from_request(request, generation, &request_id, &response, host)?;
        let semantic_pack_id = response
            .pointer("/payload/semantic_pack/semantic_pack_id")
            .and_then(Value::as_str)
            .ok_or_else(|| failure(ServiceErrorCode::InternalContractViolation))?;
        Ok::<_, ServiceError>((replay, semantic_pack_id.to_owned().into_boxed_str()))
    })();
    let (replay, semantic_pack_id) = match receipt_data {
        Ok(value) => value,
        Err(error) => {
            if !release_preissuance_retention_if_prepared(
                host,
                &continuation_store,
                &operation_id,
                &request_digest,
                &retention,
            )? {
                return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
            }
            return Err(error);
        }
    };
    let owner_binding_digest = replay.owner_binding_digest.clone();
    let draft = ReceiptDraft {
        continuation_id: token,
        parent_receipt_object_id: None,
        replay,
        completed_rounds: 1,
        semantic_pack_id,
        retention_root: retention.clone(),
    };
    let persisted = persist_prepared_operation(
        &mut continuation_store,
        OperationWrite {
            operation_id: operation_id.clone(),
            request_digest: request_digest.clone(),
            response_bytes: &response_bytes,
            receipt: Some(draft),
            history: HistoryTransition::Initial {
                original_request_id: request_id.clone(),
            },
            consumed_continuation: None,
            retention_root: retention.clone(),
            release_retention_root: false,
            owner_binding_digest,
        },
        stop,
    );
    match persisted {
        Ok((outcome, bytes)) => {
            if outcome.release_retention_root {
                return Err(failure(ServiceErrorCode::InternalContractViolation));
            }
            if outcome.retention_root != retention {
                release_retention_root(host, &retention)?;
            }
            ensure_retention_for_operation_outcome(host, &continuation_store, &outcome)?;
            checkpoint(stop)?;
            response_result(bytes)
        }
        Err(error) => {
            if release_preissuance_retention_if_prepared(
                host,
                &continuation_store,
                &operation_id,
                &request_digest,
                &retention,
            )? {
                Err(error)
            } else {
                Err(failure(ServiceErrorCode::StoreOutcomeUnknown))
            }
        }
    }
}

fn continuation_operation_id(value: &str) -> ServiceResult<OperationId> {
    OperationId::new(value.to_owned())
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))
}

fn continuation_catalog_path(value: &str) -> ServiceResult<CatalogPath> {
    CatalogPath::new(value.to_owned()).map_err(|_| failure(ServiceErrorCode::InvalidRequest))
}

pub(super) fn response_root_catalog_path(operation_id: &OperationId) -> ServiceResult<CatalogPath> {
    continuation_catalog_path(&format!("response/{}", operation_id.as_str()))
}

pub(super) fn history_catalog_path(original_request_id: &str) -> ServiceResult<CatalogPath> {
    if !valid_original_request_id(original_request_id) {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    continuation_catalog_path(&format!("history/{original_request_id}"))
}

fn valid_continuation_id(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("context-continuation:sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_original_request_id(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("context-build-request:sha256:") else {
        return false;
    };
    bounded_id(value)
        && digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn bounded_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= super::CONTEXT_MAP_ID_MAX_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn store_failure() -> ServiceError {
    failure(ServiceErrorCode::InternalContractViolation)
}

fn validate_history_record(record: &ContinuationHistoryRecord) -> ServiceResult<()> {
    if record.schema.as_ref() != HISTORY_SCHEMA
        || !valid_original_request_id(&record.original_request_id)
        || record.responses.is_empty()
        || record.responses.len() > 4
        || record.receipts.is_empty()
        || record.receipts.len() > 3
        || record.responses.len() < record.receipts.len()
        || record.responses.len() > record.receipts.len() + 1
        || record
            .responses
            .iter()
            .enumerate()
            .any(|(index, response)| {
                record.responses[..index]
                    .iter()
                    .any(|prior| prior.operation_id == response.operation_id)
            })
        || record.receipts.iter().enumerate().any(|(index, receipt)| {
            !valid_continuation_id(&receipt.continuation_id)
                || record.receipts[..index]
                    .iter()
                    .any(|prior| prior.continuation_id == receipt.continuation_id)
        })
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }

    match (
        record.active_continuation_id.as_deref(),
        record.terminal_operation_id.as_ref(),
    ) {
        (Some(active), None)
            if record.responses.len() == record.receipts.len()
                && record
                    .receipts
                    .last()
                    .is_some_and(|receipt| receipt.continuation_id.as_ref() == active) => {}
        (None, Some(terminal))
            if record.responses.len() == record.receipts.len() + 1
                && record
                    .responses
                    .last()
                    .is_some_and(|response| &response.operation_id == terminal) => {}
        _ => return Err(failure(ServiceErrorCode::IdentityMismatch)),
    }
    Ok(())
}

fn history_object(store: &Store, object_id: &ObjectId) -> ServiceResult<ContinuationHistoryRecord> {
    let object = store
        .object(object_id)
        .map_err(|_| store_failure())?
        .ok_or_else(store_failure)?;
    if object.kind() != HISTORY_KIND || object.schema_version() != RESPONSE_SCHEMA_VERSION {
        return Err(store_failure());
    }
    let record: ContinuationHistoryRecord = object.decode().map_err(|_| store_failure())?;
    validate_history_record(&record)?;
    Ok(record)
}

fn pending_history_object(record: &ContinuationHistoryRecord) -> ServiceResult<PendingObject> {
    validate_history_record(record)?;
    PendingObject::from_json(
        HISTORY_KIND,
        RESPONSE_SCHEMA_VERSION,
        record,
        continuation_store_configuration()
            .map_err(|_| store_failure())?
            .limits(),
    )
    .map_err(|_| store_failure())
}

fn load_history(
    store: &Store,
    original_request_id: &str,
) -> ServiceResult<Option<LoadedContinuationHistory>> {
    let catalog = CatalogName::new(CONTINUATION_CATALOG).map_err(|_| store_failure())?;
    let path = history_catalog_path(original_request_id)?;
    let Some(entry) = store
        .catalog_entry(&catalog, &path)
        .map_err(|_| store_failure())?
    else {
        return Ok(None);
    };
    let record = history_object(store, entry.object_id())?;
    if record.original_request_id.as_ref() != original_request_id {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    Ok(Some(LoadedContinuationHistory {
        record,
        object_id: Some(entry.object_id().clone()),
    }))
}

fn pending_json(kind: &str, value: &Value) -> ServiceResult<PendingObject> {
    PendingObject::from_json(
        kind,
        RESPONSE_SCHEMA_VERSION,
        value,
        continuation_store_configuration()
            .map_err(|_| store_failure())?
            .limits(),
    )
    .map_err(|_| store_failure())
}

fn response_value(bytes: &[u8]) -> ServiceResult<Value> {
    serde_json::from_slice(bytes).map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))
}

fn response_status(value: &Value) -> ServiceResult<ContextMapStatus> {
    match value.get("status").and_then(Value::as_str) {
        Some("complete") => Ok(ContextMapStatus::Complete),
        Some("partial") => Ok(ContextMapStatus::Partial),
        Some("truncated") => Ok(ContextMapStatus::Truncated),
        Some("failed") => Ok(ContextMapStatus::Failed),
        Some("cancelled") => Ok(ContextMapStatus::Cancelled),
        _ => Err(failure(ServiceErrorCode::InternalContractViolation)),
    }
}

fn response_result(bytes: Box<[u8]>) -> ServiceResult<ContextMapResult> {
    let value = response_value(&bytes)?;
    if bytes.is_empty() || bytes.len() > CONTEXT_MAP_RESULT_MAX_BYTES {
        return Err(failure(ServiceErrorCode::InternalContractViolation));
    }
    let status = response_status(&value)?;
    Ok(ContextMapResult {
        bytes,
        status,
        code: None,
    })
}

fn response_object(store: &Store, object_id: &ObjectId) -> ServiceResult<Box<[u8]>> {
    let object = store
        .object(object_id)
        .map_err(|_| store_failure())?
        .ok_or_else(store_failure)?;
    if object.kind() != RESPONSE_KIND || object.schema_version() != RESPONSE_SCHEMA_VERSION {
        return Err(store_failure());
    }
    let value = response_value(object.canonical_json())?;
    if wow_core::canonical_json_bytes(&value)
        .map_err(|_| store_failure())?
        .as_slice()
        != object.canonical_json()
    {
        return Err(store_failure());
    }
    Ok(object.canonical_json().into())
}

fn response_for_operation(
    store: &Store,
    operation_id: &OperationId,
    response_object_id: &ObjectId,
) -> ServiceResult<Box<[u8]>> {
    let catalog = CatalogName::new(CONTINUATION_CATALOG).map_err(|_| store_failure())?;
    let path = response_root_catalog_path(operation_id)?;
    if let Some(entry) = store
        .catalog_entry(&catalog, &path)
        .map_err(|_| store_failure())?
    {
        if entry.object_id() == response_object_id {
            return response_object(store, response_object_id);
        }
        let object = store
            .object(entry.object_id())
            .map_err(|_| store_failure())?
            .ok_or_else(store_failure)?;
        if object.kind() == HISTORY_PRUNE_RESULT_KIND {
            return Err(failure(ServiceErrorCode::OperationReleased));
        }
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }

    // Legacy rootless responses remain replayable while the exact object exists.
    // New writes add this direct root in their atomic commit_operation batch.
    response_object(store, response_object_id)
}

fn outcome_object(
    store: &Store,
    object_id: &ObjectId,
) -> ServiceResult<ContinuationOperationOutcome> {
    let object = store
        .object(object_id)
        .map_err(|_| store_failure())?
        .ok_or_else(store_failure)?;
    if object.kind() == HISTORY_PRUNE_RESULT_KIND {
        return Err(failure(ServiceErrorCode::OperationReleased));
    }
    if object.kind() != OUTCOME_KIND || object.schema_version() != RESPONSE_SCHEMA_VERSION {
        return Err(store_failure());
    }
    object.decode().map_err(|_| store_failure())
}

fn read_completed_outcome(
    store: &mut Store,
    operation_id: &OperationId,
    expected_digest: &RequestDigest,
) -> ServiceResult<(ContinuationOperationOutcome, Box<[u8]>)> {
    let operation = store
        .operation(operation_id)
        .map_err(|_| store_failure())?
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    if operation.request_digest() != expected_digest {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    if operation.state() != OperationState::Completed {
        return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
    }
    let result_id = operation.result_object_id().ok_or_else(store_failure)?;
    let outcome = outcome_object(store, result_id)?;
    if outcome.schema.as_ref() != OUTCOME_SCHEMA
        || outcome.next_continuation_id.is_some() != outcome.next_receipt_object_id.is_some()
        || outcome.next_continuation_id.is_none() != outcome.release_retention_root
    {
        return Err(store_failure());
    }
    let response = response_for_operation(store, operation_id, &outcome.response_object_id)?;
    Ok((outcome, response))
}

fn validate_receipt_issuer(
    store: &Store,
    record: &ContinuationRecord,
    receipt_object_id: &ObjectId,
) -> ServiceResult<()> {
    if record.schema.as_ref() != RECEIPT_SCHEMA
        || !valid_continuation_id(&record.continuation_id)
        || !(1..=4).contains(&record.completed_rounds)
    {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let operation = store
        .operation(&record.issuer_operation_id)
        .map_err(|_| store_failure())?
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    if operation.state() != OperationState::Completed
        || operation.request_digest() != &record.issuer_request_digest
    {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let outcome_id = operation
        .result_object_id()
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    let outcome = outcome_object(store, outcome_id)?;
    if outcome.next_continuation_id.as_deref() != Some(record.continuation_id.as_ref())
        || outcome.next_receipt_object_id.as_ref() != Some(receipt_object_id)
        || outcome.owner_binding_digest != record.replay.owner_binding_digest
        || outcome.response_object_id != record.response_object_id
        || outcome.retention_root != record.retention_root
        || outcome.release_retention_root
    {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let response = response_object(store, &record.response_object_id)?;
    let value = response_value(&response)?;
    if value.get("resource_closure").and_then(Value::as_str) != Some("closed")
        || value
            .pointer("/payload/continuation_id")
            .and_then(Value::as_str)
            != Some(record.continuation_id.as_ref())
    {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    Ok(())
}

fn receipt_from_object(store: &Store, object_id: &ObjectId) -> ServiceResult<LoadedReceipt> {
    let object = store
        .object(object_id)
        .map_err(|_| store_failure())?
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    if object.kind() != RECEIPT_KIND || object.schema_version() != RESPONSE_SCHEMA_VERSION {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let record: ContinuationRecord = object
        .decode()
        .map_err(|_| failure(ServiceErrorCode::InvalidRequest))?;
    validate_receipt_issuer(store, &record, object_id)?;
    Ok(LoadedReceipt {
        record,
        receipt_object_id: object_id.clone(),
    })
}

fn receipt_by_token(store: &Store, token: &str) -> ServiceResult<LoadedReceipt> {
    let catalog = CatalogName::new(CONTINUATION_CATALOG).map_err(|_| store_failure())?;
    let path = continuation_catalog_path(token)?;
    let entry = store
        .catalog_entry(&catalog, &path)
        .map_err(|_| store_failure())?
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    let receipt = receipt_from_object(store, entry.object_id())?;
    if receipt.record.continuation_id.as_ref() != token {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    if let Some(history) = load_history(store, &receipt.record.replay.original_request_id)? {
        if history.record.active_continuation_id.as_deref() != Some(token)
            || !history.record.receipts.iter().any(|root| {
                root.continuation_id.as_ref() == token
                    && &root.receipt_object_id == entry.object_id()
            })
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        verify_history_roots(store, &history.record)?;
    }
    Ok(receipt)
}

fn receipt_chain(store: &Store, latest: LoadedReceipt) -> ServiceResult<Vec<LoadedReceipt>> {
    let mut chain = vec![latest];
    while let Some(parent) = chain
        .last()
        .and_then(|receipt| receipt.record.parent_receipt_object_id.clone())
    {
        if chain.len() >= 4 {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        chain.push(receipt_from_object(store, &parent)?);
    }
    chain.reverse();
    let Some(first) = chain.first() else {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    };
    if first.record.completed_rounds != 1
        || first.record.parent_receipt_object_id.is_some()
        || chain.iter().enumerate().any(|(index, receipt)| {
            receipt.record.completed_rounds != u8::try_from(index + 1).unwrap_or(u8::MAX)
                || receipt.record.replay != first.record.replay
                || receipt.record.retention_root != first.record.retention_root
                || (index > 0
                    && receipt.record.parent_receipt_object_id.as_ref()
                        != Some(&chain[index - 1].receipt_object_id))
        })
    {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    Ok(chain)
}

fn history_from_active_chain(chain: &[LoadedReceipt]) -> ServiceResult<ContinuationHistoryRecord> {
    let first = chain
        .first()
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    let latest = chain
        .last()
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    let record = ContinuationHistoryRecord {
        schema: HISTORY_SCHEMA.into(),
        original_request_id: first.record.replay.original_request_id.clone(),
        owner_binding_digest: first.record.replay.owner_binding_digest.clone(),
        retention_root: first.record.retention_root.clone(),
        responses: chain
            .iter()
            .map(|receipt| ContinuationHistoryResponse {
                operation_id: receipt.record.issuer_operation_id.clone(),
                response_object_id: receipt.record.response_object_id.clone(),
            })
            .collect(),
        receipts: chain
            .iter()
            .map(|receipt| ContinuationHistoryReceipt {
                continuation_id: receipt.record.continuation_id.clone(),
                receipt_object_id: receipt.receipt_object_id.clone(),
            })
            .collect(),
        active_continuation_id: Some(latest.record.continuation_id.clone()),
        terminal_operation_id: None,
    };
    validate_history_record(&record)?;
    Ok(record)
}

fn catalog_target(
    store: &Store,
    catalog: &CatalogName,
    path: &CatalogPath,
) -> ServiceResult<Option<ObjectId>> {
    store
        .catalog_entry(catalog, path)
        .map_err(|_| store_failure())
        .map(|entry| entry.map(|value| value.object_id().clone()))
}

fn ensure_root_mutation(
    store: &Store,
    batch: &mut WriteBatch,
    catalog: &CatalogName,
    path: CatalogPath,
    target: &ObjectId,
) -> ServiceResult<()> {
    match catalog_target(store, catalog, &path)? {
        Some(existing) if &existing == target => Ok(()),
        Some(existing) => {
            let object = store
                .object(&existing)
                .map_err(|_| store_failure())?
                .ok_or_else(store_failure)?;
            if object.kind() == HISTORY_PRUNE_RESULT_KIND {
                Err(failure(ServiceErrorCode::OperationReleased))
            } else {
                Err(failure(ServiceErrorCode::IdentityMismatch))
            }
        }
        None => batch
            .add_catalog_mutation(CatalogMutation::set(
                catalog.clone(),
                path,
                CatalogExpectation::Absent,
                target.clone(),
            ))
            .map_err(|_| store_failure()),
    }
}

fn verify_history_roots(store: &Store, record: &ContinuationHistoryRecord) -> ServiceResult<()> {
    let catalog = CatalogName::new(CONTINUATION_CATALOG).map_err(|_| store_failure())?;
    for response in &record.responses {
        let path = response_root_catalog_path(&response.operation_id)?;
        if catalog_target(store, &catalog, &path)?.as_ref() != Some(&response.response_object_id) {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
    }
    for receipt in &record.receipts {
        let path = continuation_catalog_path(&receipt.continuation_id)?;
        if catalog_target(store, &catalog, &path)?.as_ref() != Some(&receipt.receipt_object_id) {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
    }
    Ok(())
}

fn history_for_active_chain(
    store: &Store,
    chain: &[LoadedReceipt],
) -> ServiceResult<LoadedContinuationHistory> {
    let expected = history_from_active_chain(chain)?;
    if let Some(history) = load_history(store, &expected.original_request_id)? {
        if history.record != expected {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        verify_history_roots(store, &history.record)?;
        return Ok(history);
    }
    // A legacy active chain has no history index. Carry its verified roots into
    // the next issuance batch; do not write them outside commit_operation.
    Ok(LoadedContinuationHistory {
        record: expected,
        object_id: None,
    })
}

impl ReplaySelection {
    fn from_request(
        request: &ContextBuildRequest,
        generation: StoreGenerationId,
        original_request_id: &str,
        response: &Value,
        host: &ContextMapHostInput,
    ) -> ServiceResult<Self> {
        let progressive = request
            .progressive
            .as_ref()
            .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
        let payload = response
            .get("payload")
            .ok_or_else(|| failure(ServiceErrorCode::InternalContractViolation))?;
        let string = |key: &str| {
            payload
                .get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| failure(ServiceErrorCode::InternalContractViolation))
        };
        let rounds = progressive
            .rounds
            .iter()
            .map(|round| {
                round
                    .iter()
                    .map(|id| ContextMapRoot::graph_entity(id.as_str()))
                    .collect::<ServiceResult<Vec<_>>>()
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        Ok(Self {
            project_store_id: request.project_store_id.clone(),
            owner_binding_digest: owner_binding_digest(host)?,
            store_generation_id: generation,
            root: request.root.clone(),
            rounds,
            render: match request.render {
                ContextRenderSelection::None => RenderMode::None,
                ContextRenderSelection::Json => RenderMode::Json,
                ContextRenderSelection::Markdown => RenderMode::Markdown,
                ContextRenderSelection::Both => RenderMode::Both,
            },
            l1_profile_id: request.l1_profile_id.map(Into::into),
            reference: request.reference.clone(),
            original_request_id: original_request_id.into(),
            resolved_request_id: string("resolved_request_id")?.into(),
            publication_set_id: string("publication_set_id")?.into(),
            universe_set_id: payload
                .pointer("/semantic_pack/core/universe_set_id")
                .and_then(Value::as_str)
                .ok_or_else(|| failure(ServiceErrorCode::InternalContractViolation))?
                .into(),
        })
    }

    fn build_request(&self) -> ServiceResult<ContextBuildRequest> {
        if self.rounds.len() < 2
            || self.rounds.len() > 4
            || self.l1_profile_id.as_deref() != Some(PRIMARY_L1_PROFILE_ID)
            || !matches!(self.root, ContextMapRoot::GraphEntity(_))
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        let render = match self.render {
            RenderMode::None => ContextRenderSelection::None,
            RenderMode::Json => ContextRenderSelection::Json,
            RenderMode::Markdown => ContextRenderSelection::Markdown,
            RenderMode::Both => ContextRenderSelection::Both,
        };
        let mut request = ContextBuildRequest::primary(
            &self.project_store_id,
            ContextMapPublication::StoreGeneration(self.store_generation_id.clone()),
            ContextInspectLevel::L1,
            self.root.clone(),
            render,
            None,
            self.reference.clone(),
        )?
        .with_progressive_expansion(self.rounds.clone())?
        .with_continuation_retention_profile(super::CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)?;
        if request.context_profile_id != PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID
            || request.progressive.as_ref().map(|value| value.profile_id)
                != Some(PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID)
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        request.schema = super::build::CONTEXT_BUILD_CONTINUATION_REQUEST_SCHEMA;
        Ok(request)
    }

    fn graph_root(&self) -> ServiceResult<ContextMapRoot> {
        match &self.root {
            ContextMapRoot::GraphEntity(id) => Ok(ContextMapRoot::GraphEntity(id.clone())),
            ContextMapRoot::Project(_) => Err(failure(ServiceErrorCode::InvalidRequest)),
        }
    }

    fn graph_rounds(&self, count: usize) -> ServiceResult<Vec<Vec<wow_graph::GraphNodeId>>> {
        let selected = self
            .rounds
            .get(..count)
            .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
        selected
            .iter()
            .map(|round| {
                round
                    .iter()
                    .map(|root| match root {
                        ContextMapRoot::GraphEntity(id) => Ok(id.clone()),
                        ContextMapRoot::Project(_) => {
                            Err(failure(ServiceErrorCode::InvalidRequest))
                        }
                    })
                    .collect::<ServiceResult<Vec<_>>>()
            })
            .collect()
    }
}

fn replay_prefix_budget(
    owner: &ReplayOwnerContext<'_, '_>,
    receipt: &LoadedReceipt,
    rounds: &[Vec<wow_graph::GraphNodeId>],
    prior_receipts: &[ContextBudgetReceipt],
) -> ServiceResult<ContextBudgetReceipt> {
    let root = owner.replay.graph_root()?;
    let ContextMapRoot::GraphEntity(root_id) = &root else {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    };
    let scope = wow_context::L0Scope::Entity(root_id.clone());
    with_native_progressive(
        owner.universe,
        owner.read,
        owner.host,
        root,
        rounds,
        owner.stop,
        |step| {
            let mut pack = wow_context::build_context_semantic_pack_from_expansion(
                owner.universe,
                &scope,
                step,
                owner.stop,
            )
            .map_err(context_error)?;
            let artifacts =
                render_pack(owner.universe, &mut pack, owner.replay.render, owner.stop)?;
            let replay_usage = pack.operation_budget_usage();
            if pack.semantic_pack_id() != receipt.record.semantic_pack_id.as_ref() {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            let response = response_object(owner.store, &receipt.record.response_object_id)?;
            let envelope = response_value(&response)?;
            let saved_pack = envelope
                .pointer("/payload/semantic_pack")
                .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
            let saved_pack_bytes = wow_core::canonical_json_bytes(saved_pack)
                .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
            let saved_artifacts = envelope
                .pointer("/payload/rendered_artifacts")
                .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
            let saved_artifact_bytes = wow_core::canonical_json_bytes(saved_artifacts)
                .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
            let actual_artifact_bytes = wow_core::canonical_json_bytes(&artifacts)
                .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
            let saved_usage = envelope
                .pointer("/payload/operation_budget_usage")
                .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
            pack.carry_verified_prior_budgets(
                prior_receipts,
                &owner.replay.original_request_id,
                owner.stop,
            )
            .map_err(context_error)?;
            let actual_usage_bytes = wow_core::canonical_json_bytes(&pack.operation_budget_usage())
                .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
            let saved_usage_bytes = wow_core::canonical_json_bytes(saved_usage)
                .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
            if saved_pack_bytes.as_slice() != pack.canonical_bytes()
                || saved_artifact_bytes != actual_artifact_bytes
                || saved_usage_bytes != actual_usage_bytes
                || envelope.get("resource_closure").and_then(Value::as_str) != Some("closed")
                || envelope
                    .pointer("/payload/continuation_id")
                    .and_then(Value::as_str)
                    != Some(receipt.record.continuation_id.as_ref())
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            pack.into_budget_receipt_after_delivery(&response, replay_usage, owner.stop)
                .map_err(context_error)
        },
    )
}

fn finalize_continue_pack<'a>(
    universe: &'a wow_context::ContextUniverseSet<'a>,
    mut pack: wow_context::ContextSemanticPack<'a, 'a>,
    request: &ContextContinueRequest,
    request_id: &str,
    replay: &ReplaySelection,
    next_continuation_id: Option<&str>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus, Box<str>)> {
    let artifacts = render_pack(universe, &mut pack, replay.render, stop)?;
    let budget = pack.operation_budget_usage();
    let status = match pack.state() {
        wow_context::ProjectMapState::Partial => ContextMapStatus::Partial,
        wow_context::ProjectMapState::Truncated => ContextMapStatus::Truncated,
    };
    let semantic_pack_id = pack.semantic_pack_id().to_owned().into_boxed_str();
    let envelope = ContinueEnvelope {
        schema: CONTEXT_CONTINUE_RESULT_SCHEMA,
        operation: "context_continue",
        request_id,
        request,
        status,
        resource_closure: "closed",
        failure_code: None,
        payload: Some(ContinuePayload {
            original_request_id: &replay.original_request_id,
            resolved_request_id: &replay.resolved_request_id,
            publication_set_id: &replay.publication_set_id,
            store_generation_id: replay.store_generation_id.as_str(),
            universe_set_id: &replay.universe_set_id,
            semantic_pack_byte_length: pack.canonical_bytes().len(),
            semantic_pack: &pack,
            rendered_artifacts: &artifacts,
            operation_budget_usage: budget,
            validation: "exact_native_binding_and_canonical_bytes",
            continuation_id: next_continuation_id,
        }),
    };
    let remaining = usize::try_from(pack.remaining_metadata_bytes())
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    let bytes = super::operation::encode(&envelope, remaining, stop)?;
    let delivered = pack
        .into_progressive_step_after_delivery(&bytes, stop)
        .map_err(context_error)?;
    drop(delivered);
    Ok((bytes, status, semantic_pack_id))
}

fn verify_replay_selection(
    read: &crate::live_project::LiveProjectRead,
    host: &ContextMapHostInput,
    replay: &ReplaySelection,
    request: &ContextBuildRequest,
    universe: &wow_context::ContextUniverseSet<'_>,
) -> ServiceResult<()> {
    if request.project_store_id != host.project_store_id
        || request.project_store_id != replay.project_store_id
        || read.store_generation_id() != &replay.store_generation_id
        || read.publication_set_id() != replay.publication_set_id.as_ref()
        || universe.universe_set_id() != replay.universe_set_id.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    if let Some(guard) = &request.reference
        && (guard.profile != *host.reference_profile.profile_id()
            || guard.generation.to_string() != host.reference.generation_id()
            || guard.view.as_ref() != host.reference.self_digest())
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let resolved = super::build::resolved_request_id(
        read,
        host,
        &replay.original_request_id,
        &request.root,
        universe,
    )?;
    if resolved != replay.resolved_request_id {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    Ok(())
}

fn continue_inner(
    host: &ContextMapHostInput,
    request: &ContextContinueRequest,
    request_id: &str,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    store: &mut Store,
    stop: &AtomicBool,
) -> Result<ContextMapResult, (&'static str, ServiceError)> {
    if let Some(result) = replay_completed_result(store, operation_id, request_digest, host, stop)
        .map_err(|error| ("closed", error))?
    {
        return Ok(result);
    }
    let latest = receipt_by_token(store, &request.continuation_id)
        .map_err(|error| ("not_acquired", error))?;
    let chain = receipt_chain(store, latest).map_err(|error| ("not_acquired", error))?;
    let latest = chain
        .last()
        .ok_or_else(|| ("not_acquired", failure(ServiceErrorCode::InvalidRequest)))?;
    if request
        .expected_original_request_id
        .as_deref()
        .is_some_and(|expected| expected != latest.record.replay.original_request_id.as_ref())
        || request
            .expected_original_semantic_pack_id
            .as_deref()
            .is_some_and(|expected| expected != latest.record.semantic_pack_id.as_ref())
    {
        return Err(("not_acquired", failure(ServiceErrorCode::IdentityMismatch)));
    }
    let replay = latest.record.replay.clone();
    verify_owner_binding(host, &replay.owner_binding_digest)
        .map_err(|error| ("not_acquired", error))?;
    if usize::from(latest.record.completed_rounds) >= replay.rounds.len()
        || chain.len() != usize::from(latest.record.completed_rounds)
    {
        return Err(("not_acquired", failure(ServiceErrorCode::InvalidRequest)));
    }
    verify_retention_root(host, &latest.record.retention_root)
        .map_err(|error| ("not_acquired", error))?;
    checkpoint(stop).map_err(|error| ("not_acquired", error))?;
    let history =
        history_for_active_chain(store, &chain).map_err(|error| ("not_acquired", error))?;
    let build_request = replay
        .build_request()
        .map_err(|error| ("not_acquired", error))?;
    let selector = ReadSelector::Exact(replay.store_generation_id.clone());
    let (native, closure) =
        with_checked_context_read(host, &replay.project_store_id, &selector, stop, |read| {
            let project = read.project();
            if project.configuration().project_id() != &host.project_id {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            let universe = wow_context::bind_context_universe_set(
                read.pair(),
                &host.reference,
                &host.reference_profile,
                host.limits.owner_limits()?,
                stop,
            )
            .map_err(context_error)?;
            verify_replay_selection(read, host, &replay, &build_request, &universe)?;
            let replay_owner = ReplayOwnerContext {
                universe: &universe,
                read,
                host,
                replay: &replay,
                store,
                stop,
            };
            let mut prior_budgets = Vec::with_capacity(chain.len());
            for (index, receipt) in chain.iter().enumerate() {
                let prefix = replay.graph_rounds(index + 1)?;
                let verified =
                    replay_prefix_budget(&replay_owner, receipt, &prefix, &prior_budgets)?;
                prior_budgets.push(verified);
            }
            let rounds = replay.graph_rounds(usize::from(latest.record.completed_rounds) + 1)?;
            let ContextMapRoot::GraphEntity(root_id) = &replay.root else {
                return Err(failure(ServiceErrorCode::InvalidRequest));
            };
            let scope = wow_context::L0Scope::Entity(root_id.clone());
            with_native_progressive(
                &universe,
                read,
                host,
                replay.root.clone(),
                &rounds,
                stop,
                |step| {
                    let mut pack = wow_context::build_context_semantic_pack_from_expansion(
                        &universe, &scope, step, stop,
                    )
                    .map_err(context_error)?;
                    pack.carry_verified_prior_budgets(
                        &prior_budgets,
                        &replay.original_request_id,
                        stop,
                    )
                    .map_err(context_error)?;
                    let new_round = latest.record.completed_rounds + 1;
                    let next = if usize::from(new_round) < replay.rounds.len() {
                        Some(continuation_id(
                            &replay.original_request_id,
                            &replay.store_generation_id,
                            new_round,
                        )?)
                    } else {
                        None
                    };
                    finalize_continue_pack(
                        &universe,
                        pack,
                        request,
                        request_id,
                        &replay,
                        next.as_deref(),
                        stop,
                    )
                },
            )
        });
    let (response_bytes, status, semantic_pack_id) = native.map_err(|error| (closure, error))?;
    verify_retention_root(host, &latest.record.retention_root)
        .map_err(|error| ("closed", error))?;
    let response = response_value(&response_bytes).map_err(|error| ("closed", error))?;
    let next_id = response
        .pointer("/payload/continuation_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let new_round = latest.record.completed_rounds + 1;
    let receipt = next_id.map(|id| ReceiptDraft {
        continuation_id: id.into_boxed_str(),
        parent_receipt_object_id: Some(latest.receipt_object_id.clone()),
        replay: replay.clone(),
        completed_rounds: new_round,
        semantic_pack_id,
        retention_root: latest.record.retention_root.clone(),
    });
    let release = receipt.is_none();
    let persisted = persist_operation(
        store,
        OperationWrite {
            operation_id: operation_id.clone(),
            request_digest: request_digest.clone(),
            response_bytes: &response_bytes,
            receipt,
            history: HistoryTransition::Continue {
                prior: Box::new(history),
            },
            consumed_continuation: Some((
                request.continuation_id.clone(),
                latest.receipt_object_id.clone(),
            )),
            retention_root: latest.record.retention_root.clone(),
            release_retention_root: release,
            owner_binding_digest: replay.owner_binding_digest.clone(),
        },
        stop,
    )
    .map_err(|error| ("closed", error))?;
    let (outcome, bytes) = persisted;
    ensure_retention_for_operation_outcome(host, store, &outcome)
        .map_err(|error| ("failed", error))?;
    checkpoint(stop).map_err(|error| ("closed", error))?;
    Ok(ContextMapResult {
        bytes,
        status,
        code: None,
    })
}

pub fn execute_context_continue(
    host: &ContextMapHostInput,
    request: &ContextContinueRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let (request_id, operation_id, request_digest) = continue_operation_identity(request)?;
    if let Err(error) = host.validate(stop) {
        return failed_continue(request, &request_id, error.code(), "not_acquired", stop);
    }
    let mut store = match open_continuation_store(host) {
        Ok(store) => store,
        Err(error) => {
            return failed_continue(request, &request_id, error.code(), "not_acquired", stop);
        }
    };
    match continue_inner(
        host,
        request,
        &request_id,
        &operation_id,
        &request_digest,
        &mut store,
        stop,
    ) {
        Ok(result) => Ok(result),
        Err((closure, error)) => failed_continue(request, &request_id, error.code(), closure, stop),
    }
}

#[derive(Serialize)]
struct ContinuePayload<'a, 'm, 'view> {
    original_request_id: &'a str,
    resolved_request_id: &'a str,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    universe_set_id: &'a str,
    semantic_pack: &'a wow_context::ContextSemanticPack<'m, 'view>,
    semantic_pack_byte_length: usize,
    rendered_artifacts: &'a [wow_context::RenderedContextArtifact],
    operation_budget_usage: wow_context::ContextOperationBudgetUsage,
    validation: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation_id: Option<&'a str>,
}

#[derive(Serialize)]
struct ContinueEnvelope<'a, 'm, 'view> {
    schema: &'static str,
    operation: &'static str,
    request_id: &'a str,
    request: &'a ContextContinueRequest,
    status: ContextMapStatus,
    resource_closure: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_code: Option<ServiceErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<ContinuePayload<'a, 'm, 'view>>,
}

fn failed_continue(
    request: &ContextContinueRequest,
    request_id: &str,
    code: ServiceErrorCode,
    closure: &'static str,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let envelope = ContinueEnvelope {
        schema: CONTEXT_CONTINUE_RESULT_SCHEMA,
        operation: "context_continue",
        request_id,
        request,
        status: if code == ServiceErrorCode::Cancelled {
            ContextMapStatus::Cancelled
        } else {
            ContextMapStatus::Failed
        },
        resource_closure: closure,
        failure_code: Some(code),
        payload: None,
    };
    let bytes = super::operation::encode(&envelope, CONTEXT_MAP_RESULT_MAX_BYTES, stop)?;
    Ok(ContextMapResult {
        bytes: bytes.into_boxed_slice(),
        status: envelope.status,
        code: Some(code),
    })
}

#[derive(Serialize)]
struct ContinuationHistoryPrunePayload<'a> {
    original_request_id: &'a str,
    terminal_operation_id: &'a str,
    pruned_response_count: usize,
    pruned_receipt_count: usize,
}

#[derive(Serialize)]
struct ContinuationHistoryPruneEnvelope<'a> {
    schema: &'static str,
    operation: &'static str,
    request_id: &'a str,
    request: &'a ContextContinuationHistoryPruneRequest,
    status: ContextMapStatus,
    resource_closure: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_code: Option<ServiceErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<ContinuationHistoryPrunePayload<'a>>,
}

fn failed_history_prune(
    request: &ContextContinuationHistoryPruneRequest,
    request_id: &str,
    code: ServiceErrorCode,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let envelope = ContinuationHistoryPruneEnvelope {
        schema: CONTEXT_CONTINUATION_HISTORY_PRUNE_RESULT_SCHEMA,
        operation: "context_continuation_history_prune",
        request_id,
        request,
        status: if code == ServiceErrorCode::Cancelled {
            ContextMapStatus::Cancelled
        } else {
            ContextMapStatus::Failed
        },
        resource_closure: "closed",
        failure_code: Some(code),
        payload: None,
    };
    let bytes = super::operation::encode(&envelope, CONTEXT_MAP_RESULT_MAX_BYTES, stop)?;
    Ok(ContextMapResult {
        bytes: bytes.into_boxed_slice(),
        status: envelope.status,
        code: Some(code),
    })
}

fn completed_history_prune_result(
    store: &Store,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    request_id: &str,
) -> ServiceResult<Option<ContextMapResult>> {
    let Some(operation) = store.operation(operation_id).map_err(|_| store_failure())? else {
        return Ok(None);
    };
    if operation.request_digest() != request_digest {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    if operation.state() != OperationState::Completed {
        return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
    }
    let result_id = operation.result_object_id().ok_or_else(store_failure)?;
    let object = store
        .object(result_id)
        .map_err(|_| store_failure())?
        .ok_or_else(store_failure)?;
    if object.kind() != HISTORY_PRUNE_RESULT_KIND
        || object.schema_version() != RESPONSE_SCHEMA_VERSION
    {
        return Err(store_failure());
    }
    let value = response_value(object.canonical_json())?;
    if value.get("schema").and_then(Value::as_str)
        != Some(CONTEXT_CONTINUATION_HISTORY_PRUNE_RESULT_SCHEMA)
        || value.get("operation").and_then(Value::as_str)
            != Some("context_continuation_history_prune")
        || value.get("request_id").and_then(Value::as_str) != Some(request_id)
    {
        return Err(store_failure());
    }
    response_result(object.canonical_json().into()).map(Some)
}

fn validate_terminal_history(
    store: &Store,
    host: &ContextMapHostInput,
    history: &LoadedContinuationHistory,
) -> ServiceResult<Vec<(OperationId, RequestDigest, ObjectId)>> {
    validate_history_record(&history.record)?;
    if history.record.active_continuation_id.is_some() {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    verify_owner_binding(host, &history.record.owner_binding_digest)?;

    let catalog = CatalogName::new(CONTINUATION_CATALOG).map_err(|_| store_failure())?;
    let history_path = history_catalog_path(&history.record.original_request_id)?;
    let history_object_id = history
        .object_id
        .as_ref()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?;
    if catalog_target(store, &catalog, &history_path)?.as_ref() != Some(history_object_id) {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }

    let latest_receipt_id = history
        .record
        .receipts
        .last()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?
        .receipt_object_id
        .clone();
    let latest_receipt = receipt_from_object(store, &latest_receipt_id)?;
    let chain = receipt_chain(store, latest_receipt)?;
    if chain.len() != history.record.receipts.len() {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    for (recorded, actual) in history.record.receipts.iter().zip(&chain) {
        if recorded.continuation_id != actual.record.continuation_id
            || recorded.receipt_object_id != actual.receipt_object_id
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        let root_path = continuation_catalog_path(&recorded.continuation_id)?;
        if catalog_target(store, &catalog, &root_path)?.as_ref()
            != Some(&recorded.receipt_object_id)
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
    }

    let terminal_operation_id = history
        .record
        .terminal_operation_id
        .as_ref()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?;
    let mut completed_results = Vec::with_capacity(history.record.responses.len());
    for (index, response) in history.record.responses.iter().enumerate() {
        let response_path = response_root_catalog_path(&response.operation_id)?;
        if catalog_target(store, &catalog, &response_path)?.as_ref()
            != Some(&response.response_object_id)
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        let operation = store
            .operation(&response.operation_id)
            .map_err(|_| store_failure())?
            .ok_or_else(|| failure(ServiceErrorCode::StoreOutcomeUnknown))?;
        if operation.state() != OperationState::Completed {
            return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
        }
        let result_id = operation
            .result_object_id()
            .ok_or_else(|| failure(ServiceErrorCode::StoreOutcomeUnknown))?;
        let outcome = outcome_object(store, result_id)?;
        if outcome.schema.as_ref() != OUTCOME_SCHEMA
            || outcome.owner_binding_digest != history.record.owner_binding_digest
            || outcome.retention_root != history.record.retention_root
            || outcome.response_object_id != response.response_object_id
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        if index < history.record.receipts.len() {
            let receipt = &history.record.receipts[index];
            if outcome.next_continuation_id.as_deref() != Some(receipt.continuation_id.as_ref())
                || outcome.next_receipt_object_id.as_ref() != Some(&receipt.receipt_object_id)
                || outcome.release_retention_root
                || chain[index].record.issuer_operation_id != response.operation_id
                || chain[index].record.response_object_id != response.response_object_id
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
        } else if &response.operation_id != terminal_operation_id
            || outcome.next_continuation_id.is_some()
            || outcome.next_receipt_object_id.is_some()
            || !outcome.release_retention_root
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        completed_results.push((
            response.operation_id.clone(),
            operation.request_digest().clone(),
            result_id.clone(),
        ));
    }
    Ok(completed_results)
}

fn prune_terminal_history(
    host: &ContextMapHostInput,
    request: &ContextContinuationHistoryPruneRequest,
    request_id: &str,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    store: &mut Store,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    if let Some(result) =
        completed_history_prune_result(store, operation_id, request_digest, request_id)?
    {
        return Ok(result);
    }
    let history = load_history(store, request.original_request_id())?
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    let completed_results = validate_terminal_history(store, host, &history)?;
    checkpoint(stop)?;

    let terminal_operation_id = history
        .record
        .terminal_operation_id
        .as_ref()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?;
    let envelope = ContinuationHistoryPruneEnvelope {
        schema: CONTEXT_CONTINUATION_HISTORY_PRUNE_RESULT_SCHEMA,
        operation: "context_continuation_history_prune",
        request_id,
        request,
        status: ContextMapStatus::Complete,
        resource_closure: "closed",
        failure_code: None,
        payload: Some(ContinuationHistoryPrunePayload {
            original_request_id: request.original_request_id(),
            terminal_operation_id: terminal_operation_id.as_str(),
            pruned_response_count: history.record.responses.len(),
            pruned_receipt_count: history.record.receipts.len(),
        }),
    };
    let response_bytes = super::operation::encode(&envelope, CONTEXT_MAP_RESULT_MAX_BYTES, stop)?;
    let prune_result = PendingObject::from_json(
        HISTORY_PRUNE_RESULT_KIND,
        RESPONSE_SCHEMA_VERSION,
        &envelope,
        continuation_store_configuration()
            .map_err(|_| store_failure())?
            .limits(),
    )
    .map_err(|_| store_failure())?;
    if prune_result.canonical_json() != response_bytes {
        return Err(failure(ServiceErrorCode::CanonicalizationFailed));
    }

    match store
        .begin_operation(operation_id.clone(), request_digest.clone())
        .map_err(|_| store_failure())?
    {
        OperationBegin::Started(_) => {}
        OperationBegin::Replay(record) if record.state() == OperationState::Completed => {
            return completed_history_prune_result(
                store,
                operation_id,
                request_digest,
                request_id,
            )?
            .ok_or_else(store_failure);
        }
        OperationBegin::Replay(_) => return Err(failure(ServiceErrorCode::StoreOutcomeUnknown)),
    }

    let catalog = CatalogName::new(CONTINUATION_CATALOG).map_err(|_| store_failure())?;
    let history_object_id = history
        .object_id
        .clone()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?;
    let mut batch = WriteBatch::new();
    batch
        .add_object(prune_result.clone())
        .map_err(|_| store_failure())?;
    for response in &history.record.responses {
        batch
            .add_catalog_mutation(CatalogMutation::delete(
                catalog.clone(),
                response_root_catalog_path(&response.operation_id)?,
                response.response_object_id.clone(),
            ))
            .map_err(|_| store_failure())?;
    }
    for receipt in &history.record.receipts {
        batch
            .add_catalog_mutation(CatalogMutation::delete(
                catalog.clone(),
                continuation_catalog_path(&receipt.continuation_id)?,
                receipt.receipt_object_id.clone(),
            ))
            .map_err(|_| store_failure())?;
    }
    for (completed_operation_id, digest, previous_result_object_id) in completed_results {
        batch
            .add_operation_result_mutation(
                OperationResultMutation::replace_completed(
                    completed_operation_id,
                    digest,
                    previous_result_object_id,
                    prune_result.object_id().clone(),
                )
                .map_err(|_| store_failure())?,
            )
            .map_err(|_| store_failure())?;
    }
    batch
        .add_catalog_mutation(CatalogMutation::set(
            catalog,
            history_catalog_path(request.original_request_id())?,
            CatalogExpectation::Exact(history_object_id),
            prune_result.object_id().clone(),
        ))
        .map_err(|_| store_failure())?;
    if batch.catalog_mutations().len() > 8 {
        return Err(failure(ServiceErrorCode::BudgetExceeded));
    }
    let completed = match store.commit_operation(
        batch,
        operation_id,
        request_digest,
        prune_result.object_id(),
    ) {
        Ok(completed) => completed,
        Err(_) => {
            let _ = store.mark_outcome_unknown(operation_id, request_digest);
            return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
        }
    };
    if completed.state() != OperationState::Completed
        || completed.request_digest() != request_digest
        || completed.result_object_id() != Some(prune_result.object_id())
    {
        return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
    }
    response_result(response_bytes.into_boxed_slice())
}

pub fn execute_context_continuation_history_prune(
    host: &ContextMapHostInput,
    request: &ContextContinuationHistoryPruneRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    checkpoint(stop)?;
    let (request_id, operation_id, request_digest) =
        history_prune_operation_identity(host, request)?;
    if let Err(error) = host.validate(stop) {
        return failed_history_prune(request, &request_id, error.code(), stop);
    }
    let mut store = match open_continuation_store(host) {
        Ok(store) => store,
        Err(error) => return failed_history_prune(request, &request_id, error.code(), stop),
    };
    match prune_terminal_history(
        host,
        request,
        &request_id,
        &operation_id,
        &request_digest,
        &mut store,
        stop,
    ) {
        Ok(result) => Ok(result),
        Err(error) => failed_history_prune(request, &request_id, error.code(), stop),
    }
}

fn render_pack<'m, 'view>(
    universe: &wow_context::ContextUniverseSet<'view>,
    pack: &mut wow_context::ContextSemanticPack<'m, 'view>,
    render: RenderMode,
    stop: &AtomicBool,
) -> ServiceResult<Vec<wow_context::RenderedContextArtifact>> {
    wow_context::validate_context_semantic_pack(universe, pack, stop).map_err(context_error)?;
    let mut artifacts = Vec::new();
    if matches!(render, RenderMode::Json | RenderMode::Both) {
        let artifact = wow_context::render_context_pack_json(pack, stop).map_err(context_error)?;
        wow_context::validate_rendered_context_artifact(pack, &artifact, stop)
            .map_err(context_error)?;
        artifacts.push(artifact);
    }
    if matches!(render, RenderMode::Markdown | RenderMode::Both) {
        let artifact =
            wow_context::render_context_pack_markdown(pack, stop).map_err(context_error)?;
        wow_context::validate_rendered_context_artifact(pack, &artifact, stop)
            .map_err(context_error)?;
        artifacts.push(artifact);
    }
    Ok(artifacts)
}

fn persist_operation(
    store: &mut Store,
    operation: OperationWrite<'_>,
    stop: &AtomicBool,
) -> ServiceResult<(ContinuationOperationOutcome, Box<[u8]>)> {
    persist_operation_inner(store, operation, stop, false)
}

fn persist_prepared_operation(
    store: &mut Store,
    operation: OperationWrite<'_>,
    stop: &AtomicBool,
) -> ServiceResult<(ContinuationOperationOutcome, Box<[u8]>)> {
    persist_operation_inner(store, operation, stop, true)
}

fn next_history_record(
    transition: HistoryTransition,
    operation_id: &OperationId,
    response_object_id: &ObjectId,
    receipt: Option<&(ContinuationRecord, PendingObject)>,
    consumed_continuation: Option<&(Box<str>, ObjectId)>,
    retention_root: &RetentionRoot,
    owner_binding_digest: &str,
) -> ServiceResult<(ContinuationHistoryRecord, CatalogExpectation)> {
    match transition {
        HistoryTransition::Initial {
            original_request_id,
        } => {
            let Some((receipt, object)) = receipt else {
                return Err(failure(ServiceErrorCode::InternalContractViolation));
            };
            if !valid_original_request_id(&original_request_id)
                || receipt.replay.original_request_id != original_request_id
                || receipt.completed_rounds != 1
                || receipt.parent_receipt_object_id.is_some()
                || receipt.retention_root != *retention_root
                || receipt.replay.owner_binding_digest.as_ref() != owner_binding_digest
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            let record = ContinuationHistoryRecord {
                schema: HISTORY_SCHEMA.into(),
                original_request_id,
                owner_binding_digest: owner_binding_digest.into(),
                retention_root: retention_root.clone(),
                responses: vec![ContinuationHistoryResponse {
                    operation_id: operation_id.clone(),
                    response_object_id: response_object_id.clone(),
                }],
                receipts: vec![ContinuationHistoryReceipt {
                    continuation_id: receipt.continuation_id.clone(),
                    receipt_object_id: object.object_id().clone(),
                }],
                active_continuation_id: Some(receipt.continuation_id.clone()),
                terminal_operation_id: None,
            };
            validate_history_record(&record)?;
            Ok((record, CatalogExpectation::Absent))
        }
        HistoryTransition::Continue { prior } => {
            let prior = *prior;
            validate_history_record(&prior.record)?;
            let Some((consumed_token, expected_receipt)) = consumed_continuation else {
                return Err(failure(ServiceErrorCode::InternalContractViolation));
            };
            if prior.record.active_continuation_id.as_deref() != Some(consumed_token.as_ref())
                || prior.record.owner_binding_digest.as_ref() != owner_binding_digest
                || prior.record.retention_root != *retention_root
                || prior
                    .record
                    .receipts
                    .last()
                    .is_none_or(|receipt| &receipt.receipt_object_id != expected_receipt)
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }

            let history_expectation = prior
                .object_id
                .map(CatalogExpectation::Exact)
                .unwrap_or(CatalogExpectation::Absent);
            let mut record = prior.record;
            record.responses.push(ContinuationHistoryResponse {
                operation_id: operation_id.clone(),
                response_object_id: response_object_id.clone(),
            });
            match receipt {
                Some((next, object)) => {
                    if next.replay.original_request_id != record.original_request_id
                        || next.replay.owner_binding_digest.as_ref() != owner_binding_digest
                        || next.retention_root != *retention_root
                        || next.parent_receipt_object_id.as_ref() != Some(expected_receipt)
                        || next.completed_rounds
                            != u8::try_from(record.receipts.len() + 1).unwrap_or(u8::MAX)
                    {
                        return Err(failure(ServiceErrorCode::IdentityMismatch));
                    }
                    record.receipts.push(ContinuationHistoryReceipt {
                        continuation_id: next.continuation_id.clone(),
                        receipt_object_id: object.object_id().clone(),
                    });
                    record.active_continuation_id = Some(next.continuation_id.clone());
                    record.terminal_operation_id = None;
                }
                None => {
                    record.active_continuation_id = None;
                    record.terminal_operation_id = Some(operation_id.clone());
                }
            }
            validate_history_record(&record)?;
            Ok((record, history_expectation))
        }
    }
}

fn persist_operation_inner(
    store: &mut Store,
    operation: OperationWrite<'_>,
    stop: &AtomicBool,
    already_prepared: bool,
) -> ServiceResult<(ContinuationOperationOutcome, Box<[u8]>)> {
    let OperationWrite {
        operation_id,
        request_digest,
        response_bytes,
        receipt,
        history,
        consumed_continuation,
        retention_root,
        release_retention_root,
        owner_binding_digest,
    } = operation;
    checkpoint(stop)?;
    let value = response_value(response_bytes)?;
    let response = pending_json(RESPONSE_KIND, &value)?;
    if response.canonical_json() != response_bytes {
        return Err(failure(ServiceErrorCode::CanonicalizationFailed));
    }

    let receipt = receipt
        .map(|draft| {
            if draft.replay.owner_binding_digest.as_ref() != owner_binding_digest.as_ref() {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            let record = ContinuationRecord {
                schema: RECEIPT_SCHEMA.into(),
                continuation_id: draft.continuation_id,
                issuer_operation_id: operation_id.clone(),
                issuer_request_digest: request_digest.clone(),
                response_object_id: response.object_id().clone(),
                parent_receipt_object_id: draft.parent_receipt_object_id,
                replay: draft.replay,
                completed_rounds: draft.completed_rounds,
                semantic_pack_id: draft.semantic_pack_id,
                retention_root: draft.retention_root,
            };
            let object = PendingObject::from_json(
                RECEIPT_KIND,
                RESPONSE_SCHEMA_VERSION,
                &record,
                continuation_store_configuration()
                    .map_err(|_| store_failure())?
                    .limits(),
            )
            .map_err(|_| store_failure())?;
            Ok::<_, ServiceError>((record, object))
        })
        .transpose()?;

    let next_id = receipt
        .as_ref()
        .map(|(record, _)| record.continuation_id.clone());
    let next_receipt_object_id = receipt
        .as_ref()
        .map(|(_, object)| object.object_id().clone());
    let outcome = ContinuationOperationOutcome {
        schema: OUTCOME_SCHEMA.into(),
        owner_binding_digest,
        response_object_id: response.object_id().clone(),
        next_continuation_id: next_id,
        next_receipt_object_id: next_receipt_object_id.clone(),
        retention_root,
        release_retention_root,
    };
    let outcome_object = PendingObject::from_json(
        OUTCOME_KIND,
        RESPONSE_SCHEMA_VERSION,
        &outcome,
        continuation_store_configuration()
            .map_err(|_| store_failure())?
            .limits(),
    )
    .map_err(|_| store_failure())?;

    let (history_record, history_expectation) = next_history_record(
        history,
        &operation_id,
        response.object_id(),
        receipt.as_ref(),
        consumed_continuation.as_ref(),
        &outcome.retention_root,
        &outcome.owner_binding_digest,
    )?;
    let history_object = pending_history_object(&history_record)?;

    if already_prepared {
        let Some(record) = store
            .operation(&operation_id)
            .map_err(|_| store_failure())?
        else {
            return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
        };
        if record.request_digest() != &request_digest {
            return Err(failure(ServiceErrorCode::OperationConflict));
        }
        match record.state() {
            OperationState::Prepared => {}
            OperationState::Completed => {
                return read_completed_outcome(store, &operation_id, &request_digest);
            }
            _ => return Err(failure(ServiceErrorCode::StoreOutcomeUnknown)),
        }
    } else {
        match store
            .begin_operation(operation_id.clone(), request_digest.clone())
            .map_err(|_| store_failure())?
        {
            OperationBegin::Replay(record) if record.state() == OperationState::Completed => {
                return read_completed_outcome(store, &operation_id, &request_digest);
            }
            OperationBegin::Replay(_) => {
                return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
            }
            OperationBegin::Started(_) => {}
        }
    }

    let mut batch = WriteBatch::new();
    let catalog = CatalogName::new(CONTINUATION_CATALOG).map_err(|_| store_failure())?;
    batch.add_object(response).map_err(|_| store_failure())?;
    batch
        .add_object(history_object.clone())
        .map_err(|_| store_failure())?;
    if let Some((_, object)) = receipt {
        batch
            .add_object(object.clone())
            .map_err(|_| store_failure())?;
    }
    for response in &history_record.responses {
        ensure_root_mutation(
            store,
            &mut batch,
            &catalog,
            response_root_catalog_path(&response.operation_id)?,
            &response.response_object_id,
        )?;
    }
    for receipt in &history_record.receipts {
        ensure_root_mutation(
            store,
            &mut batch,
            &catalog,
            continuation_catalog_path(&receipt.continuation_id)?,
            &receipt.receipt_object_id,
        )?;
    }
    batch
        .add_catalog_mutation(CatalogMutation::set(
            catalog,
            history_catalog_path(&history_record.original_request_id)?,
            history_expectation,
            history_object.object_id().clone(),
        ))
        .map_err(|_| store_failure())?;
    batch
        .add_object(outcome_object.clone())
        .map_err(|_| store_failure())?;
    if store
        .commit_operation(
            batch,
            &operation_id,
            &request_digest,
            outcome_object.object_id(),
        )
        .is_err()
    {
        let _ = store.mark_outcome_unknown(&operation_id, &request_digest);
        return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
    }
    let (actual, bytes) = read_completed_outcome(store, &operation_id, &request_digest)?;
    if actual != outcome
        || actual.next_receipt_object_id != next_receipt_object_id
        || actual.next_continuation_id != outcome.next_continuation_id
    {
        return Err(store_failure());
    }
    Ok((actual, bytes))
}
