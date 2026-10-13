//! Closed EntityV1 L1 wire and StructuralOnly joins for external packs.
//!
//! This decoder projection preserves the native source/evidence and accepted
//! relation records as data. It constructs no native custody or authority.

use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_graph::{GraphEdge, GraphNode, GraphNodeId};

use super::super::declaration_links::{
    DeclarationLinkState, DeclarationLinksError, check_l1_declaration_links,
};
use super::super::declaration_wire::DeclarationWire;
use super::super::l0_items_wire::L0ScopeWire;
use super::super::project_map_wire::{MapNavigationWire, ProjectMapWire};
use super::{PrimaryPackAdmission, SkeletonContextWire, SkeletonProfileWire, same};

/// Exact serialized L1Skeleton wrapper. Unknown fields are rejected.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct L1SkeletonWire {
    pub(super) l1_skeleton_id: String,
    pub(super) core: L1CoreWire,
}

/// Exact EntityV1 L1 core projection. Other profile families carry their own
/// owner data and remain outside this closed wire.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct L1CoreWire {
    pub(super) schema: String,
    pub(super) profile: String,
    pub(super) profile_descriptor: SkeletonProfileWire,
    pub(super) context: SkeletonContextWire,
    pub(super) root: GraphNode,
    pub(super) local_entities: Vec<GraphNode>,
    pub(super) direct_relations: Vec<GraphEdge>,
    pub(super) declarations: Vec<DeclarationWire>,
    pub(super) relation_interpretation: String,
    pub(super) not_evaluated: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum L1PackCheck {
    Valid,
    Invalid,
    UnsupportedDeclarationFamily,
    IncompleteDeclarationSupport,
}

/// The compiled descriptor and omissions form one exact default EntityV1
/// profile. Aliases, partial descriptors, and other L1 recipes do not match.
pub(super) fn entity_v1_profile_matches(skeleton: &L1SkeletonWire) -> bool {
    let core = &skeleton.core;
    let descriptor = &core.profile_descriptor;
    core.schema == "wow-context/l1-skeleton/primary/1"
        && core.profile == crate::PRIMARY_L1_PROFILE
        && descriptor.profile_id == crate::PRIMARY_L1_PROFILE
        && descriptor.scope_kinds == ["native_graph_entity"]
        && descriptor.relations == crate::profile::DETAIL_RELATIONS
        && descriptor.direction == "both"
        && descriptor.depth == 1
        && descriptor.declarations == "selected_native_callable_and_lua_xml_occurrence_metadata"
        && descriptor.types == "original_selected_native_signature_state_only"
        && descriptor.ordering == "native_node_id_then_native_edge_id"
        && descriptor.privacy == "local_metadata_only"
        && descriptor.source_bodies == "excluded"
        && descriptor.tokens == "unavailable"
        && descriptor.continuation == "not_evaluated"
        && core.relation_interpretation == "original_direct_native_assertions_no_transitive_edges"
        && core.not_evaluated
            == [
                "unselected_native_declaration_or_type_recipes",
                "reference_fact_enrichment",
                "control_effect_projection",
                "source_excerpts_and_exact_tokens",
                "deeper_paths_and_retained_continuation",
                "complete_corpus_or_runtime_behavior",
            ]
        && !core.context.absence_authoritative
        && core.context.privacy == "local_metadata_only"
        && core.context.source_bodies == "excluded"
        && core.context.tokens == "unavailable"
}

/// Check the selected L1 skeleton against the exact root and ordered direct
/// neighborhood in its already accepted Primary EntityV1 map. Declaration
/// ownership/evidence is delegated to the shared L1-aware owner join.
pub(super) fn check_entity_v1(
    skeleton: &L1SkeletonWire,
    map: &ProjectMapWire,
    root_id: &GraphNodeId,
    map_id: &str,
    universe_set_id: &str,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> crate::ContextResult<L1PackCheck> {
    let core = &skeleton.core;
    let join_steps = core
        .local_entities
        .len()
        .checked_add(core.direct_relations.len())
        .and_then(|count| count.checked_add(core.declarations.len()))
        .and_then(|count| count.checked_add(1))
        .ok_or_else(crate::budget::exhausted)?;
    admission.charge_work(join_steps)?;
    super::charge_encoded(admission, core, stop)?;
    if !entity_v1_profile_matches(skeleton)
        || core.context.universe_set_id != universe_set_id
        || core.context.origin_map_id != map_id
    {
        return Ok(L1PackCheck::Invalid);
    }

    let MapNavigationWire::Direct(view) = &map.core.navigation else {
        return Ok(L1PackCheck::Invalid);
    };
    if core.root != view.root || core.root.node_id() != root_id {
        return Ok(L1PackCheck::Invalid);
    }

    // The native producer sorts the complete root + adjacent population by ID
    // and retains the direct edge sequence unchanged.
    let Some(expected_entity_count) = view.adjacent_nodes.len().checked_add(1) else {
        return Ok(L1PackCheck::Invalid);
    };
    let root_index = view
        .adjacent_nodes
        .partition_point(|node| node.node_id() < root_id);
    if core.local_entities.len() != expected_entity_count
        || core.local_entities.get(root_index) != Some(&view.root)
        || !core.local_entities[..root_index]
            .iter()
            .eq(&view.adjacent_nodes[..root_index])
        || !core.local_entities[root_index + 1..]
            .iter()
            .eq(&view.adjacent_nodes[root_index..])
        || core.direct_relations != view.edges
    {
        return Ok(L1PackCheck::Invalid);
    }

    match check_l1_declaration_links(
        &core.declarations,
        map,
        &core.root,
        &core.local_entities,
        |amount| admission.charge_work(amount),
        stop,
    ) {
        Ok(rows) => {
            if rows
                .iter()
                .any(|row| matches!(row.state, DeclarationLinkState::Unsupported(_)))
            {
                return Ok(L1PackCheck::UnsupportedDeclarationFamily);
            }
        }
        Err(DeclarationLinksError::Incomplete(_)) => {
            return Ok(L1PackCheck::IncompleteDeclarationSupport);
        }
        Err(DeclarationLinksError::Invalid(_)) => {
            return Ok(L1PackCheck::Invalid);
        }
        Err(DeclarationLinksError::Work(error)) => return Err(error),
        Err(DeclarationLinksError::Cancelled) => {
            return Err(crate::ContextError::new(crate::ContextErrorCode::Cancelled));
        }
    }
    Ok(L1PackCheck::Valid)
}

/// Close the L1 request, item/selection, omissions, and cumulative budget
/// equations. The caller first verifies the exact map via
/// check_l1_project_map_links using this same map and root.
pub(super) fn check_l1_pack_joins(
    pack: &super::PackWire,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> crate::ContextResult<L1PackCheck> {
    let core = &pack.core;
    let Some(skeleton) = core.skeletons.l1.as_ref() else {
        return Ok(L1PackCheck::Invalid);
    };
    let l1 = skeleton;
    if core.skeletons.l0.is_some()
        || !matches!(core.request.core.level, super::LevelWire::L1)
        || !matches!(core.skeletons.level, super::LevelWire::L1)
        || core.request.core.l0_profile_id.is_some()
        || core.request.core.l1_profile_id.is_some()
    {
        return Ok(L1PackCheck::Invalid);
    }
    let L0ScopeWire::Entity(root_id) = &core.request.core.root else {
        return Ok(L1PackCheck::Invalid);
    };
    let ctx = &l1.core.context;
    let map = &ctx.origins;
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
        || core.budget.work_reserved > limits.max_context_work
        || map.core.budget.context_work_reserved > ctx.work_reserved
        || ctx.facet_bytes_reserved > core.skeletons.bytes_reserved
        || core.skeletons.bytes_reserved > core.budget.bytes_reserved
        || core.budget.bytes_reserved > limits.max_metadata_bytes
        || map.core.budget.retained_rows_bytes_reserved > ctx.facet_bytes_reserved
        || core.budget.max_metadata_bytes != limits.max_metadata_bytes
        || core.budget.max_context_work != limits.max_context_work
        || core.budget.reservation_basis != "before_semantic_pack_finalization"
        || core.budget.tokens != "unavailable"
        || ctx.absence_authoritative
        || core.absence_authoritative
    {
        return Ok(L1PackCheck::Invalid);
    }
    match check_entity_v1(
        l1,
        map,
        root_id,
        &map.project_map_id,
        &core.universe_set_id,
        admission,
        stop,
    )? {
        L1PackCheck::Valid => {}
        outcome @ L1PackCheck::Invalid
        | outcome @ L1PackCheck::UnsupportedDeclarationFamily
        | outcome @ L1PackCheck::IncompleteDeclarationSupport => return Ok(outcome),
    }

    match super::l1_items_wire::validate_l1_item_trace(pack, admission, stop) {
        Ok(()) => {}
        Err(super::l1_items_wire::L1ItemTraceError::Invalid(_)) => {
            return Ok(L1PackCheck::Invalid);
        }
        Err(super::l1_items_wire::L1ItemTraceError::Work(error)) => return Err(error),
        Err(super::l1_items_wire::L1ItemTraceError::Cancelled) => {
            return Err(crate::ContextError::new(crate::ContextErrorCode::Cancelled));
        }
    }

    let omissions = &core.omissions;
    if omissions.upstream.len() != 2
        || omissions.upstream[0].producer != "project_map"
        || omissions.upstream[0].artifact_id != map.project_map_id
        || omissions.upstream[1].producer != "l1_skeleton"
        || omissions.upstream[1].artifact_id != skeleton.l1_skeleton_id
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
        return Ok(L1PackCheck::Invalid);
    }
    Ok(L1PackCheck::Valid)
}
