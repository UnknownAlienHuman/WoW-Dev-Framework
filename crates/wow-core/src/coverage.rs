use std::collections::{BTreeMap, BTreeSet};

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    require_same_generation, CapabilityId, CoreError, CoreErrorCode, CoreResult,
    CoveragePartitionId, EvidenceId, GenerationContext, ProducerId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Complete,
    Partial,
    Unknown,
    Failed,
    NotApplicable,
    NotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CoverageRecord {
    partition: CoveragePartitionId,
    capability: CapabilityId,
    status: CoverageStatus,
    missing_inputs: Vec<String>,
    missing_capabilities: Vec<CapabilityId>,
    producer: ProducerId,
    generation: GenerationContext,
    conflicts: Vec<EvidenceId>,
    reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CoverageRecordInput {
    pub partition: CoveragePartitionId,
    pub capability: CapabilityId,
    pub status: CoverageStatus,
    pub missing_inputs: Vec<String>,
    pub missing_capabilities: Vec<CapabilityId>,
    pub producer: ProducerId,
    pub generation: GenerationContext,
    pub conflicts: Vec<EvidenceId>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CoverageRecordWire {
    partition: CoveragePartitionId,
    capability: CapabilityId,
    status: CoverageStatus,
    missing_inputs: Vec<String>,
    missing_capabilities: Vec<CapabilityId>,
    producer: ProducerId,
    generation: GenerationContext,
    conflicts: Vec<EvidenceId>,
    reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub capability: CapabilityId,
    pub status: CoverageStatus,
    pub partitions: Vec<CoveragePartitionId>,
    pub missing_inputs: Vec<String>,
    pub missing_capabilities: Vec<CapabilityId>,
    pub conflicts: Vec<EvidenceId>,
    pub reasons: Vec<String>,
    pub generation: GenerationContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegativeAuthorityStatus {
    Authoritative,
    Partial,
    Unknown,
    Failed,
    Conflict,
    NotApplicable,
    NotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegativeAuthorityDecision {
    pub status: NegativeAuthorityStatus,
    pub capability: CapabilityId,
    pub partitions: Vec<CoveragePartitionId>,
    pub reasons: Vec<String>,
}

impl CoverageRecord {
    pub fn new(input: CoverageRecordInput) -> CoreResult<Self> {
        let mut record = Self {
            partition: input.partition,
            capability: input.capability,
            status: input.status,
            missing_inputs: input.missing_inputs,
            missing_capabilities: input.missing_capabilities,
            producer: input.producer,
            generation: input.generation,
            conflicts: input.conflicts,
            reasons: input.reasons,
        };
        record.canonicalize();
        record.validate()?;
        Ok(record)
    }

    fn canonicalize(&mut self) {
        self.missing_inputs.sort();
        self.missing_inputs.dedup();
        self.missing_capabilities.sort();
        self.missing_capabilities.dedup();
        self.conflicts.sort();
        self.conflicts.dedup();
        self.reasons.sort();
        self.reasons.dedup();
    }

    pub fn validate(&self) -> CoreResult<()> {
        validate_text_items("missing input", &self.missing_inputs)?;
        validate_text_items("coverage reason", &self.reasons)?;

        if matches!(self.status, CoverageStatus::Complete | CoverageStatus::NotApplicable)
            && (!self.missing_inputs.is_empty()
                || !self.missing_capabilities.is_empty()
                || !self.conflicts.is_empty())
        {
            return Err(CoreError::new(
                CoreErrorCode::CoverageConflict,
                "validate_coverage",
                "complete/not-applicable coverage cannot carry missing inputs or conflicts",
            ));
        }

        if matches!(
            self.status,
            CoverageStatus::Partial
                | CoverageStatus::Unknown
                | CoverageStatus::Failed
                | CoverageStatus::NotEvaluated
        ) && self.missing_inputs.is_empty()
            && self.missing_capabilities.is_empty()
            && self.conflicts.is_empty()
            && self.reasons.is_empty()
        {
            return Err(CoreError::new(
                CoreErrorCode::CoverageConflict,
                "validate_coverage",
                "incomplete coverage requires a structured reason",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn partition(&self) -> &CoveragePartitionId {
        &self.partition
    }

    #[must_use]
    pub fn capability(&self) -> &CapabilityId {
        &self.capability
    }

    #[must_use]
    pub fn status(&self) -> CoverageStatus {
        self.status
    }

    #[must_use]
    pub fn missing_inputs(&self) -> &[String] {
        &self.missing_inputs
    }

    #[must_use]
    pub fn missing_capabilities(&self) -> &[CapabilityId] {
        &self.missing_capabilities
    }

    #[must_use]
    pub fn producer(&self) -> &ProducerId {
        &self.producer
    }

    #[must_use]
    pub fn generation(&self) -> &GenerationContext {
        &self.generation
    }

    #[must_use]
    pub fn conflicts(&self) -> &[EvidenceId] {
        &self.conflicts
    }

    #[must_use]
    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }
}

fn validate_text_items(kind: &'static str, values: &[String]) -> CoreResult<()> {
    if values.iter().any(|value| {
        value.is_empty() || value.len() > 1_024 || value.chars().any(char::is_control)
    }) {
        return Err(CoreError::new(
            CoreErrorCode::CoverageConflict,
            "validate_coverage",
            format!("{kind} is empty, oversized, or contains control characters"),
        ));
    }
    Ok(())
}

pub fn combine_coverage(records: &[CoverageRecord]) -> CoreResult<CoverageSummary> {
    let first = records.first().ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::CoverageConflict,
            "combine_coverage",
            "at least one coverage record is required",
        )
    })?;

    let capability = first.capability.clone();
    let generation = first.generation.clone();
    let mut partitions = BTreeSet::new();
    let mut status_by_partition = BTreeMap::new();
    let mut missing_inputs = BTreeSet::new();
    let mut missing_capabilities = BTreeSet::new();
    let mut conflicts = BTreeSet::new();
    let mut reasons = BTreeSet::new();
    let mut statuses = Vec::with_capacity(records.len());
    let mut structural_conflict = false;

    for record in records {
        record.validate()?;
        if record.capability != capability {
            return Err(CoreError::new(
                CoreErrorCode::CoverageConflict,
                "combine_coverage",
                "records for different capabilities cannot be combined",
            ));
        }
        require_same_generation(&generation, &record.generation)?;
        partitions.insert(record.partition.clone());
        if let Some(previous) = status_by_partition.insert(record.partition.clone(), record.status) {
            if previous != record.status {
                structural_conflict = true;
                reasons.insert(format!(
                    "partition {} has conflicting statuses {:?} and {:?}",
                    record.partition, previous, record.status
                ));
            }
        }
        statuses.push(record.status);
        missing_inputs.extend(record.missing_inputs.iter().cloned());
        missing_capabilities.extend(record.missing_capabilities.iter().cloned());
        conflicts.extend(record.conflicts.iter().cloned());
        reasons.extend(record.reasons.iter().cloned());
    }

    if structural_conflict && conflicts.is_empty() {
        reasons.insert("coverage records conflict without a resolved evidence reference".to_owned());
    }

    let status = combined_status(&statuses);
    Ok(CoverageSummary {
        capability,
        status,
        partitions: partitions.into_iter().collect(),
        missing_inputs: missing_inputs.into_iter().collect(),
        missing_capabilities: missing_capabilities.into_iter().collect(),
        conflicts: conflicts.into_iter().collect(),
        reasons: reasons.into_iter().collect(),
        generation,
    })
}

fn combined_status(statuses: &[CoverageStatus]) -> CoverageStatus {
    if statuses.contains(&CoverageStatus::Failed) {
        CoverageStatus::Failed
    } else if statuses.contains(&CoverageStatus::NotEvaluated) {
        CoverageStatus::NotEvaluated
    } else if statuses.contains(&CoverageStatus::Unknown) {
        CoverageStatus::Unknown
    } else if statuses.contains(&CoverageStatus::Partial) {
        CoverageStatus::Partial
    } else if statuses.contains(&CoverageStatus::Complete) {
        CoverageStatus::Complete
    } else {
        CoverageStatus::NotApplicable
    }
}

pub fn evaluate_negative_authority(summary: &CoverageSummary) -> NegativeAuthorityDecision {
    let status = if !summary.conflicts.is_empty()
        || summary
            .reasons
            .iter()
            .any(|reason| reason.contains("conflict"))
    {
        NegativeAuthorityStatus::Conflict
    } else {
        match summary.status {
            CoverageStatus::Complete => NegativeAuthorityStatus::Authoritative,
            CoverageStatus::Partial => NegativeAuthorityStatus::Partial,
            CoverageStatus::Unknown => NegativeAuthorityStatus::Unknown,
            CoverageStatus::Failed => NegativeAuthorityStatus::Failed,
            CoverageStatus::NotApplicable => NegativeAuthorityStatus::NotApplicable,
            CoverageStatus::NotEvaluated => NegativeAuthorityStatus::NotEvaluated,
        }
    };

    let mut reasons = summary.reasons.clone();
    if status != NegativeAuthorityStatus::Authoritative && reasons.is_empty() {
        reasons.push(format!("coverage status is {:?}", summary.status));
    }
    NegativeAuthorityDecision {
        status,
        capability: summary.capability.clone(),
        partitions: summary.partitions.clone(),
        reasons,
    }
}

impl<'de> Deserialize<'de> for CoverageRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CoverageRecordWire::deserialize(deserializer)?;
        Self::new(CoverageRecordInput {
            partition: wire.partition,
            capability: wire.capability,
            status: wire.status,
            missing_inputs: wire.missing_inputs,
            missing_capabilities: wire.missing_capabilities,
            producer: wire.producer,
            generation: wire.generation,
            conflicts: wire.conflicts,
            reasons: wire.reasons,
        })
        .map_err(D::Error::custom)
    }
}
