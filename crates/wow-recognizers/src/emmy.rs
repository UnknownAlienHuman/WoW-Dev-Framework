use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_emmy::{EmmyFactFileStatus, EmmyMemberCallReport, EmmyReferenceResolution};
use wow_graph::{GraphConfidence, GraphNodeId, GraphSnapshot, GraphSnapshotId};

use crate::model::normalize_ids;
use crate::{
    EmmyDirectCallAdapterId, ObservationFamily, ObservationOrigin, RecognitionCoverage,
    RecognitionCoverageState, RecognizerError, RecognizerErrorCode, RecognizerLimits,
    RecognizerResult, StructuredObservation, StructuredObservationInput,
};

pub const EMMY_DIRECT_CALL_ADAPTER_SCHEMA: &str = "wow-recognizers/emmy-direct-call-adapter/e2-b/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyDirectCallBinding {
    call_fact_id: Box<str>,
    caller: GraphNodeId,
    callee: GraphNodeId,
    evidence_ids: Vec<Box<str>>,
}

impl EmmyDirectCallBinding {
    pub fn new(
        call_fact_id: impl Into<Box<str>>,
        caller: GraphNodeId,
        callee: GraphNodeId,
        evidence_ids: Vec<Box<str>>,
        limits: RecognizerLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        let call_fact_id = call_fact_id.into();
        if !digest_id(&call_fact_id, "emmy-call:sha256:") || caller == callee {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterBindingInvalid,
                "Emmy direct-call binding identity or endpoints are invalid",
            ));
        }
        let evidence_ids = normalize_ids(
            evidence_ids,
            limits.max_evidence_per_observation as usize,
            RecognizerErrorCode::AdapterBindingInvalid,
        )?;
        Ok(Self {
            call_fact_id,
            caller,
            callee,
            evidence_ids,
        })
    }

    pub fn validate(&self, limits: RecognizerLimits) -> RecognizerResult<()> {
        if Self::new(
            self.call_fact_id.clone(),
            self.caller.clone(),
            self.callee.clone(),
            self.evidence_ids.clone(),
            limits,
        )? != *self
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterBindingInvalid,
                "Emmy direct-call binding is not canonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn call_fact_id(&self) -> &str {
        &self.call_fact_id
    }

    #[must_use]
    pub fn caller(&self) -> &GraphNodeId {
        &self.caller
    }

    #[must_use]
    pub fn callee(&self) -> &GraphNodeId {
        &self.callee
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[Box<str>] {
        &self.evidence_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyDirectCallAdaptation {
    schema: Box<str>,
    adapter_id: EmmyDirectCallAdapterId,
    source_analysis_id: Box<str>,
    source_main_snapshot_id: Box<str>,
    graph_snapshot_id: GraphSnapshotId,
    observations: Vec<StructuredObservation>,
    coverage: RecognitionCoverage,
}

impl EmmyDirectCallAdaptation {
    fn build(
        source_analysis_id: Box<str>,
        source_main_snapshot_id: Box<str>,
        graph_snapshot_id: GraphSnapshotId,
        mut observations: Vec<StructuredObservation>,
        coverage: RecognitionCoverage,
    ) -> RecognizerResult<Self> {
        observations.sort_by(|left, right| left.observation_id().cmp(right.observation_id()));
        if observations
            .windows(2)
            .any(|pair| pair[0].observation_id() == pair[1].observation_id())
            || coverage.family() != ObservationFamily::DirectCall
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterIdentityMismatch,
                "Emmy adapter observations or coverage are invalid",
            ));
        }
        let adapter_id = adapter_id(
            &source_analysis_id,
            &source_main_snapshot_id,
            &graph_snapshot_id,
            &observations,
            &coverage,
        )?;
        Ok(Self {
            schema: EMMY_DIRECT_CALL_ADAPTER_SCHEMA.into(),
            adapter_id,
            source_analysis_id,
            source_main_snapshot_id,
            graph_snapshot_id,
            observations,
            coverage,
        })
    }

    pub fn validate(&self, limits: RecognizerLimits) -> RecognizerResult<()> {
        if self.schema.as_ref() != EMMY_DIRECT_CALL_ADAPTER_SCHEMA {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterIdentityMismatch,
                "Emmy direct-call adapter schema is unsupported",
            ));
        }
        for observation in &self.observations {
            observation.validate(limits)?;
            if observation.source_snapshot_id() != &self.graph_snapshot_id
                || observation.family() != ObservationFamily::DirectCall
                || observation.origin() != ObservationOrigin::AnalyzerFact
            {
                return Err(RecognizerError::new(
                    RecognizerErrorCode::AdapterIdentityMismatch,
                    "Emmy adapter observation is bound to the wrong snapshot or family",
                ));
            }
        }
        self.coverage.validate(limits)?;
        let rebuilt = Self::build(
            self.source_analysis_id.clone(),
            self.source_main_snapshot_id.clone(),
            self.graph_snapshot_id.clone(),
            self.observations.clone(),
            self.coverage.clone(),
        )?;
        if rebuilt != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterIdentityMismatch,
                "Emmy direct-call adapter identity or order does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn adapter_id(&self) -> &EmmyDirectCallAdapterId {
        &self.adapter_id
    }

    #[must_use]
    pub fn source_analysis_id(&self) -> &str {
        &self.source_analysis_id
    }

    #[must_use]
    pub fn source_main_snapshot_id(&self) -> &str {
        &self.source_main_snapshot_id
    }

    #[must_use]
    pub fn graph_snapshot_id(&self) -> &GraphSnapshotId {
        &self.graph_snapshot_id
    }

    #[must_use]
    pub fn observations(&self) -> &[StructuredObservation] {
        &self.observations
    }

    #[must_use]
    pub fn coverage(&self) -> &RecognitionCoverage {
        &self.coverage
    }
}

/// Projects exact `wow-emmy` direct-member call facts through explicit graph endpoint bindings.
///
/// Receiver/member spellings are not interpreted. Every call fact must have exactly one binding,
/// and both endpoints must already exist in the supplied graph snapshot. Unresolved Emmy members
/// remain `Possible`; they are never converted to platform absence.
pub fn adapt_emmy_direct_calls(
    report: &EmmyMemberCallReport,
    source: &GraphSnapshot,
    mut bindings: Vec<EmmyDirectCallBinding>,
    limits: RecognizerLimits,
    cancelled: &AtomicBool,
) -> RecognizerResult<EmmyDirectCallAdaptation> {
    check_cancelled(cancelled)?;
    source.validate().map_err(|error| {
        RecognizerError::new(RecognizerErrorCode::GraphProjectionFailed, error.message())
    })?;
    limits.validate()?;
    if bindings.len() > limits.max_observations as usize {
        return Err(RecognizerError::new(
            RecognizerErrorCode::BudgetExceeded,
            "Emmy adapter binding count exceeds recognizer limits",
        ));
    }

    let nodes = source
        .nodes()
        .iter()
        .map(|node| node.node_id().clone())
        .collect::<BTreeSet<_>>();
    let mut calls = BTreeMap::new();
    for call in report.calls() {
        if calls.insert(call.fact_id(), call).is_some() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterFactMismatch,
                "Emmy report contains duplicate call fact identities",
            ));
        }
    }
    let mut references = BTreeMap::new();
    for reference in report.references() {
        if references.insert(reference.fact_id(), reference).is_some() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterFactMismatch,
                "Emmy report contains duplicate reference fact identities",
            ));
        }
    }

    bindings.sort_by(|left, right| left.call_fact_id.cmp(&right.call_fact_id));
    for binding in &bindings {
        check_cancelled(cancelled)?;
        binding.validate(limits)?;
    }
    if bindings
        .windows(2)
        .any(|pair| pair[0].call_fact_id == pair[1].call_fact_id)
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::AdapterBindingDuplicate,
            "Emmy adapter contains duplicate call bindings",
        ));
    }
    for binding in &bindings {
        if !calls.contains_key(binding.call_fact_id()) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterBindingUnknown,
                "Emmy adapter binding references an unknown call fact",
            ));
        }
        if !nodes.contains(binding.caller()) || !nodes.contains(binding.callee()) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ObservationEndpointMissing,
                "Emmy adapter binding references a missing graph endpoint",
            ));
        }
    }
    if bindings.len() != calls.len() {
        return Err(RecognizerError::new(
            RecognizerErrorCode::AdapterBindingMissing,
            "every Emmy call fact requires exactly one explicit graph binding",
        ));
    }

    let binding_by_call = bindings
        .iter()
        .map(|binding| (binding.call_fact_id(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut observations = Vec::with_capacity(calls.len());
    for call in report.calls() {
        check_cancelled(cancelled)?;
        let binding = binding_by_call.get(call.fact_id()).ok_or_else(|| {
            RecognizerError::new(
                RecognizerErrorCode::AdapterBindingMissing,
                "Emmy call fact has no explicit graph binding",
            )
        })?;
        let reference = references.get(call.reference_fact_id()).ok_or_else(|| {
            RecognizerError::new(
                RecognizerErrorCode::AdapterFactMismatch,
                "Emmy call fact references a missing member-reference fact",
            )
        })?;
        if call.path() != reference.path()
            || call.content_sha256() != reference.content_sha256()
            || !digest_id(call.fact_id(), "emmy-call:sha256:")
            || !digest_id(reference.fact_id(), "emmy-reference:sha256:")
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterFactMismatch,
                "Emmy call and reference fact identities do not agree",
            ));
        }
        let mut evidence_ids = binding.evidence_ids().to_vec();
        evidence_ids.extend([
            report.analysis_id().into(),
            report.main_snapshot_id().into(),
            call.fact_id().into(),
            reference.fact_id().into(),
        ]);
        let confidence = match reference.resolution() {
            EmmyReferenceResolution::Resolved => GraphConfidence::Derived,
            EmmyReferenceResolution::Unresolved | EmmyReferenceResolution::Possible => {
                GraphConfidence::Possible
            }
        };
        observations.push(StructuredObservation::new(
            StructuredObservationInput {
                source_snapshot_id: source.snapshot_id().clone(),
                family: ObservationFamily::DirectCall,
                from: binding.caller().clone(),
                to: binding.callee().clone(),
                origin: ObservationOrigin::AnalyzerFact,
                confidence,
                evidence_ids,
            },
            limits,
        )?);
    }

    let blockers = report
        .files()
        .iter()
        .filter(|file| file.status() == EmmyFactFileStatus::FailedParse)
        .map(|file| failed_file_blocker(file.path()))
        .collect::<Vec<_>>();
    let state = if blockers.is_empty() {
        RecognitionCoverageState::Complete
    } else {
        RecognitionCoverageState::Partial
    };
    let coverage =
        RecognitionCoverage::new(ObservationFamily::DirectCall, state, blockers, limits)?;
    check_cancelled(cancelled)?;
    EmmyDirectCallAdaptation::build(
        report.analysis_id().into(),
        report.main_snapshot_id().into(),
        source.snapshot_id().clone(),
        observations,
        coverage,
    )
}

fn adapter_id(
    source_analysis_id: &str,
    source_main_snapshot_id: &str,
    graph_snapshot_id: &GraphSnapshotId,
    observations: &[StructuredObservation],
    coverage: &RecognitionCoverage,
) -> RecognizerResult<EmmyDirectCallAdapterId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        source_analysis_id: &'a str,
        source_main_snapshot_id: &'a str,
        graph_snapshot_id: &'a GraphSnapshotId,
        observations: &'a [StructuredObservation],
        coverage: &'a RecognitionCoverage,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: EMMY_DIRECT_CALL_ADAPTER_SCHEMA,
        source_analysis_id,
        source_main_snapshot_id,
        graph_snapshot_id,
        observations,
        coverage,
    })
    .map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::AdapterIdentityMismatch,
            "Emmy direct-call adapter identity cannot be canonicalized",
        )
    })?;
    EmmyDirectCallAdapterId::new(format!(
        "recognizer-emmy-adapter:sha256:{}",
        hex(&Sha256::digest(bytes))
    ))
}

fn failed_file_blocker(path: &str) -> Box<str> {
    format!(
        "emmy-file-parse-failed:sha256:{}",
        hex(&Sha256::digest(path.as_bytes()))
    )
    .into_boxed_str()
}

fn digest_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn check_cancelled(cancelled: &AtomicBool) -> RecognizerResult<()> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(RecognizerError::new(
            RecognizerErrorCode::Cancelled,
            "Emmy adapter operation was cancelled",
        ));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
