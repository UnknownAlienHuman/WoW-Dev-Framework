use std::collections::{BTreeMap, BTreeSet};

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    canonical_json, canonical_json_digest, require_same_generation, CapabilityId,
    ContentDigest, CoreError, CoreErrorCode, CoreResult, CoveragePartitionId,
    CoverageStatus, Finding, GenerationContext, MessageArguments, MessageKey, OperationId,
    SchemaVersion,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityState {
    pub capability: CapabilityId,
    pub status: CoverageStatus,
    pub partitions: Vec<CoveragePartitionId>,
    pub reasons: Vec<String>,
}

impl CapabilityState {
    pub fn canonicalize(&mut self) -> CoreResult<()> {
        self.partitions.sort();
        self.partitions.dedup();
        self.reasons.sort();
        self.reasons.dedup();
        validate_text_list("capability reason", &self.reasons)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotEvaluatedRecord {
    pub subject: OperationId,
    pub missing_capabilities: Vec<CapabilityId>,
    pub partitions: Vec<CoveragePartitionId>,
    pub reasons: Vec<String>,
    pub context: GenerationContext,
}

impl NotEvaluatedRecord {
    pub fn canonicalize(&mut self) -> CoreResult<()> {
        self.missing_capabilities.sort();
        self.missing_capabilities.dedup();
        self.partitions.sort();
        self.partitions.dedup();
        self.reasons.sort();
        self.reasons.dedup();
        validate_text_list("NotEvaluated reason", &self.reasons)?;
        if self.missing_capabilities.is_empty()
            && self.partitions.is_empty()
            && self.reasons.is_empty()
        {
            return Err(CoreError::new(
                CoreErrorCode::ResultContextViolation,
                "validate_not_evaluated",
                "NotEvaluated requires a missing capability, partition, or reason",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultWarning {
    pub message_key: MessageKey,
    pub arguments: MessageArguments,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TruncationStatus {
    Complete,
    Truncated {
        reason: String,
        omitted_items: Option<u64>,
    },
}

impl TruncationStatus {
    fn validate(&self) -> CoreResult<()> {
        if let Self::Truncated { reason, .. } = self {
            if reason.is_empty() || reason.len() > 1_024 || reason.chars().any(char::is_control) {
                return Err(CoreError::new(
                    CoreErrorCode::BudgetInvalid,
                    "validate_result_envelope",
                    "truncation reason is empty, oversized, or contains control characters",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ResultEnvelopeInput {
    pub schema_version: SchemaVersion,
    pub operation: OperationId,
    pub context: GenerationContext,
    pub capabilities: Vec<CapabilityState>,
    pub findings: Vec<Finding>,
    pub not_evaluated: Vec<NotEvaluatedRecord>,
    pub warnings: Vec<ResultWarning>,
    pub truncation: TruncationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultEnvelope {
    schema_version: SchemaVersion,
    operation: OperationId,
    context: GenerationContext,
    capabilities: Vec<CapabilityState>,
    findings: Vec<Finding>,
    not_evaluated: Vec<NotEvaluatedRecord>,
    warnings: Vec<ResultWarning>,
    truncation: TruncationStatus,
}

#[derive(Debug, Deserialize)]
struct ResultEnvelopeWire {
    schema_version: SchemaVersion,
    operation: OperationId,
    context: GenerationContext,
    capabilities: Vec<CapabilityState>,
    findings: Vec<Finding>,
    not_evaluated: Vec<NotEvaluatedRecord>,
    warnings: Vec<ResultWarning>,
    truncation: TruncationStatus,
}

impl ResultEnvelope {
    pub fn new(input: ResultEnvelopeInput) -> CoreResult<Self> {
        let mut envelope = Self {
            schema_version: input.schema_version,
            operation: input.operation,
            context: input.context,
            capabilities: input.capabilities,
            findings: input.findings,
            not_evaluated: input.not_evaluated,
            warnings: input.warnings,
            truncation: input.truncation,
        };
        canonical_result_order(&mut envelope)?;
        validate_result_envelope(&envelope)?;
        Ok(envelope)
    }

    pub fn canonical_json(&self) -> CoreResult<String> {
        validate_result_envelope(self)?;
        canonical_json(self)
    }

    pub fn canonical_digest(&self) -> CoreResult<ContentDigest> {
        validate_result_envelope(self)?;
        canonical_json_digest(self)
    }

    #[must_use]
    pub fn schema_version(&self) -> &SchemaVersion {
        &self.schema_version
    }

    #[must_use]
    pub fn operation(&self) -> &OperationId {
        &self.operation
    }

    #[must_use]
    pub fn context(&self) -> &GenerationContext {
        &self.context
    }

    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityState] {
        &self.capabilities
    }

    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    #[must_use]
    pub fn not_evaluated(&self) -> &[NotEvaluatedRecord] {
        &self.not_evaluated
    }

    #[must_use]
    pub fn warnings(&self) -> &[ResultWarning] {
        &self.warnings
    }

    #[must_use]
    pub fn truncation(&self) -> &TruncationStatus {
        &self.truncation
    }
}

pub fn canonical_result_order(envelope: &mut ResultEnvelope) -> CoreResult<()> {
    for capability in &mut envelope.capabilities {
        capability.canonicalize()?;
    }
    envelope
        .capabilities
        .sort_by(|left, right| left.capability.cmp(&right.capability));

    envelope.findings.sort_by(|left, right| left.key().cmp(right.key()));
    let mut unique_findings = Vec::with_capacity(envelope.findings.len());
    for finding in envelope.findings.drain(..) {
        if let Some(previous) = unique_findings.last() {
            if previous.key() == finding.key() {
                if previous != &finding {
                    return Err(CoreError::new(
                        CoreErrorCode::ResultContextViolation,
                        "canonical_result_order",
                        format!("finding key {} maps to non-identical findings", finding.key()),
                    ));
                }
                continue;
            }
        }
        unique_findings.push(finding);
    }
    envelope.findings = unique_findings;

    for record in &mut envelope.not_evaluated {
        record.canonicalize()?;
    }
    envelope.not_evaluated.sort_by(|left, right| {
        (
            &left.subject,
            &left.missing_capabilities,
            &left.partitions,
            &left.reasons,
        )
            .cmp(&(
                &right.subject,
                &right.missing_capabilities,
                &right.partitions,
                &right.reasons,
            ))
    });
    envelope.not_evaluated.dedup();

    envelope.warnings.sort_by(|left, right| {
        (&left.message_key, left.arguments.as_map())
            .cmp(&(&right.message_key, right.arguments.as_map()))
    });
    envelope.warnings.dedup();
    Ok(())
}

pub fn validate_result_envelope(envelope: &ResultEnvelope) -> CoreResult<()> {
    if envelope.schema_version.as_str() != "1" {
        return Err(CoreError::new(
            CoreErrorCode::SchemaVersionUnsupported,
            "validate_result_envelope",
            format!("unsupported result schema {}", envelope.schema_version),
        ));
    }
    envelope.context.profile().validate()?;
    envelope.truncation.validate()?;

    let mut capability_ids = BTreeSet::new();
    for capability in &envelope.capabilities {
        if !capability_ids.insert(capability.capability.clone()) {
            return Err(CoreError::new(
                CoreErrorCode::ResultContextViolation,
                "validate_result_envelope",
                format!("duplicate capability state {}", capability.capability),
            ));
        }
        validate_text_list("capability reason", &capability.reasons)?;
    }

    for finding in &envelope.findings {
        finding.validate()?;
        require_same_generation(&envelope.context, finding.context())?;
    }
    for record in &envelope.not_evaluated {
        let mut canonical = record.clone();
        canonical.canonicalize()?;
        require_same_generation(&envelope.context, &record.context)?;
    }

    let capability_map = envelope
        .capabilities
        .iter()
        .map(|state| (&state.capability, state.status))
        .collect::<BTreeMap<_, _>>();
    for finding in &envelope.findings {
        for coverage in finding.coverage() {
            if let Some(status) = capability_map.get(coverage.capability()) {
                if **status == CoverageStatus::Complete
                    && coverage.status() != CoverageStatus::Complete
                    && coverage.status() != CoverageStatus::NotApplicable
                {
                    return Err(CoreError::new(
                        CoreErrorCode::ResultContextViolation,
                        "validate_result_envelope",
                        format!(
                            "capability {} is globally complete but finding coverage is {:?}",
                            coverage.capability(),
                            coverage.status()
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_text_list(kind: &'static str, values: &[String]) -> CoreResult<()> {
    if values.iter().any(|value| {
        value.is_empty() || value.len() > 1_024 || value.chars().any(char::is_control)
    }) {
        return Err(CoreError::new(
            CoreErrorCode::ResultContextViolation,
            "validate_result_envelope",
            format!("{kind} is empty, oversized, or contains control characters"),
        ));
    }
    Ok(())
}

impl<'de> Deserialize<'de> for ResultEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ResultEnvelopeWire::deserialize(deserializer)?;
        Self::new(ResultEnvelopeInput {
            schema_version: wire.schema_version,
            operation: wire.operation,
            context: wire.context,
            capabilities: wire.capabilities,
            findings: wire.findings,
            not_evaluated: wire.not_evaluated,
            warnings: wire.warnings,
            truncation: wire.truncation,
        })
        .map_err(D::Error::custom)
    }
}
