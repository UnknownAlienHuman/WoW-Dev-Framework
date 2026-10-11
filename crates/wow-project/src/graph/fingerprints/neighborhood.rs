//! Closed physical Calls and named XML-handler neighborhoods after composition.
//! No ownership constant, generic adjacency, source reread, or lineage authority.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use super::super::{producer_budget::ProducerBudget, scripts::ProjectGraphScriptQueryOutcome};
use super::{FunctionFingerprintManifest, PlatformGraphProvenance, invalid};
use crate::{ProjectResult, load::XmlScriptSource};
use serde::{Deserialize, Serialize, Serializer};
use wow_core::{ContentDigest, DigestPurpose, domain_separated_digest};
use wow_emmy::function_calls::SourceCallTarget;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence, GraphEdgeId,
    GraphGenerationId, GraphLocalAssertion, GraphNodeId, GraphPartitionSnapshot,
    GraphProducerLookup, GraphRelationKind, GraphSnapshotId,
};

pub const FUNCTION_DIRECT_NEIGHBORHOOD_RECIPE: &str =
    "wow-project/physical-function-composed-calls-and-xml-handlers/1";
const DOMAIN: &str = "wow.project.fingerprint.physical-function.composed-calls-and-xml-handlers.v1";
const CALL_PARTITION: &str = "wow-recognizers.lua-direct-calls";
const SCRIPT_PARTITION: &str = "wow-recognizers.xml-script-bindings";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FunctionDirectNeighborhoodRecipe {
    #[serde(rename = "wow-project/physical-function-composed-calls-and-xml-handlers/1")]
    ComposedCallsAndXmlHandlersV1,
}
impl<'de> Deserialize<'de> for FunctionDirectNeighborhoodRecipe {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match String::deserialize(deserializer)?.as_str() {
            FUNCTION_DIRECT_NEIGHBORHOOD_RECIPE => Ok(Self::ComposedCallsAndXmlHandlersV1),
            _ => Err(serde::de::Error::custom(
                "unsupported direct neighborhood recipe",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionDirectNeighborhoodFingerprint;
impl DigestPurpose for FunctionDirectNeighborhoodFingerprint {
    const NAME: &'static str = "function_direct_neighborhood_fingerprint";
}

/// Private application adaptation borrows real SourceCallRecognition receipts.
/// These DTOs are transport, not admission; the owner checks native facts/acceptance.
pub enum ComposedCallOutcome<'a> {
    Projected { proposal_id: &'a str },
    LibraryTarget,
    SelfRecursionUnsupported,
    Unknown,
}
pub struct ComposedCallReceipt<'a> {
    pub call_id: &'a str,
    pub outcome: ComposedCallOutcome<'a>,
}
pub struct ComposedScriptReceipt<'a> {
    pub binding_id: &'a str,
    pub proposal_id: &'a str,
}

/// Borrowed current native composition, never serialized/deserialized authority.
pub struct FunctionNeighborhoodComposition<'a> {
    pub final_lookup: &'a GraphProducerLookup<'a>,
    pub call_input_snapshot_id: &'a GraphSnapshotId,
    pub calls: &'a [ComposedCallReceipt<'a>],
    pub script_scope: &'a GraphAssertionRecordScope,
    pub scripts: &'a [ComposedScriptReceipt<'a>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionNeighborDirection {
    Incoming,
    Outgoing,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionNeighborKind {
    LuaSourceFunction,
    XmlSourceDeclaration,
}

/// Closed rows in definition/direction order; zero counts require a closed census.
#[derive(Debug, Serialize)]
pub struct FunctionNeighborGroup {
    relation_definition_id: &'static str,
    direction: FunctionNeighborDirection,
    neighbor_kind: FunctionNeighborKind,
    multiplicity: u64,
}
impl FunctionNeighborGroup {
    pub const fn relation_definition_id(&self) -> &'static str {
        self.relation_definition_id
    }
    pub const fn direction(&self) -> FunctionNeighborDirection {
        self.direction
    }
    pub const fn neighbor_kind(&self) -> FunctionNeighborKind {
        self.neighbor_kind
    }
    pub const fn multiplicity(&self) -> u64 {
        self.multiplicity
    }
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionNeighborhoodUnavailable {
    SourceScopeConflict,
    IncompletePhysicalSource,
    UnknownCallTarget,
    UnsupportedSelfRecursion,
    IncompleteXmlScope,
    DynamicXmlAssociation,
    UnsupportedNeighborKind,
}
#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FunctionNeighborhoodComponent {
    Comparable {
        digest: ContentDigest<FunctionDirectNeighborhoodFingerprint>,
        groups: [FunctionNeighborGroup; 3],
    },
    Unavailable {
        reason: FunctionNeighborhoodUnavailable,
    },
}
#[derive(Debug)]
pub enum FunctionDirectNeighborhoodSelection {
    NotSelected,
    Selected(Box<FunctionNeighborhoodComponent>),
}
impl Serialize for FunctionDirectNeighborhoodSelection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::NotSelected => serializer.serialize_str("not_selected"),
            Self::Selected(value) => value.serialize(serializer),
        }
    }
}

/// One shared prefix/final/producer binding, not repeated per function/edge.
#[derive(Debug, Serialize)]
pub struct FunctionNeighborhoodBinding {
    recipe: FunctionDirectNeighborhoodRecipe,
    prefix_snapshot_id: GraphSnapshotId,
    final_snapshot_id: GraphSnapshotId,
    final_generation: GraphGenerationId,
    registry_digest: Box<str>,
    call_partition_digest: Box<str>,
    call_validation_report_id: Box<str>,
    script_partition_digest: Box<str>,
    script_validation_report_id: Box<str>,
}
impl FunctionNeighborhoodBinding {
    pub const fn recipe(&self) -> FunctionDirectNeighborhoodRecipe {
        self.recipe
    }
    pub const fn prefix_snapshot_id(&self) -> &GraphSnapshotId {
        &self.prefix_snapshot_id
    }
    pub const fn final_snapshot_id(&self) -> &GraphSnapshotId {
        &self.final_snapshot_id
    }
    pub const fn final_generation(&self) -> &GraphGenerationId {
        &self.final_generation
    }
    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
    pub fn call_partition_digest(&self) -> &str {
        &self.call_partition_digest
    }
    pub fn call_validation_report_id(&self) -> &str {
        &self.call_validation_report_id
    }
    pub fn script_partition_digest(&self) -> &str {
        &self.script_partition_digest
    }
    pub fn script_validation_report_id(&self) -> &str {
        &self.script_validation_report_id
    }
}
pub(super) struct FunctionNeighborhoodBatch {
    pub binding: Box<FunctionNeighborhoodBinding>,
    /// Same order/cardinality as the already-proved parent records; no copied IDs.
    pub rows: Vec<FunctionDirectNeighborhoodSelection>,
}

#[derive(Default)]
struct Census<'a> {
    counts: [u64; 3],
    /// Row index identifies the EXACT definition and relative direction.
    pairs: BTreeSet<(&'a GraphEdgeId, usize)>,
    reason: Option<FunctionNeighborhoodUnavailable>,
}

/// Internal leaf: parent owns physical support, selection and the SAME live ledger.
pub(super) fn build_direct_neighborhood_components(
    direct: &PlatformGraphProvenance<'_>,
    manifest: &FunctionFingerprintManifest,
    prefix_lookup: &GraphProducerLookup<'_>,
    composition: &FunctionNeighborhoodComposition<'_>,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<FunctionNeighborhoodBatch> {
    crate::analyzer::checkpoint(stop)?;
    let lookup = composition.final_lookup;
    let owner = lookup.owner();
    if retained.limit() != manifest.limits().max_metadata_bytes()
        || manifest.records().len() > manifest.limits().max_records()
        || manifest.binding().graph_scope() != direct.scope()
        || manifest.binding().published_graph_snapshot_id()
            != direct.graph().snapshot().snapshot_id()
        || prefix_lookup.scope() != direct.scope()
        || lookup.scope() != direct.scope()
        || composition.script_scope != direct.scope()
        || composition.call_input_snapshot_id != prefix_lookup.input_view().snapshot_id()
        || owner.source_context_id() != direct.scope().source_context_id
        || owner.registry() != direct.graph().registry()
        || owner.foundation() != direct.graph().foundation()
    {
        return Err(invalid());
    }
    // One bounded prefix->final association, not O(records) physical-owner revalidation.
    for prefix in direct.graph().partitions() {
        crate::analyzer::checkpoint(stop)?;
        let final_partition = owner.partition(prefix.partition_id()).ok_or_else(invalid)?;
        if final_partition.partition_digest() != prefix.partition_digest() {
            return Err(invalid());
        }
    }
    let call_partition = owner.partition(CALL_PARTITION).ok_or_else(invalid)?;
    let script_partition = owner.partition(SCRIPT_PARTITION).ok_or_else(invalid)?;
    let report = direct.source().function_call_report().ok_or_else(invalid)?;
    if composition.calls.len() != report.calls().len()
        || composition.scripts.len() != direct.source().script_bindings().len()
    {
        return Err(invalid());
    }
    for (id, kind, source, target) in [
        (
            "lua_direct_calls",
            GraphRelationKind::Calls,
            "lua_source_function",
            "lua_source_function",
        ),
        (
            "source_xml_sets_script",
            GraphRelationKind::SetsScript,
            "xml_source_declaration",
            "lua_source_function",
        ),
    ] {
        let definition = owner.registry().relation_kind(id).ok_or_else(invalid)?;
        if definition.relation() != kind
            || !definition.allows_source_kind(source)
            || !definition.allows_target_kind(target)
            || !definition.allows_confidence(GraphConfidence::Derived)
        {
            return Err(invalid());
        }
    }
    let recipe = FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1;
    // Charge every copied field before constructing the ONE shared binding.
    retained.charge_serialized(
        &(
            recipe,
            direct.graph().snapshot().snapshot_id(),
            owner.snapshot().snapshot_id(),
            owner.snapshot().generation(),
            owner.registry().registry_digest(),
            call_partition.partition_digest(),
            call_partition.report().report_id(),
            script_partition.partition_digest(),
            script_partition.report().report_id(),
        ),
        stop,
    )?;
    let binding = Box::new(FunctionNeighborhoodBinding {
        recipe,
        prefix_snapshot_id: direct.graph().snapshot().snapshot_id().clone(),
        final_snapshot_id: owner.snapshot().snapshot_id().clone(),
        final_generation: owner.snapshot().generation().clone(),
        registry_digest: owner.registry().registry_digest().into(),
        call_partition_digest: call_partition.partition_digest().into(),
        call_validation_report_id: call_partition.report().report_id().into(),
        script_partition_digest: script_partition.partition_digest().into(),
        script_validation_report_id: script_partition.report().report_id().into(),
    });
    let mut global = if manifest.scope_conflicts().is_empty() {
        None
    } else {
        Some(FunctionNeighborhoodUnavailable::SourceScopeConflict)
    };
    for partition in [call_partition, script_partition] {
        if let Some(records) = partition.batch().assertion_records()
            && !records.conflicts.is_empty()
        {
            global = Some(FunctionNeighborhoodUnavailable::SourceScopeConflict);
        }
    }
    for file in report.files() {
        crate::analyzer::checkpoint(stop)?;
        retained.charge_serialized(&("neighborhood-health", file.parse_error_count), stop)?;
        if file.parse_error_count != 0 {
            global = Some(FunctionNeighborhoodUnavailable::IncompletePhysicalSource);
        }
    }
    let mut roots = BTreeMap::<&str, usize>::new();
    let mut node_roots = BTreeMap::<&GraphNodeId, usize>::new();
    let mut census = Vec::new();
    for (index, record) in manifest.records().iter().enumerate() {
        crate::analyzer::checkpoint(stop)?;
        retained.charge_serialized(
            &(record.function_id(), record.exact_owner().node_id()),
            stop,
        )?;
        if roots.insert(record.function_id(), index).is_some()
            || node_roots
                .insert(record.exact_owner().node_id(), index)
                .is_some()
        {
            return Err(invalid());
        }
        census.push(Census::default());
    }
    let mut call_proposals = BTreeSet::new();
    for (index, receipt) in composition.calls.iter().enumerate() {
        crate::analyzer::checkpoint(stop)?;
        // Native receipts and report calls are strictly call-ID ordered.
        let call = &report.calls()[index];
        let support = direct
            .source()
            .call_sites()
            .get(index)
            .ok_or_else(invalid)?;
        if receipt.call_id != call.fact_id() || support.call_id != call.fact_id() {
            return Err(invalid());
        }
        retained.charge_serialized(&("neighborhood-call", receipt.call_id), stop)?;
        match (call.target(), &receipt.outcome) {
            (
                SourceCallTarget::MainFunction { function_id },
                ComposedCallOutcome::Projected { proposal_id },
            ) if function_id.as_str() != call.caller_function_id() => {
                retained.charge_serialized(&("neighborhood-call-proposal", proposal_id), stop)?;
                if !call_proposals.insert(*proposal_id) {
                    return Err(invalid());
                }
                let edge = accepted_edge(
                    owner,
                    lookup,
                    CALL_PARTITION,
                    proposal_id,
                    "lua_direct_calls",
                    retained,
                    stop,
                )?;
                let proposal = call_partition
                    .batch()
                    .relation_proposal(proposal_id)
                    .ok_or_else(invalid)?;
                if !proposal.evidence_ids().contains(&support.evidence_id)
                    || !proposal
                        .source_handle_ids()
                        .contains(&support.source_handle_id)
                    || edge.confidence() != GraphConfidence::Derived
                    || edge.relation() != GraphRelationKind::Calls
                    || lookup
                        .input_view()
                        .node(edge.from())
                        .is_none_or(|n| n.kind() != "lua_source_function")
                    || lookup
                        .input_view()
                        .node(edge.to())
                        .is_none_or(|n| n.kind() != "lua_source_function")
                {
                    return Err(invalid());
                }
                // Existing parent root keys are INPUT IDs; no rebind/parser/label join.
                let from = roots.get(call.caller_function_id()).copied();
                let to = roots.get(function_id.as_str()).copied();
                if node_roots.get(edge.from()).copied() != from
                    || node_roots.get(edge.to()).copied() != to
                {
                    return Err(invalid());
                }
                if let Some(root) = from {
                    count(&mut census[root], edge.edge_id(), 1, retained, stop)?;
                }
                if let Some(root) = to {
                    count(&mut census[root], edge.edge_id(), 0, retained, stop)?;
                }
            }
            (SourceCallTarget::LibraryFunction { .. }, ComposedCallOutcome::LibraryTarget) => {}
            (
                SourceCallTarget::MainFunction { function_id },
                ComposedCallOutcome::SelfRecursionUnsupported,
            ) if function_id.as_str() == call.caller_function_id() => {
                if let Some(&root) = roots.get(function_id.as_str()) {
                    census[root].reason =
                        Some(FunctionNeighborhoodUnavailable::UnsupportedSelfRecursion);
                }
            }
            (
                SourceCallTarget::SignatureNotCaptured
                | SourceCallTarget::Unresolved
                | SourceCallTarget::Indeterminate,
                ComposedCallOutcome::Unknown,
            ) => {
                // Its potential target could be ANY selected root: incoming absence unknown.
                global = Some(FunctionNeighborhoodUnavailable::UnknownCallTarget);
            }
            _ => return Err(invalid()),
        }
    }
    if call_proposals.len() != call_partition.report().accepted_relations().len()
        || !call_partition.report().ready_for_publication()
    {
        return Err(invalid());
    }
    // No legacy error/coverage exception: consume the independent sealed native census.
    let xml_census = super::xml_census::admit(direct, retained, stop)?;
    if match xml_census {
        Some(proof) => !proof.is_for(direct, stop)?,
        None => true,
    } {
        global = Some(FunctionNeighborhoodUnavailable::IncompleteXmlScope);
    }
    let mut sources = BTreeMap::new();
    for source in direct.source().script_sources() {
        crate::analyzer::checkpoint(stop)?;
        retained.charge_serialized(&("neighborhood-script-source", &source.script_id), stop)?;
        if sources
            .insert(source.script_id.as_str(), source.source_kind)
            .is_some()
        {
            return Err(invalid());
        }
        if source.source_kind == XmlScriptSource::Unresolved {
            global = Some(FunctionNeighborhoodUnavailable::IncompleteXmlScope);
        }
    }
    for site in direct.source().script_sites() {
        crate::analyzer::checkpoint(stop)?;
        if sources.get(site.script_id.as_str()) == Some(&XmlScriptSource::ReferenceOnly) {
            if !site.blockers.is_empty() {
                global = Some(FunctionNeighborhoodUnavailable::IncompleteXmlScope);
            }
            for query in &site.queries {
                crate::analyzer::checkpoint(stop)?;
                retained.charge_serialized(&("neighborhood-script-query", &query.outcome), stop)?;
                if !matches!(
                    &query.outcome,
                    ProjectGraphScriptQueryOutcome::MainFunction { .. }
                        | ProjectGraphScriptQueryOutcome::LibraryTarget
                ) {
                    global = Some(FunctionNeighborhoodUnavailable::IncompleteXmlScope);
                }
            }
        }
    }
    let mut bindings = BTreeMap::new();
    for fact in direct.source().script_bindings() {
        crate::analyzer::checkpoint(stop)?;
        retained.charge_serialized(&("neighborhood-script-binding", &fact.binding_id), stop)?;
        if bindings.insert(fact.binding_id.as_str(), fact).is_some() {
            return Err(invalid());
        }
    }
    let mut script_proposals = BTreeSet::new();
    let mut script_bindings = BTreeSet::new();
    for receipt in composition.scripts {
        crate::analyzer::checkpoint(stop)?;
        retained.charge_serialized(&(receipt.binding_id, receipt.proposal_id), stop)?;
        if !script_proposals.insert(receipt.proposal_id)
            || !script_bindings.insert(receipt.binding_id)
        {
            return Err(invalid());
        }
        let fact = bindings.get(receipt.binding_id).ok_or_else(invalid)?;
        let edge = accepted_edge(
            owner,
            lookup,
            SCRIPT_PARTITION,
            receipt.proposal_id,
            "source_xml_sets_script",
            retained,
            stop,
        )?;
        let proposal = script_partition
            .batch()
            .relation_proposal(receipt.proposal_id)
            .ok_or_else(invalid)?;
        if edge.relation() != GraphRelationKind::SetsScript
            || edge.confidence() != fact.confidence
            || proposal.source_handle_ids() != fact.source_handle_ids.as_slice()
            || proposal.evidence_ids() != fact.evidence_ids.as_slice()
        {
            return Err(invalid());
        }
        // Mixed source producer addresses are held by the sealed native prefix.
        for (kind, proposal_id, endpoint) in [
            (
                GraphAssertionKind::Entity,
                &fact.receiver_proposal_id,
                edge.from(),
            ),
            (
                GraphAssertionKind::Entity,
                &fact.handler_proposal_id,
                edge.to(),
            ),
        ] {
            retained.charge_serialized(&("neighborhood-script-endpoint", proposal_id), stop)?;
            let key = GraphLocalAssertion {
                kind,
                proposal_id: proposal_id.as_str().into(),
            };
            let address = direct.assertion(&key).ok_or_else(invalid)?;
            let entity = lookup
                .entity(direct.scope(), address, stop)
                .map_err(super::super::platform_producers::graph_error)?;
            if entity.accepted().node().node_id() != endpoint {
                return Err(invalid());
            }
        }
        let Some(&root) = node_roots.get(edge.to()) else {
            continue;
        };
        if fact.handler_kind != "lua_source_function"
            || lookup
                .input_view()
                .node(edge.from())
                .is_none_or(|n| n.kind() != "xml_source_declaration")
        {
            census[root].reason = Some(FunctionNeighborhoodUnavailable::UnsupportedNeighborKind);
        } else if edge.confidence() != GraphConfidence::Derived {
            census[root].reason = Some(FunctionNeighborhoodUnavailable::DynamicXmlAssociation);
        } else {
            count(&mut census[root], edge.edge_id(), 2, retained, stop)?;
        }
    }
    if script_proposals.len() != script_partition.report().accepted_relations().len()
        || !script_partition.report().ready_for_publication()
    {
        return Err(invalid());
    }
    let mut rows = Vec::new();
    for item in census {
        crate::analyzer::checkpoint(stop)?;
        let value = if let Some(reason) = global.or(item.reason) {
            FunctionNeighborhoodComponent::Unavailable { reason }
        } else {
            let groups = [
                FunctionNeighborGroup {
                    relation_definition_id: "lua_direct_calls",
                    direction: FunctionNeighborDirection::Incoming,
                    neighbor_kind: FunctionNeighborKind::LuaSourceFunction,
                    multiplicity: item.counts[0],
                },
                FunctionNeighborGroup {
                    relation_definition_id: "lua_direct_calls",
                    direction: FunctionNeighborDirection::Outgoing,
                    neighbor_kind: FunctionNeighborKind::LuaSourceFunction,
                    multiplicity: item.counts[1],
                },
                FunctionNeighborGroup {
                    relation_definition_id: "source_xml_sets_script",
                    direction: FunctionNeighborDirection::Incoming,
                    neighbor_kind: FunctionNeighborKind::XmlSourceDeclaration,
                    multiplicity: item.counts[2],
                },
            ];
            retained.charge_serialized(&groups, stop)?;
            // Canonical domain envelope scratch: fixed framing plus ASCII domain.
            retained.charge(DOMAIN.len().checked_add(32).ok_or_else(invalid)?)?;
            let bytes = domain_separated_digest(DOMAIN, &groups).map_err(|_| invalid())?;
            crate::analyzer::checkpoint(stop)?;
            FunctionNeighborhoodComponent::Comparable {
                digest: ContentDigest::from_bytes(bytes),
                groups,
            }
        };
        retained.charge_serialized(&value, stop)?;
        rows.push(FunctionDirectNeighborhoodSelection::Selected(Box::new(
            value,
        )));
    }
    retained.charge_serialized(&binding, stop)?;
    retained.charge_serialized(&rows, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(FunctionNeighborhoodBatch { binding, rows })
}

fn accepted_edge<'a>(
    owner: &'a GraphPartitionSnapshot,
    lookup: &GraphProducerLookup<'a>,
    partition_id: &str,
    proposal_id: &str,
    definition: &str,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<&'a wow_graph::GraphEdge> {
    crate::analyzer::checkpoint(stop)?;
    let partition = owner.partition(partition_id).ok_or_else(invalid)?;
    retained.charge_serialized(
        &(
            "neighborhood-relation-address",
            partition_id,
            partition.batch().batch_id(),
            proposal_id,
        ),
        stop,
    )?;
    let address = GraphAssertionRef::Producer {
        partition_id: partition_id.into(),
        batch_id: partition.batch().batch_id().into(),
        assertion: GraphLocalAssertion {
            kind: GraphAssertionKind::Relation,
            proposal_id: proposal_id.into(),
        },
    };
    let resolved = lookup
        .relation(lookup.scope(), &address, stop)
        .map_err(super::super::platform_producers::graph_error)?;
    if resolved.partition().partition_digest() != partition.partition_digest()
        || resolved.proposal().relation_kind_id() != definition
        || lookup
            .input_view()
            .edge(resolved.accepted().edge().edge_id())
            != Some(resolved.accepted().edge())
    {
        return Err(invalid());
    }
    Ok(resolved.accepted().edge())
}
fn count<'a>(
    census: &mut Census<'a>,
    edge: &'a GraphEdgeId,
    row: usize,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    retained.charge_serialized(&("neighborhood-pair", edge, row), stop)?;
    if census.pairs.insert((edge, row)) {
        census.counts[row] = census.counts[row].checked_add(1).ok_or_else(invalid)?;
    }
    Ok(())
}
