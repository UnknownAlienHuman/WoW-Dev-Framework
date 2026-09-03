#![forbid(unsafe_code)]

//! Pure, deterministic boundary primitives shared by WoW Dev Framework crates.
//!
//! This crate deliberately contains no filesystem, network, clock, process,
//! database, parser, graph, search, editor, or WoW-client behavior.

pub mod canonical;
pub mod digest;
pub mod error;
pub mod generation;
pub mod ids;
pub mod profile;
pub mod source;

pub use canonical::{canonical_json_bytes, canonical_json_string, domain_separated_digest};
pub use digest::{
    CanonicalResult, ConflictId, ContentDigest, CorrectionSet, CoverageId, DigestPurpose,
    EvidenceId, ExternalGenerationId, FindingFingerprint, FindingId, GenerationContextId,
    NotEvaluatedId, ProjectGenerationId, ReferenceGenerationId, RootCauseKey, SourceContent,
    SourceLogicalSnapshot, StableHandleId, WarningId,
};
pub use error::{CoreError, CoreErrorCode, CoreResult, ErrorCategory, RetryClass};
pub use generation::{
    ExternalGeneration, GenerationContext, GenerationContextBuilder, MergeMode,
    ProducerVersionEntry,
};
pub use ids::{
    CapabilityId, CoveragePartitionId, EntityKey, MessageCode, OperationId, Parsed, ProducerId,
    ProfileId, RuleId, SchemaId, ToolVersion,
};
pub use profile::{
    ProfileComparison, ProfileIdentity, ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry,
    SourceKind,
};
pub use source::{
    NormalizedSourcePath, SourceHandle, SourceHandleBuilder, SourceHandleComparison,
    SourceOriginKind, SourceSpan, SourceSpanKind,
};
