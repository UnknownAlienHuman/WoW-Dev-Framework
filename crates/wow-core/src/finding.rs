use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::digest::{
    EvidenceId, FindingFingerprint, FindingId, GenerationContextId, RootCauseKey, StableHandleId,
    WarningId,
};
use crate::error::{CoreErrorCode, CoreResult, validation_error};
use crate::ids::{EntityKey, MessageCode, ProducerId, RuleId, ToolVersion, validate_lower_segment};

mod admission;
pub(crate) use admission::DiagnosticRegistry;

const MAX_MESSAGE_ARGUMENTS: usize = 128;
const MAX_ARGUMENT_VALUE_BYTES: usize = 4096;

/// User-visible diagnostic severity. Severity does not determine rollout policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Operational rollout policy for a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutPolicy {
    Shadow,
    Advisory,
    Blocking,
}

/// Authority class of an offered remediation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationClass {
    ExactEdit,
    ValidatedRecipe,
    PlanOnly,
    CandidateOnly,
}

/// Closed scalar kind for structured message arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageArgumentKind {
    Text,
    Integer,
    Boolean,
    Identifier,
    Path,
    Digest,
}

/// One bounded structured message argument. Arbitrary nested JSON is excluded.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageArgument {
    name: String,
    kind: MessageArgumentKind,
    value: String,
    identity_relevant: bool,
}

impl MessageArgument {
    /// Validates and constructs a message argument.
    pub fn new(
        name: impl Into<String>,
        kind: MessageArgumentKind,
        value: impl Into<String>,
        identity_relevant: bool,
    ) -> CoreResult<Self> {
        let name = name.into();
        let value = value.into();
        validate_lower_segment(&name, "validate_message_arguments", "arguments.name")?;
        validate_argument_value(kind, &value)?;
        Ok(Self {
            name,
            kind,
            value,
            identity_relevant,
        })
    }

    /// Stable argument name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Declared argument kind.
    #[must_use]
    pub const fn kind(&self) -> MessageArgumentKind {
        self.kind
    }

    /// Canonical scalar representation.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Whether this argument contributes to finding identity.
    #[must_use]
    pub const fn identity_relevant(&self) -> bool {
        self.identity_relevant
    }
}

/// Validates canonical ordering and uniqueness of structured arguments.
pub fn validate_message_arguments(arguments: &[MessageArgument]) -> CoreResult<()> {
    if arguments.len() > MAX_MESSAGE_ARGUMENTS {
        return Err(validation_error(
            "validate_message_arguments",
            CoreErrorCode::InvalidMessageArgument,
            "arguments",
        )
        .with_argument("count", arguments.len().to_string()));
    }
    let mut previous: Option<&str> = None;
    for argument in arguments {
        validate_lower_segment(
            argument.name(),
            "validate_message_arguments",
            "arguments.name",
        )?;
        validate_argument_value(argument.kind(), argument.value())?;
        if previous.is_some_and(|name| name >= argument.name()) {
            return Err(validation_error(
                "validate_message_arguments",
                CoreErrorCode::InvalidMessageArgument,
                "arguments",
            )
            .with_argument("reason", "duplicate_or_noncanonical_order"));
        }
        previous = Some(argument.name());
    }
    Ok(())
}

/// Structured remediation metadata. It never applies an edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Remediation {
    class: RemediationClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipe_id: Option<RuleId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan_handle_id: Option<StableHandleId>,
}

impl Remediation {
    /// Constructs remediation metadata under its class-specific shape.
    pub fn new(
        class: RemediationClass,
        recipe_id: Option<RuleId>,
        plan_handle_id: Option<StableHandleId>,
    ) -> CoreResult<Self> {
        let remediation = Self {
            class,
            recipe_id,
            plan_handle_id,
        };
        remediation.validate()?;
        Ok(remediation)
    }

    // Decoding bypasses `new`; keep class-specific shape admission in one place.
    fn validate(&self) -> CoreResult<()> {
        if matches!(
            self.class,
            RemediationClass::ExactEdit | RemediationClass::ValidatedRecipe
        ) && self.recipe_id.is_none()
        {
            return Err(validation_error(
                "bind_finding_to_context",
                CoreErrorCode::RemediationAuthorityViolation,
                "remediation.recipe_id",
            ));
        }
        Ok(())
    }

    /// Remediation authority class.
    #[must_use]
    pub const fn class(&self) -> RemediationClass {
        self.class
    }
}

/// Input record before a generation-bound `FindingId` is derived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingDraft {
    context_id: GenerationContextId,
    rule_id: RuleId,
    rule_version: ToolVersion,
    finding_code: MessageCode,
    severity: Severity,
    policy: RolloutPolicy,
    subject_entity_key: Option<EntityKey>,
    primary_source_handle_id: StableHandleId,
    related_source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    required_capability_ids: Vec<crate::CapabilityId>,
    coverage_status: crate::CoverageStatus,
    message_arguments: Vec<MessageArgument>,
    root_cause_key: Option<RootCauseKey>,
    caused_by_root_cause_key: Option<RootCauseKey>,
    remediation: Option<Remediation>,
}

impl FindingDraft {
    /// Creates the required semantic portion of a finding.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        context_id: GenerationContextId,
        rule_id: RuleId,
        rule_version: ToolVersion,
        finding_code: MessageCode,
        severity: Severity,
        policy: RolloutPolicy,
        primary_source_handle_id: StableHandleId,
        coverage_status: crate::CoverageStatus,
    ) -> Self {
        Self {
            context_id,
            rule_id,
            rule_version,
            finding_code,
            severity,
            policy,
            subject_entity_key: None,
            primary_source_handle_id,
            related_source_handle_ids: Vec::new(),
            evidence_ids: Vec::new(),
            required_capability_ids: Vec::new(),
            coverage_status,
            message_arguments: Vec::new(),
            root_cause_key: None,
            caused_by_root_cause_key: None,
            remediation: None,
        }
    }

    /// Associates an exact semantic subject.
    #[must_use]
    pub fn subject_entity_key(mut self, subject: EntityKey) -> Self {
        self.subject_entity_key = Some(subject);
        self
    }

    /// Replaces related source handles.
    #[must_use]
    pub fn related_source_handle_ids(mut self, mut ids: Vec<StableHandleId>) -> Self {
        ids.sort_unstable();
        ids.dedup();
        self.related_source_handle_ids = ids;
        self
    }

    /// Replaces evidence references.
    #[must_use]
    pub fn evidence_ids(mut self, mut ids: Vec<EvidenceId>) -> Self {
        ids.sort_unstable();
        ids.dedup();
        self.evidence_ids = ids;
        self
    }

    /// Replaces required capability IDs.
    #[must_use]
    pub fn required_capability_ids(mut self, mut ids: Vec<crate::CapabilityId>) -> Self {
        ids.sort();
        ids.dedup();
        self.required_capability_ids = ids;
        self
    }

    /// Replaces message arguments after canonical sorting.
    pub fn message_arguments(mut self, mut arguments: Vec<MessageArgument>) -> CoreResult<Self> {
        arguments.sort();
        validate_message_arguments(&arguments)?;
        self.message_arguments = arguments;
        Ok(self)
    }

    /// Associates causal grouping keys.
    #[must_use]
    pub const fn root_causes(
        mut self,
        root: Option<RootCauseKey>,
        caused_by: Option<RootCauseKey>,
    ) -> Self {
        self.root_cause_key = root;
        self.caused_by_root_cause_key = caused_by;
        self
    }

    /// Associates non-effecting remediation metadata.
    #[must_use]
    pub fn remediation(mut self, remediation: Remediation) -> Self {
        self.remediation = Some(remediation);
        self
    }

    /// Derives the semantic fingerprint that excludes generation context and presentation overrides.
    pub fn fingerprint(&self) -> CoreResult<FindingFingerprint> {
        #[derive(Serialize)]
        struct FingerprintProjection<'a> {
            finding_code: &'a MessageCode,
            identity_message_arguments: Vec<&'a MessageArgument>,
            primary_source_handle_id: StableHandleId,
            #[serde(skip_serializing_if = "Option::is_none")]
            root_cause_key: Option<RootCauseKey>,
            rule_id: &'a RuleId,
            rule_version: &'a ToolVersion,
            #[serde(skip_serializing_if = "Option::is_none")]
            subject_entity_key: Option<&'a EntityKey>,
        }

        let identity_message_arguments = self
            .message_arguments
            .iter()
            .filter(|argument| argument.identity_relevant())
            .collect();
        FindingFingerprint::derive(&FingerprintProjection {
            finding_code: &self.finding_code,
            identity_message_arguments,
            primary_source_handle_id: self.primary_source_handle_id,
            root_cause_key: self.root_cause_key,
            rule_id: &self.rule_id,
            rule_version: &self.rule_version,
            subject_entity_key: self.subject_entity_key.as_ref(),
        })
    }

    /// Validates references and binds the finding to its exact context.
    pub fn bind(
        self,
        context_id: GenerationContextId,
        source_handles: &[crate::SourceHandle],
        evidence_records: &[crate::EvidenceRecord],
    ) -> CoreResult<Finding> {
        if self.context_id != context_id {
            return Err(validation_error(
                "bind_finding_to_context",
                CoreErrorCode::FindingContextMismatch,
                "context_id",
            ));
        }
        validate_message_arguments(&self.message_arguments)?;
        let fingerprint = self.fingerprint()?;
        #[derive(Serialize)]
        struct FindingIdentity {
            context_id: GenerationContextId,
            finding_fingerprint: FindingFingerprint,
        }
        let finding_id = FindingId::derive(&FindingIdentity {
            context_id,
            finding_fingerprint: fingerprint,
        })?;
        let finding = Finding {
            finding_id,
            fingerprint,
            context_id: self.context_id,
            rule_id: self.rule_id,
            rule_version: self.rule_version,
            finding_code: self.finding_code,
            severity: self.severity,
            policy: self.policy,
            subject_entity_key: self.subject_entity_key,
            primary_source_handle_id: self.primary_source_handle_id,
            related_source_handle_ids: self.related_source_handle_ids,
            evidence_ids: self.evidence_ids,
            required_capability_ids: self.required_capability_ids,
            coverage_status: self.coverage_status,
            message_arguments: self.message_arguments,
            root_cause_key: self.root_cause_key,
            caused_by_root_cause_key: self.caused_by_root_cause_key,
            remediation: self.remediation,
        };
        finding.validate(context_id, source_handles, evidence_records)?;
        Ok(finding)
    }
}

/// Immutable generation-bound diagnostic finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    finding_id: FindingId,
    fingerprint: FindingFingerprint,
    context_id: GenerationContextId,
    rule_id: RuleId,
    rule_version: ToolVersion,
    finding_code: MessageCode,
    severity: Severity,
    policy: RolloutPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_entity_key: Option<EntityKey>,
    primary_source_handle_id: StableHandleId,
    related_source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    required_capability_ids: Vec<crate::CapabilityId>,
    coverage_status: crate::CoverageStatus,
    message_arguments: Vec<MessageArgument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    root_cause_key: Option<RootCauseKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    caused_by_root_cause_key: Option<RootCauseKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remediation: Option<Remediation>,
}

impl Finding {
    /// Stable context-bound finding ID.
    #[must_use]
    pub const fn finding_id(&self) -> FindingId {
        self.finding_id
    }

    /// Generation-independent semantic fingerprint.
    #[must_use]
    pub const fn fingerprint(&self) -> FindingFingerprint {
        self.fingerprint
    }

    /// Context identity.
    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    /// Primary reported source location.
    #[must_use]
    pub const fn primary_source_handle_id(&self) -> StableHandleId {
        self.primary_source_handle_id
    }

    /// Rule identity.
    #[must_use]
    pub const fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    /// Stable finding code.
    #[must_use]
    pub const fn finding_code(&self) -> &MessageCode {
        &self.finding_code
    }
}

/// Constructs a deterministic causal grouping key from explicit semantic material.
pub fn derive_root_cause_key<T: Serialize + ?Sized>(value: &T) -> CoreResult<RootCauseKey> {
    RootCauseKey::derive(value)
}

/// Sorts findings according to their source location and semantic identity.
pub fn canonical_finding_order(
    findings: &mut [Finding],
    source_handles: &[crate::SourceHandle],
) -> CoreResult<()> {
    let mut source_index = BTreeMap::new();
    for handle in source_handles {
        source_index.insert(handle.handle_id(), source_sort_key(handle));
    }
    if findings
        .iter()
        .any(|finding| !source_index.contains_key(&finding.primary_source_handle_id))
    {
        return Err(validation_error(
            "canonical_finding_order",
            CoreErrorCode::MissingSourceHandle,
            "findings.primary_source_handle_id",
        ));
    }
    findings.sort_by(|left, right| {
        let left_source = source_index.get(&left.primary_source_handle_id);
        let right_source = source_index.get(&right.primary_source_handle_id);
        left_source
            .cmp(&right_source)
            .then_with(|| left.rule_id.cmp(&right.rule_id))
            .then_with(|| left.finding_code.cmp(&right.finding_code))
            .then_with(|| left.fingerprint.cmp(&right.fingerprint))
    });
    Ok(())
}

/// Deduplicates byte-equivalent findings and rejects one ID with differing content.
pub fn deduplicate_findings(mut findings: Vec<Finding>) -> CoreResult<Vec<Finding>> {
    findings.sort_by_key(Finding::finding_id);
    let mut deduplicated: Vec<Finding> = Vec::with_capacity(findings.len());
    for finding in findings {
        if let Some(previous) = deduplicated.last()
            && previous.finding_id == finding.finding_id
        {
            if previous == &finding {
                continue;
            }
            return Err(validation_error(
                "deduplicate_findings",
                CoreErrorCode::ResultDuplicateId,
                "findings",
            ));
        }
        deduplicated.push(finding);
    }
    Ok(deduplicated)
}

/// Immutable structured operation-level warning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarningRecord {
    warning_id: WarningId,
    context_id: GenerationContextId,
    producer_id: ProducerId,
    producer_version: ToolVersion,
    warning_code: MessageCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_source_handle_id: Option<StableHandleId>,
    related_source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    message_arguments: Vec<MessageArgument>,
}

impl WarningRecord {
    /// Constructs, validates, and derives a warning ID.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context_id: GenerationContextId,
        producer_id: ProducerId,
        producer_version: ToolVersion,
        warning_code: MessageCode,
        subject: Option<(String, String)>,
        primary_source_handle_id: Option<StableHandleId>,
        mut related_source_handle_ids: Vec<StableHandleId>,
        mut evidence_ids: Vec<EvidenceId>,
        mut message_arguments: Vec<MessageArgument>,
    ) -> CoreResult<Self> {
        if let Some((kind, id)) = &subject {
            validate_lower_segment(kind, "validate_warning_record", "subject_kind")?;
            validate_bounded_text(id, "validate_warning_record", "subject_id")?;
        }
        related_source_handle_ids.sort_unstable();
        related_source_handle_ids.dedup();
        evidence_ids.sort_unstable();
        evidence_ids.dedup();
        message_arguments.sort();
        validate_message_arguments(&message_arguments)?;

        #[derive(Serialize)]
        struct Projection<'a> {
            context_id: GenerationContextId,
            evidence_ids: &'a [EvidenceId],
            message_arguments: &'a [MessageArgument],
            #[serde(skip_serializing_if = "Option::is_none")]
            primary_source_handle_id: Option<StableHandleId>,
            producer_id: &'a ProducerId,
            producer_version: &'a ToolVersion,
            related_source_handle_ids: &'a [StableHandleId],
            #[serde(skip_serializing_if = "Option::is_none")]
            subject_id: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            subject_kind: Option<&'a str>,
            warning_code: &'a MessageCode,
        }
        let warning_id = WarningId::derive(&Projection {
            context_id,
            evidence_ids: &evidence_ids,
            message_arguments: &message_arguments,
            primary_source_handle_id,
            producer_id: &producer_id,
            producer_version: &producer_version,
            related_source_handle_ids: &related_source_handle_ids,
            subject_id: subject.as_ref().map(|value| value.1.as_str()),
            subject_kind: subject.as_ref().map(|value| value.0.as_str()),
            warning_code: &warning_code,
        })?;
        Ok(Self {
            warning_id,
            context_id,
            producer_id,
            producer_version,
            warning_code,
            subject_kind: subject.as_ref().map(|value| value.0.clone()),
            subject_id: subject.map(|value| value.1),
            primary_source_handle_id,
            related_source_handle_ids,
            evidence_ids,
            message_arguments,
        })
    }

    /// Validates context and registry references.
    pub fn validate(
        &self,
        context_id: GenerationContextId,
        source_handles: &[crate::SourceHandle],
        evidence_records: &[crate::EvidenceRecord],
    ) -> CoreResult<()> {
        if self.context_id != context_id {
            return Err(validation_error(
                "validate_warning_record",
                CoreErrorCode::WarningContextMismatch,
                "context_id",
            ));
        }
        let registry = DiagnosticRegistry::new(
            context_id,
            source_handles,
            evidence_records,
            "validate_warning_record",
        )?;
        self.validate_admitted(&registry)
    }

    pub(crate) fn validate_admitted(&self, registry: &DiagnosticRegistry<'_>) -> CoreResult<()> {
        const OPERATION: &str = "validate_warning_record";
        if self.context_id != registry.context_id() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::WarningContextMismatch,
                "context_id",
            ));
        }
        match (&self.subject_kind, &self.subject_id) {
            (Some(kind), Some(id)) => {
                validate_lower_segment(kind, OPERATION, "subject_kind")?;
                validate_bounded_text(id, OPERATION, "subject_id")?;
            }
            (None, None) => {}
            _ => {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidMessageArgument,
                    "subject",
                ));
            }
        }
        validate_message_arguments(&self.message_arguments)?;
        validate_reference_set(
            &self.related_source_handle_ids,
            OPERATION,
            "related_source_handle_ids",
        )?;
        validate_reference_set(&self.evidence_ids, OPERATION, "evidence_ids")?;
        registry.require_sources(
            self.primary_source_handle_id
                .into_iter()
                .chain(self.related_source_handle_ids.iter().copied()),
            OPERATION,
            "source_handle_ids",
        )?;
        registry.require_evidence(&self.evidence_ids, OPERATION)?;
        crate::integrity::validate_warning(self)
    }

    /// Stable warning ID.
    #[must_use]
    pub const fn warning_id(&self) -> WarningId {
        self.warning_id
    }

    /// Context identity.
    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }
}

fn validate_reference_set<T: Ord>(
    values: &[T],
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(
            validation_error(operation, CoreErrorCode::ResultDuplicateId, field)
                .with_argument("reason", "noncanonical_or_duplicate_reference"),
        );
    }
    Ok(())
}

fn source_sort_key(handle: &crate::SourceHandle) -> (String, String, String, u8, u64, u64) {
    let (span_rank, start, end) = match handle.span().kind() {
        crate::SourceSpanKind::ByteRange => (
            0,
            handle.span().byte_start().unwrap_or_default(),
            handle.span().byte_end().unwrap_or_default(),
        ),
        crate::SourceSpanKind::WholeFile => (1, u64::MAX, u64::MAX),
        crate::SourceSpanKind::Unknown => (2, u64::MAX, u64::MAX),
    };
    (
        handle.origin_id().to_owned(),
        handle.revision().to_owned(),
        handle.path().as_str().to_owned(),
        span_rank,
        start,
        end,
    )
}

fn validate_argument_value(kind: MessageArgumentKind, value: &str) -> CoreResult<()> {
    if value.len() > MAX_ARGUMENT_VALUE_BYTES || value.chars().any(char::is_control) {
        return Err(validation_error(
            "validate_message_arguments",
            CoreErrorCode::InvalidMessageArgument,
            "arguments.value",
        ));
    }
    let valid = match kind {
        MessageArgumentKind::Text => true,
        MessageArgumentKind::Integer => value
            .parse::<u64>()
            .is_ok_and(|number| number <= 9_007_199_254_740_991 && number.to_string() == value),
        MessageArgumentKind::Boolean => matches!(value, "true" | "false"),
        MessageArgumentKind::Identifier => {
            !value.is_empty() && !value.chars().any(char::is_whitespace)
        }
        MessageArgumentKind::Path => {
            crate::NormalizedSourcePath::parse(value).is_ok_and(|path| path.was_canonical())
        }
        MessageArgumentKind::Digest => value.strip_prefix("sha256:").is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        }),
    };
    if valid {
        Ok(())
    } else {
        Err(validation_error(
            "validate_message_arguments",
            CoreErrorCode::InvalidMessageArgument,
            "arguments.value",
        ))
    }
}

fn validate_bounded_text(
    value: &str,
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    if value.is_empty()
        || value.len() > MAX_ARGUMENT_VALUE_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        Err(validation_error(
            operation,
            CoreErrorCode::InvalidMessageArgument,
            field,
        ))
    } else {
        Ok(())
    }
}

impl Finding {
    /// Revalidates a deserialized finding against its exact registries.
    pub fn validate(
        &self,
        context_id: GenerationContextId,
        source_handles: &[crate::SourceHandle],
        evidence_records: &[crate::EvidenceRecord],
    ) -> CoreResult<()> {
        if self.context_id != context_id {
            return Err(validation_error(
                "bind_finding_to_context",
                CoreErrorCode::FindingContextMismatch,
                "context_id",
            ));
        }
        let registry = DiagnosticRegistry::new(
            context_id,
            source_handles,
            evidence_records,
            "bind_finding_to_context",
        )?;
        self.validate_admitted(&registry)
    }

    pub(crate) fn validate_admitted(&self, registry: &DiagnosticRegistry<'_>) -> CoreResult<()> {
        const OPERATION: &str = "bind_finding_to_context";
        if self.context_id != registry.context_id() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::FindingContextMismatch,
                "context_id",
            ));
        }
        validate_message_arguments(&self.message_arguments)?;
        validate_reference_set(
            &self.related_source_handle_ids,
            OPERATION,
            "related_source_handle_ids",
        )?;
        validate_reference_set(&self.evidence_ids, OPERATION, "evidence_ids")?;
        validate_reference_set(
            &self.required_capability_ids,
            OPERATION,
            "required_capability_ids",
        )?;
        registry.require_sources(
            std::iter::once(self.primary_source_handle_id)
                .chain(self.related_source_handle_ids.iter().copied()),
            OPERATION,
            "source_handle_ids",
        )?;
        let has_candidate = registry.require_evidence(&self.evidence_ids, OPERATION)?;
        if let Some(remediation) = &self.remediation {
            remediation.validate()?;
            registry.require_sources(
                remediation.plan_handle_id,
                OPERATION,
                "remediation.plan_handle_id",
            )?;
            if remediation.class() == RemediationClass::ExactEdit
                && (has_candidate || self.evidence_ids.is_empty())
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::RemediationAuthorityViolation,
                    "remediation.class",
                ));
            }
        }
        crate::integrity::validate_finding(self)
    }
}

/// Recomputes a finding fingerprint before context binding.
pub fn derive_finding_fingerprint(draft: &FindingDraft) -> CoreResult<FindingFingerprint> {
    draft.fingerprint()
}

/// Binds a validated finding draft to one exact context and registries.
pub fn bind_finding_to_context(
    draft: FindingDraft,
    context_id: GenerationContextId,
    source_handles: &[crate::SourceHandle],
    evidence_records: &[crate::EvidenceRecord],
) -> CoreResult<Finding> {
    draft.bind(context_id, source_handles, evidence_records)
}

/// Recomputes the identity of one warning record.
pub fn derive_warning_id(record: &WarningRecord) -> CoreResult<WarningId> {
    #[derive(Serialize)]
    struct Projection<'a> {
        context_id: GenerationContextId,
        evidence_ids: &'a [EvidenceId],
        message_arguments: &'a [MessageArgument],
        #[serde(skip_serializing_if = "Option::is_none")]
        primary_source_handle_id: Option<StableHandleId>,
        producer_id: &'a ProducerId,
        producer_version: &'a ToolVersion,
        related_source_handle_ids: &'a [StableHandleId],
        #[serde(skip_serializing_if = "Option::is_none")]
        subject_id: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subject_kind: Option<&'a str>,
        warning_code: &'a MessageCode,
    }
    WarningId::derive(&Projection {
        context_id: record.context_id,
        evidence_ids: &record.evidence_ids,
        message_arguments: &record.message_arguments,
        primary_source_handle_id: record.primary_source_handle_id,
        producer_id: &record.producer_id,
        producer_version: &record.producer_version,
        related_source_handle_ids: &record.related_source_handle_ids,
        subject_id: record.subject_id.as_deref(),
        subject_kind: record.subject_kind.as_deref(),
        warning_code: &record.warning_code,
    })
}

/// Operation wrapper for warning validation.
pub fn validate_warning_record(
    record: &WarningRecord,
    context_id: GenerationContextId,
    source_handles: &[crate::SourceHandle],
    evidence_records: &[crate::EvidenceRecord],
) -> CoreResult<()> {
    record.validate(context_id, source_handles, evidence_records)
}
