//! Complete decode-only model for the selected current writer. None of these
//! records can construct a native owner, acquired view, or semantic pack.
use std::io::{self, BufReader, Read};
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_graph::{GraphEdge, GraphNode, GraphNodeId, GraphRelationKind};

use super::declaration_links::{
    DeclarationLinkState, DeclarationLinksError, check_declaration_links,
};
use super::declaration_wire::DeclarationWire;
use super::l0_items_wire::{
    ContextItemWire, ItemPayloadWire, L0ItemTraceError, L0ScopeWire, SelectionRecordWire,
    validate_l0_item_trace,
};
use super::project_map_wire::{
    MapNavigationWire, PrimaryMapLimitsWire, ProjectMapStateWire, ProjectMapWire,
    ProjectMapWireCheckError, ProjectMapWireOutcome, UniverseBindingWire, check_project_map_links,
};
use super::{
    PrimaryPackAdmission, PrimaryPackDecoderCheck, PrimaryPackValidationCode as Code,
    PrimaryPackValidationState as State,
};
use crate::budget::{canonical, encoded_len, identity};
use crate::{ContextError, ContextErrorCode, ContextResult};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackWire {
    semantic_pack_id: String,
    core: PackCoreWire,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackCoreWire {
    schema: String,
    profile: PackProfileWire,
    universe_set_id: String,
    binding: UniverseBindingWire,
    request: RequestWire,
    boundary_notice: String,
    skeletons: SkeletonsWire,
    items: Vec<ContextItemWire>,
    selection_trace: Vec<SelectionRecordWire>,
    omissions: OmissionsWire,
    budget: PackBudgetWire,
    state: ProjectMapStateWire,
    absence_authoritative: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackProfileWire {
    authority: String,
    boundary: String,
    budgets: String,
    continuation: String,
    deduplication: String,
    item_kinds: Vec<String>,
    layer_order: Vec<String>,
    license: String,
    ordering: String,
    privacy: String,
    profile_id: String,
    selection: String,
    source_bodies: String,
    tokens: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestWire {
    context_request_id: String,
    core: RequestCoreWire,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestCoreWire {
    schema: String,
    universe_set_id: String,
    root: L0ScopeWire,
    level: LevelWire,
    profile_id: String,
    limits: PrimaryMapLimitsWire,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LevelWire {
    L0,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkeletonsWire {
    schema: String,
    level: LevelWire,
    l0: L0Wire,
    work_reserved: u64,
    bytes_reserved: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct L0Wire {
    l0_skeleton_id: String,
    core: L0CoreWire,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct L0CoreWire {
    schema: String,
    profile: String,
    profile_descriptor: SkeletonProfileWire,
    context: SkeletonContextWire,
    scope: ScopeWire,
    direct_members: Vec<GraphNode>,
    selected_member_count: usize,
    member_enumeration: String,
    declarations: Vec<DeclarationWire>,
    direct_relations: Vec<GraphEdge>,
    detail_routes: Vec<GraphNodeId>,
    not_evaluated: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkeletonProfileWire {
    profile_id: String,
    scope_kinds: Vec<String>,
    relations: Vec<GraphRelationKind>,
    direction: String,
    depth: u8,
    declarations: String,
    types: String,
    ordering: String,
    privacy: String,
    source_bodies: String,
    tokens: String,
    continuation: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkeletonContextWire {
    universe_set_id: String,
    origin_map_id: String,
    origins: ProjectMapWire,
    state: ProjectMapStateWire,
    absence_authoritative: bool,
    privacy: String,
    source_bodies: String,
    tokens: String,
    work_reserved: u64,
    facet_bytes_reserved: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ScopeWire {
    Entity { node: GraphNode },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OmissionsWire {
    upstream: Vec<OmissionRefWire>,
    interpretation: String,
    not_evaluated: Vec<String>,
    continuation: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OmissionRefWire {
    producer: String,
    artifact_id: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackBudgetWire {
    max_metadata_bytes: u64,
    max_context_work: u64,
    reservation_basis: String,
    work_reserved: u64,
    bytes_reserved: u64,
    tokens: String,
}

/// Decode once after admission. Full canonical equality prevents serde leaf
/// normalization/defaults from silently repairing identity material.
pub(super) fn check(
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> ContextResult<PrimaryPackDecoderCheck> {
    let Some(input) = admission.decoded_input() else {
        return Ok(result(
            admission,
            None,
            State::NotEvaluated,
            State::NotEvaluated,
            State::NotEvaluated,
            State::NotEvaluated,
            None,
        ));
    };
    let (parsed, work_error) = {
        let mut reader = AdmissionReader {
            input,
            offset: 0,
            admission,
            stop,
            work_error: None,
        };
        let parsed = {
            let mut buffered = BufReader::with_capacity(8192, &mut reader);
            let parsed: Result<PackWire, serde_json::Error> =
                serde_json::from_reader(&mut buffered);
            drop(buffered);
            parsed
        };
        (parsed, reader.work_error.take())
    };
    if let Some(error) = work_error {
        return Err(error);
    }
    let parsed: PackWire = match parsed {
        Ok(parsed) => parsed,
        Err(_) => {
            admission.checkpoint()?;
            return Ok(result(
                admission,
                None,
                State::Invalid,
                State::NotEvaluated,
                State::NotEvaluated,
                State::NotEvaluated,
                Some(Code::WireSchemaMismatch),
            ));
        }
    };
    admission.checkpoint()?;
    let limit = admission.limits().max_metadata_bytes();
    charge_encoded(admission, &parsed, stop)?;
    if canonical(&parsed, limit, stop)? != input {
        return Ok(result(
            admission,
            None,
            State::Invalid,
            State::NotEvaluated,
            State::NotEvaluated,
            State::NotEvaluated,
            Some(Code::WireSchemaMismatch),
        ));
    }
    if !profiles(&parsed, admission, stop)? {
        return Ok(result(
            admission,
            None,
            State::Valid,
            State::Invalid,
            State::NotEvaluated,
            State::NotEvaluated,
            Some(Code::CompiledProfileMismatch),
        ));
    }
    if !digests(&parsed, admission, stop)? {
        return Ok(result(
            admission,
            None,
            State::Valid,
            State::Valid,
            State::Invalid,
            State::NotEvaluated,
            Some(Code::DigestIdentityMismatch),
        ));
    }
    let joined = references(&parsed, admission, stop)?;
    Ok(result(
        admission,
        Some(parsed.semantic_pack_id.into_boxed_str()),
        State::Valid,
        State::Valid,
        State::Valid,
        joined.0,
        joined.1,
    ))
}

/// Charge every buffered source refill to the admission ledger before copying
/// bytes into serde's input buffer. The caller checks the stop flag between
/// refill calls and maps cancellation separately from malformed wire data.
struct AdmissionReader<'a, 'b> {
    input: &'a [u8],
    offset: usize,
    admission: &'b mut PrimaryPackAdmission<'a>,
    stop: &'b AtomicBool,
    work_error: Option<ContextError>,
}

impl Read for AdmissionReader<'_, '_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.stop.load(std::sync::atomic::Ordering::Acquire) {
            return Err(io::Error::other("context cancelled"));
        }
        if output.is_empty() || self.offset == self.input.len() {
            return Ok(0);
        }
        let length = output
            .len()
            .min(8192)
            .min(self.input.len().saturating_sub(self.offset));
        if let Err(error) = self.admission.charge_work(1) {
            self.work_error = Some(error);
            return Err(io::Error::other("context work budget"));
        }
        output[..length].copy_from_slice(&self.input[self.offset..self.offset + length]);
        self.offset += length;
        Ok(length)
    }
}

fn result(
    admission: &PrimaryPackAdmission<'_>,
    id: Option<Box<str>>,
    schema: State,
    profile: State,
    digest: State,
    references: State,
    code: Option<Code>,
) -> PrimaryPackDecoderCheck {
    PrimaryPackDecoderCheck {
        artifact_schema: Some(super::PACK_SCHEMA.into()),
        profile_id: Some(super::PACK_PROFILE.into()),
        selection: admission.selection(),
        schema,
        compiled_profile: profile,
        digest_and_identity: digest,
        internal_references: references,
        semantic_pack_id: id,
        code,
    }
}

fn charge_encoded<T: Serialize + ?Sized>(
    admission: &mut PrimaryPackAdmission<'_>,
    value: &T,
    stop: &AtomicBool,
) -> ContextResult<()> {
    // Each projection is a subrecord of the admitted document. Charge its
    // whole-input upper bound before counting/canonicalizing/hashing it.
    let units = admission
        .decoded_input()
        .map_or(1, |input| input.len().div_ceil(8192).max(1));
    admission.charge_work(units)?;
    encoded_len(value, admission.limits().max_metadata_bytes(), stop)?;
    Ok(())
}

fn same<A: Serialize + ?Sized, B: Serialize + ?Sized>(
    left: &A,
    right: &B,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> ContextResult<bool> {
    charge_encoded(admission, left, stop)?;
    charge_encoded(admission, right, stop)?;
    let limit = admission.limits().max_metadata_bytes();
    Ok(canonical(left, limit, stop)? == canonical(right, limit, stop)?)
}

fn profiles(
    pack: &PackWire,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> ContextResult<bool> {
    let core = &pack.core;
    let l0 = &core.skeletons.l0.core;
    let d = &l0.profile_descriptor;
    if core.schema != super::PACK_SCHEMA
        || !same(&core.profile, &super::super::PACK_PROFILE, admission, stop)?
        || core.request.core.schema != "wow-context/primary-pack-request/1"
        || core.request.core.profile_id != super::PACK_PROFILE
        || core.boundary_notice != super::super::BOUNDARY_NOTICE
        || core.skeletons.schema != "wow-context/primary-skeleton-inspection/1"
        || l0.schema != "wow-context/l0-skeleton/primary/1"
        || l0.profile != crate::PRIMARY_L0_PROFILE
        || d.profile_id != crate::PRIMARY_L0_PROFILE
        || d.scope_kinds != ["project", "source_file", "source_package"]
        || d.relations != crate::profile::RELATIONS
        || d.direction != "both"
        || d.depth != 1
        || d.declarations != "selected_native_callable_and_lua_xml_occurrence_metadata"
        || d.types != "original_selected_native_signature_state_only"
        || d.ordering != "native_node_id_then_native_edge_id"
        || d.privacy != "local_metadata_only"
        || d.source_bodies != "excluded"
        || d.tokens != "unavailable"
        || d.continuation != "not_evaluated"
        || l0.member_enumeration != "selected_outgoing_native_owns_or_contains"
        || l0.not_evaluated
            != [
                "unselected_native_declaration_or_type_recipes",
                "reference_fact_enrichment",
                "control_effect_projection",
                "source_excerpts_and_exact_tokens",
                "deeper_paths_and_retained_continuation",
                "complete_corpus_or_runtime_behavior",
            ]
    {
        return Ok(false);
    }
    let ctx = &l0.context;
    let binding = &core.binding;
    if binding.target_profile.validate().is_err() {
        return Ok(false);
    }
    Ok(binding.schema == "wow-context/primary-universe-binding/1"
        && binding.compatibility_basis == "exact_configured_generation_and_explicit_caller_profile"
        && binding.platform_absence == "profile_excluded_optional_platform_context"
        && ctx.privacy == "local_metadata_only"
        && ctx.source_bodies == "excluded"
        && ctx.tokens == "unavailable"
        && !ctx.absence_authoritative
        && !core.absence_authoritative)
}

fn matches_id<T: Serialize + ?Sized>(
    supplied: &str,
    domain: &'static str,
    prefix: &'static str,
    value: &T,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> ContextResult<bool> {
    charge_encoded(admission, value, stop)?;
    Ok(supplied
        == identity(
            domain,
            prefix,
            value,
            admission.limits().max_metadata_bytes(),
            stop,
        )?
        .as_ref())
}

fn digests(
    pack: &PackWire,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> ContextResult<bool> {
    let core = &pack.core;
    let l0 = &core.skeletons.l0;
    let map = &l0.core.context.origins;
    if !matches_id(
        &core.universe_set_id,
        "wow-context/primary-universe-binding/1",
        "context-universe:sha256:",
        &core.binding,
        admission,
        stop,
    )? || !matches_id(
        &core.request.context_request_id,
        "wow-context/primary-pack-request/1",
        "context-pack-request:sha256:",
        &core.request.core,
        admission,
        stop,
    )? || !matches_id(
        &l0.l0_skeleton_id,
        "wow-context/l0-skeleton/primary/1",
        "l0-skeleton:sha256:",
        &l0.core,
        admission,
        stop,
    )? || !matches_id(
        &map.project_map_id,
        "wow-context/project-map/primary-direct-navigation/1",
        "project-map:sha256:",
        &map.core,
        admission,
        stop,
    )? || !matches_id(
        &pack.semantic_pack_id,
        super::PACK_SCHEMA,
        "context-pack:sha256:",
        core,
        admission,
        stop,
    )? {
        return Ok(false);
    }
    for item in &core.items {
        admission.charge_work(1)?;
        if !matches_id(
            &item.core.payload_digest,
            "wow-context/context-item-payload/1",
            "sha256:",
            &item.core.payload,
            admission,
            stop,
        )? || !matches_id(
            &item.context_item_id,
            "wow-context/context-item/primary/1",
            "context-item:sha256:",
            &item.core,
            admission,
            stop,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn references(
    pack: &PackWire,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> ContextResult<(State, Option<Code>)> {
    let invalid = (State::Invalid, Some(Code::InternalReferenceMismatch));
    let core = &pack.core;
    let l0 = &core.skeletons.l0.core;
    let ctx = &l0.context;
    let map = &ctx.origins;
    let ScopeWire::Entity { node: root } = &l0.scope;
    let L0ScopeWire::Entity(root_id) = &core.request.core.root else {
        return Ok(invalid);
    };
    let MapNavigationWire::Direct(view) = &map.core.navigation else {
        return Ok(invalid);
    };
    let limits = &core.request.core.limits;
    let explain = &limits.explain;
    if crate::PrimaryMapLimits::new(
        limits.neighbors,
        wow_graph::GraphExplainLimits {
            max_scanned_assertions: explain.max_scanned_assertions,
            max_supports: explain.max_supports,
            max_output_bytes: explain.max_output_bytes,
            max_derivation_depth: explain.max_derivation_depth,
        },
        limits.max_direct_edges,
        limits.max_input_bytes,
        limits.max_metadata_bytes,
        limits.max_context_work,
    )
    .is_err()
    {
        return Ok(invalid);
    }
    if root.node_id() != root_id
        || root != &view.root
        || !matches!(root.kind(), "source_file" | "source_package")
        || ctx.universe_set_id != core.universe_set_id
        || core.request.core.universe_set_id != core.universe_set_id
        || map.core.universe_set_id != core.universe_set_id
        || ctx.origin_map_id != map.project_map_id
        || !same(&core.binding, &map.core.binding, admission, stop)?
        || !same(limits, &map.core.budget.limits, admission, stop)?
        || !same(&ctx.state, &map.core.state, admission, stop)?
        || !same(&ctx.state, &core.state, admission, stop)?
        || ctx.work_reserved > core.skeletons.work_reserved
        || core.skeletons.work_reserved > core.budget.work_reserved
        || ctx.facet_bytes_reserved > core.skeletons.bytes_reserved
        || core.skeletons.bytes_reserved > core.budget.bytes_reserved
        || core.budget.work_reserved > limits.max_context_work
        || core.budget.bytes_reserved > limits.max_metadata_bytes
        || core.budget.max_metadata_bytes != limits.max_metadata_bytes
        || core.budget.max_context_work != limits.max_context_work
        || core.budget.reservation_basis != "before_semantic_pack_finalization"
        || core.budget.tokens != "unavailable"
    {
        return Ok(invalid);
    }
    match check_project_map_links(map, root_id, |n| admission.charge_work(n), stop) {
        Ok(ProjectMapWireOutcome::SupportedPrimaryL0) => {}
        Ok(ProjectMapWireOutcome::NotEvaluated { .. }) => {
            return Ok((State::NotEvaluated, Some(Code::UnsupportedLoadCustody)));
        }
        Err(ProjectMapWireCheckError::Invalid(reason)) => {
            let _ = reason;
            return Ok(invalid);
        }
        Err(ProjectMapWireCheckError::Work(error)) => return Err(error),
    }
    // Native Map view nodes are sorted root + adjacent nodes; its edge sequence
    // is already the exact ordered native one-hop sequence.
    let mut nodes: Vec<&GraphNode> = std::iter::once(&view.root)
        .chain(&view.adjacent_nodes)
        .collect();
    admission.charge_work(
        nodes
            .len()
            .checked_mul(nodes.len().max(1).ilog2() as usize + 1)
            .ok_or_else(crate::budget::exhausted)?,
    )?;
    nodes.sort_by(|a, b| a.node_id().cmp(b.node_id()));
    if nodes
        .windows(2)
        .any(|pair| pair[0].node_id() >= pair[1].node_id())
        || core.items.len()
            != nodes
                .len()
                .checked_add(view.edges.len())
                .ok_or_else(crate::budget::exhausted)?
        || l0.selected_member_count != l0.direct_members.len()
        || l0.detail_routes.len() != l0.direct_members.len()
        || l0.direct_relations != view.edges
    {
        return Ok(invalid);
    }
    let mut expected_members = Vec::new();
    for node in &nodes {
        admission.charge_work(
            view.edges
                .len()
                .checked_add(1)
                .ok_or_else(crate::budget::exhausted)?,
        )?;
        if view.edges.iter().any(|edge| {
            edge.from() == root_id
                && edge.to() == node.node_id()
                && matches!(
                    edge.relation(),
                    GraphRelationKind::Owns | GraphRelationKind::Contains
                )
        }) {
            expected_members.push(*node);
        }
    }
    if !l0.direct_members.iter().eq(expected_members)
        || !l0
            .detail_routes
            .iter()
            .eq(l0.direct_members.iter().map(GraphNode::node_id))
    {
        return Ok(invalid);
    }
    for (item, node) in core.items.iter().take(nodes.len()).zip(&nodes) {
        admission.charge_work(1)?;
        if item.core.universe_set_id != core.universe_set_id
            || !matches!(&item.core.payload, ItemPayloadWire::Entity { node: actual } if actual == *node)
        {
            return Ok(invalid);
        }
    }
    for (item, edge) in core.items.iter().skip(nodes.len()).zip(&view.edges) {
        admission.charge_work(1)?;
        if item.core.universe_set_id != core.universe_set_id
            || !matches!(&item.core.payload, ItemPayloadWire::Relation { edge: actual } if actual == edge)
        {
            return Ok(invalid);
        }
    }
    match validate_l0_item_trace(
        &core.request.core.root,
        &map.project_map_id,
        &core.items,
        &core.selection_trace,
        |amount| admission.charge_work(amount),
        stop,
    ) {
        Ok(()) => {}
        Err(L0ItemTraceError::Invalid(_)) => return Ok(invalid),
        Err(L0ItemTraceError::Work(error)) => return Err(error),
        Err(L0ItemTraceError::Cancelled) => {
            return Err(ContextError::new(ContextErrorCode::Cancelled));
        }
    }
    match check_declaration_links(
        &l0.declarations,
        map,
        root,
        &l0.direct_members,
        |n| admission.charge_work(n),
        stop,
    ) {
        Ok(rows)
            if rows
                .iter()
                .any(|row| matches!(row.state, DeclarationLinkState::Unsupported(_))) =>
        {
            return Ok((
                State::NotEvaluated,
                Some(Code::UnsupportedDeclarationFamily),
            ));
        }
        Ok(_) => {}
        Err(DeclarationLinksError::Invalid(_)) => return Ok(invalid),
        Err(DeclarationLinksError::Work(error)) => return Err(error),
        Err(DeclarationLinksError::Cancelled) => {
            return Err(ContextError::new(ContextErrorCode::Cancelled));
        }
    }
    let omissions = &core.omissions;
    if omissions.upstream.len() != 2
        || omissions.upstream[0].producer != "project_map"
        || omissions.upstream[0].artifact_id != map.project_map_id
        || omissions.upstream[1].producer != "l0_skeleton"
        || omissions.upstream[1].artifact_id != core.skeletons.l0.l0_skeleton_id
        || omissions.interpretation != "producer_local_omissions_retained_in_original_artifacts"
        || omissions.continuation != "not_evaluated"
        || omissions.not_evaluated
            != [
                "reference_fact_enrichment",
                "declaration_signature_qualification",
                "control_effect_projection",
                "source_excerpts_and_tokens",
                "deeper_expansion_and_continuation",
                "optional_platform_universe",
                "complete_corpus_and_runtime",
            ]
    {
        return Ok(invalid);
    }
    Ok((State::Valid, None))
}
