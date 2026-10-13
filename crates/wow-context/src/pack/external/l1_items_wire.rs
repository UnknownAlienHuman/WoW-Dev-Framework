//! EntityV1 item and selection joins over the single typed PackWire decode.
//! This reuses the existing ContextItem and SelectionRecord wire owners;
//! it does not add another decoder or normalize imported item records.

use std::sync::atomic::{AtomicBool, Ordering};

use super::super::l0_items_wire::{ItemPayloadWire, L0ScopeWire};
use super::PrimaryPackAdmission;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum L1ItemTraceError<E> {
    Work(E),
    Cancelled,
    Invalid(&'static str),
}

/// Check the exact writer sequence: native map nodes, then native map edges;
/// each selection preserves its item identity, origin, required reason, and
/// nonexpanding status.
pub(super) fn validate_l1_item_trace(
    pack: &super::PackWire,
    admission: &mut PrimaryPackAdmission<'_>,
    stop: &AtomicBool,
) -> Result<(), L1ItemTraceError<crate::ContextError>> {
    let core = &pack.core;
    let skeleton = core.skeletons.l1.as_ref().ok_or(L1ItemTraceError::Invalid(
        "L1 skeleton required for its item trace",
    ))?;
    let root = &skeleton.core.root;
    let root_scope = &core.request.core.root;
    let project_map_id = skeleton.core.context.origin_map_id.as_str();
    let nodes = &skeleton.core.local_entities;
    let edges = &skeleton.core.direct_relations;
    let universe_set_id = core.universe_set_id.as_str();
    let items = &core.items;
    let trace = &core.selection_trace;
    let total = nodes
        .len()
        .checked_add(edges.len())
        .ok_or(L1ItemTraceError::Invalid("item count overflow"))?;
    if !matches!(root_scope, L0ScopeWire::Entity(_))
        || root_scope != &L0ScopeWire::Entity(root.node_id().clone())
        || items.len() != total
        || trace.len() != total
    {
        return Err(L1ItemTraceError::Invalid(
            "EntityV1 root or item cardinality mismatch",
        ));
    }

    let L0ScopeWire::Entity(root_id) = root_scope else {
        return Err(L1ItemTraceError::Invalid("L1 GraphEntity root required"));
    };
    let mut root_count = 0usize;
    for (index, (item, selected)) in items.iter().zip(trace).enumerate() {
        if stop.load(Ordering::Acquire) {
            return Err(L1ItemTraceError::Cancelled);
        }
        admission.charge_work(1).map_err(L1ItemTraceError::Work)?;
        if &item.core.root != root_scope
            || item.core.schema != "wow-context/context-item/primary/1"
            || item.core.universe_set_id != universe_set_id
            || item.core.origin_map_id != project_map_id
            || selected.origin_map_id != project_map_id
            || selected.item_id != item.context_item_id
            || selected.reason != item.core.inclusion_reason
            || !selected.required
            || selected.expansion != item.core.expansion
            || selected.expansion.is_some()
            || item.core.expansion.is_some()
            || item.core.privacy != "local_metadata_only"
            || item.core.license != "source_inclusion_not_permitted"
            || item.core.source_boundary != "untrusted_native_metadata"
        {
            return Err(L1ItemTraceError::Invalid(
                "L1 item and selection metadata disagree",
            ));
        }

        match (index < nodes.len(), &item.core.payload) {
            (true, ItemPayloadWire::Entity { node }) if node == &nodes[index] => {
                let is_root = node.node_id() == root_id;
                if is_root {
                    root_count = root_count
                        .checked_add(1)
                        .ok_or(L1ItemTraceError::Invalid("root count overflow"))?;
                    if node != root || item.core.inclusion_reason != "exact_native_root" {
                        return Err(L1ItemTraceError::Invalid(
                            "L1 root payload or inclusion reason mismatch",
                        ));
                    }
                } else if item.core.inclusion_reason != "selected_direct_native_neighbor" {
                    return Err(L1ItemTraceError::Invalid(
                        "L1 neighbor inclusion reason mismatch",
                    ));
                }
            }
            (false, ItemPayloadWire::Relation { edge })
                if edge == &edges[index - nodes.len()]
                    && item.core.inclusion_reason
                        == "original_selected_direct_native_assertion" => {}
            _ => {
                return Err(L1ItemTraceError::Invalid(
                    "L1 item payload differs from the selected native map sequence",
                ));
            }
        }
    }
    if root_count != 1 {
        return Err(L1ItemTraceError::Invalid(
            "exact L1 root item cardinality mismatch",
        ));
    }
    Ok(())
}
