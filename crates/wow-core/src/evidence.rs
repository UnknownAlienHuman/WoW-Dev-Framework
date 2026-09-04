use serde::{Deserialize, Serialize};

use crate::digest::{ConflictId, EvidenceId, GenerationContextId, StableHandleId};
use crate::error::{CoreErrorCode, CoreResult, validation_error};
use crate::ids::{
    CapabilityId, CoveragePartitionId, EntityKey, MessageCode, ProducerId, ToolVersion,
};
use crate::integrity;

/// Provenance class for one immutable evidence statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceClass {
    PlatformSource,
    ProjectSource,
    RuntimeProbe,
    CuratedCorrection,
    DifferentialOracle,
    ExternalImplementation,
    SemanticCandidate,
    HistoricalRecord,
    ModelInference,
}

/// Confidence carried by one evidence statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceConfidence {
    Proven,
    Derived,
    Possible,
    Candidate,
}

/// Contract name retained for callers that use the data-model terminology.
pub type EvidenceLevel = EvidenceConfidence;

/// Semantic scope of the claim supported by evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimScope {
    SourceObservation,
    PlatformContract,
    ProjectFact,
    RuntimeScenario,
    HistoricalRelation,
    CandidateRelation,
}

/// Semantic coverage reference used by evidence without creating an ID cycle.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceCoverageRef {
    capability_id: CapabilityId,
    partition_id: CoveragePartitionId,
    producer_id: ProducerId,
}

/// Contract name retained for callers that use the data-model terminology.
pub type CoverageReference = EvidenceCoverageRef;

impl EvidenceCoverageRef {
    #[must_use]
    pub const fn new(
        capability_id: CapabilityId,
        partition_id: CoveragePartitionId,
        producer_id: ProducerId,
    ) -> Self {
        Self {
            capability_id,
            partition_id,
            producer_id,
        }
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
    pub const fn producer_id(&self) -> &ProducerId {
        &self.producer_id
    }
}

/// Immutable evidence record bound to one generation context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    evidence_id: EvidenceId,
    context_id: GenerationContextId,
    provenance: ProvenanceClass,
    confidence: EvidenceConfidence,
    claim_scope: ClaimScope,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    source_handle_ids: Vec<StableHandleId>,
    coverage_refs: Vec<EvidenceCoverageRef>,
    derivation_input_ids: Vec<EvidenceId>,
}

impl EvidenceRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context_id: GenerationContextId,
        provenance: ProvenanceClass,
        confidence: EvidenceConfidence,
        claim_scope: ClaimScope,
        producer_id: ProducerId,
        producer_version: ToolVersion,
        mut source_handle_ids: Vec<StableHandleId>,
        mut coverage_refs: Vec<EvidenceCoverageRef>,
        mut derivation_input_ids: Vec<EvidenceId>,
    ) -> CoreResult<Self> {
        sort_unique(
            &mut source_handle_ids,
            "validate_evidence_record",
            "source_handle_ids",
            CoreErrorCode::DuplicateEvidenceReference,
        )?;
        sort_unique(
            &mut coverage_refs,
            "validate_evidence_record",
            "coverage_refs",
            CoreErrorCode::DuplicateCoverageRecord,
        )?;
        sort_unique(
            &mut derivation_input_ids,
            "validate_evidence_record",
            "derivation_input_ids",
            CoreErrorCode::DuplicateEvidenceReference,
        )?;

        let evidence_id = EvidenceId::derive(&EvidenceIdentity {
            context_id,
            provenance,
            confidence,
            claim_scope,
            producer_id: &producer_id,
            producer_version: &producer_version,
            source_handle_ids: &source_handle_ids,
            coverage_refs: &coverage_refs,
            derivation_input_ids: &derivation_input_ids,
        })?;
        let record = Self {
            evidence_id,
            context_id,
            provenance,
            confidence,
            claim_scope,
            producer_id,
            producer_version,
            source_handle_ids,
            coverage_refs,
            derivation_input_ids,
        };
        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> CoreResult<()> {
        validate_evidence_authority(self)?;
        integrity::validate_evidence(self)
    }

    #[must_use]
    pub const fn evidence_id(&self) -> EvidenceId {
        self.evidence_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub const fn provenance(&self) -> ProvenanceClass {
        self.provenance
    }

    #[must_use]
    pub const fn confidence(&self) -> EvidenceConfidence {
        self.confidence
    }

    #[must_use]
    pub const fn claim_scope(&self) -> ClaimScope {
        self.claim_scope
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
    pub fn source_handle_ids(&self) -> &[StableHandleId] {
        &self.source_handle_ids
    }

    #[must_use]
    pub fn coverage_refs(&self) -> &[EvidenceCoverageRef] {
        &self.coverage_refs
    }

    #[must_use]
    pub fn derivation_input_ids(&self) -> &[EvidenceId] {
        &self.derivation_input_ids
    }
}

#[derive(Serialize)]
struct EvidenceIdentity<'a> {
    context_id: GenerationContextId,
    provenance: ProvenanceClass,
    confidence: EvidenceConfidence,
    claim_scope: ClaimScope,
    producer_id: &'a ProducerId,
    producer_version: &'a ToolVersion,
    source_handle_ids: &'a [StableHandleId],
    coverage_refs: &'a [EvidenceCoverageRef],
    derivation_input_ids: &'a [EvidenceId],
}

/// One exact capability/partition scope affected by an unresolved conflict.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictAffectedRef {
    capability_id: CapabilityId,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition_id: Option<CoveragePartitionId>,
}

impl ConflictAffectedRef {
    #[must_use]
    pub const fn new(
        capability_id: CapabilityId,
        partition_id: Option<CoveragePartitionId>,
    ) -> Self {
        Self {
            capability_id,
            partition_id,
        }
    }

    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    #[must_use]
    pub const fn partition_id(&self) -> Option<&CoveragePartitionId> {
        self.partition_id.as_ref()
    }
}

/// Immutable unresolved conflict over two or more evidence records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictRecord {
    conflict_id: ConflictId,
    context_id: GenerationContextId,
    conflict_code: MessageCode,
    evidence_ids: Vec<EvidenceId>,
    affected_refs: Vec<ConflictAffectedRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_entity_key: Option<EntityKey>,
}

impl ConflictRecord {
    pub fn new(
        context_id: GenerationContextId,
        conflict_code: MessageCode,
        mut evidence_ids: Vec<EvidenceId>,
        mut affected_refs: Vec<ConflictAffectedRef>,
        subject_entity_key: Option<EntityKey>,
    ) -> CoreResult<Self> {
        if evidence_ids.len() < 2 {
            return Err(validation_error(
                "relate_evidence_conflict",
                CoreErrorCode::ConflictScopeEmpty,
                "evidence_ids",
            ));
        }
        if affected_refs.is_empty() {
            return Err(validation_error(
                "relate_evidence_conflict",
                CoreErrorCode::ConflictScopeEmpty,
                "affected_refs",
            ));
        }
        sort_unique(
            &mut evidence_ids,
            "relate_evidence_conflict",
            "evidence_ids",
            CoreErrorCode::DuplicateEvidenceReference,
        )?;
        sort_unique(
            &mut affected_refs,
            "relate_evidence_conflict",
            "affected_refs",
            CoreErrorCode::ConflictScopeEmpty,
        )?;

        let conflict_id = ConflictId::derive(&ConflictIdentity {
            context_id,
            conflict_code: &conflict_code,
            evidence_ids: &evidence_ids,
            affected_refs: &affected_refs,
            subject_entity_key: subject_entity_key.as_ref(),
        })?;
        let record = Self {
            conflict_id,
            context_id,
            conflict_code,
            evidence_ids,
            affected_refs,
            subject_entity_key,
        };
        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> CoreResult<()> {
        integrity::validate_conflict(self)
    }

    #[must_use]
    pub const fn conflict_id(&self) -> ConflictId {
        self.conflict_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub const fn conflict_code(&self) -> &MessageCode {
        &self.conflict_code
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[EvidenceId] {
        &self.evidence_ids
    }

    #[must_use]
    pub fn affected_refs(&self) -> &[ConflictAffectedRef] {
        &self.affected_refs
    }

    #[must_use]
    pub const fn subject_entity_key(&self) -> Option<&EntityKey> {
        self.subject_entity_key.as_ref()
    }
}

#[derive(Serialize)]
struct ConflictIdentity<'a> {
    context_id: GenerationContextId,
    conflict_code: &'a MessageCode,
    evidence_ids: &'a [EvidenceId],
    affected_refs: &'a [ConflictAffectedRef],
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_entity_key: Option<&'a EntityKey>,
}

fn validate_evidence_authority(record: &EvidenceRecord) -> CoreResult<()> {
    if matches!(
        record.provenance,
        ProvenanceClass::SemanticCandidate | ProvenanceClass::ModelInference
    ) && record.confidence != EvidenceConfidence::Candidate
    {
        return Err(validation_error(
            "validate_evidence_record",
            CoreErrorCode::EvidenceAuthorityViolation,
            "confidence",
        ));
    }
    if record.confidence == EvidenceConfidence::Derived && record.derivation_input_ids.is_empty() {
        return Err(validation_error(
            "validate_evidence_record",
            CoreErrorCode::DerivedEvidenceMissingInputs,
            "derivation_input_ids",
        ));
    }
    if record.confidence == EvidenceConfidence::Proven && !record.derivation_input_ids.is_empty() {
        return Err(validation_error(
            "validate_evidence_record",
            CoreErrorCode::EvidenceAuthorityViolation,
            "derivation_input_ids",
        ));
    }
    if record.provenance == ProvenanceClass::RuntimeProbe
        && record.confidence == EvidenceConfidence::Proven
        && record.claim_scope != ClaimScope::RuntimeScenario
    {
        return Err(validation_error(
            "validate_evidence_record",
            CoreErrorCode::EvidenceAuthorityViolation,
            "claim_scope",
        ));
    }
    Ok(())
}

fn sort_unique<T: Ord>(
    values: &mut [T],
    operation: &'static str,
    field: &'static str,
    code: CoreErrorCode,
) -> CoreResult<()> {
    values.sort_unstable();
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(validation_error(operation, code, field))
    } else {
        Ok(())
    }
}

/// Operation wrapper for validating one evidence record.
pub fn validate_evidence_record(record: &EvidenceRecord) -> CoreResult<()> {
    record.validate()
}

/// Recomputes the identity of one evidence record from all non-ID fields.
pub fn derive_evidence_id(record: &EvidenceRecord) -> CoreResult<EvidenceId> {
    EvidenceId::derive(&EvidenceIdentity {
        context_id: record.context_id,
        provenance: record.provenance,
        confidence: record.confidence,
        claim_scope: record.claim_scope,
        producer_id: &record.producer_id,
        producer_version: &record.producer_version,
        source_handle_ids: &record.source_handle_ids,
        coverage_refs: &record.coverage_refs,
        derivation_input_ids: &record.derivation_input_ids,
    })
}

/// Validates reference closure, authority monotonicity, and acyclicity.
pub fn validate_evidence_derivation_graph(records: &[EvidenceRecord]) -> CoreResult<()> {
    use std::collections::{BTreeMap, BTreeSet};

    for record in records {
        record.validate()?;
    }
    let index = records
        .iter()
        .map(|record| (record.evidence_id, record))
        .collect::<BTreeMap<_, _>>();
    if index.len() != records.len() {
        return Err(validation_error(
            "validate_evidence_derivation_graph",
            CoreErrorCode::DuplicateEvidenceReference,
            "evidence_records.evidence_id",
        ));
    }
    for record in records {
        for input_id in &record.derivation_input_ids {
            let input = index.get(input_id).ok_or_else(|| {
                validation_error(
                    "validate_evidence_derivation_graph",
                    CoreErrorCode::MissingEvidenceReference,
                    "evidence_records.derivation_input_ids",
                )
            })?;
            if record.confidence == EvidenceConfidence::Derived
                && input.confidence == EvidenceConfidence::Candidate
            {
                return Err(validation_error(
                    "validate_evidence_derivation_graph",
                    CoreErrorCode::EvidenceAuthorityViolation,
                    "evidence_records.derivation_input_ids",
                ));
            }
            if record.claim_scope == ClaimScope::PlatformContract
                && input.provenance == ProvenanceClass::RuntimeProbe
            {
                return Err(validation_error(
                    "validate_evidence_derivation_graph",
                    CoreErrorCode::EvidenceAuthorityViolation,
                    "evidence_records.claim_scope",
                ));
            }
        }
    }

    fn visit(
        id: EvidenceId,
        index: &BTreeMap<EvidenceId, &EvidenceRecord>,
        temporary: &mut BTreeSet<EvidenceId>,
        permanent: &mut BTreeSet<EvidenceId>,
    ) -> CoreResult<()> {
        if permanent.contains(&id) {
            return Ok(());
        }
        if !temporary.insert(id) {
            return Err(validation_error(
                "validate_evidence_derivation_graph",
                CoreErrorCode::EvidenceDerivationCycle,
                "evidence_records.derivation_input_ids",
            ));
        }
        let record = index.get(&id).ok_or_else(|| {
            validation_error(
                "validate_evidence_derivation_graph",
                CoreErrorCode::MissingEvidenceReference,
                "evidence_records.evidence_id",
            )
        })?;
        for input in &record.derivation_input_ids {
            visit(*input, index, temporary, permanent)?;
        }
        temporary.remove(&id);
        permanent.insert(id);
        Ok(())
    }

    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for id in index.keys().copied() {
        visit(id, &index, &mut temporary, &mut permanent)?;
    }
    Ok(())
}

/// Constructs one immutable conflict relation.
pub fn relate_evidence_conflict(
    context_id: GenerationContextId,
    conflict_code: MessageCode,
    evidence_ids: Vec<EvidenceId>,
    affected_refs: Vec<ConflictAffectedRef>,
    subject_entity_key: Option<EntityKey>,
) -> CoreResult<ConflictRecord> {
    ConflictRecord::new(
        context_id,
        conflict_code,
        evidence_ids,
        affected_refs,
        subject_entity_key,
    )
}

/// Recomputes the identity of one conflict record.
pub fn derive_conflict_id(record: &ConflictRecord) -> CoreResult<ConflictId> {
    ConflictId::derive(&ConflictIdentity {
        context_id: record.context_id,
        conflict_code: &record.conflict_code,
        evidence_ids: &record.evidence_ids,
        affected_refs: &record.affected_refs,
        subject_entity_key: record.subject_entity_key.as_ref(),
    })
}

/// Operation wrapper for validating one conflict record.
pub fn validate_conflict_record(record: &ConflictRecord) -> CoreResult<()> {
    record.validate()
}

/// Constructs one immutable evidence record.
#[allow(clippy::too_many_arguments)]
pub fn derive_evidence(
    context_id: GenerationContextId,
    provenance: ProvenanceClass,
    confidence: EvidenceConfidence,
    claim_scope: ClaimScope,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    source_handle_ids: Vec<StableHandleId>,
    coverage_refs: Vec<EvidenceCoverageRef>,
    derivation_input_ids: Vec<EvidenceId>,
) -> CoreResult<EvidenceRecord> {
    EvidenceRecord::new(
        context_id,
        provenance,
        confidence,
        claim_scope,
        producer_id,
        producer_version,
        source_handle_ids,
        coverage_refs,
        derivation_input_ids,
    )
}
