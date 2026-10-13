//! Strict wire fragment for the finite Primary L0 GraphEntity pack item layer.
//!
//! The enclosing external validator checks the full pack, ProjectMap, skeleton,
//! identities and references. This module owns only the item/selection joins.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

use wow_graph::{GraphEdge, GraphNode};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum L0ItemTraceError<E> {
    Work(E),
    Cancelled,
    Invalid(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum L0ScopeWire {
    Project(wow_project::ProjectId),
    Entity(wow_graph::GraphNodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ItemPayloadWire {
    Entity { node: GraphNode },
    Relation { edge: GraphEdge },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemCoreWire {
    pub schema: String,
    pub universe_set_id: String,
    pub root: L0ScopeWire,
    pub origin_map_id: String,
    pub payload: ItemPayloadWire,
    pub payload_digest: String,
    pub inclusion_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expansion: Option<ExpansionOriginWire>,
    pub privacy: String,
    pub license: String,
    pub source_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpansionOriginWire {
    pub expansion_id: String,
    pub item_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextItemWire {
    pub context_item_id: String,
    pub core: ItemCoreWire,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionRecordWire {
    pub item_id: String,
    pub origin_map_id: String,
    pub reason: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expansion: Option<ExpansionOriginWire>,
}

/// Validates the exact current item/trace identities and their shared selection
/// links. Call only after strict recursive decoding; this does not validate the
/// containing ProjectMap, native support receipts, or L0 skeleton identity.
pub(crate) fn validate_l0_item_trace<E>(
    root: &L0ScopeWire,
    project_map_id: &str,
    items: &[ContextItemWire],
    trace: &[SelectionRecordWire],
    mut reserve_work: impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), L0ItemTraceError<E>> {
    if !matches!(root, L0ScopeWire::Entity(_)) || items.len() != trace.len() {
        return Err(L0ItemTraceError::Invalid(
            "wrong root kind or selection cardinality",
        ));
    }

    let L0ScopeWire::Entity(root_id) = root else {
        return Err(L0ItemTraceError::Invalid("L0 GraphEntity root required"));
    };
    let mut root_count = 0usize;
    for (item, selected) in items.iter().zip(trace) {
        if stop.load(Ordering::Acquire) {
            return Err(L0ItemTraceError::Cancelled);
        }
        reserve_work(1).map_err(L0ItemTraceError::Work)?;
        if &item.core.root != root
            || item.core.schema != "wow-context/context-item/primary/1"
            || item.core.origin_map_id != project_map_id
            || selected.origin_map_id != project_map_id
            || selected.item_id != item.context_item_id
            || selected.reason != item.core.inclusion_reason
            || !selected.required
            || selected.expansion != item.core.expansion
            || item.core.expansion.is_some()
            || item.core.privacy != "local_metadata_only"
            || item.core.license != "source_inclusion_not_permitted"
            || item.core.source_boundary != "untrusted_native_metadata"
        {
            return Err(L0ItemTraceError::Invalid(
                "item and selection trace disagree",
            ));
        }

        match &item.core.payload {
            ItemPayloadWire::Entity { node } if node.node_id() == root_id => {
                root_count = root_count
                    .checked_add(1)
                    .ok_or(L0ItemTraceError::Invalid("root count overflow"))?;
                if !matches!(node.kind(), "source_file" | "source_package")
                    || item.core.inclusion_reason != "exact_native_root"
                {
                    return Err(L0ItemTraceError::Invalid(
                        "root entity kind or inclusion reason is invalid",
                    ));
                }
            }
            ItemPayloadWire::Entity { .. }
                if item.core.inclusion_reason != "selected_direct_native_neighbor" =>
            {
                return Err(L0ItemTraceError::Invalid(
                    "neighbor entity has the wrong inclusion reason",
                ));
            }
            ItemPayloadWire::Relation { .. }
                if item.core.inclusion_reason != "original_selected_direct_native_assertion" =>
            {
                return Err(L0ItemTraceError::Invalid(
                    "relation has the wrong inclusion reason",
                ));
            }
            _ => {}
        }
    }
    if root_count != 1 {
        return Err(L0ItemTraceError::Invalid(
            "exact root item cardinality mismatch",
        ));
    }
    Ok(())
}
