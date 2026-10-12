//! The selected frontier family consumes already-admitted native custody once.
use super::*;
use crate::ContextExpansionStep;
use crate::map::ExpansionItem;

pub const PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE: &str = "wow-context/primary-frontier-pack/1";
pub const PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE: &str =
    "wow-context/primary-frontier-pack-json/1";
pub const PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE: &str =
    "wow-context/primary-frontier-pack-markdown/1";
pub(crate) const FRONTIER_PACK_SCHEMA: &str = "wow-context/semantic-pack/primary-frontier/1";
const FRONTIER_PACK_PROFILE: PackProfile = PackProfile {
    profile_id: PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE,
    deduplication: "original_native_node_or_edge_id_with_all_embedded_expansion_reasons",
    layer_order: &["l1", "expansion"],
    ordering: "original_expansion_item_order_native_node_id_then_native_edge_id",
    selection: "mandatory_exact_root_and_original_expansion_items",
    ..PACK_PROFILE
};
pub(crate) const FRONTIER_JSON_PROFILE: RendererProfile = RendererProfile {
    profile_id: PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE,
    semantic_schema: FRONTIER_PACK_SCHEMA,
    ..JSON_PROFILE
};
pub(crate) const FRONTIER_MARKDOWN_PROFILE: RendererProfile = RendererProfile {
    profile_id: PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE,
    semantic_schema: FRONTIER_PACK_SCHEMA,
    layout: "fixed_typed_native_frontier_metadata_records",
    templates: &[
        "pack_header",
        "pack_profile",
        "universe_binding",
        "pack_request",
        "boundary_notice",
        "native_skeletons",
        "native_frontier_expansion",
        "native_entity_item",
        "native_relation_item",
        "selection_trace",
        "coverage_and_omissions",
        "pack_budget",
    ],
    ..MARKDOWN_PROFILE
};
const FROZEN_PROFILES: &[u8] = include_bytes!("../../e3/native-frontier-pack/profiles.json");

fn validate_frontier_profiles(limit: u64, stop: &AtomicBool) -> ContextResult<()> {
    #[derive(Serialize)]
    struct Profiles {
        json: &'static RendererProfile,
        markdown: &'static RendererProfile,
        pack: &'static PackProfile,
    }
    let bytes = canonical(
        &Profiles {
            json: &FRONTIER_JSON_PROFILE,
            markdown: &FRONTIER_MARKDOWN_PROFILE,
            pack: &FRONTIER_PACK_PROFILE,
        },
        limit,
        stop,
    )?;
    // The owned fixture's final LF is file framing, outside descriptor bytes.
    if bytes
        != FROZEN_PROFILES
            .strip_suffix(b"\n")
            .unwrap_or(FROZEN_PROFILES)
    {
        return Err(encoding());
    }
    Ok(())
}

/// Move the exact artifact, original skeletons and actual private ledger into
/// one native pack. No graph operation or skeleton reconstruction occurs here.
pub fn build_context_semantic_pack_from_expansion<'m, 'a: 'm>(
    universe: &'m ContextUniverseSet<'a>,
    root: &'m L0Scope,
    step: ContextExpansionStep<'a>,
    stop: &AtomicBool,
) -> ContextResult<ContextSemanticPack<'m, 'a>> {
    let (expansion, skeletons, mut budget) = step.into_pack_parts(universe, root, stop)?;
    validate_frontier_profiles(universe.limits.max_metadata_bytes, stop)?;
    budget.retain(expansion.expansion_id(), stop)?;
    let request_core = PackRequestCore {
        schema: "wow-context/primary-pack-request/1",
        universe_set_id: universe.universe_set_id(),
        root,
        level: SkeletonLevel::L1,
        profile_id: PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE,
        limits: universe.limits,
        l0_profile_id: None,
        l1_profile_id: None,
        expansion_id: Some(expansion.expansion_id().into()),
    };
    budget.retain(&request_core, stop)?;
    let request = PackRequest {
        context_request_id: identity(
            request_core.schema,
            "context-pack-request:sha256:",
            &request_core,
            universe.limits.max_metadata_bytes,
            stop,
        )?,
        core: request_core,
    };
    budget.work(expansion.items().len(), stop)?;
    let mut items = Vec::new();
    for (item_index, original) in expansion.items().iter().enumerate() {
        checkpoint(stop)?;
        let payload = match original {
            ExpansionItem::Entity { node, .. } => ContextItemPayload::Entity { node },
            ExpansionItem::Relation { edge, .. } => ContextItemPayload::Relation { edge },
        };
        let origin_map_id = expansion.origins().project_map_id();
        budget.retain(origin_map_id, stop)?;
        budget.retain(expansion.expansion_id(), stop)?;
        items.push(item(
            universe,
            ItemOrigin {
                map_id: Cow::Owned(origin_map_id.into()),
                reason: "exact_embedded_expansion_item",
                expansion: Some(ExpansionItemOrigin {
                    expansion_id: expansion.expansion_id().into(),
                    item_index,
                }),
            },
            root,
            payload,
            &mut budget,
            stop,
        )?);
    }
    let selection_trace = build_selection_trace(&items, &mut budget, stop)?;
    let mut upstream = Vec::new();
    #[derive(Serialize)]
    struct OmissionRef<'p> {
        producer: &'static str,
        artifact_id: &'p str,
    }
    for (producer, id) in [
        ("project_map", Some(expansion.origins().project_map_id())),
        ("native_frontier_expansion", Some(expansion.expansion_id())),
        ("l1_skeleton", skeletons.l1_id()),
    ] {
        budget.work(1, stop)?;
        if let Some(artifact_id) = id {
            budget.retain(
                &OmissionRef {
                    producer,
                    artifact_id,
                },
                stop,
            )?;
            upstream.push(UpstreamOmissionRef {
                producer,
                artifact_id: artifact_id.into(),
            });
        }
    }
    let state = expansion.state();
    let core = PackCore {
        schema: FRONTIER_PACK_SCHEMA,
        profile: &FRONTIER_PACK_PROFILE,
        universe_set_id: universe.universe_set_id(),
        binding: &universe.binding,
        request,
        boundary_notice: BOUNDARY_NOTICE,
        skeletons,
        expansion: Some(expansion),
        items,
        selection_trace,
        omissions: PackOmissions {
            upstream,
            interpretation: "producer_local_omissions_retained_in_original_artifacts",
            not_evaluated: &[
                "reference_fact_enrichment",
                "declaration_signature_qualification",
                "control_effect_projection",
                "source_excerpts_and_tokens",
                "further_rounds_and_continuation",
                "optional_platform_universe",
                "complete_corpus_and_runtime",
            ],
            continuation: "not_evaluated",
        },
        budget: PackBudget {
            max_metadata_bytes: universe.limits.max_metadata_bytes,
            max_context_work: universe.limits.max_context_work,
            reservation_basis: "before_semantic_pack_finalization",
            work_reserved: budget.work,
            bytes_reserved: budget.retained_bytes,
            tokens: "unavailable",
        },
        state,
        absence_authoritative: false,
    };
    finalize_pack(universe, core, budget, stop)
}

pub(super) fn validate_expansion(
    pack: &ContextSemanticPack<'_, '_>,
    stop: &AtomicBool,
) -> ContextResult<()> {
    let Some(expansion) = &pack.core.expansion else {
        if pack.core.schema != PACK_SCHEMA
            || pack.core.profile.profile_id != PRIMARY_CONTEXT_PACK_PROFILE
        {
            return Err(encoding());
        }
        return Ok(());
    };
    if pack.core.schema != FRONTIER_PACK_SCHEMA
        || pack.core.profile.profile_id != PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE
        || pack.core.request.core.expansion_id.as_deref() != Some(expansion.expansion_id())
        || pack.core.items.len() != expansion.items().len()
        || !open_project_map_view(expansion.origins(), stop)?.bound_to(pack.universe)
        || canonical(expansion, pack.render_limit(), stop)?.as_slice()
            != expansion.canonical_bytes()
    {
        return Err(encoding());
    }
    for (index, (selected, original)) in pack.core.items.iter().zip(expansion.items()).enumerate() {
        checkpoint(stop)?;
        let Some(origin) = &selected.core.expansion else {
            return Err(encoding());
        };
        let native = match (&selected.core.payload, original) {
            (ContextItemPayload::Entity { node }, ExpansionItem::Entity { node: native, .. }) => {
                std::ptr::eq(*node, *native)
            }
            (
                ContextItemPayload::Relation { edge },
                ExpansionItem::Relation { edge: native, .. },
            ) => std::ptr::eq(*edge, *native),
            _ => false,
        };
        let trace = &pack.core.selection_trace[index];
        if !native
            || origin.expansion_id.as_ref() != expansion.expansion_id()
            || origin.item_index != index
            || selected.core.origin_map_id != expansion.origins().project_map_id()
            || trace.item_id.as_ref() != selected.context_item_id()
            || trace.origin_map_id != selected.core.origin_map_id
            || trace.expansion.as_ref().is_none_or(|item| {
                item.expansion_id != origin.expansion_id || item.item_index != index
            })
        {
            return Err(encoding());
        }
    }
    Ok(())
}
