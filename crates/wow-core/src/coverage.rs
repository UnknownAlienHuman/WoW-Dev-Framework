use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::budget::TruncationState;
use crate::digest::{ConflictId, CoverageId, EvidenceId, GenerationContextId, NotEvaluatedId};
use crate::error::{CoreErrorCode, CoreResult, validation_error};
use crate::evidence::ConflictRecord;
use crate::ids::{
    CapabilityId, CoveragePartitionId, MessageCode, ProducerId, ToolVersion, validate_lower_segment,
};
use crate::integrity;

/// Exact coverage state for one capability partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Complete,
    Partial,
    Unknown,
    Failed,
    NotApplicable,
}

impl CoverageStatus {
    const fn precedence(self) -> u8 {
        match self {
            Self::Complete | Self::NotApplicable => 0,
            Self::Partial => 1,
            Self::Unknown => 2,
            Self::Failed => 3,
        }
    }
}

/// Typed truncation reason attached to coverage and summaries.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageTruncationRef {
    collection_id: String,
    reason_code: MessageCode,
}

impl CoverageTruncationRef {
    pub fn new(collection_id: impl Into<String>, reason_code: MessageCode) -> CoreResult<Self> {
        let collection_id = collection_id.into();
        validate_lower_segment(
            &collection_id,
            "validate_coverage_record",
            "truncation_refs.collection_id",
        )?;
        Ok(Self {
            collection_id,
            reason_code,
        })
    }

    #[must_use]
    pub fn collection_id(&self) -> &str {
        &self.collection_id
    }

    #[must_use]
    pub const fn reason_code(&self) -> &MessageCode {
        &self.reason_code
    }
}

/// Producer-owned coverage truth for one capability partition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRecord {
    coverage_id: CoverageId,
    context_id: GenerationContextId,
    capability_id: CapabilityId,
    partition_id: CoveragePartitionId,
    status: CoverageStatus,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    missing_input_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_code: Option<MessageCode>,
    conflict_ids: Vec<ConflictId>,
    truncation_refs: Vec<CoverageTruncationRef>,
}

impl CoverageRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context_id: GenerationContextId,
        capability_id: CapabilityId,
        partition_id: CoveragePartitionId,
        status: CoverageStatus,
        producer_id: ProducerId,
        producer_version: ToolVersion,
        mut missing_input_ids: Vec<String>,
        failure_code: Option<MessageCode>,
        mut conflict_ids: Vec<ConflictId>,
        mut truncation_refs: Vec<CoverageTruncationRef>,
    ) -> CoreResult<Self> {
        sort_unique_strings(&mut missing_input_ids, "missing_input_ids")?;
        sort_unique(
            &mut conflict_ids,
            "validate_coverage_record",
            "conflict_ids",
            CoreErrorCode::DuplicateConflictReference,
        )?;
        sort_unique(
            &mut truncation_refs,
            "validate_coverage_record",
            "truncation_refs",
            CoreErrorCode::CoverageConflict,
        )?;
        validate_coverage_state(
            status,
            &missing_input_ids,
            failure_code.as_ref(),
            &conflict_ids,
            &truncation_refs,
        )?;

        let coverage_id = CoverageId::derive(&CoverageIdentity {
            context_id,
            capability_id: &capability_id,
            partition_id: &partition_id,
            status,
            producer_id: &producer_id,
            producer_version: &producer_version,
            missing_input_ids: &missing_input_ids,
            failure_code: failure_code.as_ref(),
            conflict_ids: &conflict_ids,
            truncation_refs: &truncation_refs,
        })?;
        let record = Self {
            coverage_id,
            context_id,
            capability_id,
            partition_id,
            status,
            producer_id,
            producer_version,
            missing_input_ids,
            failure_code,
            conflict_ids,
            truncation_refs,
        };
        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> CoreResult<()> {
        validate_coverage_state(
            self.status,
            &self.missing_input_ids,
            self.failure_code.as_ref(),
            &self.conflict_ids,
            &self.truncation_refs,
        )?;
        ensure_sorted_unique(
            &self.missing_input_ids,
            "validate_coverage_record",
            "missing_input_ids",
            CoreErrorCode::DuplicateCoverageRecord,
        )?;
        ensure_sorted_unique(
            &self.conflict_ids,
            "validate_coverage_record",
            "conflict_ids",
            CoreErrorCode::DuplicateConflictReference,
        )?;
        ensure_sorted_unique(
            &self.truncation_refs,
            "validate_coverage_record",
            "truncation_refs",
            CoreErrorCode::CoverageConflict,
        )?;
        integrity::validate_coverage(self)
    }

    #[must_use]
    pub const fn coverage_id(&self) -> CoverageId {
        self.coverage_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    #[must_use]
    pub const fn partition_id(&self) -> &CoveragePartitionId {
        &self.partition_id
    }

    #[must_use]
    pub const fn status(&self) -> CoverageStatus {
        self.status
    }

    #[must_use]
    pub const fn producer_id(&self) -> &ProducerId {
        &self.producer_id
    }

    #[must_use]
    pub const fn producer_version(&self) -> &ToolVersion {
        &self.producer_version
    }

    #[must_use]
    pub fn missing_input_ids(&self) -> &[String] {
        &self.missing_input_ids
    }

    #[must_use]
    pub const fn failure_code(&self) -> Option<&MessageCode> {
        self.failure_code.as_ref()
    }

    #[must_use]
    pub fn conflict_ids(&self) -> &[ConflictId] {
        &self.conflict_ids
    }

    #[must_use]
    pub fn truncation_refs(&self) -> &[CoverageTruncationRef] {
        &self.truncation_refs
    }
}

#[derive(Serialize)]
struct CoverageIdentity<'a> {
    context_id: GenerationContextId,
    capability_id: &'a CapabilityId,
    partition_id: &'a CoveragePartitionId,
    status: CoverageStatus,
    producer_id: &'a ProducerId,
    producer_version: &'a ToolVersion,
    missing_input_ids: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_code: Option<&'a MessageCode>,
    conflict_ids: &'a [ConflictId],
    truncation_refs: &'a [CoverageTruncationRef],
}

/// Exact partition retained by a capability summary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoveragePartitionRef {
    coverage_id: CoverageId,
    partition_id: CoveragePartitionId,
    status: CoverageStatus,
}

impl CoveragePartitionRef {
    #[must_use]
    pub const fn coverage_id(&self) -> CoverageId {
        self.coverage_id
    }

    #[must_use]
    pub const fn partition_id(&self) -> &CoveragePartitionId {
        &self.partition_id
    }

    #[must_use]
    pub const fn status(&self) -> CoverageStatus {
        self.status
    }
}

/// Conservative capability state derived from exact coverage records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilitySummary {
    context_id: GenerationContextId,
    capability_id: CapabilityId,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    status: CoverageStatus,
    partition_refs: Vec<CoveragePartitionRef>,
    conflict_ids: Vec<ConflictId>,
    truncation_refs: Vec<CoverageTruncationRef>,
}

impl CapabilitySummary {
    pub fn from_records(
        context_id: GenerationContextId,
        capability_id: CapabilityId,
        producer_id: ProducerId,
        producer_version: ToolVersion,
        records: &[CoverageRecord],
    ) -> CoreResult<Self> {
        if records.is_empty() {
            return Err(validation_error(
                "combine_coverage",
                CoreErrorCode::CoverageRecordMissing,
                "records",
            ));
        }

        let mut partition_refs = Vec::with_capacity(records.len());
        let mut conflict_ids = Vec::new();
        let mut truncation_refs = Vec::new();
        let mut applicable_status: Option<CoverageStatus> = None;
        let mut applicable_count = 0_usize;

        for record in records {
            record.validate()?;
            if record.context_id != context_id {
                return Err(validation_error(
                    "combine_coverage",
                    CoreErrorCode::CoverageContextMismatch,
                    "records.context_id",
                ));
            }
            if record.capability_id != capability_id {
                return Err(validation_error(
                    "combine_coverage",
                    CoreErrorCode::CoverageConflict,
                    "records.capability_id",
                ));
            }
            partition_refs.push(CoveragePartitionRef {
                coverage_id: record.coverage_id,
                partition_id: record.partition_id.clone(),
                status: record.status,
            });
            conflict_ids.extend(record.conflict_ids.iter().copied());
            truncation_refs.extend(record.truncation_refs.iter().cloned());
            if record.status != CoverageStatus::NotApplicable {
                applicable_count += 1;
                applicable_status = Some(match applicable_status {
                    Some(current) if current.precedence() >= record.status.precedence() => current,
                    _ => record.status,
                });
            }
        }

        partition_refs.sort();
        if partition_refs
            .windows(2)
            .any(|pair| pair[0].partition_id == pair[1].partition_id)
        {
            return Err(validation_error(
                "combine_coverage",
                CoreErrorCode::DuplicateCoverageRecord,
                "records.partition_id",
            ));
        }
        conflict_ids.sort_unstable();
        conflict_ids.dedup();
        truncation_refs.sort();
        truncation_refs.dedup();

        let status = if applicable_count == 0 {
            CoverageStatus::NotApplicable
        } else {
            applicable_status.ok_or_else(|| {
                validation_error(
                    "combine_coverage",
                    CoreErrorCode::CoverageRecordMissing,
                    "records",
                )
            })?
        };
        Ok(Self {
            context_id,
            capability_id,
            producer_id,
            producer_version,
            status,
            partition_refs,
            conflict_ids,
            truncation_refs,
        })
    }

    pub fn validate(&self, records: &[CoverageRecord]) -> CoreResult<()> {
        let expected = Self::from_records(
            self.context_id,
            self.capability_id.clone(),
            self.producer_id.clone(),
            self.producer_version.clone(),
            records,
        )?;
        if &expected == self {
            Ok(())
        } else {
            Err(validation_error(
                "validate_capability_summary",
                CoreErrorCode::CoverageConflict,
                "capability_summary",
            ))
        }
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    #[must_use]
    pub const fn producer_id(&self) -> &ProducerId {
        &self.producer_id
    }

    #[must_use]
    pub const fn producer_version(&self) -> &ToolVersion {
        &self.producer_version
    }

    #[must_use]
    pub const fn status(&self) -> CoverageStatus {
        self.status
    }

    #[must_use]
    pub fn partition_refs(&self) -> &[CoveragePartitionRef] {
        &self.partition_refs
    }

    #[must_use]
    pub fn conflict_ids(&self) -> &[ConflictId] {
        &self.conflict_ids
    }

    #[must_use]
    pub fn truncation_refs(&self) -> &[CoverageTruncationRef] {
        &self.truncation_refs
    }
}

/// Exact partition blocking subject evaluation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockingPartitionRef {
    capability_id: CapabilityId,
    coverage_id: CoverageId,
    partition_id: CoveragePartitionId,
    status: CoverageStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    conflict_ids: Vec<ConflictId>,
}

impl BlockingPartitionRef {
    #[must_use]
    pub fn from_record(record: &CoverageRecord) -> Self {
        Self {
            capability_id: record.capability_id.clone(),
            coverage_id: record.coverage_id,
            partition_id: record.partition_id.clone(),
            status: record.status,
            conflict_ids: record.conflict_ids.clone(),
        }
    }

    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    #[must_use]
    pub const fn coverage_id(&self) -> CoverageId {
        self.coverage_id
    }

    #[must_use]
    pub const fn partition_id(&self) -> &CoveragePartitionId {
        &self.partition_id
    }

    #[must_use]
    pub const fn status(&self) -> CoverageStatus {
        self.status
    }

    #[must_use]
    pub fn conflict_ids(&self) -> &[ConflictId] {
        &self.conflict_ids
    }
}

/// Canonical explanation that a requested subject could not be evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotEvaluatedRecord {
    not_evaluated_id: NotEvaluatedId,
    context_id: GenerationContextId,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    subject_kind: String,
    subject_id: String,
    reason_code: MessageCode,
    blocking_capability_ids: Vec<CapabilityId>,
    blocking_partitions: Vec<BlockingPartitionRef>,
    conflict_ids: Vec<ConflictId>,
}

impl NotEvaluatedRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context_id: GenerationContextId,
        producer_id: ProducerId,
        producer_version: ToolVersion,
        subject_kind: impl Into<String>,
        subject_id: impl Into<String>,
        reason_code: MessageCode,
        mut blocking_capability_ids: Vec<CapabilityId>,
        mut blocking_partitions: Vec<BlockingPartitionRef>,
        mut conflict_ids: Vec<ConflictId>,
    ) -> CoreResult<Self> {
        let subject_kind = subject_kind.into();
        let subject_id = subject_id.into();
        validate_subject(&subject_kind, &subject_id)?;
        if blocking_capability_ids.is_empty() {
            return Err(validation_error(
                "validate_not_evaluated_record",
                CoreErrorCode::CoverageRecordMissing,
                "blocking_capability_ids",
            ));
        }
        sort_unique(
            &mut blocking_capability_ids,
            "validate_not_evaluated_record",
            "blocking_capability_ids",
            CoreErrorCode::DuplicateCoverageRecord,
        )?;
        sort_unique(
            &mut blocking_partitions,
            "validate_not_evaluated_record",
            "blocking_partitions",
            CoreErrorCode::DuplicateCoverageRecord,
        )?;
        sort_unique(
            &mut conflict_ids,
            "validate_not_evaluated_record",
            "conflict_ids",
            CoreErrorCode::DuplicateConflictReference,
        )?;

        let not_evaluated_id = NotEvaluatedId::derive(&NotEvaluatedIdentity {
            context_id,
            producer_id: &producer_id,
            producer_version: &producer_version,
            subject_kind: &subject_kind,
            subject_id: &subject_id,
            reason_code: &reason_code,
            blocking_capability_ids: &blocking_capability_ids,
            blocking_partitions: &blocking_partitions,
            conflict_ids: &conflict_ids,
        })?;
        let record = Self {
            not_evaluated_id,
            context_id,
            producer_id,
            producer_version,
            subject_kind,
            subject_id,
            reason_code,
            blocking_capability_ids,
            blocking_partitions,
            conflict_ids,
        };
        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> CoreResult<()> {
        validate_subject(&self.subject_kind, &self.subject_id)?;
        ensure_sorted_unique(
            &self.blocking_capability_ids,
            "validate_not_evaluated_record",
            "blocking_capability_ids",
            CoreErrorCode::DuplicateCoverageRecord,
        )?;
        ensure_sorted_unique(
            &self.blocking_partitions,
            "validate_not_evaluated_record",
            "blocking_partitions",
            CoreErrorCode::DuplicateCoverageRecord,
        )?;
        ensure_sorted_unique(
            &self.conflict_ids,
            "validate_not_evaluated_record",
            "conflict_ids",
            CoreErrorCode::DuplicateConflictReference,
        )?;
        integrity::validate_not_evaluated(self)
    }

    #[must_use]
    pub const fn not_evaluated_id(&self) -> NotEvaluatedId {
        self.not_evaluated_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub fn blocking_capability_ids(&self) -> &[CapabilityId] {
        &self.blocking_capability_ids
    }

    #[must_use]
    pub fn blocking_partitions(&self) -> &[BlockingPartitionRef] {
        &self.blocking_partitions
    }

    #[must_use]
    pub fn conflict_ids(&self) -> &[ConflictId] {
        &self.conflict_ids
    }
}

#[derive(Serialize)]
struct NotEvaluatedIdentity<'a> {
    context_id: GenerationContextId,
    producer_id: &'a ProducerId,
    producer_version: &'a ToolVersion,
    subject_kind: &'a str,
    subject_id: &'a str,
    reason_code: &'a MessageCode,
    blocking_capability_ids: &'a [CapabilityId],
    blocking_partitions: &'a [BlockingPartitionRef],
    conflict_ids: &'a [ConflictId],
}

/// Result of evaluating whether all required capabilities are available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityAvailability {
    Runnable,
    NotEvaluated(Box<NotEvaluatedRecord>),
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_capability_availability(
    context_id: GenerationContextId,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    subject_kind: impl Into<String>,
    subject_id: impl Into<String>,
    reason_code: MessageCode,
    summaries: &[CapabilitySummary],
    coverage_records: &[CoverageRecord],
) -> CoreResult<CapabilityAvailability> {
    let mut blocking_capability_ids = Vec::new();
    let mut blocking_partitions = Vec::new();
    let mut conflict_ids = Vec::new();

    for summary in summaries {
        if summary.context_id != context_id {
            return Err(validation_error(
                "evaluate_capability_availability",
                CoreErrorCode::CoverageContextMismatch,
                "summaries.context_id",
            ));
        }
        let blocked = summary.status != CoverageStatus::Complete
            || !summary.conflict_ids.is_empty()
            || !summary.truncation_refs.is_empty();
        if !blocked {
            continue;
        }
        blocking_capability_ids.push(summary.capability_id.clone());
        conflict_ids.extend(summary.conflict_ids.iter().copied());
        for partition in &summary.partition_refs {
            let record = coverage_records
                .iter()
                .find(|record| record.coverage_id == partition.coverage_id)
                .ok_or_else(|| {
                    validation_error(
                        "evaluate_capability_availability",
                        CoreErrorCode::CoverageRecordMissing,
                        "coverage_records",
                    )
                })?;
            if record.status != CoverageStatus::Complete
                || !record.conflict_ids.is_empty()
                || !record.truncation_refs.is_empty()
            {
                blocking_partitions.push(BlockingPartitionRef::from_record(record));
            }
        }
    }
    if blocking_capability_ids.is_empty() {
        return Ok(CapabilityAvailability::Runnable);
    }
    Ok(CapabilityAvailability::NotEvaluated(Box::new(
        NotEvaluatedRecord::new(
            context_id,
            producer_id,
            producer_version,
            subject_kind,
            subject_id,
            reason_code,
            blocking_capability_ids,
            blocking_partitions,
            conflict_ids,
        )?,
    )))
}

/// Outcome of a conservative negative-authority evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegativeAuthorityOutcome {
    AuthoritativeAbsent,
    NotAuthoritative,
    NotApplicable,
}

/// Exact reason denying negative authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegativeAuthorityReason {
    ProfileUnavailable,
    ReferenceGenerationUnavailable,
    GenerationMismatch,
    PartitionPartial,
    PartitionUnknown,
    PartitionFailed,
    UnresolvedConflict,
    CapabilityNotEvaluated,
    CandidateOnlyEvidence,
    ScopeUnknown,
    ResultTruncated,
}

/// Typed negative-authority decision retaining every bounded blocker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeAuthorityDecision {
    outcome: NegativeAuthorityOutcome,
    reasons: Vec<NegativeAuthorityReason>,
    capability_ids: Vec<CapabilityId>,
    coverage_ids: Vec<CoverageId>,
    conflict_ids: Vec<ConflictId>,
    candidate_evidence_ids: Vec<EvidenceId>,
}

impl NegativeAuthorityDecision {
    #[must_use]
    pub const fn outcome(&self) -> NegativeAuthorityOutcome {
        self.outcome
    }

    #[must_use]
    pub fn reasons(&self) -> &[NegativeAuthorityReason] {
        &self.reasons
    }
}

pub fn evaluate_negative_authority(
    scope_known: bool,
    lookup_completed: bool,
    summaries: &[CapabilitySummary],
    conflicts: &[ConflictRecord],
    candidate_evidence_ids: Vec<EvidenceId>,
    evaluation: Option<&NotEvaluatedRecord>,
    truncation: &TruncationState,
) -> NegativeAuthorityDecision {
    let mut reasons = BTreeSet::new();
    let mut capability_ids = BTreeSet::new();
    let mut coverage_ids = BTreeSet::new();
    let mut conflict_ids = BTreeSet::new();

    if !scope_known || !lookup_completed {
        reasons.insert(NegativeAuthorityReason::ScopeUnknown);
    }
    for summary in summaries {
        capability_ids.insert(summary.capability_id.clone());
        coverage_ids.extend(summary.partition_refs.iter().map(|item| item.coverage_id));
        conflict_ids.extend(summary.conflict_ids.iter().copied());
        if !summary.truncation_refs.is_empty() {
            reasons.insert(NegativeAuthorityReason::ResultTruncated);
        }
        match summary.status {
            CoverageStatus::Complete => {}
            CoverageStatus::Partial => {
                reasons.insert(NegativeAuthorityReason::PartitionPartial);
            }
            CoverageStatus::Unknown | CoverageStatus::NotApplicable => {
                reasons.insert(NegativeAuthorityReason::PartitionUnknown);
            }
            CoverageStatus::Failed => {
                reasons.insert(NegativeAuthorityReason::PartitionFailed);
            }
        }
    }
    if summaries
        .iter()
        .all(|summary| summary.status == CoverageStatus::NotApplicable)
        && !summaries.is_empty()
        && reasons.is_empty()
    {
        return NegativeAuthorityDecision {
            outcome: NegativeAuthorityOutcome::NotApplicable,
            reasons: Vec::new(),
            capability_ids: capability_ids.into_iter().collect(),
            coverage_ids: coverage_ids.into_iter().collect(),
            conflict_ids: conflict_ids.into_iter().collect(),
            candidate_evidence_ids: Vec::new(),
        };
    }
    if !conflicts.is_empty() || !conflict_ids.is_empty() {
        reasons.insert(NegativeAuthorityReason::UnresolvedConflict);
        conflict_ids.extend(conflicts.iter().map(ConflictRecord::conflict_id));
    }
    if !candidate_evidence_ids.is_empty() {
        reasons.insert(NegativeAuthorityReason::CandidateOnlyEvidence);
    }
    if evaluation.is_some() {
        reasons.insert(NegativeAuthorityReason::CapabilityNotEvaluated);
    }
    if truncation.is_truncated() {
        reasons.insert(NegativeAuthorityReason::ResultTruncated);
    }

    let mut candidate_evidence_ids = candidate_evidence_ids;
    candidate_evidence_ids.sort_unstable();
    candidate_evidence_ids.dedup();
    let outcome = if reasons.is_empty() {
        NegativeAuthorityOutcome::AuthoritativeAbsent
    } else {
        NegativeAuthorityOutcome::NotAuthoritative
    };
    NegativeAuthorityDecision {
        outcome,
        reasons: reasons.into_iter().collect(),
        capability_ids: capability_ids.into_iter().collect(),
        coverage_ids: coverage_ids.into_iter().collect(),
        conflict_ids: conflict_ids.into_iter().collect(),
        candidate_evidence_ids,
    }
}

pub(crate) fn conflict_affects_coverage(
    conflict: &ConflictRecord,
    record: &CoverageRecord,
) -> bool {
    conflict.affected_refs().iter().any(|affected| {
        affected.capability_id() == record.capability_id()
            && affected
                .partition_id()
                .is_none_or(|partition| partition == record.partition_id())
    })
}

fn validate_coverage_state(
    status: CoverageStatus,
    missing_input_ids: &[String],
    failure_code: Option<&MessageCode>,
    conflict_ids: &[ConflictId],
    truncation_refs: &[CoverageTruncationRef],
) -> CoreResult<()> {
    let valid = match status {
        CoverageStatus::Complete => {
            missing_input_ids.is_empty() && failure_code.is_none() && truncation_refs.is_empty()
        }
        CoverageStatus::Partial => {
            failure_code.is_none() && (!missing_input_ids.is_empty() || !truncation_refs.is_empty())
        }
        CoverageStatus::Unknown => !missing_input_ids.is_empty() && failure_code.is_none(),
        CoverageStatus::Failed => failure_code.is_some(),
        CoverageStatus::NotApplicable => {
            missing_input_ids.is_empty()
                && failure_code.is_none()
                && conflict_ids.is_empty()
                && truncation_refs.is_empty()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(validation_error(
            "validate_coverage_record",
            CoreErrorCode::CoverageConflict,
            "status",
        ))
    }
}

fn validate_subject(subject_kind: &str, subject_id: &str) -> CoreResult<()> {
    validate_lower_segment(
        subject_kind,
        "validate_not_evaluated_record",
        "subject_kind",
    )?;
    if subject_id.is_empty()
        || subject_id.len() > 512
        || subject_id.chars().any(char::is_control)
        || subject_id.trim() != subject_id
    {
        return Err(validation_error(
            "validate_not_evaluated_record",
            CoreErrorCode::InvalidIdentifier,
            "subject_id",
        ));
    }
    Ok(())
}

fn sort_unique_strings(values: &mut [String], field: &'static str) -> CoreResult<()> {
    values.sort_unstable();
    if values.iter().any(|value| {
        value.is_empty()
            || value.len() > 512
            || value.chars().any(char::is_control)
            || value.trim() != value
    }) {
        return Err(validation_error(
            "validate_coverage_record",
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }
    ensure_sorted_unique(
        values,
        "validate_coverage_record",
        field,
        CoreErrorCode::DuplicateCoverageRecord,
    )
}

fn sort_unique<T: Ord>(
    values: &mut [T],
    operation: &'static str,
    field: &'static str,
    code: CoreErrorCode,
) -> CoreResult<()> {
    values.sort_unstable();
    ensure_sorted_unique(values, operation, field, code)
}

fn ensure_sorted_unique<T: Ord>(
    values: &[T],
    operation: &'static str,
    field: &'static str,
    code: CoreErrorCode,
) -> CoreResult<()> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(validation_error(operation, code, field))
    } else {
        Ok(())
    }
}

/// Operation wrapper for validating one coverage record.
pub fn validate_coverage_record(record: &CoverageRecord) -> CoreResult<()> {
    record.validate()
}

/// Recomputes the identity of one coverage record.
pub fn derive_coverage_id(record: &CoverageRecord) -> CoreResult<CoverageId> {
    CoverageId::derive(&CoverageIdentity {
        context_id: record.context_id,
        capability_id: &record.capability_id,
        partition_id: &record.partition_id,
        status: record.status,
        producer_id: &record.producer_id,
        producer_version: &record.producer_version,
        missing_input_ids: &record.missing_input_ids,
        failure_code: record.failure_code.as_ref(),
        conflict_ids: &record.conflict_ids,
        truncation_refs: &record.truncation_refs,
    })
}

/// Combines exact coverage records into one conservative capability summary.
pub fn combine_coverage(
    context_id: GenerationContextId,
    capability_id: CapabilityId,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    records: &[CoverageRecord],
) -> CoreResult<CapabilitySummary> {
    CapabilitySummary::from_records(
        context_id,
        capability_id,
        producer_id,
        producer_version,
        records,
    )
}

/// Recomputes and validates one capability summary against its exact records.
pub fn validate_capability_summary(
    summary: &CapabilitySummary,
    records: &[CoverageRecord],
) -> CoreResult<()> {
    summary.validate(records)
}

/// Recomputes the identity of one not-evaluated record.
pub fn derive_not_evaluated_id(record: &NotEvaluatedRecord) -> CoreResult<NotEvaluatedId> {
    NotEvaluatedId::derive(&NotEvaluatedIdentity {
        context_id: record.context_id,
        producer_id: &record.producer_id,
        producer_version: &record.producer_version,
        subject_kind: &record.subject_kind,
        subject_id: &record.subject_id,
        reason_code: &record.reason_code,
        blocking_capability_ids: &record.blocking_capability_ids,
        blocking_partitions: &record.blocking_partitions,
        conflict_ids: &record.conflict_ids,
    })
}

/// Operation wrapper for validating one not-evaluated record.
pub fn validate_not_evaluated_record(record: &NotEvaluatedRecord) -> CoreResult<()> {
    record.validate()
}
