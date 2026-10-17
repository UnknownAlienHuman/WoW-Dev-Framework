//! Closed exact queries over an admitted native Candidate snapshot.
use super::*;
use crate::{
    GraphAcceptedEntityProposal, GraphAcceptedRelationProposal, GraphAssertionRecordScope,
    GraphEntityProposal, GraphNode, GraphPartitionRead, GraphRelationProposal,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wow_core::{EvidenceId, EvidenceRecord, SourceHandle, StableHandleId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum LineageCandidateQuery {
    Status {},
    Validate {},
    Compare {
        before: LineageEntityAddress,
        after: LineageEntityAddress,
    },
    Explain {
        target: LineageExplanationTarget,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LineageExplanationTarget {
    Proposal { id: Box<str> },
    Component { id: Box<str> },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageQueryLimits {
    pub max_work: u64,
    pub max_metadata_bytes: usize,
    pub max_output_bytes: usize,
    pub max_components: usize,
    pub max_entities: usize,
    pub max_proposals: usize,
}
impl LineageQueryLimits {
    pub fn validate(self) -> GraphResult<()> {
        if self.max_work == 0
            || self.max_work > 1_000_000_000
            || self.max_metadata_bytes == 0
            || self.max_metadata_bytes > 64 * 1024 * 1024
            || self.max_output_bytes == 0
            || self.max_output_bytes > 64 * 1024 * 1024
            || self.max_components == 0
            || self.max_components > 16_384
            || self.max_entities == 0
            || self.max_entities > 16_384
            || self.max_proposals == 0
            || self.max_proposals > 1_000_000
        {
            return Err(query_invalid());
        }
        Ok(())
    }
    pub(super) fn ledger(self, stop: &AtomicBool) -> GraphResult<OperationBudget<'_>> {
        self.validate()?;
        OperationBudget::resume(
            LineageLimits {
                max_entities: 16_384,
                max_bucket_members: 16_384,
                max_pairs: 1_000_000,
                max_component_nodes: 16_384,
                max_component_edges: 1_000_000,
                max_work: self.max_work,
                max_metadata_bytes: self.max_metadata_bytes,
            },
            LineageUsage::default(),
            stop,
        )
    }
}
impl LineageCandidateQuery {
    pub fn validate(&self) -> GraphResult<()> {
        match self {
            Self::Compare { before, after } => {
                if before.role != LineageRole::Before || after.role != LineageRole::After {
                    return Err(query_invalid());
                }
                crate::GraphNodeId::new(before.node_id.as_str())?;
                crate::GraphNodeId::new(after.node_id.as_str())?;
            }
            Self::Explain { target } => {
                let (id, prefix) = match target {
                    LineageExplanationTarget::Proposal { id } => {
                        (id, "wow-graph/lineage-candidate-proposal/1:sha256:")
                    }
                    LineageExplanationTarget::Component { id } => {
                        (id, "wow-graph/lineage-candidate-component/1:sha256:")
                    }
                };
                let hex = id.strip_prefix(prefix).ok_or_else(query_invalid)?;
                if hex.len() != 64
                    || !hex
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err(query_invalid());
                }
            }
            _ => {}
        }
        Ok(())
    }
}
/// Maps are borrowed from the exact retained Project owner. Graph resolves only
/// typed IDs against them and the matching native graph; no source is opened.
pub struct LineageNativeSourceSide<'a, 'g> {
    pub graph: &'a GraphPartitionRead<'g>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}
pub struct LineageNativeSources<'a, 'g> {
    pub before: LineageNativeSourceSide<'a, 'g>,
    pub after: LineageNativeSourceSide<'a, 'g>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageNativeFunctionSupport {
    pub entity: LineageEntityAddress,
    pub scope: GraphAssertionRecordScope,
    pub entity_proposal: GraphEntityProposal,
    pub entity_accepted: GraphAcceptedEntityProposal,
    pub ownership_proposal: GraphRelationProposal,
    pub ownership_accepted: GraphAcceptedRelationProposal,
    pub original_source_file_node: GraphNode,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageRoleSource {
    pub role: LineageRole,
    pub source: SourceHandle,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageRoleEvidence {
    pub role: LineageRole,
    pub evidence: EvidenceRecord,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageQueryClosure {
    pub components: Vec<LineageCandidateComponent>,
    pub proposals: Vec<LineageCandidateProposal>,
    pub functions: Vec<LineageFunctionRecord>,
    pub native_support: Vec<LineageNativeFunctionSupport>,
    pub sources: Vec<LineageRoleSource>,
    pub evidence: Vec<LineageRoleEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReadSummary {
    pub functions: usize,
    pub proposals: usize,
    pub components: usize,
    pub ambiguous_components: usize,
    pub skipped_buckets: usize,
    pub unavailable_features: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageQueryResult {
    pub schema: Box<str>,
    pub result_id: Box<str>,
    pub snapshot_id: Box<str>,
    pub profile_id: Box<str>,
    pub input_partition_id: Box<str>,
    pub binding: LineageComparisonBinding,
    pub query: LineageCandidateQuery,
    pub status: Box<str>,
    pub selected_validation: Box<str>,
    pub full_e4_b_validation: Box<str>,
    pub summary: LineageReadSummary,
    pub input_partitions: Vec<LineagePartitionCoverage>,
    pub skipped_buckets: Vec<LineageSkippedBucket>,
    pub unavailable_features: Vec<LineageUnavailableFeature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closure: Option<LineageQueryClosure>,
    pub accepted_assertion_count: usize,
    pub promotion: Box<str>,
    pub negative_authority: bool,
    pub usage: LineageUsage,
}

#[derive(Serialize)]
struct ClosureIdentity<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    closure: Option<&'a LineageQueryClosure>,
}
impl LineageQueryResult {
    fn semantic_identity(&self) -> impl Serialize + '_ {
        (
            &self.schema,
            &self.snapshot_id,
            &self.profile_id,
            &self.input_partition_id,
            &self.binding,
            &self.query,
            &self.status,
            (&self.selected_validation, &self.full_e4_b_validation),
            &self.summary,
            &self.input_partitions,
            &self.skipped_buckets,
            &self.unavailable_features,
            ClosureIdentity {
                closure: self.closure.as_ref(),
            },
            (
                self.accepted_assertion_count,
                &self.promotion,
                self.negative_authority,
            ),
        )
    }
    /// Validates recorded observation data; this never constructs a native view
    /// or asserts that the original snapshot is still admitted or retained.
    pub fn validate_capture(
        &self,
        snapshot_id: &str,
        query: &LineageCandidateQuery,
    ) -> GraphResult<()> {
        self.validate_capture_profile(
            snapshot_id,
            query,
            PROJECT_SPAN_LINEAGE_PROFILE,
            "native_project_source_span_candidates_valid",
        )
    }
    pub fn validate_source_capture(
        &self,
        snapshot_id: &str,
        query: &LineageCandidateQuery,
    ) -> GraphResult<()> {
        self.validate_capture_profile(
            snapshot_id,
            query,
            SOURCE_SPAN_LINEAGE_PROFILE,
            "native_blizzard_ui_source_span_candidates_valid",
        )
    }
    fn validate_capture_profile(
        &self,
        snapshot_id: &str,
        query: &LineageCandidateQuery,
        profile: &str,
        validation: &str,
    ) -> GraphResult<()> {
        if self.schema.as_ref() != "wow-graph/lineage-candidate-query-result/1"
            || self.snapshot_id.as_ref() != snapshot_id
            || self.profile_id.as_ref() != profile
            || &self.query != query
            || self.accepted_assertion_count != 0
            || self.promotion.as_ref() != "not_evaluated"
            || self.negative_authority
            || self.selected_validation.as_ref() != validation
            || self.full_e4_b_validation.as_ref() != "not_evaluated"
            || !matches!(self.status.as_ref(), "candidate_only" | "truncated")
            || self.result_id
                != identity(
                    "wow-graph/lineage-candidate-query/1",
                    &self.semantic_identity(),
                )?
        {
            return Err(query_invalid());
        }
        if let Some(closure) = &self.closure {
            for component in &closure.components {
                if component.negative_authority {
                    return Err(query_invalid());
                }
            }
            for proposal in &closure.proposals {
                if proposal.confidence != crate::GraphConfidence::Candidate
                    || proposal.proof_ceiling != crate::GraphConfidence::Candidate
                    || proposal.relation.as_ref() != "lineage_successor_of"
                    || proposal.input_partition_id != self.input_partition_id
                {
                    return Err(query_invalid());
                }
            }
        }
        if matches!(
            query,
            LineageCandidateQuery::Compare { .. } | LineageCandidateQuery::Explain { .. }
        ) && self.closure.is_none()
        {
            return Err(query_invalid());
        }
        Ok(())
    }
}

pub(super) fn execute(
    record: &super::snapshot::SnapshotRecord,
    query: &LineageCandidateQuery,
    sources: &LineageNativeSources<'_, '_>,
    limits: LineageQueryLimits,
    ledger: &mut OperationBudget<'_>,
    stop: &AtomicBool,
) -> GraphResult<LineageQueryResult> {
    query.validate()?;
    let data = &record.input;
    let candidates = &record.candidates;
    ledger.work(candidates.components.len())?;
    let summary = LineageReadSummary {
        functions: data.records.len(),
        proposals: candidates.proposals.len(),
        components: candidates.components.len(),
        ambiguous_components: candidates.components.iter().filter(|c| c.ambiguous).count(),
        skipped_buckets: candidates.skipped_buckets.len(),
        unavailable_features: candidates.unavailable_features.len(),
    };
    let mut selected = BTreeSet::new();
    match query {
        LineageCandidateQuery::Compare { before, after } => {
            for address in [before, after] {
                ledger.work(data.records.len())?;
                if !data
                    .records
                    .iter()
                    .any(|r| r.role == address.role && r.node.node_id() == &address.node_id)
                {
                    return Err(query_invalid());
                }
                let mut found = false;
                for c in &candidates.components {
                    ledger.work(
                        c.before
                            .len()
                            .checked_add(c.after.len())
                            .ok_or_else(budget)?,
                    )?;
                    if c.before.contains(address) || c.after.contains(address) {
                        ledger.metadata(&c.component_id)?;
                        selected.insert(c.component_id.as_ref());
                        found = true;
                    }
                }
                if !found {
                    return Err(query_invalid());
                }
            }
        }
        LineageCandidateQuery::Explain { target } => match target {
            LineageExplanationTarget::Component { id } => {
                ledger.work(candidates.components.len())?;
                let c = candidates
                    .components
                    .iter()
                    .find(|c| c.component_id == *id)
                    .ok_or_else(query_invalid)?;
                ledger.metadata(&c.component_id)?;
                selected.insert(c.component_id.as_ref());
            }
            LineageExplanationTarget::Proposal { id } => {
                ledger.work(candidates.proposals.len())?;
                if !candidates.proposals.iter().any(|p| p.proposal_id == *id) {
                    return Err(query_invalid());
                }
                let mut found = false;
                for c in &candidates.components {
                    ledger.work(c.proposal_ids.len())?;
                    if c.proposal_ids.binary_search(id).is_ok() {
                        ledger.metadata(&c.component_id)?;
                        selected.insert(c.component_id.as_ref());
                        found = true;
                    }
                }
                if !found {
                    return Err(query_invalid());
                }
            }
        },
        _ => {}
    }
    let closure = if selected.is_empty() {
        None
    } else {
        Some(build_closure(
            record, &selected, sources, limits, ledger, stop,
        )?)
    };
    ledger.metadata(&(
        &data.binding,
        &data.partitions,
        &candidates.skipped_buckets,
        &candidates.unavailable_features,
    ))?;
    let mut result = LineageQueryResult {
        schema: "wow-graph/lineage-candidate-query-result/1".into(),
        result_id: "".into(),
        snapshot_id: record.snapshot_id.clone(),
        profile_id: record.profile_id.clone(),
        input_partition_id: record.input_partition_id.clone(),
        binding: data.binding.clone(),
        query: query.clone(),
        status: if candidates.status.as_ref() == "truncated" {
            "truncated"
        } else {
            "candidate_only"
        }
        .into(),
        selected_validation: match record.profile_id.as_ref() {
            SOURCE_SPAN_LINEAGE_PROFILE => "native_blizzard_ui_source_span_candidates_valid",
            _ => "native_project_source_span_candidates_valid",
        }
        .into(),
        full_e4_b_validation: "not_evaluated".into(),
        summary,
        input_partitions: data.partitions.clone(),
        skipped_buckets: candidates.skipped_buckets.clone(),
        unavailable_features: candidates.unavailable_features.clone(),
        closure,
        accepted_assertion_count: 0,
        promotion: "not_evaluated".into(),
        negative_authority: false,
        usage: LineageUsage::default(),
    };
    // Reserve result metadata before identity serialization; selected payloads
    // were separately charged before cloning. No partial component is returned.
    ledger.metadata(&result.semantic_identity())?;
    let result_id = identity(
        "wow-graph/lineage-candidate-query/1",
        &result.semantic_identity(),
    )?;
    result.result_id = result_id;
    ledger.metadata(&result.result_id)?;
    // Semantic identity excludes observation counters. Reserve the maximum
    // decimal counter widths before final serialization, then freeze actual usage.
    result.usage = LineageUsage {
        work: u64::MAX,
        metadata_bytes: usize::MAX,
    };
    let size = ledger.metadata(&result)?;
    if size
        .checked_add(1)
        .is_none_or(|n| n > limits.max_output_bytes)
    {
        return Err(budget());
    }
    result.usage = ledger.usage;
    checkpoint(stop)?;
    Ok(result)
}

fn build_closure(
    record: &super::snapshot::SnapshotRecord,
    selected: &BTreeSet<&str>,
    sources: &LineageNativeSources<'_, '_>,
    limits: LineageQueryLimits,
    ledger: &mut OperationBudget<'_>,
    stop: &AtomicBool,
) -> GraphResult<LineageQueryClosure> {
    if selected.len() > limits.max_components {
        return Err(budget());
    }
    let mut output = LineageQueryClosure {
        components: Vec::new(),
        proposals: Vec::new(),
        functions: Vec::new(),
        native_support: Vec::new(),
        sources: Vec::new(),
        evidence: Vec::new(),
    };
    let mut addresses = BTreeSet::new();
    let mut proposals = BTreeSet::new();
    for c in &record.candidates.components {
        ledger.work(1)?;
        if !selected.contains(c.component_id.as_ref()) {
            continue;
        }
        ledger.metadata(c)?;
        ledger.work(
            c.before
                .len()
                .saturating_add(c.after.len())
                .saturating_add(c.proposal_ids.len()),
        )?;
        for n in c.before.iter().chain(&c.after) {
            ledger.metadata(n)?;
            addresses.insert(n.clone());
        }
        for p in &c.proposal_ids {
            ledger.metadata(p)?;
            proposals.insert(p.as_ref());
        }
        if addresses.len() > limits.max_entities || proposals.len() > limits.max_proposals {
            return Err(budget());
        }
        output.components.push(c.clone());
    }
    for p in &record.candidates.proposals {
        ledger.work(1)?;
        if proposals.contains(p.proposal_id.as_ref()) {
            if !addresses.contains(&p.before) || !addresses.contains(&p.after) {
                return Err(query_invalid());
            }
            ledger.metadata(p)?;
            output.proposals.push(p.clone());
        }
    }
    if output.proposals.len() != proposals.len() {
        return Err(query_invalid());
    }
    for r in &record.input.records {
        ledger.work(1)?;
        let key = LineageEntityAddress {
            role: r.role,
            node_id: r.node.node_id().clone(),
        };
        if addresses.contains(&key) {
            ledger.metadata(r)?;
            output.functions.push(r.clone());
        }
    }
    if output.functions.len() != addresses.len() {
        return Err(query_invalid());
    }
    native_support(record, sources, ledger, stop, &mut output)?;
    Ok(output)
}

fn native_support(
    record: &super::snapshot::SnapshotRecord,
    sources: &LineageNativeSources<'_, '_>,
    ledger: &mut OperationBudget<'_>,
    stop: &AtomicBool,
    output: &mut LineageQueryClosure,
) -> GraphResult<()> {
    for (role, side, binding) in [
        (
            LineageRole::Before,
            &sources.before,
            &record.input.binding.before,
        ),
        (
            LineageRole::After,
            &sources.after,
            &record.input.binding.after,
        ),
    ] {
        if side.graph.owner().snapshot().snapshot_id() != &binding.published_graph_snapshot_id
            || side.graph.owner().snapshot().generation() != &binding.published_graph_generation_id
            || side.graph.owner().source_context_id()
                != binding.original_graph_scope.source_context_id
        {
            return Err(query_invalid());
        }
        // Reserve native lookup scratch before reconstructing its exact input view.
        ledger.metadata(side.graph.owner().snapshot())?;
        let lookup = side.graph.owner().producer_lookup(stop)?;
        if lookup.scope() != &binding.original_graph_scope {
            return Err(query_invalid());
        }
        let mut evidence_ids = BTreeSet::new();
        let mut handle_ids = BTreeSet::new();
        for r in output.functions.iter().filter(|r| r.role == role) {
            ledger.work(1)?;
            let entity = lookup.entity(&binding.original_graph_scope, &r.source_assertion, stop)?;
            let ownership =
                lookup.relation(&binding.original_graph_scope, &r.ownership_assertion, stop)?;
            let parent = lookup
                .input_view()
                .node(ownership.accepted().edge().from())
                .ok_or_else(query_invalid)?;
            if ownership.accepted().edge().to() != entity.accepted().node().node_id()
                || parent.kind() != "source_file"
            {
                return Err(query_invalid());
            }
            for id in entity
                .proposal()
                .evidence_ids()
                .iter()
                .chain(ownership.proposal().evidence_ids())
            {
                ledger.metadata(id)?;
                evidence_ids.insert(*id);
            }
            for id in parent.evidence_ids() {
                ledger.work(1)?;
                let parsed = id.parse::<EvidenceId>().map_err(|_| query_invalid())?;
                ledger.metadata(&parsed)?;
                evidence_ids.insert(parsed);
            }
            for id in entity
                .proposal()
                .source_handle_ids()
                .iter()
                .chain(ownership.proposal().source_handle_ids())
            {
                ledger.metadata(id)?;
                handle_ids.insert(*id);
            }
            ledger.metadata(&(
                entity.proposal(),
                entity.accepted(),
                ownership.proposal(),
                ownership.accepted(),
                parent,
            ))?;
            output.native_support.push(LineageNativeFunctionSupport {
                entity: LineageEntityAddress {
                    role,
                    node_id: r.node.node_id().clone(),
                },
                scope: binding.original_graph_scope.clone(),
                entity_proposal: entity.proposal().clone(),
                entity_accepted: entity.accepted().clone(),
                ownership_proposal: ownership.proposal().clone(),
                ownership_accepted: ownership.accepted().clone(),
                original_source_file_node: parent.clone(),
            });
        }
        for id in evidence_ids {
            ledger.work(1)?;
            let evidence = side.evidence.get(&id).ok_or_else(query_invalid)?;
            if evidence.context_id() != binding.original_graph_scope.source_context_id {
                return Err(query_invalid());
            }
            ledger.metadata(evidence)?;
            for handle in evidence.source_handle_ids() {
                ledger.metadata(handle)?;
                handle_ids.insert(*handle);
            }
            output.evidence.push(LineageRoleEvidence {
                role,
                evidence: evidence.clone(),
            });
        }
        for id in handle_ids {
            ledger.work(1)?;
            let source = side.source_handles.get(&id).ok_or_else(query_invalid)?;
            ledger.metadata(source)?;
            output.sources.push(LineageRoleSource {
                role,
                source: source.clone(),
            });
        }
    }
    Ok(())
}
pub(super) fn query_invalid() -> GraphError {
    GraphError::new(
        GraphErrorCode::LineageQueryInvalid,
        "exact lineage query or native support is invalid",
    )
}
