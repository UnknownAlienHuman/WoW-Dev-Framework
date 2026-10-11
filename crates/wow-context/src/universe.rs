use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{
    CanonicalResult, ContentDigest, GenerationContextId, ProfileIdentity, ProjectGenerationId,
    ReferenceGenerationId,
};
use wow_graph::{GraphGenerationId, GraphSnapshotId, GraphUniverseId};
use wow_project::replay::publication::AcquiredProjectPair;
use wow_project::{ProjectId, ProjectKind};
use wow_reference::{CoverageStatus, ReferenceView};

use crate::budget::{canonical, checkpoint, encoded_len, identity};
use crate::profile::{PRIMARY_MAP_PROFILE_BYTES, PROFILE};
use crate::{ContextError, ContextErrorCode, ContextResult, PrimaryMapLimits};

#[derive(Debug, Serialize)]
pub(crate) struct UniverseBinding<'a> {
    schema: &'static str,
    pub(crate) project_id: &'a ProjectId,
    pub(crate) project_kind: ProjectKind,
    pub(crate) project_snapshot_id: &'a str,
    pub(crate) project_generation: ProjectGenerationId,
    configuration_digest: ContentDigest<CanonicalResult>,
    graph_universe: &'a GraphUniverseId,
    graph_generation: &'a GraphGenerationId,
    pub(crate) graph_snapshot_id: &'a GraphSnapshotId,
    pub(crate) source_context_id: GenerationContextId,
    source_registry_id: &'a str,
    analyzer_snapshot_id: &'a str,
    registry_digest: &'a str,
    target_profile: &'a ProfileIdentity,
    pub(crate) reference_generation: ReferenceGenerationId,
    reference_schema: &'a str,
    reference_view_digest: &'a str,
    /// ReferenceView has no ProfileIdentity field. This is the exact caller
    /// association checked against ProjectConfiguration, not reference attestation.
    compatibility_basis: &'static str,
    platform_absence: &'static str,
}

/// Sealed borrowed native owners. Cannot be reconstructed by deserializing a DTO.
/// The acquired pair already validated project/graph/source closure on one read.
pub struct ContextUniverseSet<'a> {
    pub(crate) pair: &'a AcquiredProjectPair,
    pub(crate) reference: &'a ReferenceView,
    pub(crate) binding: UniverseBinding<'a>,
    pub(crate) limits: PrimaryMapLimits,
    universe_set_id: Box<str>,
    pub(crate) input_bytes_reserved: u64,
}
impl ContextUniverseSet<'_> {
    #[must_use]
    pub fn universe_set_id(&self) -> &str {
        &self.universe_set_id
    }
    #[must_use]
    pub const fn limits(&self) -> PrimaryMapLimits {
        self.limits
    }
}

/// Primary-only operation; optional platform absence is selected by this closed
/// profile, recorded canonically, and affects platform-context coverage.
/// Reference content is validated under a byte preflight, never enriched.
pub fn bind_context_universe_set<'a>(
    pair: &'a AcquiredProjectPair,
    reference: &'a ReferenceView,
    reference_profile: &'a ProfileIdentity,
    limits: PrimaryMapLimits,
    stop: &AtomicBool,
) -> ContextResult<ContextUniverseSet<'a>> {
    checkpoint(stop)?;
    let project = pair.project();
    let configuration = project.configuration();
    let owner = pair.graph();
    if configuration.project_kind() == ProjectKind::BlizzardUiPlatformSource
        || owner.source_context_id() != project.snapshot().generation_context().context_id()
        || configuration.selected_profile() != reference_profile
    {
        return Err(mismatch());
    }
    reference_profile.validate().map_err(|_| mismatch())?;
    let generation: ReferenceGenerationId =
        reference.generation_id().parse().map_err(|_| mismatch())?;
    if generation != configuration.reference_generation() {
        return Err(mismatch());
    }
    // Native validation hashes the full reference view. Bound it first, retain
    // none of its payload strings, and checkpoint before/after that bounded call.
    let input_bytes_reserved = encoded_len(reference, limits.max_input_bytes, stop)?;
    let validation = reference.validate();
    checkpoint(stop)?;
    validation.map_err(|_| mismatch())?;
    if canonical(&PROFILE, limits.max_metadata_bytes, stop)?.as_slice() != PRIMARY_MAP_PROFILE_BYTES
    {
        return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
    }
    let binding = UniverseBinding {
        schema: "wow-context/primary-universe-binding/1",
        project_id: configuration.project_id(),
        project_kind: configuration.project_kind(),
        project_snapshot_id: project.snapshot_id(),
        project_generation: project.project_generation(),
        configuration_digest: configuration.configuration_digest(),
        graph_universe: owner.snapshot().universe(),
        graph_generation: owner.snapshot().generation(),
        graph_snapshot_id: owner.snapshot().snapshot_id(),
        source_context_id: owner.source_context_id(),
        source_registry_id: project.snapshot().source_registry().registry_id(),
        analyzer_snapshot_id: project.analyzer_snapshot_id(),
        registry_digest: owner.registry().registry_digest(),
        target_profile: reference_profile,
        reference_generation: generation,
        reference_schema: reference.schema(),
        reference_view_digest: reference.self_digest(),
        compatibility_basis: "exact_configured_generation_and_explicit_caller_profile",
        platform_absence: "profile_excluded_optional_platform_context",
    };
    let universe_set_id = identity(
        "wow-context/primary-universe-binding/1",
        "context-universe:sha256:",
        &binding,
        limits.max_metadata_bytes,
        stop,
    )?;
    Ok(ContextUniverseSet {
        pair,
        reference,
        binding,
        limits,
        universe_set_id,
        input_bytes_reserved,
    })
}

#[derive(Debug, Serialize)]
pub(crate) struct ReferenceCoverage<'a> {
    pub(crate) partition_id: &'a str,
    pub(crate) state: CoverageStatus,
}
fn mismatch() -> ContextError {
    ContextError::new(ContextErrorCode::UniverseMismatch)
}
