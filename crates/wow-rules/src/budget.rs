//! Request-local admission budget for rule evaluation.
//!
//! One continuing budget carries across both providers, so the caller's selected
//! limits bound result materialization rather than only what is finally returned.
//! Every retained evaluation, finding, source handle and evidence record is
//! admitted before it is retained, so a request whose construction would exceed
//! the selected budget fails as one whole operation instead of returning a
//! silently complete result.
//!
//! Borrowed, closed owner plans count serde lengths before constructors, copies
//! and Core identity materialization. Fixed-width digest slots stand for future
//! IDs; they are never used to derive IDs. Only Core canonicalizes and validates.

use std::io::{self, Write};

use crate::{
    RuleBlockerKind, RuleCleanClaimKind, RuleDescriptor, RuleGuardClassification,
    RuleReferenceLookupRecord,
};
use crate::{
    RuleBudgetUsage, RuleError, RuleErrorCode, RuleEvaluationRecord, RuleExecutionBudget,
    RuleExecutionContext, RuleResult,
};
use serde::Serialize;
use wow_core::GenerationContextId;
use wow_core::{
    BlockingPartitionRef, CapabilityId, ClaimScope, ContentDigest, CoverageId, CoverageStatus,
    EntityKey, EvidenceConfidence, MessageArgumentKind, ProjectGenerationId, ProvenanceClass,
    ReferenceGenerationId, RolloutPolicy, SourceContent, SourceOriginKind, SourceSpan,
};

/// Cumulative request-local admissions measured against the caller's selected
/// limits. The budget is created once per request and is never reset between
/// providers, so the second provider cannot spend a fresh independent limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Admission {
    evaluations: u64,
    findings: u64,
    source_handles: u64,
    evidence_records: u64,
    evaluation_bytes: u64,
    encoded_evaluations: u64,
    limits: RuleExecutionBudget,
}

/// Closed owner fields used to preflight report identity and output bytes.
pub(crate) struct ReportMaterial<'a> {
    pub(crate) registry_id: &'a str,
    pub(crate) context_id: GenerationContextId,
    pub(crate) project_snapshot_id: &'a str,
    pub(crate) analyzer_snapshot_id: &'a str,
    pub(crate) reference_generation: &'a str,
    pub(crate) reference_view_digest: &'a str,
    pub(crate) fixture_policy_id: &'a str,
    pub(crate) evaluations: &'a [RuleEvaluationRecord],
    pub(crate) budget_usage: RuleBudgetUsage,
}

#[derive(Serialize)]
struct ReportIdentityView<'a> {
    schema: &'static str,
    registry_id: &'a str,
    context_id: GenerationContextId,
    project_snapshot_id: &'a str,
    analyzer_snapshot_id: &'a str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    fixture_policy_id: &'a str,
    evaluations: &'a [RuleEvaluationRecord],
    budget_usage: RuleBudgetUsage,
}

#[derive(Serialize)]
struct IdentityMaterial<'a, T: Serialize + ?Sized> {
    domain: &'static str,
    value: &'a T,
}

#[derive(Serialize)]
struct ReportOutputView<'a> {
    schema: &'static str,
    report_id: &'a str,
    registry_id: &'a str,
    context_id: GenerationContextId,
    project_snapshot_id: &'a str,
    analyzer_snapshot_id: &'a str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    fixture_policy_id: &'a str,
    evaluations: &'a [RuleEvaluationRecord],
    budget_usage: RuleBudgetUsage,
}

/// Counts the immutable owner length before Core derives the report identity.
/// The placeholder has the exact prefix and SHA-256 digest width of report IDs.
pub(crate) fn preflight_report(material: &ReportMaterial<'_>, max_bytes: u64) -> RuleResult<u64> {
    let identity = ReportIdentityView {
        schema: "wow-rules/execution-report/1",
        registry_id: material.registry_id,
        context_id: material.context_id,
        project_snapshot_id: material.project_snapshot_id,
        analyzer_snapshot_id: material.analyzer_snapshot_id,
        reference_generation: material.reference_generation,
        reference_view_digest: material.reference_view_digest,
        fixture_policy_id: material.fixture_policy_id,
        evaluations: material.evaluations,
        budget_usage: material.budget_usage,
    };
    let identity_material = IdentityMaterial {
        domain: "wow-rules/execution-report/e0-e/1",
        value: &identity,
    };
    encoded_len(&identity_material, max_bytes)?;

    let mut report_id = String::with_capacity("rule-execution:sha256:".len() + 64);
    report_id.push_str("rule-execution:sha256:");
    report_id.extend(std::iter::repeat_n('0', 64));
    let report = ReportOutputView {
        schema: identity.schema,
        report_id: &report_id,
        registry_id: material.registry_id,
        context_id: material.context_id,
        project_snapshot_id: material.project_snapshot_id,
        analyzer_snapshot_id: material.analyzer_snapshot_id,
        reference_generation: material.reference_generation,
        reference_view_digest: material.reference_view_digest,
        fixture_policy_id: material.fixture_policy_id,
        evaluations: material.evaluations,
        budget_usage: material.budget_usage,
    };
    encoded_len(&report, max_bytes)
}

/// Compact serde counting does not canonicalize or validate. Sorting the fixed
/// owner object's keys changes byte order, not its encoded length.
fn encoded_len<T: Serialize + ?Sized>(value: &T, limit: u64) -> RuleResult<u64> {
    struct Counter {
        bytes: u64,
        limit: u64,
        exceeded: bool,
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let length = u64::try_from(bytes.len())
                .map_err(|_| io::Error::other("serialized byte count overflow"))?;
            if length > self.limit.saturating_sub(self.bytes) {
                self.exceeded = true;
                return Err(io::Error::other("serialized output budget exceeded"));
            }
            self.bytes += length;
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut counter = Counter {
        bytes: 0,
        limit,
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut counter, value);
    if counter.exceeded {
        return Err(output_budget_error());
    }
    result.map_err(|_| {
        RuleError::new(
            RuleErrorCode::CanonicalizationFailed,
            "closed rule output owner could not be counted",
        )
    })?;
    Ok(counter.bytes)
}

/// Runs Core's canonical encoder only after preflight proves the complete
/// output fits the caller's byte ceiling.
pub(crate) fn verify_canonical_output<T: Serialize + ?Sized>(
    value: &T,
    expected_bytes: u64,
    max_bytes: u64,
) -> RuleResult<()> {
    let bytes = wow_core::canonical_json_bytes(value).map_err(|_| {
        RuleError::new(
            RuleErrorCode::CanonicalizationFailed,
            "rule report could not be canonically encoded",
        )
    })?;
    let actual_bytes = u64::try_from(bytes.len()).map_err(|_| output_budget_error())?;
    if actual_bytes > max_bytes {
        return Err(output_budget_error());
    }
    if actual_bytes != expected_bytes {
        return Err(RuleError::new(
            RuleErrorCode::CanonicalizationFailed,
            "canonical report length differs from its closed owner preflight",
        ));
    }
    Ok(())
}

impl Admission {
    /// Starts one continuing request admission from the selected limits.
    pub(crate) fn new(context: &RuleExecutionContext<'_>) -> Self {
        Self {
            evaluations: 0,
            findings: 0,
            source_handles: 0,
            evidence_records: 0,
            evaluation_bytes: 0,
            encoded_evaluations: 0,
            limits: context.budget(),
        }
    }

    /// Admits one evaluation record before it is retained.
    pub(crate) fn admit_evaluation(&mut self) -> RuleResult<()> {
        let after = self.evaluations.checked_add(1).ok_or_else(overflow)?;
        if after > self.limits.max_evaluations() {
            return Err(budget_exceeded());
        }
        self.evaluations = after;
        Ok(())
    }

    /// Admits one finding before it is retained.
    pub(crate) fn admit_finding(&mut self) -> RuleResult<()> {
        let after = self.findings.checked_add(1).ok_or_else(overflow)?;
        if after > self.limits.max_findings() {
            return Err(budget_exceeded());
        }
        self.findings = after;
        Ok(())
    }

    /// Admits the source handles a finding will publish before the finding is
    /// retained. `source_handles` is the exact canonical, de-duplicated handle
    /// count the finding publishes, so the admitted usage equals the published
    /// usage for every successful request.
    pub(crate) fn admit_source_handles(&mut self, source_handles: usize) -> RuleResult<()> {
        let after = self
            .source_handles
            .checked_add(usize_to_u64(source_handles)?)
            .ok_or_else(overflow)?;
        if after > self.limits.max_source_handles() {
            return Err(budget_exceeded());
        }
        self.source_handles = after;
        Ok(())
    }

    /// Admits the evidence records a finding will publish before the finding is
    /// retained. `evidence_records` is the exact canonical evidence count the
    /// finding publishes, so the admitted usage equals the published usage for
    /// every successful request.
    pub(crate) fn admit_evidence_records(&mut self, evidence_records: usize) -> RuleResult<()> {
        let after = self
            .evidence_records
            .checked_add(usize_to_u64(evidence_records)?)
            .ok_or_else(overflow)?;
        if after > self.limits.max_evidence_records() {
            return Err(budget_exceeded());
        }
        self.evidence_records = after;
        Ok(())
    }

    /// Reserves the exact future evaluation output, including the report envelope
    /// and the current cumulative counters, before any of its owners are built.
    fn reserve_output<T: Serialize>(
        &mut self,
        context: &RuleExecutionContext<'_>,
        value: &T,
    ) -> RuleResult<()> {
        let limit = self.limits.max_serialized_output_bytes();
        let bytes = encoded_len(value, limit)?;
        let after = self
            .evaluation_bytes
            .checked_add(bytes)
            .and_then(|v| v.checked_add(u64::from(self.encoded_evaluations != 0)))
            .ok_or_else(output_budget_error)?;
        let material = ReportMaterial {
            registry_id: context.registry().registry_id(),
            context_id: context
                .project()
                .snapshot()
                .generation_context()
                .context_id(),
            project_snapshot_id: context.project().snapshot_id(),
            analyzer_snapshot_id: context.project().analyzer_snapshot_id(),
            reference_generation: context.reference().generation_id(),
            reference_view_digest: context.reference().self_digest(),
            fixture_policy_id: context.policy_id(),
            evaluations: &[],
            budget_usage: self.usage(),
        };
        let envelope = preflight_report(&material, limit)?;
        if after > limit.saturating_sub(envelope) {
            return Err(output_budget_error());
        }
        self.evaluation_bytes = after;
        self.encoded_evaluations += 1;
        Ok(())
    }

    pub(crate) fn preflight_identity<T: Serialize + ?Sized>(
        &self,
        domain: &'static str,
        value: &T,
    ) -> RuleResult<()> {
        encoded_len(
            &IdentityMaterial { domain, value },
            self.limits.max_serialized_output_bytes(),
        )?;
        Ok(())
    }

    pub(crate) fn max_bytes(&self) -> u64 {
        self.limits.max_serialized_output_bytes()
    }

    pub(crate) fn preflight_value<T: Serialize + ?Sized>(&self, value: &T) -> RuleResult<()> {
        encoded_len(value, self.max_bytes())?;
        Ok(())
    }

    pub(crate) fn usage(&self) -> RuleBudgetUsage {
        RuleBudgetUsage {
            evaluations: self.evaluations,
            findings: self.findings,
            source_handles: self.source_handles,
            evidence_records: self.evidence_records,
        }
    }

    pub(crate) fn verify_report_plan(
        &self,
        material: &ReportMaterial<'_>,
        actual_bytes: u64,
    ) -> RuleResult<()> {
        let empty = ReportMaterial {
            evaluations: &[],
            ..*material
        };
        let envelope = preflight_report(&empty, self.max_bytes())?;
        if material.budget_usage != self.usage()
            || self.encoded_evaluations != self.evaluations
            || envelope.checked_add(self.evaluation_bytes) != Some(actual_bytes)
        {
            return Err(RuleError::new(
                RuleErrorCode::CanonicalizationFailed,
                "constructed report differs from its admitted owner plans",
            ));
        }
        Ok(())
    }
}

// These slots have exactly the owner's wire prefix and 64 hexadecimal digits.
// Order and digest contents cannot affect compact serde length.
const HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
struct IdSlot(&'static str);
impl Serialize for IdSlot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format_args!("{}{}", self.0, HEX))
    }
}
struct IdSlots {
    prefix: &'static str,
    count: usize,
}
impl Serialize for IdSlots {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.count))?;
        for _ in 0..self.count {
            seq.serialize_element(&IdSlot(self.prefix))?;
        }
        seq.end()
    }
}

/// Complete prospective SourceHandle identity. Equality is checked on all inputs
/// before construction, so duplicate XML pieces/local uses spend one handle.
#[derive(Serialize, PartialEq, Eq)]
pub(crate) struct SourcePlan<'a> {
    pub(crate) origin_kind: SourceOriginKind,
    pub(crate) origin_id: &'a str,
    pub(crate) revision: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reference_generation: Option<ReferenceGenerationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) project_generation: Option<ProjectGenerationId>,
    pub(crate) path: &'a str,
    pub(crate) span: SourceSpan,
    pub(crate) content_digest: ContentDigest<SourceContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) entity_key: Option<&'a EntityKey>,
}

#[derive(Serialize)]
struct SourceOutput<'a> {
    handle_id: IdSlot,
    #[serde(flatten)]
    owner: &'a SourcePlan<'a>,
}
struct SourceOutputs<'a>(&'a [SourcePlan<'a>]);
impl Serialize for SourceOutputs<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for plan in self.0 {
            seq.serialize_element(&SourceOutput {
                handle_id: IdSlot("handle:sha256:"),
                owner: plan,
            })?;
        }
        seq.end()
    }
}

pub(crate) fn admit_source_plan<'a>(
    admission: &mut Admission,
    plans: &mut Vec<SourcePlan<'a>>,
    plan: SourcePlan<'a>,
) -> RuleResult<()> {
    if !plans.contains(&plan) {
        admission.admit_source_handles(1)?;
        admission.preflight_identity("wow-core/source-handle/e0-1", &plan)?;
        plans.push(plan);
    }
    Ok(())
}

#[derive(Serialize)]
pub(crate) struct ArgumentPlan<'a> {
    pub(crate) name: &'a str,
    pub(crate) kind: MessageArgumentKind,
    pub(crate) value: &'a str,
    identity_relevant: bool,
}
impl<'a> ArgumentPlan<'a> {
    pub(crate) const fn new(name: &'a str, kind: MessageArgumentKind, value: &'a str) -> Self {
        Self {
            name,
            kind,
            value,
            identity_relevant: true,
        }
    }
}

#[derive(Serialize)]
struct EvidenceOwner {
    context_id: GenerationContextId,
    provenance: ProvenanceClass,
    confidence: EvidenceConfidence,
    claim_scope: ClaimScope,
    producer_id: &'static str,
    producer_version: &'static str,
    source_handle_ids: IdSlots,
    coverage_refs: [(); 0],
    derivation_input_ids: IdSlots,
}
#[derive(Serialize)]
struct EvidenceOutput {
    evidence_id: IdSlot,
    #[serde(flatten)]
    owner: EvidenceOwner,
}

#[derive(Serialize)]
struct RemediationPlan {
    class: &'static str,
}
#[derive(Serialize)]
struct FindingOutput<'a> {
    finding_id: IdSlot,
    fingerprint: IdSlot,
    context_id: GenerationContextId,
    rule_id: &'a wow_core::RuleId,
    rule_version: &'a wow_core::ToolVersion,
    finding_code: &'a wow_core::MessageCode,
    severity: wow_core::Severity,
    policy: RolloutPolicy,
    subject_entity_key: &'a EntityKey,
    primary_source_handle_id: IdSlot,
    related_source_handle_ids: IdSlots,
    evidence_ids: IdSlots,
    required_capability_ids: &'a [CapabilityId],
    coverage_status: CoverageStatus,
    message_arguments: &'a [ArgumentPlan<'a>],
    root_cause_key: IdSlot,
    remediation: RemediationPlan,
}

pub(crate) struct FindingMaterial<'a> {
    pub(crate) scope_id: &'a str,
    pub(crate) descriptor: &'a RuleDescriptor,
    pub(crate) subject: &'a EntityKey,
    pub(crate) sources: &'a [SourcePlan<'a>],
    pub(crate) arguments: &'a [ArgumentPlan<'a>],
    pub(crate) fact_ids: &'a [Box<str>],
    pub(crate) coverage_ids: &'a [CoverageId],
    pub(crate) lookup: &'a RuleReferenceLookupRecord,
    pub(crate) guard: Option<RuleGuardClassification>,
}

#[derive(Serialize)]
struct FindingSetOutput<'a> {
    evaluation_id: IdSlot,
    fixture_policy_id: &'a str,
    input_fact_ids: &'a [Box<str>],
    findings: [FindingOutput<'a>; 1],
    source_handles: SourceOutputs<'a>,
    evidence_records: [EvidenceOutput; 3],
    coverage_ids: &'a [CoverageId],
    reference_lookup: &'a RuleReferenceLookupRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    guard_classification: Option<RuleGuardClassification>,
    budget_usage: RuleBudgetUsage,
}
#[derive(Serialize)]
struct EvaluationOutput<'a, T: Serialize> {
    rule_id: &'a wow_core::RuleId,
    rule_version: &'a wow_core::ToolVersion,
    scope_id: &'a str,
    outcome: T,
}
#[derive(Serialize)]
struct FindingsOutcome<'a> {
    status: &'static str,
    result: &'a FindingSetOutput<'a>,
}

pub(crate) fn preflight_finding(
    admission: &mut Admission,
    context: &RuleExecutionContext<'_>,
    material: &FindingMaterial<'_>,
) -> RuleResult<()> {
    admission.admit_evidence_records(3)?;
    let context_id = context
        .project()
        .snapshot()
        .generation_context()
        .context_id();
    let evidence_owner = |provenance, confidence, claim_scope, sources, inputs| EvidenceOwner {
        context_id,
        provenance,
        confidence,
        claim_scope,
        producer_id: "wow.rules",
        producer_version: "1.0.0",
        source_handle_ids: IdSlots {
            prefix: "handle:sha256:",
            count: sources,
        },
        coverage_refs: [],
        derivation_input_ids: IdSlots {
            prefix: "evidence:sha256:",
            count: inputs,
        },
    };
    let evidence = [
        evidence_owner(
            ProvenanceClass::ProjectSource,
            EvidenceConfidence::Proven,
            ClaimScope::ProjectFact,
            material.sources.len() - 1,
            0,
        ),
        evidence_owner(
            ProvenanceClass::PlatformSource,
            EvidenceConfidence::Proven,
            ClaimScope::PlatformContract,
            1,
            0,
        ),
        evidence_owner(
            ProvenanceClass::ExternalImplementation,
            EvidenceConfidence::Derived,
            ClaimScope::ProjectFact,
            0,
            2,
        ),
    ];
    for owner in &evidence {
        admission.preflight_identity("wow-core/evidence/e0-1", owner)?;
    }
    let finding = FindingOutput {
        finding_id: IdSlot("finding:sha256:"),
        fingerprint: IdSlot("finding-fingerprint:sha256:"),
        context_id,
        rule_id: material.descriptor.rule_id(),
        rule_version: material.descriptor.rule_version(),
        finding_code: material.descriptor.semantic_category(),
        severity: material.descriptor.technical_severity(),
        policy: material.descriptor.rollout_policy(),
        subject_entity_key: material.subject,
        primary_source_handle_id: IdSlot("handle:sha256:"),
        related_source_handle_ids: IdSlots {
            prefix: "handle:sha256:",
            count: material.sources.len() - 1,
        },
        evidence_ids: IdSlots {
            prefix: "evidence:sha256:",
            count: 3,
        },
        required_capability_ids: material.descriptor.required_capabilities(),
        coverage_status: CoverageStatus::Complete,
        message_arguments: material.arguments,
        root_cause_key: IdSlot("root-cause:sha256:"),
        remediation: RemediationPlan { class: "plan_only" },
    };
    // This closed whole-finding view is an upper bound on both Core identity
    // projections (fingerprint and context binding), including their envelopes.
    admission.preflight_identity("wow-core/finding-fingerprint/e0-1", &finding)?;
    let usage = RuleBudgetUsage {
        evaluations: 1,
        findings: 1,
        source_handles: usize_to_u64(material.sources.len())?,
        evidence_records: 3,
    };
    let [project, reference, derivation] = evidence;
    let result = FindingSetOutput {
        evaluation_id: IdSlot("rule-evaluation:sha256:"),
        fixture_policy_id: context.policy_id(),
        input_fact_ids: material.fact_ids,
        findings: [finding],
        source_handles: SourceOutputs(material.sources),
        evidence_records: [
            EvidenceOutput {
                evidence_id: IdSlot("evidence:sha256:"),
                owner: project,
            },
            EvidenceOutput {
                evidence_id: IdSlot("evidence:sha256:"),
                owner: reference,
            },
            EvidenceOutput {
                evidence_id: IdSlot("evidence:sha256:"),
                owner: derivation,
            },
        ],
        coverage_ids: material.coverage_ids,
        reference_lookup: material.lookup,
        guard_classification: material.guard,
        budget_usage: usage,
    };
    // The set's identity is a strict subset of these owner fields and IDs.
    admission.preflight_identity("wow-rules/findings/e0-e/1", &result)?;
    admission.reserve_output(
        context,
        &EvaluationOutput {
            rule_id: material.descriptor.rule_id(),
            rule_version: material.descriptor.rule_version(),
            scope_id: material.scope_id,
            outcome: FindingsOutcome {
                status: "findings",
                result: &result,
            },
        },
    )
}

#[derive(Serialize)]
struct CleanOwner<'a> {
    rule_id: &'a wow_core::RuleId,
    rule_version: &'a wow_core::ToolVersion,
    context_id: GenerationContextId,
    fixture_policy_id: &'a str,
    scope_id: &'a str,
    clean_claim_kind: RuleCleanClaimKind,
    input_fact_ids: &'a [Box<str>],
    reference_lookup_id: &'a str,
    capability_ids: &'a [CapabilityId],
    coverage_ids: &'a [CoverageId],
    #[serde(skip_serializing_if = "Option::is_none")]
    guard_classification: Option<RuleGuardClassification>,
    budget_usage: RuleBudgetUsage,
}
#[derive(Serialize)]
struct CleanOutput<'a> {
    evaluation_id: IdSlot,
    #[serde(flatten)]
    owner: CleanOwner<'a>,
}
#[derive(Serialize)]
struct CleanOutcome<'a> {
    status: &'static str,
    record: CleanOutput<'a>,
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn preflight_clean(
    admission: &mut Admission,
    context: &RuleExecutionContext<'_>,
    descriptor: &RuleDescriptor,
    scope_id: &str,
    claim: RuleCleanClaimKind,
    fact_ids: &[Box<str>],
    lookup: &RuleReferenceLookupRecord,
    coverage_ids: &[CoverageId],
    guard: Option<RuleGuardClassification>,
) -> RuleResult<()> {
    let owner = CleanOwner {
        rule_id: descriptor.rule_id(),
        rule_version: descriptor.rule_version(),
        context_id: context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        fixture_policy_id: context.policy_id(),
        scope_id,
        clean_claim_kind: claim,
        input_fact_ids: fact_ids,
        reference_lookup_id: lookup.lookup_id(),
        capability_ids: descriptor.required_capabilities(),
        coverage_ids,
        guard_classification: guard,
        budget_usage: RuleBudgetUsage {
            evaluations: 1,
            ..RuleBudgetUsage::default()
        },
    };
    admission.preflight_identity("wow-rules/clean-evaluation/e0-e/1", &owner)?;
    admission.reserve_output(
        context,
        &EvaluationOutput {
            rule_id: descriptor.rule_id(),
            rule_version: descriptor.rule_version(),
            scope_id,
            outcome: CleanOutcome {
                status: "evaluated_clean",
                record: CleanOutput {
                    evaluation_id: IdSlot("rule-evaluation:sha256:"),
                    owner,
                },
            },
        },
    )
}

#[derive(Serialize)]
struct NotEvaluatedOwner<'a> {
    context_id: GenerationContextId,
    producer_id: &'static str,
    producer_version: &'static str,
    subject_kind: &'static str,
    subject_id: &'a str,
    reason_code: &'static str,
    blocking_capability_ids: &'a [CapabilityId],
    blocking_partitions: &'a [BlockingPartitionRef],
    conflict_ids: [(); 0],
}
#[derive(Serialize)]
struct NotEvaluatedOutput<'a> {
    not_evaluated_id: IdSlot,
    #[serde(flatten)]
    owner: NotEvaluatedOwner<'a>,
}
#[derive(Serialize)]
struct DetailOutput<'a> {
    evaluation_id: IdSlot,
    rule_id: &'a wow_core::RuleId,
    rule_version: &'a wow_core::ToolVersion,
    context_id: GenerationContextId,
    scope_id: &'a str,
    blockers: &'a [RuleBlockerKind],
    core_record: NotEvaluatedOutput<'a>,
    input_fact_ids: &'a [Box<str>],
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_lookup_id: Option<&'a str>,
}
#[derive(Serialize)]
struct NotEvaluatedOutcome<'a> {
    status: &'static str,
    detail: DetailOutput<'a>,
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn preflight_not_evaluated(
    admission: &mut Admission,
    context: &RuleExecutionContext<'_>,
    descriptor: &RuleDescriptor,
    scope_id: &str,
    blockers: &[RuleBlockerKind],
    capabilities: &[CapabilityId],
    partitions: &[BlockingPartitionRef],
    fact_ids: &[Box<str>],
    lookup: Option<&RuleReferenceLookupRecord>,
) -> RuleResult<()> {
    let context_id = context
        .project()
        .snapshot()
        .generation_context()
        .context_id();
    let owner = NotEvaluatedOwner {
        context_id,
        producer_id: "wow.rules",
        producer_version: "1.0.0",
        subject_kind: "rule",
        subject_id: scope_id,
        reason_code: "rule_required_capability_unavailable",
        blocking_capability_ids: capabilities,
        blocking_partitions: partitions,
        conflict_ids: [],
    };
    admission.preflight_identity("wow-core/not-evaluated/e0-1", &owner)?;
    let detail = DetailOutput {
        evaluation_id: IdSlot("rule-evaluation:sha256:"),
        rule_id: descriptor.rule_id(),
        rule_version: descriptor.rule_version(),
        context_id,
        scope_id,
        blockers,
        core_record: NotEvaluatedOutput {
            not_evaluated_id: IdSlot("not-evaluated:sha256:"),
            owner,
        },
        input_fact_ids: fact_ids,
        reference_lookup_id: lookup.map(RuleReferenceLookupRecord::lookup_id),
    };
    admission.preflight_identity("wow-rules/not-evaluated/e0-e/1", &detail)?;
    admission.reserve_output(
        context,
        &EvaluationOutput {
            rule_id: descriptor.rule_id(),
            rule_version: descriptor.rule_version(),
            scope_id,
            outcome: NotEvaluatedOutcome {
                status: "not_evaluated",
                detail,
            },
        },
    )
}

#[derive(Serialize)]
struct CancelledOutcome {
    status: &'static str,
}
pub(crate) fn preflight_cancelled(
    admission: &mut Admission,
    context: &RuleExecutionContext<'_>,
    rule: &wow_core::RuleId,
    version: &wow_core::ToolVersion,
    scope_id: &str,
) -> RuleResult<()> {
    admission.reserve_output(
        context,
        &EvaluationOutput {
            rule_id: rule,
            rule_version: version,
            scope_id,
            outcome: CancelledOutcome {
                status: "cancelled",
            },
        },
    )
}

#[derive(Serialize)]
struct LookupOwner<'a> {
    partition_id: &'a str,
    entity_key: &'a str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    outcome: crate::RuleReferenceOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    record_digest: Option<IdSlot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conflict_digest: Option<IdSlot>,
}
pub(crate) fn preflight_lookup(
    admission: &Admission,
    context: &RuleExecutionContext<'_>,
    partition: &str,
    key: &str,
    outcome: crate::RuleReferenceOutcome,
) -> RuleResult<()> {
    admission.preflight_identity(
        "wow-rules/reference-lookup/e0-e/1",
        &LookupOwner {
            partition_id: partition,
            entity_key: key,
            reference_generation: context.reference().generation_id(),
            reference_view_digest: context.reference().self_digest(),
            outcome,
            record_digest: (outcome == crate::RuleReferenceOutcome::Found)
                .then_some(IdSlot("sha256:")),
            conflict_digest: (outcome == crate::RuleReferenceOutcome::Conflict)
                .then_some(IdSlot("reference-conflict:sha256:")),
        },
    )
}

fn budget_exceeded() -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleExecutionBudgetExceeded,
        "rule execution exceeded its deterministic budget",
    )
}

fn overflow() -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleExecutionBudgetExceeded,
        "rule execution accounting overflowed its budget",
    )
}

fn usize_to_u64(value: usize) -> RuleResult<u64> {
    u64::try_from(value).map_err(|_| overflow())
}

fn output_budget_error() -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleOutputBudgetExceeded,
        "serialized rule report exceeds the configured output budget",
    )
}
