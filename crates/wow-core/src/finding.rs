use std::collections::BTreeMap;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    canonical_evidence_key, canonical_json_digest, require_same_generation, CapabilityId,
    CoreError, CoreErrorCode, CoreResult, CoverageRecord, CoverageStatus, EvidenceId,
    EvidenceLevel, EvidenceRecord, FindingKey, GenerationContext, MessageKey, RootCauseKey,
    RuleId, SourceHandle, ToolVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Hint,
    Information,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingPolicy {
    Shadow,
    Advisory,
    Blocking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationClass {
    ExactEdit,
    ValidatedRecipe,
    PlanOnly,
    CandidateOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct MessageArguments(BTreeMap<String, String>);

impl MessageArguments {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> CoreResult<Option<String>> {
        let key = key.into();
        let value = value.into();
        validate_message_argument(&key, &value)?;
        Ok(self.0.insert(key, value))
    }

    pub fn from_map(values: BTreeMap<String, String>) -> CoreResult<Self> {
        for (key, value) in &values {
            validate_message_argument(key, value)?;
        }
        Ok(Self(values))
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    #[must_use]
    pub fn as_map(&self) -> &BTreeMap<String, String> {
        &self.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn validate_message_argument(key: &str, value: &str) -> CoreResult<()> {
    let key_bytes = key.as_bytes();
    if key.is_empty()
        || key.len() > 96
        || !key.is_ascii()
        || !key_bytes[0].is_ascii_alphanumeric()
        || !key_bytes[key_bytes.len() - 1].is_ascii_alphanumeric()
        || !key.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '-' | '_')
        })
    {
        return Err(CoreError::new(
            CoreErrorCode::ResultContextViolation,
            "validate_message_arguments",
            "message argument key is not canonical",
        ));
    }
    if value.len() > 2_048 || value.chars().any(char::is_control) {
        return Err(CoreError::new(
            CoreErrorCode::ResultContextViolation,
            "validate_message_arguments",
            "message argument value is oversized or contains control characters",
        ));
    }
    Ok(())
}

impl<'de> Deserialize<'de> for MessageArguments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = BTreeMap::<String, String>::deserialize(deserializer)?;
        Self::from_map(values).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub struct FindingInput {
    pub rule_id: RuleId,
    pub rule_version: ToolVersion,
    pub severity: FindingSeverity,
    pub policy: FindingPolicy,
    pub message_key: MessageKey,
    pub message_arguments: MessageArguments,
    pub primary_source: Option<SourceHandle>,
    pub related_sources: Vec<SourceHandle>,
    pub evidence: Vec<EvidenceRecord>,
    pub required_capabilities: Vec<CapabilityId>,
    pub coverage: Vec<CoverageRecord>,
    pub context: GenerationContext,
    pub root_cause: Option<RootCauseKey>,
    pub remediation: Option<RemediationClass>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    key: FindingKey,
    rule_id: RuleId,
    rule_version: ToolVersion,
    severity: FindingSeverity,
    policy: FindingPolicy,
    message_key: MessageKey,
    message_arguments: MessageArguments,
    primary_source: Option<SourceHandle>,
    related_sources: Vec<SourceHandle>,
    evidence: Vec<EvidenceRecord>,
    required_capabilities: Vec<CapabilityId>,
    coverage: Vec<CoverageRecord>,
    context: GenerationContext,
    root_cause: Option<RootCauseKey>,
    remediation: Option<RemediationClass>,
}

#[derive(Debug, Deserialize)]
struct FindingWire {
    key: FindingKey,
    rule_id: RuleId,
    rule_version: ToolVersion,
    severity: FindingSeverity,
    policy: FindingPolicy,
    message_key: MessageKey,
    message_arguments: MessageArguments,
    primary_source: Option<SourceHandle>,
    related_sources: Vec<SourceHandle>,
    evidence: Vec<EvidenceRecord>,
    required_capabilities: Vec<CapabilityId>,
    coverage: Vec<CoverageRecord>,
    context: GenerationContext,
    root_cause: Option<RootCauseKey>,
    remediation: Option<RemediationClass>,
}

#[derive(Serialize)]
struct FindingIdentity<'a> {
    rule_id: &'a RuleId,
    rule_version: &'a ToolVersion,
    message_key: &'a MessageKey,
    message_arguments: &'a MessageArguments,
    primary_source: Option<&'a SourceHandle>,
    related_source_ids: Vec<&'a crate::StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    required_capabilities: &'a [CapabilityId],
    coverage: &'a [CoverageRecord],
    context: &'a GenerationContext,
    root_cause: Option<&'a RootCauseKey>,
}

impl Finding {
    pub fn new(mut input: FindingInput) -> CoreResult<Self> {
        canonicalize_input(&mut input)?;
        validate_input(&input)?;
        let key = canonical_finding_key_from_input(&input)?;
        Ok(Self {
            key,
            rule_id: input.rule_id,
            rule_version: input.rule_version,
            severity: input.severity,
            policy: input.policy,
            message_key: input.message_key,
            message_arguments: input.message_arguments,
            primary_source: input.primary_source,
            related_sources: input.related_sources,
            evidence: input.evidence,
            required_capabilities: input.required_capabilities,
            coverage: input.coverage,
            context: input.context,
            root_cause: input.root_cause,
            remediation: input.remediation,
        })
    }

    pub fn validate(&self) -> CoreResult<()> {
        let input = FindingInput {
            rule_id: self.rule_id.clone(),
            rule_version: self.rule_version.clone(),
            severity: self.severity,
            policy: self.policy,
            message_key: self.message_key.clone(),
            message_arguments: self.message_arguments.clone(),
            primary_source: self.primary_source.clone(),
            related_sources: self.related_sources.clone(),
            evidence: self.evidence.clone(),
            required_capabilities: self.required_capabilities.clone(),
            coverage: self.coverage.clone(),
            context: self.context.clone(),
            root_cause: self.root_cause.clone(),
            remediation: self.remediation,
        };
        validate_input(&input)?;
        if canonical_finding_key_from_input(&input)? != self.key {
            return Err(CoreError::new(
                CoreErrorCode::ResultContextViolation,
                "validate_finding",
                "finding key does not match canonical structured identity",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn key(&self) -> &FindingKey {
        &self.key
    }

    #[must_use]
    pub fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    #[must_use]
    pub fn severity(&self) -> FindingSeverity {
        self.severity
    }

    #[must_use]
    pub fn policy(&self) -> FindingPolicy {
        self.policy
    }

    #[must_use]
    pub fn context(&self) -> &GenerationContext {
        &self.context
    }

    #[must_use]
    pub fn primary_source(&self) -> Option<&SourceHandle> {
        self.primary_source.as_ref()
    }

    #[must_use]
    pub fn evidence(&self) -> &[EvidenceRecord] {
        &self.evidence
    }

    #[must_use]
    pub fn coverage(&self) -> &[CoverageRecord] {
        &self.coverage
    }

    #[must_use]
    pub fn remediation(&self) -> Option<RemediationClass> {
        self.remediation
    }
}

fn canonicalize_input(input: &mut FindingInput) -> CoreResult<()> {
    input.related_sources.sort_by(|left, right| left.id().cmp(right.id()));
    input.related_sources.dedup_by(|left, right| left.id() == right.id());

    let mut evidence_with_keys = input
        .evidence
        .drain(..)
        .map(|record| canonical_evidence_key(&record).map(|key| (key, record)))
        .collect::<CoreResult<Vec<_>>>()?;
    evidence_with_keys.sort_by(|left, right| left.0.cmp(&right.0));
    evidence_with_keys.dedup_by(|left, right| left.0 == right.0);
    input.evidence = evidence_with_keys
        .into_iter()
        .map(|(_, record)| record)
        .collect();

    input.required_capabilities.sort();
    input.required_capabilities.dedup();
    input.coverage.sort_by(|left, right| {
        (
            left.capability(),
            left.partition(),
            left.producer(),
            left.status(),
        )
            .cmp(&(
                right.capability(),
                right.partition(),
                right.producer(),
                right.status(),
            ))
    });
    Ok(())
}

fn validate_input(input: &FindingInput) -> CoreResult<()> {
    if let Some(source) = &input.primary_source {
        source.validate()?;
    }
    for source in &input.related_sources {
        source.validate()?;
    }
    for evidence in &input.evidence {
        evidence.validate()?;
        evidence.require_context(&input.context)?;
    }
    for coverage in &input.coverage {
        coverage.validate()?;
        require_same_generation(&input.context, coverage.generation())?;
    }

    if input.remediation == Some(RemediationClass::ExactEdit) {
        let proven_basis = input.evidence.iter().any(|record| {
            matches!(record.confidence(), EvidenceLevel::Proven | EvidenceLevel::Derived)
        });
        let complete_coverage = !input.coverage.is_empty()
            && input.coverage.iter().all(|record| {
                matches!(
                    record.status(),
                    CoverageStatus::Complete | CoverageStatus::NotApplicable
                )
            });
        if !proven_basis || !complete_coverage {
            return Err(CoreError::new(
                CoreErrorCode::ResultContextViolation,
                "validate_finding",
                "exact_edit requires proven/derived evidence and complete applicable coverage",
            ));
        }
    }
    Ok(())
}

fn canonical_finding_key_from_input(input: &FindingInput) -> CoreResult<FindingKey> {
    let evidence_ids = input
        .evidence
        .iter()
        .map(canonical_evidence_key)
        .collect::<CoreResult<Vec<_>>>()?;
    let identity = FindingIdentity {
        rule_id: &input.rule_id,
        rule_version: &input.rule_version,
        message_key: &input.message_key,
        message_arguments: &input.message_arguments,
        primary_source: input.primary_source.as_ref(),
        related_source_ids: input.related_sources.iter().map(SourceHandle::id).collect(),
        evidence_ids,
        required_capabilities: &input.required_capabilities,
        coverage: &input.coverage,
        context: &input.context,
        root_cause: input.root_cause.as_ref(),
    };
    let digest = canonical_json_digest(&identity)?;
    FindingKey::parse(format!(
        "finding:{}",
        digest.canonical_string().trim_start_matches("sha256:")
    ))
}

pub fn canonical_finding_key(finding: &Finding) -> CoreResult<FindingKey> {
    finding.validate()?;
    Ok(finding.key.clone())
}

impl<'de> Deserialize<'de> for Finding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let FindingWire {
            key,
            rule_id,
            rule_version,
            severity,
            policy,
            message_key,
            message_arguments,
            primary_source,
            related_sources,
            evidence,
            required_capabilities,
            coverage,
            context,
            root_cause,
            remediation,
        } = FindingWire::deserialize(deserializer)?;
        let finding = Finding::new(FindingInput {
            rule_id,
            rule_version,
            severity,
            policy,
            message_key,
            message_arguments,
            primary_source,
            related_sources,
            evidence,
            required_capabilities,
            coverage,
            context,
            root_cause,
            remediation,
        })
        .map_err(D::Error::custom)?;
        if finding.key != key {
            return Err(D::Error::custom("serialized finding key is not canonical"));
        }
        Ok(finding)
    }
}
