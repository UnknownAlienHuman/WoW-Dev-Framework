//! Selected platform metadata/handles over one genuine retained Skeleton input.
//! Primary-only universe admission and its serialized artifacts stay unchanged.
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_core::{
    EvidenceRecord, GenerationContext, ProfileIdentity, ReferenceGenerationId, SourceHandle,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphAssertionSupport, GraphLocalAssertion,
    GraphNeighborView, GraphNode, GraphQueryState, GraphResolvedExplanation,
};
use wow_project::skeleton_input::{
    OpenSkeletonInputRequest, SkeletonInputNotEvaluated, SkeletonInputPage, SkeletonInputView,
};
use wow_project::{
    ProjectError, ProjectErrorCode, ProjectKind,
    graph::{ProjectCallableMetadata, ProjectGraphInlineHandler, ProjectGraphScriptSource},
    load::XmlScriptSource,
};
use wow_reference::{ReferenceConflict, ReferenceView, ReferenceViewError};

use crate::budget::{Budget, canonical, checkpoint, encoded_len, exhausted, identity};
use crate::universe::ReferenceCoverage;
use crate::{ContextError, ContextErrorCode, ContextResult, PrimaryMapLimits};

/// One selected family; artifact roles distinguish universe, map and inspect.
pub const PLATFORM_NATIVE_ONE_HOP_PROFILE: &str = "wow-context/platform-native-one-hop/1";
pub const PLATFORM_NATIVE_ONE_HOP_PROFILE_BYTES: &[u8] = br#"{"native":"genuine_admitted_platform_skeleton_input","neighborhood":"one_exact_native_query","profile":"wow-context/platform-native-one-hop/1","reference":"configured_profile_generation_and_validated_full_content_binding","runtime":"not_evaluated","source":"metadata_and_handles_only","support":"original_root_support_neighbor_evidence_not_evaluated","tokens":"not_evaluated"}"#;

/// Closed inert intent; decoding this selector supplies no native authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformRootFacet {
    CallableMetadata,
    XmlHandlerMetadata,
}

#[derive(Serialize)]
struct PlatformProfile {
    native: &'static str,
    neighborhood: &'static str,
    profile: &'static str,
    reference: &'static str,
    runtime: &'static str,
    source: &'static str,
    support: &'static str,
    tokens: &'static str,
}
const PROFILE: PlatformProfile = PlatformProfile {
    native: "genuine_admitted_platform_skeleton_input",
    neighborhood: "one_exact_native_query",
    profile: PLATFORM_NATIVE_ONE_HOP_PROFILE,
    reference: "configured_profile_generation_and_validated_full_content_binding",
    runtime: "not_evaluated",
    source: "metadata_and_handles_only",
    support: "original_root_support_neighbor_evidence_not_evaluated",
    tokens: "not_evaluated",
};

#[derive(Serialize)]
struct PlatformBinding<'a> {
    family: &'static str,
    artifact_kind: &'static str,
    profile: &'static PlatformProfile,
    native_request: &'a OpenSkeletonInputRequest,
    target_profile: &'a ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    reference_schema: &'a str,
    reference_view_digest: &'a str,
    reference_coverage: Vec<ReferenceCoverage<'a>>,
    reference_conflicts: &'a [ReferenceConflict],
    compatibility_basis: &'static str,
    limits: PrimaryMapLimits,
    input_bytes_reserved: u64,
}

/// Borrowed qualified view and the original continuing context ledger.
/// No decoding, cloning or public field constructor can create this authority.
pub struct PlatformContextUniverse<'a> {
    view: &'a SkeletonInputView<'a>,
    binding: PlatformBinding<'a>,
    universe_id: Box<str>,
    budget: Budget,
}
impl PlatformContextUniverse<'_> {
    #[must_use]
    pub fn universe_id(&self) -> &str {
        &self.universe_id
    }
}

/// Admit the selected platform universe without weakening primary admission.
pub fn bind_platform_context_universe<'a>(
    view: &'a SkeletonInputView<'a>,
    reference: &'a ReferenceView,
    reference_profile: &'a ProfileIdentity,
    limits: PrimaryMapLimits,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextUniverse<'a>> {
    bind_platform_context_universe_with_budget(
        view,
        reference,
        reference_profile,
        limits,
        Budget::new(limits),
        0,
        stop,
    )
}

// The standalone wrapper creates its own ledger; composition transfers the
// actual Primary ledger here before any additional platform validation/read.
pub(crate) fn bind_platform_context_universe_with_budget<'a>(
    view: &'a SkeletonInputView<'a>,
    reference: &'a ReferenceView,
    reference_profile: &'a ProfileIdentity,
    limits: PrimaryMapLimits,
    mut budget: Budget,
    prior_input_bytes: u64,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextUniverse<'a>> {
    checkpoint(stop)?;
    let (source, accepted, request) = view.context_parts();
    let configuration = source.project().configuration();
    if configuration.project_kind() != ProjectKind::BlizzardUiPlatformSource
        || configuration.selected_profile() != reference_profile
        || accepted.source_context_id() != source.scope().source_context_id
        || request.neighborhood().snapshot_id() != accepted.snapshot().snapshot_id()
        || request.neighborhood().limits().max_scanned_edges > limits.neighbors.max_scanned_edges
        || request.neighborhood().limits().max_output_bytes > limits.neighbors.max_output_bytes
        || request.neighborhood().query().max_edges() > limits.max_direct_edges
        || request.limits().root_explanation.max_scanned_assertions
            > limits.explain.max_scanned_assertions
        || request.limits().root_explanation.max_supports > limits.explain.max_supports
        || request.limits().root_explanation.max_output_bytes > limits.explain.max_output_bytes
        || request.limits().root_explanation.max_derivation_depth
            > limits.explain.max_derivation_depth
    {
        return Err(mismatch());
    }
    budget.work(1, stop)?;
    let input_bytes_reserved = encoded_len(
        &(reference_profile, reference, request),
        limits
            .max_input_bytes
            .checked_sub(prior_input_bytes)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let profile_validation = reference_profile.validate();
    checkpoint(stop)?;
    profile_validation.map_err(|_| mismatch())?;
    let generation: ReferenceGenerationId =
        reference.generation_id().parse().map_err(|_| mismatch())?;
    if generation != configuration.reference_generation() {
        return Err(mismatch());
    }
    let validation = reference.validate_bounded(
        usize::try_from(limits.max_input_bytes).map_err(|_| exhausted())?,
        usize::try_from(
            limits
                .max_context_work
                .checked_sub(budget.work)
                .ok_or_else(exhausted)?,
        )
        .map_err(|_| exhausted())?,
        stop,
    );
    checkpoint(stop)?;
    let (_, steps) = validation.map_err(reference_error)?;
    budget.work(steps, stop)?;
    budget.retain(&PROFILE, stop)?;
    if canonical(
        &PROFILE,
        u64::try_from(PLATFORM_NATIVE_ONE_HOP_PROFILE_BYTES.len()).map_err(|_| exhausted())?,
        stop,
    )?
    .as_slice()
        != PLATFORM_NATIVE_ONE_HOP_PROFILE_BYTES
    {
        return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
    }
    let mut reference_coverage = Vec::new();
    for partition in reference.partitions() {
        budget.work(1, stop)?;
        let row = ReferenceCoverage {
            partition_id: partition.id(),
            state: partition.coverage(),
        };
        budget.retain(&row, stop)?;
        reference_coverage.push(row);
    }
    let binding = PlatformBinding {
        family: PLATFORM_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "universe",
        profile: &PROFILE,
        native_request: request,
        target_profile: reference_profile,
        reference_generation: generation,
        reference_schema: reference.schema(),
        reference_view_digest: reference.self_digest(),
        reference_coverage,
        reference_conflicts: reference.conflicts(),
        compatibility_basis: "exact_configured_generation_and_explicit_caller_profile_checked_full_view",
        limits,
        input_bytes_reserved,
    };
    budget.retain(&binding, stop)?;
    let universe_id = identity(
        PLATFORM_NATIVE_ONE_HOP_PROFILE,
        "platform-context-universe:sha256:",
        &binding,
        remaining(&budget, limits)?,
        stop,
    )?;
    budget.retain(&universe_id, stop)?;
    checkpoint(stop)?;
    Ok(PlatformContextUniverse {
        view,
        binding,
        universe_id,
        budget,
    })
}

#[derive(Serialize)]
struct PlatformMapCore<'a> {
    universe_id: Box<str>,
    binding: PlatformBinding<'a>,
    native: SkeletonInputPage<'a>,
    absence_authoritative: bool,
}
#[derive(Serialize)]
pub(crate) struct PlatformMapArtifact<'a> {
    family: &'static str,
    artifact_kind: &'static str,
    map_id: Box<str>,
    core: PlatformMapCore<'a>,
    #[serde(skip)]
    native_view: &'a SkeletonInputView<'a>,
}

impl<'a> PlatformMapArtifact<'a> {
    pub(crate) fn map_id(&self) -> &str {
        &self.map_id
    }
    pub(crate) fn page(&self) -> &SkeletonInputPage<'a> {
        &self.core.native
    }
    pub(crate) fn input_bytes_reserved(&self) -> u64 {
        self.core.binding.input_bytes_reserved
    }
}

/// Serialize-only output retaining the original native page and its view lifetime.
#[derive(Serialize)]
pub struct PlatformContextMap<'a> {
    #[serde(flatten)]
    artifact: PlatformMapArtifact<'a>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: Budget,
}
impl<'a> PlatformContextMap<'a> {
    pub(crate) fn into_parts(self) -> (PlatformMapArtifact<'a>, Box<[u8]>, Budget) {
        (self.artifact, self.canonical_bytes, self.budget)
    }
    #[must_use]
    pub fn map_id(&self) -> &str {
        &self.artifact.map_id
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn page(&self) -> &SkeletonInputPage<'_> {
        &self.artifact.core.native
    }
    #[must_use]
    pub fn remaining_metadata_bytes(&self) -> u64 {
        self.artifact.core.binding.limits.max_metadata_bytes - self.budget.retained_bytes
    }
}

/// Consume the admitted ledger; invoke the genuine native one-hop read once.
pub fn build_platform_context_map<'a>(
    universe: PlatformContextUniverse<'a>,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextMap<'a>> {
    checkpoint(stop)?;
    let PlatformContextUniverse {
        view,
        binding,
        universe_id,
        mut budget,
    } = universe;
    let limits = binding.limits;
    // Bound native materialization before it occurs; commit exact retained bytes
    // below. The original immutable request and native ceilings are unchanged.
    if u64::from(view.context_parts().2.limits().max_output_bytes) > remaining(&budget, limits)? {
        return Err(exhausted());
    }
    // Native neighbors scans the full edge population even when few rows match.
    // Reserve that exact scan plus the caller's root explanation/evidence ceilings
    // before executing; returned-row charges below cover the context projection.
    let (_, accepted, request) = view.context_parts();
    let native_scan_work = accepted
        .snapshot()
        .edges()
        .len()
        .checked_add(request.limits().root_explanation.max_scanned_assertions as usize)
        .and_then(|work| work.checked_add(request.limits().root_evidence.max_work as usize))
        .and_then(|work| work.checked_add(1))
        .ok_or_else(exhausted)?;
    budget.work(native_scan_work, stop)?;
    let page = view.read(stop).map_err(project_error)?;
    checkpoint(stop)?;
    let counts = page.counts();
    budget.work(
        counts
            .returned_entity_records
            .checked_add(counts.returned_relations)
            .and_then(|n| n.checked_add(counts.returned_root_support_records))
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let core = PlatformMapCore {
        universe_id,
        binding,
        native: page,
        absence_authoritative: false,
    };
    budget.retain(&core, stop)?;
    let map_id = identity(
        PLATFORM_NATIVE_ONE_HOP_PROFILE,
        "platform-context-map:sha256:",
        &("map", &core),
        remaining(&budget, limits)?,
        stop,
    )?;
    let artifact = PlatformMapArtifact {
        family: PLATFORM_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "map",
        map_id,
        core,
        native_view: view,
    };
    let canonical_bytes = encode_retained(&artifact, &mut budget, limits, stop)?;
    Ok(PlatformContextMap {
        artifact,
        canonical_bytes,
        budget,
    })
}

#[derive(Serialize)]
struct PlatformInspectArtifact<M> {
    family: &'static str,
    artifact_kind: &'static str,
    inspect_id: Box<str>,
    map: M,
}

/// Selected root/native-neighborhood metadata and original root support only.
#[derive(Serialize)]
pub struct PlatformContextInspect<'a> {
    #[serde(flatten)]
    artifact: PlatformInspectArtifact<PlatformMapArtifact<'a>>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    map_canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: Budget,
}
impl PlatformContextInspect<'_> {
    #[must_use]
    pub fn inspect_id(&self) -> &str {
        &self.artifact.inspect_id
    }
    #[must_use]
    pub fn map_id(&self) -> &str {
        &self.artifact.map.map_id
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn map_canonical_bytes(&self) -> &[u8] {
        &self.map_canonical_bytes
    }
    #[must_use]
    pub fn page(&self) -> &SkeletonInputPage<'_> {
        &self.artifact.map.core.native
    }
    #[must_use]
    pub fn root(&self) -> &GraphNode {
        self.page().neighborhood().root()
    }
    #[must_use]
    pub fn neighborhood(&self) -> &GraphNeighborView<'_> {
        self.page().neighborhood()
    }
    #[must_use]
    pub fn root_support(&self) -> &GraphResolvedExplanation<'_> {
        self.page().root_support()
    }
    #[must_use]
    pub fn not_evaluated(&self) -> &[SkeletonInputNotEvaluated] {
        self.page().not_evaluated()
    }
    #[must_use]
    pub fn state(&self) -> GraphQueryState {
        self.page().state()
    }
    #[must_use]
    pub fn remaining_metadata_bytes(&self) -> u64 {
        self.artifact.map.core.binding.limits.max_metadata_bytes - self.budget.retained_bytes
    }
}

/// Consume the actual map/ledger without another native read or receipt copy.
pub fn build_platform_context_inspect<'a>(
    map: PlatformContextMap<'a>,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextInspect<'a>> {
    checkpoint(stop)?;
    let PlatformContextMap {
        artifact: map,
        canonical_bytes: map_canonical_bytes,
        mut budget,
    } = map;
    let limits = map.core.binding.limits;
    let (artifact, canonical_bytes) = platform_inspect_artifact(map, &mut budget, limits, stop)?;
    Ok(PlatformContextInspect {
        artifact,
        canonical_bytes,
        map_canonical_bytes,
        budget,
    })
}

// Owned standalone and borrowed combined inspections use identical identity and
// encoding steps; references serialize the original immutable map exactly.
fn platform_inspect_artifact<M: Serialize>(
    map: M,
    budget: &mut Budget,
    limits: PrimaryMapLimits,
    stop: &AtomicBool,
) -> ContextResult<(PlatformInspectArtifact<M>, Box<[u8]>)> {
    budget.work(1, stop)?;
    let inspect_id = identity(
        PLATFORM_NATIVE_ONE_HOP_PROFILE,
        "platform-context-inspect:sha256:",
        &("inspect", &map),
        remaining(budget, limits)?,
        stop,
    )?;
    let artifact = PlatformInspectArtifact {
        family: PLATFORM_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "inspect",
        inspect_id,
        map,
    };
    let canonical_bytes = encode_retained(&artifact, budget, limits, stop)?;
    Ok((artifact, canonical_bytes))
}

/// Borrowed original platform inspection, sealed by the same owner engine.
/// It owns only its inspection identity/canonical cache, never a native page copy.
#[derive(Serialize)]
pub struct PlatformContextInspectView<'m, 'a> {
    #[serde(flatten)]
    artifact: PlatformInspectArtifact<&'m PlatformMapArtifact<'a>>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    map_canonical_bytes: &'m [u8],
}
impl<'a> PlatformContextInspectView<'_, 'a> {
    #[must_use]
    pub fn inspect_id(&self) -> &str {
        &self.artifact.inspect_id
    }
    #[must_use]
    pub fn map_id(&self) -> &str {
        self.artifact.map.map_id()
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn map_canonical_bytes(&self) -> &[u8] {
        self.map_canonical_bytes
    }
    #[must_use]
    pub fn page(&self) -> &SkeletonInputPage<'a> {
        self.artifact.map.page()
    }
    #[must_use]
    pub fn root(&self) -> &GraphNode {
        self.page().neighborhood().root()
    }
    #[must_use]
    pub fn neighborhood(&self) -> &GraphNeighborView<'_> {
        self.page().neighborhood()
    }
    #[must_use]
    pub fn root_support(&self) -> &GraphResolvedExplanation<'_> {
        self.page().root_support()
    }
    #[must_use]
    pub fn not_evaluated(&self) -> &[SkeletonInputNotEvaluated] {
        self.page().not_evaluated()
    }
    #[must_use]
    pub fn state(&self) -> GraphQueryState {
        self.page().state()
    }
}

pub(crate) fn build_platform_context_inspect_on_budget<'m, 'a>(
    map: &'m PlatformMapArtifact<'a>,
    map_canonical_bytes: &'m [u8],
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextInspectView<'m, 'a>> {
    checkpoint(stop)?;
    let (artifact, canonical_bytes) =
        platform_inspect_artifact(map, budget, map.core.binding.limits, stop)?;
    Ok(PlatformContextInspectView {
        artifact,
        canonical_bytes,
        map_canonical_bytes,
    })
}

/// Original selected-root metadata and custody; no public constructor or decoder.
#[derive(Serialize)]
pub(crate) struct PlatformRootCallableFacet<'m, 'a> {
    selection: PlatformRootFacet,
    interpretation: &'static str,
    source_context: &'a GenerationContext,
    assertion: &'a GraphAssertionRef,
    root: &'a GraphNode,
    metadata: ProjectCallableMetadata<'a>,
    source_handle: &'a SourceHandle,
    evidence: &'a EvidenceRecord,
    root_support: &'m GraphResolvedExplanation<'m>,
}

pub(crate) fn build_platform_root_callable_facet_on_budget<'m, 'a>(
    map: &'m PlatformMapArtifact<'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<PlatformRootCallableFacet<'m, 'a>> {
    checkpoint(stop)?;
    let native_view: &'a SkeletonInputView<'a> = map.native_view;
    let (native, accepted, _) = native_view.context_parts();
    let source = native.source();
    let root = map.page().neighborhood().root();
    let root_support = map.page().root_support();
    if root.kind() != "lua_source_function" {
        return Err(ContextError::new(ContextErrorCode::RootInvalid));
    }
    if source.context().context_id() != accepted.source_context_id()
        || source.context().context_id() != native.scope().source_context_id
    {
        return Err(mismatch());
    }
    let explanation = root_support.explanation();
    if !explanation.support_complete() || !explanation.derivation_complete() {
        return Err(facet_missing());
    }
    if !explanation.conflicts().is_empty() {
        return Err(facet_mismatch());
    }

    // Search exact native proposal IDs in the already resolved root support.
    // Reserve the full search before visiting either collection; no name join.
    let supports = explanation.supports();
    let functions = source.functions();
    budget.work(
        functions
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(supports.len()))
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let depth = |count: usize| count.max(1).ilog2() as usize + 1;
    // The original producer charges every serialized address row. Its charged
    // byte count therefore bounds address count without scanning any graph.
    let address_depth = depth(native.charged_metadata_bytes());
    let mut selected = None;
    for support in supports {
        checkpoint(stop)?;
        let GraphAssertionSupport::ProducerEntity {
            producer,
            proposal,
            accepted: receipt,
            ..
        } = support
        else {
            continue;
        };
        // Restrict the search to the exact selected entity identity, never a
        // callable dependency which happens to occur in this root explanation.
        if receipt.node().kind() != root.kind() || receipt.node().owner_key() != root.owner_key() {
            continue;
        }
        for function in functions {
            checkpoint(stop)?;
            if function.proposal_id.as_str() != proposal.proposal_id() {
                continue;
            }
            budget.work(
                function
                    .proposal_id
                    .len()
                    .checked_add(address_depth)
                    .ok_or_else(exhausted)?,
                stop,
            )?;
            budget.retain(&function.proposal_id, stop)?;
            let local = GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: function.proposal_id.clone().into(),
            };
            let assertion = native.assertion(&local).ok_or_else(facet_missing)?;
            let GraphAssertionRef::Producer {
                partition_id,
                batch_id,
                assertion: retained_local,
            } = assertion
            else {
                return Err(facet_mismatch());
            };
            if partition_id.as_ref() != producer.partition_id
                || batch_id.as_ref() != producer.batch_id
            {
                continue;
            }
            // These are original GraphExplainQuery proposal/receipt witnesses
            // from the already admitted Skeleton view, not decoded authority.
            if retained_local != &local
                || receipt.proposal_id() != proposal.proposal_id()
                || proposal.entity_kind_id() != "lua_source_function"
                || receipt.node().universe() != &native.scope().universe
                || receipt.node().generation() != &native.scope().generation
            {
                return Err(facet_mismatch());
            }
            if selected
                .replace((function, assertion, *proposal, *receipt))
                .is_some()
            {
                return Err(facet_mismatch());
            }
        }
    }
    let (function, assertion, proposal, receipt) = selected.ok_or_else(facet_missing)?;

    // The input-generation accepted node and the final selected node are distinct.
    // Rebind only this original witness and resolve its exact final node.
    let snapshot = accepted.snapshot();
    let evidence_count = receipt.node().evidence_ids().len();
    let node_work = depth(snapshot.nodes().len())
        .checked_add(8)
        .and_then(|n| {
            evidence_count
                .checked_mul(depth(evidence_count) + 1)
                .and_then(|e| n.checked_add(e))
        })
        .ok_or_else(exhausted)?;
    budget.work(node_work, stop)?;
    // Reserve the selected original node (including its identity/evidence) and
    // exact target identities before constructing the one transient rebound.
    let original = receipt.node();
    let node_material = (original, snapshot.universe(), snapshot.generation());
    budget.retain(&node_material, stop)?;
    let node_bytes = encoded_len(&node_material, budget.remaining_context_work(), stop)?;
    budget.work(usize::try_from(node_bytes).map_err(|_| exhausted())?, stop)?;
    let rebound = GraphNode::new(
        snapshot.universe().clone(),
        snapshot.generation().clone(),
        original.kind(),
        original.owner_key(),
        original.evidence_ids().to_vec(),
        snapshot.limits(),
    )
    .map_err(crate::error::graph_error)?;
    let final_node = snapshot.node(rebound.node_id()).ok_or_else(facet_missing)?;
    if !std::ptr::eq(final_node, root) {
        return Err(facet_mismatch());
    }

    budget.work(
        depth(source.source_handles().len())
            .checked_add(depth(source.evidence().len()))
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let source_handle = source
        .source_handles()
        .get(&function.source_handle_id)
        .ok_or_else(facet_missing)?;
    let evidence = source
        .evidence()
        .get(&function.evidence_id)
        .ok_or_else(facet_missing)?;
    let report_rows = source
        .function_call_report()
        .map_or(0, |r| r.functions().len());
    let metadata_work = [
        depth(functions.len()),
        depth(report_rows),
        depth(report_rows),
        depth(source.files().len()),
        depth(source.source_handles().len()),
        depth(source.evidence().len()),
        evidence.source_handle_ids().len(),
        root_support.evidence_resolution().source_handles().len(),
        root_support.evidence_resolution().evidence_records().len(),
        proposal.source_handle_ids().len(),
        proposal.evidence_ids().len(),
        8,
    ]
    .into_iter()
    .try_fold(0usize, |sum, n| sum.checked_add(n).ok_or_else(exhausted))?;
    budget.work(metadata_work, stop)?;
    // Also reserve structural comparisons of original support records before
    // they run. Catalog records may be retained copies; never decode substitutes.
    let comparison_bytes = encoded_len(
        &(source_handle, evidence, root_support.evidence_resolution()),
        budget
            .remaining_metadata_bytes()?
            .min(budget.remaining_context_work()),
        stop,
    )?;
    budget.work(
        usize::try_from(comparison_bytes).map_err(|_| exhausted())?,
        stop,
    )?;
    let metadata = source
        .callable_metadata(&function.function_id)
        .map_err(project_error)?
        .ok_or_else(facet_missing)?;
    checkpoint(stop)?;
    if !std::ptr::eq(metadata.function(), function)
        || !proposal
            .source_handle_ids()
            .contains(&source_handle.handle_id())
        || !proposal.evidence_ids().contains(&evidence.evidence_id())
        || !root_support
            .evidence_resolution()
            .source_handles()
            .contains(&source_handle)
        || !root_support
            .evidence_resolution()
            .evidence_records()
            .contains(&evidence)
    {
        return Err(facet_mismatch());
    }
    let facet = PlatformRootCallableFacet {
        selection: PlatformRootFacet::CallableMetadata,
        interpretation: "original_accepted_root_callable_metadata_no_runtime_or_reference_resolution",
        source_context: source.context(),
        assertion,
        root: final_node,
        metadata,
        source_handle,
        evidence,
        root_support,
    };
    budget.retain(&facet, stop)?;
    Ok(facet)
}

/// Original accepted inline-handler metadata and custody; the borrowed source
/// record contains identity/span metadata only and never a body.
#[derive(Serialize)]
pub(crate) struct PlatformRootXmlHandlerFacet<'m, 'a> {
    selection: PlatformRootFacet,
    interpretation: &'static str,
    source_context: &'a GenerationContext,
    assertion: &'a GraphAssertionRef,
    root: &'a GraphNode,
    handler: &'a ProjectGraphInlineHandler,
    script_source: &'a ProjectGraphScriptSource,
    source_handle: &'a SourceHandle,
    evidence: &'a EvidenceRecord,
    root_support: &'m GraphResolvedExplanation<'m>,
}

pub(crate) fn build_platform_root_xml_handler_facet_on_budget<'m, 'a>(
    map: &'m PlatformMapArtifact<'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<PlatformRootXmlHandlerFacet<'m, 'a>> {
    checkpoint(stop)?;
    let native_view: &'a SkeletonInputView<'a> = map.native_view;
    let (native, accepted, _) = native_view.context_parts();
    let source = native.source();
    let root = map.page().neighborhood().root();
    let root_support = map.page().root_support();
    if root.kind() != "xml_source_handler" {
        return Err(ContextError::new(ContextErrorCode::RootInvalid));
    }
    if source.context().context_id() != accepted.source_context_id()
        || source.context().context_id() != native.scope().source_context_id
    {
        return Err(mismatch());
    }
    let explanation = root_support.explanation();
    if !explanation.support_complete() || !explanation.derivation_complete() {
        return Err(facet_missing());
    }
    if !explanation.conflicts().is_empty() {
        return Err(facet_mismatch());
    }

    // Search only the exact root's original ProducerEntity receipts and native
    // handler records. Reserve the complete cross-product before visiting it.
    let supports = explanation.supports();
    let handlers = source.inline_handlers();
    budget.work(
        handlers
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(supports.len()))
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let depth = |count: usize| count.max(1).ilog2() as usize + 1;
    let address_depth = depth(native.charged_metadata_bytes());
    let mut selected = None;
    for support in supports {
        checkpoint(stop)?;
        let GraphAssertionSupport::ProducerEntity {
            producer,
            proposal,
            accepted: receipt,
            ..
        } = support
        else {
            continue;
        };
        if receipt.node().kind() != root.kind() || receipt.node().owner_key() != root.owner_key() {
            continue;
        }
        for handler in handlers {
            checkpoint(stop)?;
            budget.work(
                handler
                    .proposal_id
                    .len()
                    .checked_add(address_depth)
                    .ok_or_else(exhausted)?,
                stop,
            )?;
            if handler.proposal_id != proposal.proposal_id() {
                continue;
            }
            if !handler.semantic_context.admits_static_source_association() {
                return Err(facet_mismatch());
            }
            budget.retain(handler, stop)?;
            let local = GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: handler.proposal_id.clone().into(),
            };
            let assertion = native.assertion(&local).ok_or_else(facet_missing)?;
            let GraphAssertionRef::Producer {
                partition_id,
                batch_id,
                assertion: retained_local,
            } = assertion
            else {
                return Err(facet_mismatch());
            };
            if partition_id.as_ref() != producer.partition_id
                || batch_id.as_ref() != producer.batch_id
            {
                continue;
            }
            if retained_local != &local
                || receipt.proposal_id() != proposal.proposal_id()
                || proposal.proposal_id() != handler.proposal_id
                || proposal.entity_kind_id() != "xml_source_handler"
                || receipt.node().universe() != &native.scope().universe
                || receipt.node().generation() != &native.scope().generation
            {
                return Err(facet_mismatch());
            }
            if selected
                .replace((handler, assertion, *proposal, *receipt))
                .is_some()
            {
                return Err(facet_mismatch());
            }
        }
    }
    let (handler, assertion, proposal, receipt) = selected.ok_or_else(facet_missing)?;

    // Rebind this original accepted root into the held final candidate snapshot.
    let snapshot = accepted.snapshot();
    let evidence_count = receipt.node().evidence_ids().len();
    let node_work = depth(snapshot.nodes().len())
        .checked_add(8)
        .and_then(|n| {
            evidence_count
                .checked_mul(depth(evidence_count) + 1)
                .and_then(|e| n.checked_add(e))
        })
        .ok_or_else(exhausted)?;
    budget.work(node_work, stop)?;
    let original = receipt.node();
    let node_material = (original, snapshot.universe(), snapshot.generation());
    budget.retain(&node_material, stop)?;
    let node_bytes = encoded_len(&node_material, budget.remaining_context_work(), stop)?;
    budget.work(usize::try_from(node_bytes).map_err(|_| exhausted())?, stop)?;
    let rebound = GraphNode::new(
        snapshot.universe().clone(),
        snapshot.generation().clone(),
        original.kind(),
        original.owner_key(),
        original.evidence_ids().to_vec(),
        snapshot.limits(),
    )
    .map_err(crate::error::graph_error)?;
    let final_node = snapshot.node(rebound.node_id()).ok_or_else(facet_missing)?;
    if !std::ptr::eq(final_node, root) {
        return Err(facet_mismatch());
    }

    let script_sources = source.script_sources();
    budget.work(script_sources.len(), stop)?;
    let script_comparison_work = handler
        .script_id
        .len()
        .checked_add(handler.document.len())
        .and_then(|per_candidate| per_candidate.checked_mul(script_sources.len()))
        .ok_or_else(exhausted)?;
    budget.work(script_comparison_work, stop)?;
    let mut matched_source = None;
    for script_source in script_sources {
        checkpoint(stop)?;
        budget.work(1, stop)?;
        if script_source.script_id == handler.script_id
            && script_source.document == handler.document
            && script_source.source_kind == XmlScriptSource::InlineBody
            && script_source.source_handle_id == handler.source_handle_id
            && script_source.evidence_id == handler.evidence_id
            && matched_source.replace(script_source).is_some()
        {
            return Err(facet_mismatch());
        }
    }
    let script_source = matched_source.ok_or_else(facet_missing)?;

    // Charge both native catalog lookups and all support-membership comparisons
    // before running them; the output keeps only the original borrowed records.
    let handles = source.source_handles();
    let evidence_records = source.evidence();
    budget.work(
        depth(handles.len())
            .checked_add(depth(evidence_records.len()))
            .and_then(|n| n.checked_add(script_source.source_handle_id.as_hash().len()))
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let source_handle = handles
        .get(&handler.source_handle_id)
        .ok_or_else(facet_missing)?;
    let evidence = evidence_records
        .get(&handler.evidence_id)
        .ok_or_else(facet_missing)?;
    let evidence_resolution = root_support.evidence_resolution();
    let contains_work = evidence_resolution
        .source_handles()
        .len()
        .checked_add(evidence_resolution.evidence_records().len())
        .and_then(|n| n.checked_add(proposal.source_handle_ids().len()))
        .and_then(|n| n.checked_add(proposal.evidence_ids().len()))
        .and_then(|n| n.checked_add(evidence.source_handle_ids().len()))
        .ok_or_else(exhausted)?;
    budget.work(contains_work, stop)?;
    let comparison_bytes = encoded_len(
        &(
            handler,
            script_source,
            source_handle,
            evidence,
            evidence_resolution,
        ),
        budget
            .remaining_metadata_bytes()?
            .min(budget.remaining_context_work()),
        stop,
    )?;
    budget.work(
        usize::try_from(comparison_bytes).map_err(|_| exhausted())?,
        stop,
    )?;
    if script_source.span != source_handle.span()
        || source_handle.path().as_str() != handler.document
        || evidence.context_id() != source.context().context_id()
        || !proposal
            .source_handle_ids()
            .contains(&source_handle.handle_id())
        || !proposal.evidence_ids().contains(&evidence.evidence_id())
        || !evidence
            .source_handle_ids()
            .contains(&source_handle.handle_id())
        || !evidence_resolution
            .source_handles()
            .contains(&source_handle)
        || !evidence_resolution.evidence_records().contains(&evidence)
    {
        return Err(facet_mismatch());
    }

    let facet = PlatformRootXmlHandlerFacet {
        selection: PlatformRootFacet::XmlHandlerMetadata,
        interpretation: "original_accepted_root_xml_handler_metadata_no_receiver_or_runtime_resolution",
        source_context: source.context(),
        assertion,
        root: final_node,
        handler,
        script_source,
        source_handle,
        evidence,
        root_support,
    };
    budget.retain(&facet, stop)?;
    Ok(facet)
}

fn facet_missing() -> ContextError {
    ContextError::new(ContextErrorCode::MissingRequiredCustody)
}
fn facet_mismatch() -> ContextError {
    ContextError::new(ContextErrorCode::SupportMismatch)
}

fn remaining(budget: &Budget, limits: PrimaryMapLimits) -> ContextResult<u64> {
    limits
        .max_metadata_bytes
        .checked_sub(budget.retained_bytes)
        .ok_or_else(exhausted)
}
pub(crate) fn encode_retained<T: Serialize + ?Sized>(
    artifact: &T,
    budget: &mut Budget,
    limits: PrimaryMapLimits,
    stop: &AtomicBool,
) -> ContextResult<Box<[u8]>> {
    let before = budget.retained_bytes;
    budget.retain(artifact, stop)?;
    let reserved = budget
        .retained_bytes
        .checked_sub(before)
        .ok_or_else(exhausted)?;
    // This allowance is the exact reservation just committed on the same ledger.
    let bytes = canonical(artifact, reserved, stop)?;
    checkpoint(stop)?;
    // The caller's whole-envelope allowance is still the continuing remainder.
    remaining(budget, limits)?;
    Ok(bytes.into_boxed_slice())
}
fn mismatch() -> ContextError {
    ContextError::new(ContextErrorCode::UniverseMismatch)
}
fn reference_error(error: ReferenceViewError) -> ContextError {
    match error {
        ReferenceViewError::Cancelled => ContextError::new(ContextErrorCode::Cancelled),
        ReferenceViewError::LimitExceeded => exhausted(),
        _ => mismatch(),
    }
}
fn project_error(error: ProjectError) -> ContextError {
    ContextError::new(match error.code() {
        ProjectErrorCode::AnalysisCancelled => ContextErrorCode::Cancelled,
        ProjectErrorCode::SourceBudgetExceeded => {
            ContextErrorCode::MinimumRequiredContentExceedsBudget
        }
        _ => ContextErrorCode::NativeOwnerRefused,
    })
}
