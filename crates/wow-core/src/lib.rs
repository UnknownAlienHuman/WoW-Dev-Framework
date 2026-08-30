#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod canonical;
mod coverage;
mod error;
mod evidence;
mod finding;
mod generation;
mod id;
mod profile;
mod result;
mod source;

pub use canonical::{canonical_json, canonical_json_digest};
pub use coverage::{
    combine_coverage, evaluate_negative_authority, CoverageRecord, CoverageRecordInput,
    CoverageStatus, CoverageSummary, NegativeAuthorityDecision, NegativeAuthorityStatus,
};
pub use error::{CoreError, CoreErrorCode, CoreResult};
pub use evidence::{
    canonical_evidence_key, combine_evidence_levels, EvidenceLevel, EvidenceRecord,
    ProvenanceClass,
};
pub use finding::{
    canonical_finding_key, Finding, FindingInput, FindingPolicy, FindingSeverity,
    MessageArguments, RemediationClass,
};
pub use generation::{
    merge_generation_context, require_same_generation, ExternalGeneration,
    GenerationContext,
};
pub use id::{
    CapabilityId, ContentDigest, CoveragePartitionId, EntityKey, EvidenceId,
    ExternalGenerationId, FindingKey, FlavorId, MessageKey, OperationId, ProducerId,
    ProfileId, ProjectGenerationId, ReferenceGenerationId, RepositoryId, RevisionId,
    RootCauseKey, RuleId, SchemaVersion, StableHandleId, ToolVersion,
};
pub use profile::{ProfileIdentity, ProfileKind};
pub use result::{
    canonical_result_order, validate_result_envelope, CapabilityState, NotEvaluatedRecord,
    ResultEnvelope, ResultEnvelopeInput, ResultWarning, TruncationStatus,
};
pub use source::{
    build_source_handle, normalize_source_path, verify_source_handle_content, ByteSpan,
    LinePosition, LineSpan, SourceHandle, SourceHandleInput, SourceOwner,
};
