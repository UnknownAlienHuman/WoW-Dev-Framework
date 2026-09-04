use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::budget::{Budget, BudgetLimits, BudgetUsage, TruncationState};
use crate::canonical::{CANONICALIZATION_VERSION, canonical_json_bytes};
use crate::digest::{CanonicalResult, ContentDigest, GenerationContextId};
use crate::error::{
    CoreError, CoreErrorCode, CoreResult, ErrorCategory, RetryClass, validation_error,
};
use crate::finding::{Finding, WarningRecord, canonical_finding_order, deduplicate_findings};
use crate::ids::{OperationId, SchemaId, ToolVersion};
use crate::profile::SchemaVersionEntry;

const FINALIZATION_PASSES: usize = 6;

/// Canonical E0 check-result state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    Complete,
    Partial,
    Failed,
}

/// Schema compatibility result for an exact-major E0 schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaCompatibility {
    ExactSupported,
    CompatibleSupported,
}

/// Validated, byte-stable E0 check result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct E0CheckResultEnvelope {
    schema: SchemaVersionEntry,
    canonicalization_version: String,
    operation_id: OperationId,
    context: crate::GenerationContext,
    status: ResultStatus,
    coverage_records: Vec<crate::CoverageRecord>,
    capability_summaries: Vec<crate::CapabilitySummary>,
    source_handles: Vec<crate::SourceHandle>,
    evidence_records: Vec<crate::EvidenceRecord>,
    conflicts: Vec<crate::ConflictRecord>,
    findings: Vec<Finding>,
    not_evaluated: Vec<crate::NotEvaluatedRecord>,
    warnings: Vec<WarningRecord>,
    budget: Budget,
    canonical_digest: ContentDigest<CanonicalResult>,
}

impl E0CheckResultEnvelope {
    /// Validates identities, references, ordering, status, budgets, and digest.
    pub fn validate(&self) -> CoreResult<()> {
        validate_schema_version(
            self.schema.schema_id(),
            self.schema.version(),
            self.schema.schema_id(),
            self.schema.version(),
        )?;
        if self.canonicalization_version != CANONICALIZATION_VERSION {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::SchemaVersionUnsupported,
                "canonicalization_version",
            ));
        }
        if self.operation_id.as_str() != "wow.check" {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::ContractViolation,
                "operation_id",
            ));
        }
        self.context.validate()?;
        let context_id = self.context.context_id();
        validate_context_bound_collection(&self.coverage_records, "coverage_records", context_id)?;
        validate_context_bound_collection(
            &self.capability_summaries,
            "capability_summaries",
            context_id,
        )?;
        validate_source_handle_contexts(&self.source_handles, &self.context)?;
        validate_records(self)?;
        validate_context_bound_collection(&self.evidence_records, "evidence_records", context_id)?;
        validate_context_bound_collection(&self.conflicts, "conflicts", context_id)?;
        validate_context_bound_collection(&self.findings, "findings", context_id)?;
        validate_context_bound_collection(&self.not_evaluated, "not_evaluated", context_id)?;
        validate_context_bound_collection(&self.warnings, "warnings", context_id)?;

        validate_unique_ids(&self.coverage_records, "coverage_id", "coverage_records")?;
        validate_unique_ids(&self.source_handles, "handle_id", "source_handles")?;
        validate_unique_ids(&self.evidence_records, "evidence_id", "evidence_records")?;
        validate_unique_ids(&self.conflicts, "conflict_id", "conflicts")?;
        validate_unique_ids(&self.findings, "finding_id", "findings")?;
        validate_unique_ids(&self.not_evaluated, "not_evaluated_id", "not_evaluated")?;
        validate_unique_ids(&self.warnings, "warning_id", "warnings")?;
        validate_unique_summary_keys(&self.capability_summaries)?;
        validate_reference_closure(self)?;
        validate_status(self)?;
        validate_budget_counts(self)?;
        self.budget.validate_limits()?;

        let expected_digest = canonical_result_digest(self)?;
        if expected_digest != self.canonical_digest {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::CanonicalDigestMismatch,
                "canonical_digest",
            ));
        }
        let bytes = canonical_json_bytes(self)?;
        let byte_count = u64::try_from(bytes.len()).map_err(|_| usage_overflow("output_bytes"))?;
        if self.budget.usage().output_bytes != byte_count {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::BudgetInvalid,
                "budget.usage.output_bytes",
            ));
        }
        Ok(())
    }

    /// Returns exact final canonical JSON bytes after validation.
    pub fn canonical_bytes(&self) -> CoreResult<Vec<u8>> {
        self.validate()?;
        canonical_json_bytes(self)
    }

    /// Exact canonical result digest.
    #[must_use]
    pub const fn canonical_digest(&self) -> &ContentDigest<CanonicalResult> {
        &self.canonical_digest
    }

    /// Exact generation context.
    #[must_use]
    pub const fn context(&self) -> &crate::GenerationContext {
        &self.context
    }

    /// Result status.
    #[must_use]
    pub const fn status(&self) -> ResultStatus {
        self.status
    }
}

/// Mutable assembly input that cannot claim a digest until finalization succeeds.
#[derive(Debug, Clone)]
pub struct E0CheckResultDraft {
    schema: SchemaVersionEntry,
    operation_id: OperationId,
    context: crate::GenerationContext,
    status: ResultStatus,
    coverage_records: Vec<crate::CoverageRecord>,
    capability_summaries: Vec<crate::CapabilitySummary>,
    source_handles: Vec<crate::SourceHandle>,
    evidence_records: Vec<crate::EvidenceRecord>,
    conflicts: Vec<crate::ConflictRecord>,
    findings: Vec<Finding>,
    not_evaluated: Vec<crate::NotEvaluatedRecord>,
    warnings: Vec<WarningRecord>,
    budget_limits: BudgetLimits,
    truncation: TruncationState,
}

impl E0CheckResultDraft {
    /// Creates a draft from complete typed result collections.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        schema: SchemaVersionEntry,
        operation_id: OperationId,
        context: crate::GenerationContext,
        status: ResultStatus,
        coverage_records: Vec<crate::CoverageRecord>,
        capability_summaries: Vec<crate::CapabilitySummary>,
        source_handles: Vec<crate::SourceHandle>,
        evidence_records: Vec<crate::EvidenceRecord>,
        conflicts: Vec<crate::ConflictRecord>,
        findings: Vec<Finding>,
        not_evaluated: Vec<crate::NotEvaluatedRecord>,
        warnings: Vec<WarningRecord>,
        budget_limits: BudgetLimits,
        truncation: TruncationState,
    ) -> Self {
        Self {
            schema,
            operation_id,
            context,
            status,
            coverage_records,
            capability_summaries,
            source_handles,
            evidence_records,
            conflicts,
            findings,
            not_evaluated,
            warnings,
            budget_limits,
            truncation,
        }
    }

    /// Canonically orders, validates, hashes, and fixes final output-byte usage.
    pub fn finalize(mut self) -> CoreResult<E0CheckResultEnvelope> {
        self.context.validate()?;
        self.coverage_records
            .sort_by_cached_key(coverage_sort_key_lossless);
        self.capability_summaries
            .sort_by_cached_key(summary_sort_key_lossless);
        self.source_handles
            .sort_by_key(crate::SourceHandle::handle_id);
        self.evidence_records
            .sort_by_cached_key(|record| record_id(record, "evidence_id").unwrap_or_default());
        self.conflicts
            .sort_by_cached_key(|record| record_id(record, "conflict_id").unwrap_or_default());
        canonical_finding_order(&mut self.findings, &self.source_handles)?;
        self.findings = deduplicate_findings(self.findings)?;
        self.not_evaluated
            .sort_by_cached_key(|record| record_id(record, "not_evaluated_id").unwrap_or_default());
        self.warnings
            .sort_by_cached_key(|record| record_id(record, "warning_id").unwrap_or_default());

        let usage = collection_usage(
            &self.coverage_records,
            &self.capability_summaries,
            &self.source_handles,
            &self.evidence_records,
            &self.conflicts,
            &self.findings,
            &self.not_evaluated,
            &self.warnings,
            0,
        )?;
        let mut budget = Budget::new(self.budget_limits, usage, self.truncation)?;
        let zero_digest = ContentDigest::<CanonicalResult>::from_bytes([0; 32]);
        let mut envelope = E0CheckResultEnvelope {
            schema: self.schema,
            canonicalization_version: CANONICALIZATION_VERSION.to_owned(),
            operation_id: self.operation_id,
            context: self.context,
            status: self.status,
            coverage_records: self.coverage_records,
            capability_summaries: self.capability_summaries,
            source_handles: self.source_handles,
            evidence_records: self.evidence_records,
            conflicts: self.conflicts,
            findings: self.findings,
            not_evaluated: self.not_evaluated,
            warnings: self.warnings,
            budget: budget.clone(),
            canonical_digest: zero_digest,
        };

        for _ in 0..FINALIZATION_PASSES {
            envelope.budget = budget.clone();
            envelope.canonical_digest = canonical_result_digest(&envelope)?;
            let bytes = canonical_json_bytes(&envelope)?;
            let output_bytes =
                u64::try_from(bytes.len()).map_err(|_| usage_overflow("output_bytes"))?;
            if budget.usage().output_bytes == output_bytes {
                envelope.validate()?;
                return Ok(envelope);
            }
            budget = budget.with_output_bytes(output_bytes)?;
        }
        Err(CoreError::new(
            CoreErrorCode::CanonicalizationFailure,
            ErrorCategory::Invariant,
            "finalize_result_envelope",
            RetryClass::AfterInputChange,
        )
        .with_argument("reason", "output_bytes_fixed_point_did_not_converge"))
    }
}

/// Strict canonical operation-error envelope. It cannot contain analysis records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct E0OperationErrorEnvelope {
    schema: SchemaVersionEntry,
    canonicalization_version: String,
    operation_id: OperationId,
    error: CoreError,
    canonical_digest: ContentDigest<CanonicalResult>,
}

impl E0OperationErrorEnvelope {
    /// Constructs and hashes an operation error envelope.
    pub fn finalize(
        schema: SchemaVersionEntry,
        operation_id: OperationId,
        error: CoreError,
    ) -> CoreResult<Self> {
        error.validate()?;
        let zero = ContentDigest::<CanonicalResult>::from_bytes([0; 32]);
        let mut envelope = Self {
            schema,
            canonicalization_version: CANONICALIZATION_VERSION.to_owned(),
            operation_id,
            error,
            canonical_digest: zero,
        };
        envelope.canonical_digest = error_envelope_digest(&envelope)?;
        Ok(envelope)
    }

    /// Verifies schema, canonicalization profile, and digest.
    pub fn validate(&self) -> CoreResult<()> {
        if self.canonicalization_version != CANONICALIZATION_VERSION {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::SchemaVersionUnsupported,
                "canonicalization_version",
            ));
        }
        self.error.validate()?;
        if error_envelope_digest(self)? != self.canonical_digest {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::CanonicalDigestMismatch,
                "canonical_digest",
            ));
        }
        Ok(())
    }

    /// Canonical JSON bytes.
    pub fn canonical_bytes(&self) -> CoreResult<Vec<u8>> {
        self.validate()?;
        canonical_json_bytes(self)
    }
}

/// Validates exact-major schema compatibility.
pub fn validate_schema_version(
    encountered_id: &SchemaId,
    encountered_version: &ToolVersion,
    supported_id: &SchemaId,
    supported_version: &ToolVersion,
) -> CoreResult<SchemaCompatibility> {
    if encountered_id != supported_id
        || encountered_version.version().major != supported_version.version().major
        || encountered_version.version() > supported_version.version()
    {
        return Err(validation_error(
            "validate_schema_version",
            CoreErrorCode::SchemaVersionUnsupported,
            "schema",
        ));
    }
    if encountered_version == supported_version {
        Ok(SchemaCompatibility::ExactSupported)
    } else {
        Ok(SchemaCompatibility::CompatibleSupported)
    }
}

/// Recomputes the canonical result digest over the envelope without its digest field.
pub fn canonical_result_digest(
    envelope: &E0CheckResultEnvelope,
) -> CoreResult<ContentDigest<CanonicalResult>> {
    #[derive(Serialize)]
    struct Projection<'a> {
        budget: &'a Budget,
        canonicalization_version: &'a str,
        capability_summaries: &'a [crate::CapabilitySummary],
        conflicts: &'a [crate::ConflictRecord],
        context: &'a crate::GenerationContext,
        coverage_records: &'a [crate::CoverageRecord],
        evidence_records: &'a [crate::EvidenceRecord],
        findings: &'a [Finding],
        not_evaluated: &'a [crate::NotEvaluatedRecord],
        operation_id: &'a OperationId,
        schema: &'a SchemaVersionEntry,
        source_handles: &'a [crate::SourceHandle],
        status: ResultStatus,
        warnings: &'a [WarningRecord],
    }
    let projection = Projection {
        budget: &envelope.budget,
        canonicalization_version: &envelope.canonicalization_version,
        capability_summaries: &envelope.capability_summaries,
        conflicts: &envelope.conflicts,
        context: &envelope.context,
        coverage_records: &envelope.coverage_records,
        evidence_records: &envelope.evidence_records,
        findings: &envelope.findings,
        not_evaluated: &envelope.not_evaluated,
        operation_id: &envelope.operation_id,
        schema: &envelope.schema,
        source_handles: &envelope.source_handles,
        status: envelope.status,
        warnings: &envelope.warnings,
    };
    let digest = crate::domain_separated_digest("wow-core/result/e0-1", &projection)?;
    Ok(ContentDigest::from_bytes(digest))
}

fn error_envelope_digest(
    envelope: &E0OperationErrorEnvelope,
) -> CoreResult<ContentDigest<CanonicalResult>> {
    #[derive(Serialize)]
    struct Projection<'a> {
        canonicalization_version: &'a str,
        error: &'a CoreError,
        operation_id: &'a OperationId,
        schema: &'a SchemaVersionEntry,
    }
    let digest = crate::domain_separated_digest(
        "wow-core/result/e0-1",
        &Projection {
            canonicalization_version: &envelope.canonicalization_version,
            error: &envelope.error,
            operation_id: &envelope.operation_id,
            schema: &envelope.schema,
        },
    )?;
    Ok(ContentDigest::from_bytes(digest))
}

fn validate_context_bound_collection<T: Serialize>(
    records: &[T],
    collection: &'static str,
    context_id: GenerationContextId,
) -> CoreResult<()> {
    let expected = context_id.to_string();
    for record in records {
        let value = record_value(record, collection)?;
        let actual = value
            .get("context_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                validation_error(
                    "validate_result_envelope",
                    CoreErrorCode::ResultContextViolation,
                    collection,
                )
            })?;
        if actual != expected {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::ResultContextViolation,
                collection,
            ));
        }
    }
    Ok(())
}

fn validate_source_handle_contexts(
    handles: &[crate::SourceHandle],
    context: &crate::GenerationContext,
) -> CoreResult<()> {
    let reference = context.reference_generation().to_string();
    let project = context.project_generation().map(|id| id.to_string());
    for handle in handles {
        let value = record_value(handle, "source_handles")?;
        if let Some(actual) = value.get("reference_generation").and_then(Value::as_str)
            && actual != reference
        {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::ResultContextViolation,
                "source_handles.reference_generation",
            ));
        }
        if let Some(actual) = value.get("project_generation").and_then(Value::as_str)
            && project.as_deref() != Some(actual)
        {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::ResultContextViolation,
                "source_handles.project_generation",
            ));
        }
    }
    Ok(())
}

fn validate_unique_ids<T: Serialize>(
    records: &[T],
    id_field: &'static str,
    collection: &'static str,
) -> CoreResult<()> {
    let mut ids = BTreeSet::new();
    for record in records {
        let id = record_id(record, id_field)?;
        if !ids.insert(id) {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::ResultDuplicateId,
                collection,
            ));
        }
    }
    Ok(())
}

fn validate_unique_summary_keys(records: &[crate::CapabilitySummary]) -> CoreResult<()> {
    let mut keys = BTreeSet::new();
    for record in records {
        let value = record_value(record, "capability_summaries")?;
        let key = (
            required_string(&value, "capability_id", "capability_summaries")?,
            required_string(&value, "producer_id", "capability_summaries")?,
            required_string(&value, "producer_version", "capability_summaries")?,
        );
        if !keys.insert(key) {
            return Err(validation_error(
                "validate_result_envelope",
                CoreErrorCode::CoverageConflict,
                "capability_summaries",
            ));
        }
    }
    Ok(())
}

fn validate_reference_closure(envelope: &E0CheckResultEnvelope) -> CoreResult<()> {
    let handles = id_set(&envelope.source_handles, "handle_id", "source_handles")?;
    let evidence = id_set(
        &envelope.evidence_records,
        "evidence_id",
        "evidence_records",
    )?;
    let conflicts = id_set(&envelope.conflicts, "conflict_id", "conflicts")?;
    let coverage = id_set(
        &envelope.coverage_records,
        "coverage_id",
        "coverage_records",
    )?;

    for record in &envelope.evidence_records {
        validate_string_array_refs(record, "source_handle_ids", &handles, "evidence_records")?;
        validate_string_array_refs(
            record,
            "derivation_input_ids",
            &evidence,
            "evidence_records",
        )?;
    }
    for record in &envelope.conflicts {
        validate_string_array_refs(record, "evidence_ids", &evidence, "conflicts")?;
    }
    for record in &envelope.coverage_records {
        validate_string_array_refs(record, "conflict_ids", &conflicts, "coverage_records")?;
    }
    for record in &envelope.capability_summaries {
        validate_string_array_refs(record, "conflict_ids", &conflicts, "capability_summaries")?;
        let value = record_value(record, "capability_summaries")?;
        if let Some(Value::Array(partitions)) = value.get("partition_refs") {
            for partition in partitions {
                let id = partition
                    .get("coverage_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| reference_error("capability_summaries.partition_refs"))?;
                if !coverage.contains(id) {
                    return Err(reference_error("capability_summaries.partition_refs"));
                }
            }
        }
    }
    for record in &envelope.not_evaluated {
        validate_string_array_refs(record, "conflict_ids", &conflicts, "not_evaluated")?;
        let value = record_value(record, "not_evaluated")?;
        if let Some(Value::Array(partitions)) = value.get("blocking_partitions") {
            for partition in partitions {
                let id = partition
                    .get("coverage_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| reference_error("not_evaluated.blocking_partitions"))?;
                if !coverage.contains(id) {
                    return Err(reference_error("not_evaluated.blocking_partitions"));
                }
            }
        }
    }
    for record in &envelope.findings {
        validate_scalar_ref(record, "primary_source_handle_id", &handles, "findings")?;
        validate_string_array_refs(record, "related_source_handle_ids", &handles, "findings")?;
        validate_string_array_refs(record, "evidence_ids", &evidence, "findings")?;
    }
    for record in &envelope.warnings {
        validate_optional_scalar_ref(record, "primary_source_handle_id", &handles, "warnings")?;
        validate_string_array_refs(record, "related_source_handle_ids", &handles, "warnings")?;
        validate_string_array_refs(record, "evidence_ids", &evidence, "warnings")?;
    }
    validate_evidence_acyclic(&envelope.evidence_records)?;
    Ok(())
}

fn validate_evidence_acyclic(records: &[crate::EvidenceRecord]) -> CoreResult<()> {
    let mut edges = std::collections::BTreeMap::<String, Vec<String>>::new();
    for record in records {
        let value = record_value(record, "evidence_records")?;
        let id = required_string(&value, "evidence_id", "evidence_records")?;
        let inputs = value
            .get("derivation_input_ids")
            .and_then(Value::as_array)
            .map_or_else(Vec::new, |items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            });
        edges.insert(id, inputs);
    }
    let mut permanent = BTreeSet::new();
    let mut temporary = BTreeSet::new();
    for node in edges.keys() {
        visit_evidence(node, &edges, &mut temporary, &mut permanent)?;
    }
    Ok(())
}

fn visit_evidence(
    node: &str,
    edges: &std::collections::BTreeMap<String, Vec<String>>,
    temporary: &mut BTreeSet<String>,
    permanent: &mut BTreeSet<String>,
) -> CoreResult<()> {
    if permanent.contains(node) {
        return Ok(());
    }
    if !temporary.insert(node.to_owned()) {
        return Err(validation_error(
            "validate_evidence_derivation_graph",
            CoreErrorCode::EvidenceDerivationCycle,
            "evidence_records.derivation_input_ids",
        ));
    }
    if let Some(inputs) = edges.get(node) {
        for input in inputs {
            if !edges.contains_key(input) {
                return Err(reference_error("evidence_records.derivation_input_ids"));
            }
            visit_evidence(input, edges, temporary, permanent)?;
        }
    }
    temporary.remove(node);
    permanent.insert(node.to_owned());
    Ok(())
}

fn validate_status(envelope: &E0CheckResultEnvelope) -> CoreResult<()> {
    let partial_state = !envelope.not_evaluated.is_empty()
        || !envelope.conflicts.is_empty()
        || envelope.budget.truncation().is_truncated()
        || envelope.coverage_records.iter().any(|record| {
            record_status(record)
                .is_some_and(|status| status != "complete" && status != "not_applicable")
        });
    match envelope.status {
        ResultStatus::Complete if partial_state => Err(validation_error(
            "validate_result_envelope",
            CoreErrorCode::ResultStatusViolation,
            "status",
        )),
        ResultStatus::Failed if !envelope.findings.is_empty() => Err(validation_error(
            "validate_result_envelope",
            CoreErrorCode::ResultStatusViolation,
            "status",
        )),
        ResultStatus::Partial if !partial_state => Err(validation_error(
            "validate_result_envelope",
            CoreErrorCode::ResultStatusViolation,
            "status",
        )),
        _ => Ok(()),
    }
}

fn validate_budget_counts(envelope: &E0CheckResultEnvelope) -> CoreResult<()> {
    let expected = collection_usage(
        &envelope.coverage_records,
        &envelope.capability_summaries,
        &envelope.source_handles,
        &envelope.evidence_records,
        &envelope.conflicts,
        &envelope.findings,
        &envelope.not_evaluated,
        &envelope.warnings,
        envelope.budget.usage().output_bytes,
    )?;
    if envelope.budget.usage() == expected {
        Ok(())
    } else {
        Err(validation_error(
            "validate_result_envelope",
            CoreErrorCode::BudgetInvalid,
            "budget.usage",
        ))
    }
}

#[allow(clippy::too_many_arguments)]
fn collection_usage(
    coverage_records: &[crate::CoverageRecord],
    capability_summaries: &[crate::CapabilitySummary],
    source_handles: &[crate::SourceHandle],
    evidence_records: &[crate::EvidenceRecord],
    conflicts: &[crate::ConflictRecord],
    findings: &[Finding],
    not_evaluated: &[crate::NotEvaluatedRecord],
    warnings: &[WarningRecord],
    output_bytes: u64,
) -> CoreResult<BudgetUsage> {
    Ok(BudgetUsage {
        coverage_records: count(coverage_records.len())?,
        capability_summaries: count(capability_summaries.len())?,
        source_handles: count(source_handles.len())?,
        evidence_records: count(evidence_records.len())?,
        conflicts: count(conflicts.len())?,
        findings: count(findings.len())?,
        not_evaluated: count(not_evaluated.len())?,
        warnings: count(warnings.len())?,
        output_bytes,
    })
}

fn count(value: usize) -> CoreResult<u64> {
    u64::try_from(value).map_err(|_| usage_overflow("collection_count"))
}

fn usage_overflow(field: &'static str) -> CoreError {
    CoreError::new(
        CoreErrorCode::UsageOverflow,
        ErrorCategory::Invariant,
        "accumulate_budget_usage",
        RetryClass::Never,
    )
    .at_field(field)
}

fn record_value<T: Serialize>(record: &T, field: &'static str) -> CoreResult<Value> {
    serde_json::to_value(record).map_err(|error| {
        validation_error(
            "validate_result_envelope",
            CoreErrorCode::ContractViolation,
            field,
        )
        .with_argument("reason", error.to_string())
    })
}

fn record_id<T: Serialize>(record: &T, field: &'static str) -> CoreResult<String> {
    let value = record_value(record, field)?;
    required_string(&value, field, field)
}

fn required_string(
    value: &Value,
    field: &'static str,
    collection: &'static str,
) -> CoreResult<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            validation_error(
                "validate_result_envelope",
                CoreErrorCode::ContractViolation,
                collection,
            )
        })
}

fn coverage_sort_key_lossless(
    record: &crate::CoverageRecord,
) -> (String, String, String, String, String) {
    (
        record.capability_id().as_str().to_owned(),
        record.partition_id().canonical(),
        record.producer_id().as_str().to_owned(),
        record.producer_version().to_string(),
        record.coverage_id().canonical(),
    )
}

fn summary_sort_key_lossless(record: &crate::CapabilitySummary) -> Vec<u8> {
    canonical_json_bytes(record).unwrap_or_default()
}

fn id_set<T: Serialize>(
    records: &[T],
    field: &'static str,
    collection: &'static str,
) -> CoreResult<BTreeSet<String>> {
    records
        .iter()
        .map(|record| record_id(record, field))
        .collect::<CoreResult<BTreeSet<_>>>()
        .map_err(|error| error.at_field(collection))
}

fn validate_string_array_refs<T: Serialize>(
    record: &T,
    field: &'static str,
    registry: &BTreeSet<String>,
    collection: &'static str,
) -> CoreResult<()> {
    let value = record_value(record, collection)?;
    let Some(items) = value.get(field).and_then(Value::as_array) else {
        return Ok(());
    };
    for item in items {
        let Some(id) = item.as_str() else {
            return Err(reference_error(collection));
        };
        if !registry.contains(id) {
            return Err(reference_error(collection));
        }
    }
    Ok(())
}

fn validate_scalar_ref<T: Serialize>(
    record: &T,
    field: &'static str,
    registry: &BTreeSet<String>,
    collection: &'static str,
) -> CoreResult<()> {
    let value = record_value(record, collection)?;
    let id = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| reference_error(collection))?;
    if registry.contains(id) {
        Ok(())
    } else {
        Err(reference_error(collection))
    }
}

fn validate_optional_scalar_ref<T: Serialize>(
    record: &T,
    field: &'static str,
    registry: &BTreeSet<String>,
    collection: &'static str,
) -> CoreResult<()> {
    let value = record_value(record, collection)?;
    match value.get(field).and_then(Value::as_str) {
        Some(id) if !registry.contains(id) => Err(reference_error(collection)),
        _ => Ok(()),
    }
}

fn reference_error(field: &'static str) -> CoreError {
    validation_error(
        "validate_result_envelope",
        CoreErrorCode::ResultReferenceViolation,
        field,
    )
}

fn record_status<T: Serialize>(record: &T) -> Option<String> {
    serde_json::to_value(record).ok().and_then(|value| {
        value
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_owned)
    })
}

fn validate_records(envelope: &E0CheckResultEnvelope) -> CoreResult<()> {
    for handle in &envelope.source_handles {
        handle.validate()?;
    }
    crate::validate_evidence_derivation_graph(&envelope.evidence_records)?;
    for conflict in &envelope.conflicts {
        conflict.validate()?;
    }
    for coverage in &envelope.coverage_records {
        coverage.validate()?;
        for conflict_id in coverage.conflict_ids() {
            let conflict = envelope
                .conflicts
                .iter()
                .find(|conflict| conflict.conflict_id() == *conflict_id)
                .ok_or_else(|| reference_error("coverage_records.conflict_ids"))?;
            if !crate::coverage::conflict_affects_coverage(conflict, coverage) {
                return Err(validation_error(
                    "validate_coverage_record",
                    CoreErrorCode::CoverageConflict,
                    "coverage_records.conflict_ids",
                ));
            }
        }
    }
    for evidence in &envelope.evidence_records {
        for reference in evidence.coverage_refs() {
            let matches = envelope
                .coverage_records
                .iter()
                .filter(|coverage| {
                    coverage.capability_id() == reference.capability_id()
                        && coverage.partition_id() == reference.partition_id()
                        && coverage.producer_id() == reference.producer_id()
                })
                .count();
            if matches != 1 {
                return Err(reference_error("evidence_records.coverage_refs"));
            }
        }
    }
    for summary in &envelope.capability_summaries {
        let selected = summary
            .partition_refs()
            .iter()
            .map(|partition| {
                envelope
                    .coverage_records
                    .iter()
                    .find(|coverage| coverage.coverage_id() == partition.coverage_id())
                    .cloned()
                    .ok_or_else(|| reference_error("capability_summaries.partition_refs"))
            })
            .collect::<CoreResult<Vec<_>>>()?;
        summary.validate(&selected)?;
    }
    for record in &envelope.not_evaluated {
        record.validate()?;
        for blocker in record.blocking_partitions() {
            let coverage = envelope
                .coverage_records
                .iter()
                .find(|coverage| coverage.coverage_id() == blocker.coverage_id())
                .ok_or_else(|| reference_error("not_evaluated.blocking_partitions"))?;
            if coverage.capability_id() != blocker.capability_id()
                || coverage.partition_id() != blocker.partition_id()
                || coverage.status() != blocker.status()
            {
                return Err(reference_error("not_evaluated.blocking_partitions"));
            }
        }
    }
    let context_id = envelope.context.context_id();
    for finding in &envelope.findings {
        finding.validate(
            context_id,
            &envelope.source_handles,
            &envelope.evidence_records,
        )?;
    }
    for warning in &envelope.warnings {
        warning.validate(
            context_id,
            &envelope.source_handles,
            &envelope.evidence_records,
        )?;
    }
    Ok(())
}

/// Operation wrapper for strict result-envelope validation.
pub fn validate_result_envelope(envelope: &E0CheckResultEnvelope) -> CoreResult<()> {
    envelope.validate()
}

/// Returns a canonically ordered, rehashed equivalent result envelope.
pub fn canonical_result_order(
    mut envelope: E0CheckResultEnvelope,
) -> CoreResult<E0CheckResultEnvelope> {
    envelope.validate()?;
    envelope
        .coverage_records
        .sort_by_cached_key(coverage_sort_key_lossless);
    envelope
        .capability_summaries
        .sort_by_cached_key(summary_sort_key_lossless);
    envelope
        .source_handles
        .sort_by_key(crate::SourceHandle::handle_id);
    envelope
        .evidence_records
        .sort_by_key(crate::EvidenceRecord::evidence_id);
    envelope
        .conflicts
        .sort_by_key(crate::ConflictRecord::conflict_id);
    canonical_finding_order(&mut envelope.findings, &envelope.source_handles)?;
    envelope
        .not_evaluated
        .sort_by_key(crate::NotEvaluatedRecord::not_evaluated_id);
    envelope
        .warnings
        .sort_by_key(crate::WarningRecord::warning_id);

    let mut budget = envelope.budget.clone();
    for _ in 0..FINALIZATION_PASSES {
        envelope.budget = budget.clone();
        envelope.canonical_digest = canonical_result_digest(&envelope)?;
        let bytes = canonical_json_bytes(&envelope)?;
        let output_bytes =
            u64::try_from(bytes.len()).map_err(|_| usage_overflow("output_bytes"))?;
        if budget.usage().output_bytes == output_bytes {
            envelope.validate()?;
            return Ok(envelope);
        }
        budget = budget.with_output_bytes(output_bytes)?;
    }
    Err(CoreError::new(
        CoreErrorCode::CanonicalizationFailure,
        ErrorCategory::Invariant,
        "canonical_result_order",
        RetryClass::AfterInputChange,
    )
    .with_argument("reason", "output_bytes_fixed_point_did_not_converge"))
}

/// Operation wrapper for the result finalization pipeline.
pub fn finalize_result_envelope(draft: E0CheckResultDraft) -> CoreResult<E0CheckResultEnvelope> {
    draft.finalize()
}
