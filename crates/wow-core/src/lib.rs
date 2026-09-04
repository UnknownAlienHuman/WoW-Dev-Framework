#![forbid(unsafe_code)]

//! Pure, deterministic boundary primitives shared by WoW Dev Framework crates.
//!
//! This crate deliberately contains no filesystem, network, clock, process,
//! database, parser, graph, search, editor, or WoW-client behavior.

pub mod budget;
pub mod canonical;
pub mod contract;
pub mod coverage;
pub mod digest;
pub mod envelope;
pub mod error;
pub mod evidence;
pub mod finding;
pub mod generation;
pub mod ids;
mod integrity;
pub mod profile;
pub mod source;

pub use budget::{
    Budget, BudgetLimits, BudgetUsage, TruncationEntry, TruncationState, accumulate_budget_usage,
    classify_truncation, validate_budget,
};
pub use canonical::{
    CANONICALIZATION_VERSION, canonical_json_bytes, canonical_json_string, domain_separated_digest,
};
pub use contract::E0_OPERATION_IDS;
pub use coverage::{
    BlockingPartitionRef, CapabilityAvailability, CapabilitySummary, CoveragePartitionRef,
    CoverageRecord, CoverageStatus, CoverageTruncationRef, NegativeAuthorityDecision,
    NegativeAuthorityOutcome, NegativeAuthorityReason, NotEvaluatedRecord, combine_coverage,
    derive_coverage_id, derive_not_evaluated_id, evaluate_capability_availability,
    evaluate_negative_authority, validate_capability_summary, validate_coverage_record,
    validate_not_evaluated_record,
};
pub use digest::{
    CanonicalResult, ConflictId, ContentDigest, CorrectionSet, CoverageId, DigestPurpose,
    EvidenceId, ExternalGenerationId, FindingFingerprint, FindingId, GenerationContextId,
    NotEvaluatedId, ProjectGenerationId, ReferenceGenerationId, RootCauseKey, SourceContent,
    SourceLogicalSnapshot, StableHandleId, WarningId, derive_typed_digest_id, parse_content_digest,
};
pub use envelope::{
    E0CheckResultDraft, E0CheckResultEnvelope, E0OperationErrorEnvelope, ResultStatus,
    SchemaCompatibility, canonical_result_digest, canonical_result_order, finalize_result_envelope,
    validate_result_envelope, validate_schema_version,
};
pub use error::{
    CoreError, CoreErrorCode, CoreResult, ErrorArgument, ErrorArgumentKind, ErrorCategory,
    RetryClass,
};
pub use evidence::{
    ClaimScope, ConflictAffectedRef, ConflictRecord, CoverageReference, EvidenceConfidence,
    EvidenceCoverageRef, EvidenceLevel, EvidenceRecord, ProvenanceClass, derive_conflict_id,
    derive_evidence, derive_evidence_id, relate_evidence_conflict, validate_conflict_record,
    validate_evidence_derivation_graph, validate_evidence_record,
};
pub use finding::{
    Finding, FindingDraft, MessageArgument, MessageArgumentKind, Remediation, RemediationClass,
    RolloutPolicy, Severity, WarningRecord, bind_finding_to_context, canonical_finding_order,
    deduplicate_findings, derive_finding_fingerprint, derive_root_cause_key, derive_warning_id,
    validate_message_arguments, validate_warning_record,
};
pub use generation::{
    ExternalGeneration, GenerationContext, GenerationContextBuilder, MergeMode,
    ProducerVersionEntry, derive_generation_context_id, merge_generation_context,
    require_same_generation, validate_generation_context,
};
pub use ids::{
    CapabilityId, CoveragePartitionId, EntityKey, MessageCode, OperationId, Parsed, ProducerId,
    ProfileId, RuleId, SchemaId, ToolVersion, parse_capability_id, parse_coverage_partition_id,
    parse_entity_key, parse_operation_id, parse_producer_id, parse_profile_id, parse_rule_id,
};
pub use profile::{
    ProfileComparison, ProfileIdentity, ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry,
    SourceKind, compare_profile_identity, require_profile_identity_match,
    validate_profile_identity,
};
pub use source::{
    NormalizedSourcePath, SourceHandle, SourceHandleBuilder, SourceHandleComparison,
    SourceOriginKind, SourceSpan, SourceSpanKind, build_source_handle, compare_source_handles,
    normalize_source_path, validate_source_span, verify_source_handle_content,
};
