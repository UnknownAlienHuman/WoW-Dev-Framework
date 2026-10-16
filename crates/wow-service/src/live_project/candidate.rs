//! One selected companion record in the existing manifested generation.
use super::*;
use crate::graph::{
    BlizzardUiIndexCandidate, CANDIDATE_RECORD_KEY, CANDIDATE_STORAGE_CHECK,
    CANDIDATE_STORAGE_SCHEMA, candidate_storage_record, capture_candidate_from_backend,
    read_candidate_pair,
};
use std::collections::BTreeMap;
use wow_store::project::PartitionRecord;

/// Created only from a genuine sealed candidate plus its retained publisher.
/// No receipt decoder or public caller-data constructor can create this handoff.
pub(super) enum PublicationHandoff {
    Pair(ProjectPublicationBundle),
    Candidate(CandidatePublication),
}

pub(crate) struct CandidatePublication {
    pair: ProjectPublicationBundle,
    receipt: PartitionRecord,
}
impl CandidatePublication {
    pub(crate) fn capture(
        publisher: &ProjectPublisher,
        candidate: BlizzardUiIndexCandidate,
        namespace: &ProjectStoreNamespace,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        publication_checkpoint(stop)?;
        let pair = ProjectPublicationBundle::build_with_native_parts(
            publisher,
            candidate.graph(),
            candidate.native(),
            Some(namespace),
            stop,
        )
        .map_err(project_error)?;
        let receipt = candidate_storage_record(&candidate, stop)?;
        // Dropping the original candidate avoids holding two candidate bodies
        // during inactive readback. No semantic body is cloned into the receipt.
        drop(candidate);
        publication_checkpoint(stop)?;
        Ok(Self { pair, receipt })
    }
    pub(super) fn matches_receipt(&self, previous: &PartitionRecord) -> bool {
        self.receipt.key() == previous.key() && self.receipt.version() == previous.version()
    }
    pub(super) fn into_parts(self) -> (Vec<PartitionRecord>, BTreeMap<String, String>) {
        let (mut records, bindings) = self.pair.into_parts();
        records.push(self.receipt);
        (records, bindings)
    }
}

pub(super) fn catalog(project_schemas: &[&'static str]) -> ServiceResult<RecordCatalog> {
    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    schemas.extend_from_slice(project_schemas);
    schemas.push(CANDIDATE_STORAGE_SCHEMA);
    RecordCatalog::new(
        &schemas,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            publication::STORAGE_CHECK,
            CANDIDATE_STORAGE_CHECK,
        ],
    )
    .map_err(store_error)
}

/// Epoch selection is exact. No optional receipt fallback and no old widening.
pub(crate) fn is_selected(actual: &RecordCatalog) -> ServiceResult<bool> {
    for &schemas in publication::STORAGE_SCHEMA_CATALOGS {
        if actual == &catalog(schemas)? {
            return Ok(true);
        }
        if actual == &catalog_for(schemas)? {
            return Ok(false);
        }
    }
    Err(fail(ServiceErrorCode::IdentityMismatch))
}

pub(crate) fn read_owners(
    read: &ReadSnapshot,
    stop: &AtomicBool,
) -> ServiceResult<(AcquiredProjectPair, Option<BlizzardUiIndexCandidate>)> {
    publication_checkpoint(stop)?;
    if is_selected(read.epoch().catalog())? {
        if read.epoch().namespace().is_none() {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        let (pair, candidate) = read_candidate_pair(read, stop)?;
        Ok((pair, Some(candidate)))
    } else {
        // Ordinary generic reader continues to reject every additional record.
        Ok((
            AcquiredProjectPair::read(read, stop).map_err(project_error)?,
            None,
        ))
    }
}

pub(super) fn completed_checks(selected: bool) -> Vec<&'static str> {
    let mut checks = vec![
        GraphPartitionSnapshot::STORAGE_CHECK,
        publication::STORAGE_CHECK,
    ];
    if selected {
        checks.push(CANDIDATE_STORAGE_CHECK);
    }
    checks
}

/// Explicit selection is catalog intent, not a candidate success flag.
pub(super) fn capture_input(
    input: crate::LocalProjectInput,
    request: &crate::graph::GraphBuildRequest,
    selection: &super::PlatformStoreSelection,
    stop: &AtomicBool,
) -> ServiceResult<CandidatePublication> {
    let backend = crate::LocalProjectBackend::for_graph(input)?;
    let candidate = capture_candidate_from_backend(&backend, request, stop)?;
    selection.require_configuration(candidate.native().project().configuration())?;
    backend.capture_candidate_publication(candidate, selection.namespace(), stop)
}

impl LiveProjectStore {
    pub(super) fn publish_candidate_bundle(
        &mut self,
        bundle: CandidatePublication,
        operation_id: &str,
        expected_current: Option<CurrentRecordId>,
        stop: &AtomicBool,
    ) -> ServiceResult<PublicationOperation> {
        if !is_selected(self.store.epoch().catalog())? {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        let (records, bindings) = bundle.into_parts();
        if records
            .iter()
            .filter(|record| record.key() == CANDIDATE_RECORD_KEY)
            .count()
            != 1
        {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        self.publish_parts(records, bindings, operation_id, expected_current, stop)
    }
}
