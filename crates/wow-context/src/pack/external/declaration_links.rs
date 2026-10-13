//! Bounded structural joins for default Primary L0 declaration rows.
//!
//! Caller contract: invoke only after strict single-pass decode and after
//! `check_project_map_links` has accepted `map`. Pass the exact checked L0 root
//! and direct-member node slice, plus the caller's existing cumulative work
//! reservation callback and cancellation flag. This module creates no native
//! owner, custody, parser, or work allowance. A `Supported` result is only a
//! structural join over decoded bytes; it does not restore native authority.

use std::sync::atomic::{AtomicBool, Ordering};

use wow_core::{EvidenceId, SourceHandle, StableHandleId};
use wow_graph::{
    GraphEntityProposal, GraphLimits, GraphNode, GraphProposalEndpoint, GraphProposalValue,
};

use super::declaration_wire::{
    DeclarationWire, DefaultPrimaryL0Admission, ProjectCallableFacetStateWire,
    ProjectCallableMetadataWire, ProjectCallableScopeWire, SourceDeclaredParameterWire,
    SourceFunctionDeclarationFormWire, SourceFunctionDeclarationWire, SourceFunctionKindWire,
    SourceFunctionSignatureWire,
};
use super::project_map_wire::{
    AssertionSupportWire, MapNavigationWire, NeighborViewWire, ProjectMapWire, ResolvedSupportWire,
    project_node_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnsupportedDeclarationFamily {
    XmlHandler,
    StateRoot,
    StatePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeclarationLinkState {
    /// The index addresses the original decoded declaration; the row itself is
    /// deliberately retained by the caller without normalization or rewriting.
    Supported,
    /// The current default L0 profile does not consume this writer family.
    Unsupported(UnsupportedDeclarationFamily),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclarationLink {
    pub declaration_index: usize,
    pub node_id: wow_graph::GraphNodeId,
    pub state: DeclarationLinkState,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DeclarationLinksError<E> {
    Work(E),
    Cancelled,
    Invalid(&'static str),
}

/// Validate one selected root's declaration rows against its checked map.
///
/// Every scan step first checks cancellation and reserves one unit from the
/// caller's continuing ledger. `direct_members` must be the already checked
/// native member set for this exact root; the function independently confirms
/// each member against the checked map's original adjacent GraphNodes.
pub(crate) fn check_declaration_links<E>(
    declarations: &[DeclarationWire],
    map: &ProjectMapWire,
    root: &GraphNode,
    direct_members: &[GraphNode],
    mut reserve_work: impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<Vec<DeclarationLink>, DeclarationLinksError<E>> {
    let view = root_view(map, root, &mut reserve_work, stop)?;
    let mut output = Vec::with_capacity(declarations.len());
    let mut previous: Option<&wow_graph::GraphNodeId> = None;
    for (index, declaration) in declarations.iter().enumerate() {
        step(&mut reserve_work, stop)?;
        let node = declaration_node(declaration);
        if previous.is_some_and(|prior| prior >= node.node_id()) {
            return Err(DeclarationLinksError::Invalid(
                "declaration nodes are duplicated or not in native node order",
            ));
        }
        previous = Some(node.node_id());

        match declaration.default_primary_l0_admission() {
            DefaultPrimaryL0Admission::UnsupportedProfileVariant => {
                output.push(DeclarationLink {
                    declaration_index: index,
                    node_id: node.node_id().clone(),
                    state: DeclarationLinkState::Unsupported(unsupported_family(declaration)),
                });
            }
            DefaultPrimaryL0Admission::SupportedMetadata => {
                check_supported_declaration(
                    declaration,
                    map,
                    view,
                    root,
                    direct_members,
                    &mut reserve_work,
                    stop,
                )?;
                output.push(DeclarationLink {
                    declaration_index: index,
                    node_id: node.node_id().clone(),
                    state: DeclarationLinkState::Supported,
                });
            }
        }
    }
    validate_callable_parent_links(declarations, &mut reserve_work, stop)?;
    Ok(output)
}

fn step<E>(
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    if stop.load(Ordering::Acquire) {
        return Err(DeclarationLinksError::Cancelled);
    }
    reserve_work(1).map_err(DeclarationLinksError::Work)
}

fn root_view<'a, E>(
    map: &'a ProjectMapWire,
    root: &GraphNode,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<&'a NeighborViewWire, DeclarationLinksError<E>> {
    let mut found: Option<&NeighborViewWire> = None;
    match &map.core.navigation {
        MapNavigationWire::Direct(view) => {
            step(reserve_work, stop)?;
            if view.root.node_id() == root.node_id() {
                found = Some(view);
            }
        }
        MapNavigationWire::Project(project) => {
            for view in &project.neighborhoods {
                step(reserve_work, stop)?;
                if view.root.node_id() == root.node_id() && found.replace(view).is_some() {
                    return Err(DeclarationLinksError::Invalid(
                        "selected root has multiple ProjectMap neighborhoods",
                    ));
                }
            }
        }
    }
    let view = found.ok_or(DeclarationLinksError::Invalid(
        "selected root is absent from checked ProjectMap",
    ))?;
    if &view.root != root
        || root.universe() != &map.core.binding.graph_universe
        || root.generation() != &map.core.binding.graph_generation
    {
        return Err(DeclarationLinksError::Invalid(
            "selected root node identity differs from ProjectMap binding",
        ));
    }
    validate_graph_node(root, reserve_work, stop)?;
    Ok(view)
}

fn declaration_node(row: &DeclarationWire) -> &GraphNode {
    match row {
        DeclarationWire::Callable { node, .. }
        | DeclarationWire::Lua { node, .. }
        | DeclarationWire::Xml { node, .. }
        | DeclarationWire::XmlHandler { node, .. }
        | DeclarationWire::StateRoot { node, .. }
        | DeclarationWire::StatePath { node, .. } => node,
    }
}

fn unsupported_family(row: &DeclarationWire) -> UnsupportedDeclarationFamily {
    match row {
        DeclarationWire::XmlHandler { .. } => UnsupportedDeclarationFamily::XmlHandler,
        DeclarationWire::StateRoot { .. } => UnsupportedDeclarationFamily::StateRoot,
        DeclarationWire::StatePath { .. } => UnsupportedDeclarationFamily::StatePath,
        // The admission match above makes these unreachable in normal flow;
        // mapping them conservatively cannot accidentally admit the row.
        DeclarationWire::Callable { .. }
        | DeclarationWire::Lua { .. }
        | DeclarationWire::Xml { .. } => UnsupportedDeclarationFamily::StateRoot,
    }
}

fn check_supported_declaration<E>(
    row: &DeclarationWire,
    map: &ProjectMapWire,
    view: &NeighborViewWire,
    root: &GraphNode,
    direct_members: &[GraphNode],
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    let node = declaration_node(row);
    let (proposal_id, ownership_id, handle_id, evidence_id, expected_kind, key) = match row {
        DeclarationWire::Callable { metadata, .. } => {
            let function = &metadata.function;
            if function.function_id.is_empty()
                || function.path.is_empty()
                || metadata.function_analysis_id.is_empty()
                || !callable_facets_are_preserved_coherently(metadata)
                || !callable_declaration_matches_kind(function.kind, metadata)
                || !matches!(
                    (
                        function.kind,
                        metadata.scope,
                        metadata.parent_function_id.as_deref()
                    ),
                    (
                        SourceFunctionKindWire::Chunk,
                        ProjectCallableScopeWire::Chunk,
                        None
                    ) | (
                        SourceFunctionKindWire::Closure,
                        ProjectCallableScopeWire::ChunkScopeClosure
                            | ProjectCallableScopeWire::NestedClosure,
                        Some(_)
                    )
                )
            {
                return Err(DeclarationLinksError::Invalid(
                    "callable metadata identity, scope, or facet state is inconsistent",
                ));
            }
            validate_callable_spans(function.span, metadata, reserve_work, stop)?;
            let handle = support_handle(map, &function.source_handle_id, reserve_work, stop)?;
            if handle.path().as_str() != function.path
                || handle.span() != function.span
                || handle.content_digest() != &metadata.content_digest
            {
                return Err(DeclarationLinksError::Invalid(
                    "callable origin path, span, or content digest differs from its source handle",
                ));
            }
            (
                function.proposal_id.as_str(),
                function.ownership_proposal_id.as_str(),
                function.source_handle_id,
                function.evidence_id,
                "lua_source_function",
                EntityKeyCheck::Callable {
                    path: &function.path,
                    function_id: &function.function_id,
                },
            )
        }
        DeclarationWire::Lua { origin, .. } => {
            if origin.path.is_empty() || origin.declaration_id.is_empty() {
                return Err(DeclarationLinksError::Invalid(
                    "Lua declaration identity is empty",
                ));
            }
            let handle = support_handle(map, &origin.source_handle_id, reserve_work, stop)?;
            if handle.path().as_str() != origin.path || handle.span() != origin.span {
                return Err(DeclarationLinksError::Invalid(
                    "Lua declaration path or span differs from its source handle",
                ));
            }
            (
                origin.proposal_id.as_str(),
                origin.ownership_proposal_id.as_str(),
                origin.source_handle_id,
                origin.evidence_id,
                "lua_source_declaration",
                EntityKeyCheck::Lua {
                    path: &origin.path,
                    span: origin.span,
                },
            )
        }
        DeclarationWire::Xml { origin, .. } => {
            if origin.path.is_empty() || origin.occurrence_id.is_empty() {
                return Err(DeclarationLinksError::Invalid(
                    "XML declaration identity is empty",
                ));
            }
            let handle = support_handle(map, &origin.source_handle_id, reserve_work, stop)?;
            if handle.path().as_str() != origin.path {
                return Err(DeclarationLinksError::Invalid(
                    "XML declaration path differs from its source handle",
                ));
            }
            (
                origin.proposal_id.as_str(),
                origin.ownership_proposal_id.as_str(),
                origin.source_handle_id,
                origin.evidence_id,
                "xml_source_declaration",
                EntityKeyCheck::Xml {
                    path: &origin.path,
                    occurrence_id: &origin.occurrence_id,
                },
            )
        }
        DeclarationWire::XmlHandler { .. }
        | DeclarationWire::StateRoot { .. }
        | DeclarationWire::StatePath { .. } => {
            return Err(DeclarationLinksError::Invalid(
                "unsupported declaration reached supported join path",
            ));
        }
    };

    if node.kind() != expected_kind
        || node.universe() != &map.core.binding.graph_universe
        || node.generation() != &map.core.binding.graph_generation
        || !single_evidence(node, evidence_id)
    {
        return Err(DeclarationLinksError::Invalid(
            "declaration GraphNode kind, binding, or original evidence differs",
        ));
    }
    validate_graph_node(node, reserve_work, stop)?;
    if !contains_equal_node(&view.adjacent_nodes, node, reserve_work, stop)?
        || !contains_equal_node(direct_members, node, reserve_work, stop)?
    {
        return Err(DeclarationLinksError::Invalid(
            "declaration node is not an exact checked direct member",
        ));
    }

    let mut direct_ownership_edge = None;
    for edge in &view.edges {
        step(reserve_work, stop)?;
        if edge.relation() == wow_graph::GraphRelationKind::Owns
            && edge.from() == root.node_id()
            && edge.to() == node.node_id()
            && direct_ownership_edge.replace(edge).is_some()
        {
            return Err(DeclarationLinksError::Invalid(
                "declaration has duplicate direct ownership edges",
            ));
        }
    }
    let ownership_edge = direct_ownership_edge.ok_or(DeclarationLinksError::Invalid(
        "declaration has no direct native Owns edge from the selected root",
    ))?;

    check_entity_support(
        map,
        proposal_id,
        node,
        (expected_kind, &key),
        (handle_id, evidence_id),
        reserve_work,
        stop,
    )?;
    check_ownership_support(
        map,
        (ownership_id, proposal_id),
        (root, node, ownership_edge),
        (handle_id, evidence_id),
        reserve_work,
        stop,
    )?;
    check_evidence_pair(map, handle_id, evidence_id, reserve_work, stop)?;
    Ok(())
}

enum EntityKeyCheck<'a> {
    Callable {
        path: &'a str,
        function_id: &'a str,
    },
    Lua {
        path: &'a str,
        span: wow_core::SourceSpan,
    },
    Xml {
        path: &'a str,
        occurrence_id: &'a str,
    },
}

fn callable_facets_are_preserved_coherently(metadata: &ProjectCallableMetadataWire) -> bool {
    use ProjectCallableFacetStateWire as State;
    let declaration_ok = match metadata.declaration.state {
        State::NotSelected => {
            metadata.declaration.recipe.is_none() && metadata.declaration.observation.is_none()
        }
        State::Observed => {
            matches!(
                metadata.declaration.observation.as_ref(),
                Some(SourceFunctionDeclarationWire::Observed { .. })
            ) && metadata.declaration.recipe.is_some()
        }
        State::Unavailable => {
            matches!(
                metadata.declaration.observation.as_ref(),
                Some(SourceFunctionDeclarationWire::Unavailable { .. })
            ) && metadata.declaration.recipe.is_some()
        }
        State::Unsupported => {
            matches!(
                metadata.declaration.observation.as_ref(),
                Some(SourceFunctionDeclarationWire::Unsupported { .. })
            ) && metadata.declaration.recipe.is_some()
        }
        // The native Project writer has no Partial declaration state.
        State::Partial => false,
    };
    let signature_ok = match (
        &metadata.signature.state,
        metadata.signature.observation.as_ref(),
    ) {
        (State::NotSelected, None) => metadata.signature.recipe.is_none(),
        (
            State::Observed,
            Some(SourceFunctionSignatureWire::Observed {
                unavailable_reasons,
                ..
            }),
        ) => metadata.signature.recipe.is_some() && unavailable_reasons.is_empty(),
        (
            State::Partial,
            Some(SourceFunctionSignatureWire::Observed {
                unavailable_reasons,
                ..
            }),
        ) => metadata.signature.recipe.is_some() && !unavailable_reasons.is_empty(),
        (
            State::Unavailable,
            Some(SourceFunctionSignatureWire::Unavailable {
                reason:
                    super::declaration_wire::SourceSignatureUnavailableWire::MissingSignatureIndex,
            }),
        ) => metadata.signature.recipe.is_some(),
        // The native Project writer never emits Unsupported for signatures.
        _ => false,
    };
    declaration_ok && signature_ok
}

fn callable_declaration_matches_kind(
    kind: SourceFunctionKindWire,
    metadata: &ProjectCallableMetadataWire,
) -> bool {
    match (kind, metadata.declaration.observation.as_ref()) {
        (
            SourceFunctionKindWire::Chunk,
            Some(SourceFunctionDeclarationWire::Unavailable {
                reason:
                    super::declaration_wire::SourceFunctionDeclarationUnavailableWire::ChunkHasNoDeclaration,
            }),
        ) => true,
        (
            SourceFunctionKindWire::Chunk,
            Some(SourceFunctionDeclarationWire::Observed { .. }),
        ) => false,
        (
            SourceFunctionKindWire::Closure,
            Some(SourceFunctionDeclarationWire::Unavailable {
                reason:
                    super::declaration_wire::SourceFunctionDeclarationUnavailableWire::ChunkHasNoDeclaration,
            }),
        ) => false,
        _ => true,
    }
}

fn validate_callable_spans<E>(
    function_span: wow_core::SourceSpan,
    metadata: &ProjectCallableMetadataWire,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    let Some(SourceFunctionDeclarationWire::Observed {
        form,
        colon_definition,
        declaration_span,
        parameters,
    }) = metadata.declaration.observation.as_ref()
    else {
        return Ok(());
    };
    let (Some(function_start), Some(function_end)) =
        (function_span.byte_start(), function_span.byte_end())
    else {
        return Err(DeclarationLinksError::Invalid(
            "observed callable declaration requires an exact function byte span",
        ));
    };
    let (Some(declaration_start), Some(declaration_end)) =
        (declaration_span.byte_start(), declaration_span.byte_end())
    else {
        return Err(DeclarationLinksError::Invalid(
            "observed callable declaration requires an exact declaration byte span",
        ));
    };
    if function_start >= function_end
        || declaration_start >= declaration_end
        || declaration_start > function_start
        || function_end > declaration_end
        || (*colon_definition && *form != SourceFunctionDeclarationFormWire::NamedFunction)
        || (*form == SourceFunctionDeclarationFormWire::ClosureExpression
            && *declaration_span != function_span)
    {
        return Err(DeclarationLinksError::Invalid(
            "callable declaration observation does not contain its exact function span",
        ));
    }
    let mut previous_end = function_start;
    let mut vararg_seen = false;
    for parameter in parameters {
        step(reserve_work, stop)?;
        let (name, span, is_vararg) = match parameter {
            SourceDeclaredParameterWire::Name { name, span } => (Some(name), *span, false),
            SourceDeclaredParameterWire::Vararg { span } => (None, *span, true),
        };
        let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
            return Err(DeclarationLinksError::Invalid(
                "callable parameter requires an exact byte span",
            ));
        };
        if start < previous_end
            || start >= end
            || end > function_end
            || vararg_seen
            || name.is_some_and(|name| name.is_empty() || (end - start) as usize != name.len())
            || is_vararg && end - start != 3
        {
            return Err(DeclarationLinksError::Invalid(
                "callable parameter span or order is invalid for its function declaration",
            ));
        }
        previous_end = end;
        vararg_seen |= is_vararg;
    }
    Ok(())
}

fn contains_equal_node<E>(
    nodes: &[GraphNode],
    expected: &GraphNode,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<bool, DeclarationLinksError<E>> {
    let mut found = false;
    for node in nodes {
        step(reserve_work, stop)?;
        if node.node_id() == expected.node_id() {
            if found || node != expected {
                return Err(DeclarationLinksError::Invalid(
                    "same GraphNode identity has conflicting or duplicate records",
                ));
            }
            found = true;
        }
    }
    Ok(found)
}

fn support_handle<'a, E>(
    map: &'a ProjectMapWire,
    id: &StableHandleId,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<&'a SourceHandle, DeclarationLinksError<E>> {
    let mut found = None;
    for receipt in &map.core.supports {
        step(reserve_work, stop)?;
        for handle in &receipt.evidence_resolution.source_handles {
            step(reserve_work, stop)?;
            if handle.handle_id() == *id {
                if found.is_some_and(|previous| previous != handle) {
                    return Err(DeclarationLinksError::Invalid(
                        "source-handle ID resolves to conflicting native support records",
                    ));
                }
                step(reserve_work, stop)?;
                handle.validate().map_err(|_| {
                    DeclarationLinksError::Invalid("declaration source handle identity is invalid")
                })?;
                found = Some(handle);
            }
        }
    }
    found.ok_or(DeclarationLinksError::Invalid(
        "declaration source handle is absent from ProjectMap support",
    ))
}

fn check_entity_support<E>(
    map: &ProjectMapWire,
    proposal_id: &str,
    node: &GraphNode,
    kind_and_key: (&str, &EntityKeyCheck<'_>),
    support_ids: (StableHandleId, EvidenceId),
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    let (kind, key) = kind_and_key;
    let (handle_id, evidence_id) = support_ids;
    let mut matches = 0usize;
    for receipt in &map.core.supports {
        step(reserve_work, stop)?;
        for support in &receipt.explanation.supports {
            step(reserve_work, stop)?;
            if let AssertionSupportWire::ProducerEntity {
                proposal, accepted, ..
            } = support
                && proposal.proposal_id() == proposal_id
            {
                reserve_work(accepted.node().evidence_ids().len().saturating_add(1))
                    .map_err(DeclarationLinksError::Work)?;
                matches = matches
                    .checked_add(1)
                    .ok_or(DeclarationLinksError::Invalid(
                        "entity support count overflow",
                    ))?;
                if proposal.entity_kind_id() != kind
                    || accepted.proposal_id() != proposal_id
                    || !project_node_id(accepted.node(), map).is_ok_and(|id| &id == node.node_id())
                    || !single_handle(proposal.source_handle_ids(), handle_id)
                    || !single_evidence_ids(proposal.evidence_ids(), evidence_id)
                    || !entity_key_matches(proposal, key)
                {
                    return Err(DeclarationLinksError::Invalid(
                        "entity proposal or accepted native node differs from declaration origin",
                    ));
                }
                check_support_evidence_pair(receipt, handle_id, evidence_id, reserve_work, stop)?;
            }
        }
    }
    if matches != 1 {
        return Err(DeclarationLinksError::Invalid(
            "declaration does not resolve to exactly one producer entity support",
        ));
    }
    Ok(())
}

fn entity_key_matches(proposal: &GraphEntityProposal, key: &EntityKeyCheck<'_>) -> bool {
    let values = proposal.semantic_key();
    match key {
        EntityKeyCheck::Callable { path, function_id } => {
            values.len() == 2
                && values
                    .get("document")
                    .is_some_and(|v| v == &GraphProposalValue::String((*path).into()))
                && values
                    .get("function")
                    .is_some_and(|v| v == &GraphProposalValue::String((*function_id).into()))
        }
        EntityKeyCheck::Lua { path, span } => {
            let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
                return false;
            };
            let (Ok(start), Ok(end)) = (i64::try_from(start), i64::try_from(end)) else {
                return false;
            };
            values.len() == 3
                && values
                    .get("document")
                    .is_some_and(|v| v == &GraphProposalValue::String((*path).into()))
                && values
                    .get("span_start")
                    .is_some_and(|v| v == &GraphProposalValue::Integer(start))
                && values
                    .get("span_end")
                    .is_some_and(|v| v == &GraphProposalValue::Integer(end))
        }
        EntityKeyCheck::Xml {
            path,
            occurrence_id,
        } => {
            values.len() == 2
                && values
                    .get("document")
                    .is_some_and(|v| v == &GraphProposalValue::String((*path).into()))
                && values
                    .get("occurrence")
                    .is_some_and(|v| v == &GraphProposalValue::Identifier((*occurrence_id).into()))
        }
    }
}

fn check_ownership_support<E>(
    map: &ProjectMapWire,
    proposal_ids: (&str, &str),
    direct_join: (&GraphNode, &GraphNode, &wow_graph::GraphEdge),
    support_ids: (StableHandleId, EvidenceId),
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    let (ownership_id, entity_proposal_id) = proposal_ids;
    let (root, node, direct_edge) = direct_join;
    let (handle_id, evidence_id) = support_ids;
    let root_proposal_id = root_entity_proposal_id(map, root, reserve_work, stop)?;
    let mut matches = 0usize;
    for receipt in &map.core.supports {
        step(reserve_work, stop)?;
        for support in &receipt.explanation.supports {
            step(reserve_work, stop)?;
            if let AssertionSupportWire::ProducerRelation {
                proposal, accepted, ..
            } = support
                && proposal.proposal_id() == ownership_id
            {
                matches = matches
                    .checked_add(1)
                    .ok_or(DeclarationLinksError::Invalid(
                        "ownership support count overflow",
                    ))?;
                let endpoints = proposal.endpoints();
                let target_matches = matches!(
                    endpoints[1],
                    GraphProposalEndpoint::Proposed(id) if id.as_ref() == entity_proposal_id
                );
                let source_matches = matches!(
                    endpoints[0],
                    GraphProposalEndpoint::Proposed(id) if id.as_ref() == root_proposal_id.as_ref()
                );
                if proposal.relation_kind_id() != "source_declaration_owns"
                    || !source_matches
                    || !target_matches
                    || !single_handle(proposal.source_handle_ids(), handle_id)
                    || !single_evidence_ids(proposal.evidence_ids(), evidence_id)
                    || accepted.proposal_id() != ownership_id
                    || !published_edge_matches(accepted.edge(), direct_edge, reserve_work, stop)?
                    || direct_edge.from() != root.node_id()
                    || direct_edge.to() != node.node_id()
                    || direct_edge.relation() != wow_graph::GraphRelationKind::Owns
                    || !single_evidence_boxed(direct_edge.evidence_ids(), evidence_id)
                {
                    return Err(DeclarationLinksError::Invalid(
                        "ownership proposal/accepted edge differs from the exact direct declaration join",
                    ));
                }
                validate_graph_edge(direct_edge, reserve_work, stop)?;
                check_support_evidence_pair(receipt, handle_id, evidence_id, reserve_work, stop)?;
            }
        }
    }
    if matches != 1 {
        return Err(DeclarationLinksError::Invalid(
            "declaration does not resolve to exactly one producer ownership support",
        ));
    }
    Ok(())
}

// The preceding Map check already bound the original edge endpoints to the
// selected graph nodes. Preserve its original semantic fields while applying
// that published endpoint identity; direct original/published equality is false.
fn published_edge_matches<E>(
    original: &wow_graph::GraphEdge,
    published: &wow_graph::GraphEdge,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<bool, DeclarationLinksError<E>> {
    step(reserve_work, stop)?;
    reserve_work(original.evidence_ids().len().saturating_add(1))
        .map_err(DeclarationLinksError::Work)?;
    let rebound = wow_graph::GraphEdge::new(
        published.from().clone(),
        published.to().clone(),
        original.relation(),
        original.confidence(),
        original.evidence_ids().to_vec(),
        GraphLimits {
            max_nodes: 1,
            max_edges: 1,
            max_coverage_records: 1,
            max_evidence_per_record: 256,
            max_query_edges: 1,
        },
    )
    .map_err(|_| DeclarationLinksError::Invalid("declaration edge cannot be projected"))?;
    Ok(rebound == *published)
}

fn root_entity_proposal_id<E>(
    map: &ProjectMapWire,
    root: &GraphNode,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<Box<str>, DeclarationLinksError<E>> {
    let mut found: Option<Box<str>> = None;
    for receipt in &map.core.supports {
        step(reserve_work, stop)?;
        for support in &receipt.explanation.supports {
            step(reserve_work, stop)?;
            if let AssertionSupportWire::ProducerEntity {
                proposal, accepted, ..
            } = support
            {
                reserve_work(accepted.node().evidence_ids().len().saturating_add(1))
                    .map_err(DeclarationLinksError::Work)?;
                if project_node_id(accepted.node(), map).is_ok_and(|id| &id == root.node_id()) {
                    if accepted.proposal_id() != proposal.proposal_id() || found.is_some() {
                        return Err(DeclarationLinksError::Invalid(
                            "selected root has conflicting producer entity support",
                        ));
                    }
                    found = Some(proposal.proposal_id().into());
                }
            }
        }
    }
    found.ok_or(DeclarationLinksError::Invalid(
        "selected root has no exact producer entity support",
    ))
}

fn validate_callable_parent_links<E>(
    declarations: &[DeclarationWire],
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    for declaration in declarations {
        step(reserve_work, stop)?;
        let DeclarationWire::Callable { metadata, .. } = declaration else {
            continue;
        };
        let Some(parent_id) = metadata.parent_function_id.as_deref() else {
            continue;
        };
        let mut parent_found = false;
        for candidate in declarations {
            step(reserve_work, stop)?;
            if let DeclarationWire::Callable {
                metadata: parent, ..
            } = candidate
                && parent.function.function_id == parent_id
                && parent.function.function_id != metadata.function.function_id
                && parent.function.path == metadata.function.path
                && parent.content_digest == metadata.content_digest
                && parent_scope_contains(parent.function.span, metadata.function.span)
                && matches!(
                    (metadata.scope, parent.function.kind),
                    (
                        ProjectCallableScopeWire::ChunkScopeClosure,
                        SourceFunctionKindWire::Chunk
                    ) | (
                        ProjectCallableScopeWire::NestedClosure,
                        SourceFunctionKindWire::Closure
                    )
                )
            {
                if parent_found {
                    return Err(DeclarationLinksError::Invalid(
                        "callable parent function ID resolves more than once",
                    ));
                }
                parent_found = true;
            }
        }
        if !parent_found {
            return Err(DeclarationLinksError::Invalid(
                "callable parent function ID is not present in the selected declaration set",
            ));
        }
    }
    Ok(())
}

fn parent_scope_contains(parent: wow_core::SourceSpan, child: wow_core::SourceSpan) -> bool {
    match (
        parent.byte_start(),
        parent.byte_end(),
        child.byte_start(),
        child.byte_end(),
        parent.kind(),
    ) {
        (None, None, Some(start), Some(end), wow_core::SourceSpanKind::WholeFile) => start < end,
        (Some(parent_start), Some(parent_end), Some(child_start), Some(child_end), _) => {
            parent_start <= child_start
                && child_start < child_end
                && child_end <= parent_end
                && (parent_start < child_start || child_end < parent_end)
        }
        _ => false,
    }
}

fn check_evidence_pair<E>(
    map: &ProjectMapWire,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    let mut found = false;
    for receipt in &map.core.supports {
        step(reserve_work, stop)?;
        for evidence in &receipt.evidence_resolution.evidence_records {
            step(reserve_work, stop)?;
            if evidence.evidence_id() == evidence_id
                && evidence_has_handle(evidence.source_handle_ids(), handle_id, reserve_work, stop)?
            {
                validate_evidence_record(evidence, reserve_work, stop)?;
                found = true;
            }
        }
    }
    if found {
        Ok(())
    } else {
        Err(DeclarationLinksError::Invalid(
            "declaration evidence ID does not resolve with its source handle",
        ))
    }
}

fn check_support_evidence_pair<E>(
    receipt: &ResolvedSupportWire,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    for evidence in &receipt.evidence_resolution.evidence_records {
        step(reserve_work, stop)?;
        if evidence.evidence_id() == evidence_id
            && evidence_has_handle(evidence.source_handle_ids(), handle_id, reserve_work, stop)?
        {
            validate_evidence_record(evidence, reserve_work, stop)?;
            return Ok(());
        }
    }
    Err(DeclarationLinksError::Invalid(
        "producer support receipt lacks the declaration's original evidence/handle pair",
    ))
}

fn validate_graph_node<E>(
    node: &GraphNode,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    const MAX_EVIDENCE: usize = 256;
    if node.evidence_ids().is_empty() || node.evidence_ids().len() > MAX_EVIDENCE {
        return Err(DeclarationLinksError::Invalid(
            "GraphNode evidence cardinality is outside the graph owner's finite bound",
        ));
    }
    for _ in node.evidence_ids() {
        step(reserve_work, stop)?;
    }
    node.validate(graph_limits()).map_err(|_| {
        DeclarationLinksError::Invalid(
            "GraphNode content-derived identity or evidence order is invalid",
        )
    })
}

fn validate_graph_edge<E>(
    edge: &wow_graph::GraphEdge,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    const MAX_EVIDENCE: usize = 256;
    if edge.evidence_ids().is_empty() || edge.evidence_ids().len() > MAX_EVIDENCE {
        return Err(DeclarationLinksError::Invalid(
            "GraphEdge evidence cardinality is outside the graph owner's finite bound",
        ));
    }
    for _ in edge.evidence_ids() {
        step(reserve_work, stop)?;
    }
    edge.validate(graph_limits()).map_err(|_| {
        DeclarationLinksError::Invalid(
            "GraphEdge content-derived identity or evidence order is invalid",
        )
    })
}

fn graph_limits() -> GraphLimits {
    GraphLimits {
        max_nodes: 1,
        max_edges: 1,
        max_coverage_records: 1,
        max_evidence_per_record: 256,
        max_query_edges: 1,
    }
}

fn validate_evidence_record<E>(
    evidence: &wow_core::EvidenceRecord,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<(), DeclarationLinksError<E>> {
    let total = evidence
        .source_handle_ids()
        .len()
        .checked_add(evidence.coverage_refs().len())
        .and_then(|n| n.checked_add(evidence.derivation_input_ids().len()))
        .ok_or(DeclarationLinksError::Invalid(
            "evidence work count overflow",
        ))?;
    if total > 4096 {
        return Err(DeclarationLinksError::Invalid(
            "evidence record validation exceeds its finite structural bound",
        ));
    }
    if stop.load(Ordering::Acquire) {
        return Err(DeclarationLinksError::Cancelled);
    }
    reserve_work(total).map_err(DeclarationLinksError::Work)?;
    if stop.load(Ordering::Acquire) {
        return Err(DeclarationLinksError::Cancelled);
    }
    evidence.validate().map_err(|_| {
        DeclarationLinksError::Invalid("declaration evidence content-derived identity is invalid")
    })
}

fn evidence_has_handle<E>(
    handles: &[StableHandleId],
    expected: StableHandleId,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<bool, DeclarationLinksError<E>> {
    let mut found = false;
    for handle in handles {
        step(reserve_work, stop)?;
        found |= *handle == expected;
    }
    Ok(found)
}

fn single_evidence(node: &GraphNode, expected: EvidenceId) -> bool {
    node.evidence_ids().len() == 1 && node.evidence_ids()[0].as_ref() == expected.to_string()
}

fn single_handle(values: &[StableHandleId], expected: StableHandleId) -> bool {
    values.len() == 1 && values[0] == expected
}

fn single_evidence_ids(values: &[EvidenceId], expected: EvidenceId) -> bool {
    values.len() == 1 && values[0] == expected
}

fn single_evidence_boxed(values: &[Box<str>], expected: EvidenceId) -> bool {
    values.len() == 1 && values[0].as_ref() == expected.to_string()
}
