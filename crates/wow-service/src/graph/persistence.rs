//! Explicit retained-artifact publication. Storage coherence is not full E2
//! semantic acceptance, live analyzer rehydration or provenance authentication.
use super::{
    GRAPH_BUNDLE_MAX_BYTES, GraphReadOperation, GraphReadResult, GraphReadStatus, bundle, input,
};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path, sync::atomic::AtomicBool};
use wow_graph::GraphPartitionSnapshot;
use wow_project::graph::persistence as project;
use wow_store::project::{
    CurrentPublication, CurrentRecordId, PartitionRecord, ProjectStore, PublicationOperation,
    PublicationRequest, PublicationState, ReadSelector, ReadSnapshot, RecordCatalog,
    StoreGenerationId,
};
use wow_store::{OperationId, StoreError, StoreErrorCode};

const RECEIPT_SCHEMA: &str = "wow-service.graph-build-receipt.v7";
const RECEIPT_KEY: &str = "service.receipt";
const RECEIPT_CHECK: &str = "wow-service.graph-build-receipt-integrity.v7";
const SCOPE: &str = "retained-graph-bundle-v7";

pub struct GraphStorePublishRequest {
    operation_id: OperationId,
    expected: Option<CurrentRecordId>,
    initialize: bool,
}
impl GraphStorePublishRequest {
    pub fn new(
        operation_id: &str,
        expected_current: &str,
        initialize: bool,
        allow_partial: bool,
    ) -> ServiceResult<Self> {
        if !allow_partial {
            return Err(fail(ServiceErrorCode::InvalidRequest));
        }
        let operation_id = OperationId::new(operation_id).map_err(store_error)?;
        let expected = if expected_current == "absent" {
            None
        } else {
            Some(CurrentRecordId::parse(expected_current).map_err(store_error)?)
        };
        Ok(Self {
            operation_id,
            expected,
            initialize,
        })
    }
}
#[derive(Debug, Serialize)]
pub struct GraphStorePublishResult {
    schema: &'static str,
    scope: &'static str,
    operation_id: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<PublicationOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<CurrentPublication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<ServiceErrorCode>,
    boundaries: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_digest: Option<Box<str>>,
}
impl GraphStorePublishResult {
    pub fn exit_code(&self) -> u8 {
        match self.status {
            "activated" => 2,
            "cancelled" => 130,
            "operation_not_retained" => 3,
            "observed" => 0,
            _ => 4,
        }
    }
    pub fn canonical_bytes(&self) -> ServiceResult<Vec<u8>> {
        let bytes = wow_core::canonical_json_bytes(self)
            .map_err(|_| fail(ServiceErrorCode::CanonicalizationFailed))?;
        if bytes.len() > 128 * 1024 {
            return Err(fail(ServiceErrorCode::BudgetExceeded));
        }
        Ok(bytes)
    }
    fn new(id: &OperationId) -> Self {
        Self {
            schema: "wow-service/graph-store-publication-result/1",
            scope: SCOPE,
            operation_id: id.as_str().into(),
            status: "failed",
            operation: None,
            current: None,
            failure: None,
            result_digest: None,
            boundaries: vec![
                "partial_retained_metadata_only",
                "not_full_e2_project_publication",
                "source_and_runtime_not_verified",
                "no_automatic_retry_rollback_or_garbage_collection",
                "power_loss_and_platform_acceptance_not_established",
            ],
        }
    }
    fn seal(mut self) -> ServiceResult<Self> {
        self.result_digest = Some(super::hash(&self.canonical_bytes()?));
        self.canonical_bytes()?;
        Ok(self)
    }
}
#[derive(Debug, Clone, Serialize)]
pub(super) struct StoredContext {
    scope: &'static str,
    epoch_id: String,
    store_generation_id: String,
    bindings: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_at_acquisition: Option<CurrentPublication>,
}

pub fn publish_graph_bundle(
    bytes: &[u8],
    root: &Path,
    request: &GraphStorePublishRequest,
    stop: &AtomicBool,
) -> ServiceResult<GraphStorePublishResult> {
    let mut result = GraphStorePublishResult::new(&request.operation_id);
    // All input admission and owner record planning precedes store creation.
    let admitted = bundle::admit(bytes, stop).map_err(admission_error)?;
    let mut envelope: serde_json::Map<String, Value> =
        input::decode_bundle(bytes, GRAPH_BUNDLE_MAX_BYTES, stop).map_err(admission_error)?;
    envelope
        .remove("snapshot")
        .ok_or_else(|| fail(ServiceErrorCode::InvalidRequest))?;
    let provenance = envelope
        .remove("provenance")
        .ok_or_else(|| fail(ServiceErrorCode::InvalidRequest))?;
    let mut records = admitted.owner.storage_records(stop).map_err(graph_error)?;
    records.extend(project::records(&provenance, &admitted.owner, stop).map_err(project_error)?);
    records
        .push(PartitionRecord::new(RECEIPT_KEY, RECEIPT_SCHEMA, &envelope).map_err(store_error)?);
    let mut bindings = project::bindings(&provenance, &admitted.owner)
        .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?;
    bindings.insert(
        "bundle_result_digest".into(),
        admitted.result_digest.to_string(),
    );
    bindings.insert("scope".into(), SCOPE.into());
    let catalog = catalog()?;
    let owner = admitted.owner.snapshot().universe().as_str();
    let mut store = if request.initialize {
        ProjectStore::open_or_create(root, owner, catalog).map_err(store_error)?
    } else {
        ProjectStore::open(root, &catalog).map_err(store_error)?
    };
    if store.epoch().owner() != owner {
        return Err(fail(ServiceErrorCode::IdentityMismatch));
    }
    let publication = PublicationRequest::new(
        store.epoch(),
        request.operation_id.clone(),
        request.expected.clone(),
        bindings,
        records,
    )
    .map_err(store_error)?;
    let outcome: ServiceResult<PublicationOperation> = (|| {
        let operation = store.prepare(&publication, stop).map_err(store_error)?;
        let read = store
            .read(
                &ReadSelector::Exact(publication.generation().generation_id.clone()),
                stop,
            )
            .map_err(store_error)?;
        let retained = restore(&read, stop)?;
        // Check exactly what was committed, through a fresh read transaction.
        let checked = bundle::admit(&retained, stop).map_err(admission_error)?;
        if checked.result_digest != admitted.result_digest {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        let validated = read
            .owner_validation(&[
                GraphPartitionSnapshot::STORAGE_CHECK,
                project::STORAGE_CHECK,
                RECEIPT_CHECK,
            ])
            .map_err(store_error)?;
        drop(read);
        if operation.state == PublicationState::Activated {
            return Ok(operation);
        }
        store
            .validate_inactive(
                &request.operation_id,
                publication.request_digest(),
                validated,
                stop,
            )
            .map_err(store_error)?;
        store
            .activate(&request.operation_id, publication.request_digest(), stop)
            .map_err(store_error)
    })();
    match outcome {
        Ok(operation) => {
            result.status = "activated";
            result.operation = Some(operation);
        }
        Err(error) => {
            result.status = if error.code() == ServiceErrorCode::Cancelled {
                "cancelled"
            } else if error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                "outcome_unknown"
            } else {
                "failed"
            };
            result.failure = Some(error.code());
            // Observe rather than repeat an uncertain effect. Never replace an
            // activated receipt with cancellation after a noninterruptible commit.
            if let Ok(Some(operation)) = store.reconcile(&request.operation_id) {
                if operation.request_digest == publication.request_digest()
                    && operation.state == PublicationState::Activated
                {
                    result.status = "activated";
                    result.failure = None;
                }
                result.operation = Some(operation);
            }
        }
    }
    result.current = store.current().map_err(store_error)?;
    result.seal()
}

pub fn reconcile_graph_publication(
    root: &Path,
    operation_id: &str,
) -> ServiceResult<GraphStorePublishResult> {
    let id = OperationId::new(operation_id).map_err(store_error)?;
    let store = ProjectStore::open(root, &catalog()?).map_err(store_error)?;
    let mut result = GraphStorePublishResult::new(&id);
    result.operation = store.reconcile(&id).map_err(store_error)?;
    result.current = store.current().map_err(store_error)?;
    result.status = if result.operation.is_some() {
        "observed"
    } else {
        "operation_not_retained"
    };
    result.seal()
}

pub fn execute_graph_store_read(
    operation: GraphReadOperation,
    root: &Path,
    generation: &str,
    request: &[u8],
    source_root: Option<&Path>,
    stop: &AtomicBool,
) -> ServiceResult<GraphReadResult> {
    super::checkpoint(stop).map_err(admission_error)?;
    let selector = if generation == "current" {
        ReadSelector::Current
    } else {
        ReadSelector::Exact(StoreGenerationId::parse(generation).map_err(store_error)?)
    };
    let store = ProjectStore::open(root, &catalog()?).map_err(store_error)?;
    let read = store.read(&selector, stop).map_err(store_error)?;
    let bytes = restore(&read, stop)?;
    let mut result = super::execute_read(operation, &bytes, request, true, source_root, stop)?;
    result.envelope.schema = "wow-service/graph-stored-read-result/1";
    result.envelope.store_context = Some(StoredContext {
        scope: SCOPE,
        epoch_id: read.manifest().epoch_id.as_str().into(),
        store_generation_id: read.manifest().generation_id.as_str().into(),
        bindings: read.manifest().bindings.clone(),
        current_at_acquisition: read.current_at_acquisition().cloned(),
    });
    result
        .envelope
        .boundaries
        .push("exact_retained_store_generation_acquired");
    result
        .envelope
        .boundaries
        .push("full_e2_project_publication_not_implemented");
    if matches!(result.envelope.status, GraphReadStatus::Complete) {
        result.envelope.status = GraphReadStatus::Partial;
    }
    result.result_digest =
        super::hash(&super::encode(&result.envelope).map_err(super::encoding_error)?);
    result.canonical_bytes()?;
    // The read transaction and writer lease close here, before transport output.
    Ok(result)
}
fn restore(read: &ReadSnapshot, stop: &AtomicBool) -> ServiceResult<Vec<u8>> {
    let graph = GraphPartitionSnapshot::read_stored(read, stop).map_err(graph_error)?;
    let provenance = project::read_provenance(read, &graph, stop).map_err(project_error)?;
    if !read
        .manifest()
        .members
        .iter()
        .any(|m| m.key == RECEIPT_KEY && m.schema == RECEIPT_SCHEMA)
        || read.manifest().members.iter().any(|m| {
            !m.key.starts_with("graph.") && !m.key.starts_with("project.") && m.key != RECEIPT_KEY
        })
        || read.manifest().owner != graph.snapshot().universe().as_str()
    {
        return Err(fail(ServiceErrorCode::IdentityMismatch));
    }
    let mut envelope: serde_json::Map<String, Value> = read
        .record(RECEIPT_KEY, stop)
        .map_err(store_error)?
        .ok_or_else(|| fail(ServiceErrorCode::IdentityMismatch))?
        .decode()
        .map_err(store_error)?;
    let mut bindings = project::bindings(&provenance, &graph)
        .map_err(|_| fail(ServiceErrorCode::IdentityMismatch))?;
    bindings.insert(
        "bundle_result_digest".into(),
        envelope
            .get("result_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| fail(ServiceErrorCode::IdentityMismatch))?
            .into(),
    );
    bindings.insert("scope".into(), SCOPE.into());
    if bindings != read.manifest().bindings
        || envelope.contains_key("snapshot")
        || envelope.contains_key("provenance")
    {
        return Err(fail(ServiceErrorCode::IdentityMismatch));
    }
    envelope.insert(
        "snapshot".into(),
        serde_json::to_value(graph).map_err(|_| fail(ServiceErrorCode::CanonicalizationFailed))?,
    );
    envelope.insert("provenance".into(), provenance);
    let bytes = wow_core::canonical_json_bytes(&envelope)
        .map_err(|_| fail(ServiceErrorCode::CanonicalizationFailed))?;
    if bytes.len() > GRAPH_BUNDLE_MAX_BYTES {
        return Err(fail(ServiceErrorCode::BudgetExceeded));
    }
    Ok(bytes)
}
fn catalog() -> ServiceResult<RecordCatalog> {
    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    schemas.extend_from_slice(project::STORAGE_SCHEMAS);
    schemas.push(RECEIPT_SCHEMA);
    RecordCatalog::new(
        &schemas,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            project::STORAGE_CHECK,
            RECEIPT_CHECK,
        ],
    )
    .map_err(store_error)
}
fn store_error(error: StoreError) -> ServiceError {
    fail(match error.code() {
        StoreErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        StoreErrorCode::WriterBusy => ServiceErrorCode::OperationBusy,
        StoreErrorCode::CurrentConflict => ServiceErrorCode::StoreCurrentConflict,
        StoreErrorCode::OperationConflict => ServiceErrorCode::OperationConflict,
        StoreErrorCode::OutcomeUnknown => ServiceErrorCode::StoreOutcomeUnknown,
        StoreErrorCode::BudgetExceeded | StoreErrorCode::ObjectTooLarge => {
            ServiceErrorCode::BudgetExceeded
        }
        StoreErrorCode::GenerationMissing => ServiceErrorCode::ExactGenerationUnavailable,
        StoreErrorCode::IntegrityViolation => ServiceErrorCode::IdentityMismatch,
        _ => ServiceErrorCode::ComponentUnavailable,
    })
}
fn admission_error(error: super::GraphReadFailure) -> ServiceError {
    fail(error.code)
}
fn fail(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(
        code,
        "retained graph store operation failed; inspect its typed outcome",
    )
}

fn graph_error(e: wow_graph::GraphError) -> ServiceError {
    fail(match e.code() {
        wow_graph::GraphErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::IdentityMismatch,
    })
}
fn project_error(e: wow_project::ProjectError) -> ServiceError {
    fail(match e.code() {
        wow_project::ProjectErrorCode::AnalysisCancelled => ServiceErrorCode::Cancelled,
        wow_project::ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::IdentityMismatch,
    })
}
