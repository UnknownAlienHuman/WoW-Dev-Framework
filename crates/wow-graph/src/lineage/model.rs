//! Closed owner-neutral input records. Native Graph membership is validated here;
//! exact source-span computation remains the Project producer's responsibility.
use super::*;
use crate::{
    GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence, GraphConflictRecord,
    GraphCoverageRecord, GraphNode, GraphPartitionRead,
};
use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest, EvidenceRecord, SourceContent, SourceHandle};

pub const PROJECT_SPAN_LINEAGE_PROFILE: &str =
    "wow-graph/lineage/ordinary-project-source-span-candidates/1";
pub const PROJECT_SPAN_INPUT_SCHEMA: &str = "wow-graph/lineage-project-source-span-input/1";
pub const PROJECT_SPAN_PRODUCER: &str = "wow.project.lineage.source_fingerprint";
pub const SOURCE_SPAN_FEATURE_PROFILE: &str = "wow-project/physical-function-source-span/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageRole {
    Before,
    After,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageFunctionKind {
    Chunk,
    Closure,
    ReferenceCallable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageUniverseClass {
    UserProject,
    BlizzardUiSource,
    ReferencePlatformContract,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDeferredCapability {
    pub capability_id: wow_core::CapabilityId,
    pub record: wow_core::NotEvaluatedRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageGenerationBinding {
    pub project_id: Box<str>,
    pub project_kind: Box<str>,
    pub selected_profile: wow_core::ProfileIdentity,
    pub project_generation_schema_version: u64,
    pub analyzer_pin_id: Box<str>,
    pub analyzer_configuration_digest: Box<str>,
    pub project_coverage: Vec<wow_core::CoverageRecord>,
    pub project_deferred: Vec<LineageDeferredCapability>,
    pub store_epoch_id: Box<str>,
    pub store_generation_id: Box<str>,
    pub publication_set_id: Box<str>,
    pub project_snapshot_id: Box<str>,
    pub project_generation_id: Box<str>,
    pub analyzer_snapshot_id: Box<str>,
    pub configuration_digest: Box<str>,
    pub reference_generation_id: Box<str>,
    pub original_graph_scope: GraphAssertionRecordScope,
    pub published_graph_snapshot_id: crate::GraphSnapshotId,
    pub published_graph_generation_id: crate::GraphGenerationId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageComparisonBinding {
    pub universe_class: LineageUniverseClass,
    pub profile_id: Box<str>,
    pub pair_id: Box<str>,
    pub before: LineageGenerationBinding,
    pub after: LineageGenerationBinding,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<LineageSourceComparisonBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<LineageReferenceComparisonBinding>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceSpanFeature {
    pub profile_id: Box<str>,
    pub fingerprint: ContentDigest<CanonicalResult>,
    pub content_sha256: ContentDigest<SourceContent>,
    pub byte_length: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum LineageFingerprintFeature {
    Available {
        source_span: LineageSourceSpanFeature,
    },
    MissingExactSpan,
    ConflictBlocked,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageFunctionRecord {
    pub role: LineageRole,
    pub node: GraphNode,
    pub function_id: Box<str>,
    pub function_kind: LineageFunctionKind,
    pub source_assertion: GraphAssertionRef,
    pub ownership_assertion: GraphAssertionRef,
    pub source_handle: SourceHandle,
    pub evidence: EvidenceRecord,
    pub fingerprint: LineageFingerprintFeature,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineagePartitionCoverage {
    pub role: LineageRole,
    pub partition_id: Box<str>,
    pub partition_digest: Box<str>,
    pub producer_version: Box<str>,
    pub coverage: Vec<GraphCoverageRecord>,
    pub conflicts: Vec<GraphConflictRecord>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectFingerprintPartition {
    pub schema: Box<str>,
    pub producer_id: Box<str>,
    pub binding: LineageComparisonBinding,
    pub records: Vec<LineageFunctionRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_records: Vec<LineageReferenceCallableRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_omissions: Vec<LineageReferenceOmission>,
    pub partitions: Vec<LineagePartitionCoverage>,
    pub coverage: Box<str>,
    pub source_bodies: Box<str>,
    pub proof_ceiling: GraphConfidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageLimits {
    pub max_entities: usize,
    pub max_bucket_members: usize,
    pub max_pairs: usize,
    pub max_component_nodes: usize,
    pub max_component_edges: usize,
    pub max_work: u64,
    pub max_metadata_bytes: usize,
}
impl LineageLimits {
    pub fn validate(&self) -> GraphResult<()> {
        if self.max_entities == 0
            || self.max_entities > 16_384
            || self.max_bucket_members == 0
            || self.max_bucket_members > self.max_entities
            || self.max_pairs == 0
            || self.max_pairs > 1_000_000
            || self.max_component_nodes == 0
            || self.max_component_nodes > self.max_entities
            || self.max_component_edges == 0
            || self.max_component_edges > self.max_pairs
            || self.max_work == 0
            || self.max_work > 1_000_000_000
            || self.max_metadata_bytes == 0
            || self.max_metadata_bytes > 64 * 1024 * 1024
        {
            return Err(GraphError::new(
                GraphErrorCode::LineageProfileInvalid,
                "invalid lineage limits",
            ));
        }
        Ok(())
    }
}

/// Membership-qualified logical inputs, not a retained native Project proof.
/// Project must validate its computation before this independent Graph check.
pub struct ValidatedLineageInput {
    pub(super) data: ProjectFingerprintPartition,
    pub(super) partition_id: Box<str>,
    pub(super) limits: LineageLimits,
    pub(super) usage: LineageUsage,
}
impl ValidatedLineageInput {
    pub fn data(&self) -> &ProjectFingerprintPartition {
        &self.data
    }
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }
}

pub fn validate_project_fingerprint_partition(
    data: ProjectFingerprintPartition,
    before: &GraphPartitionRead<'_>,
    after: &GraphPartitionRead<'_>,
    limits: LineageLimits,
    stop: &AtomicBool,
) -> GraphResult<ValidatedLineageInput> {
    if data.binding.universe_class != LineageUniverseClass::UserProject {
        return Err(invalid());
    }
    validate_fingerprint_partition(data, before, after, limits, stop)
}
pub fn validate_source_fingerprint_partition(
    data: ProjectFingerprintPartition,
    before: &GraphPartitionRead<'_>,
    after: &GraphPartitionRead<'_>,
    limits: LineageLimits,
    stop: &AtomicBool,
) -> GraphResult<ValidatedLineageInput> {
    if data.binding.universe_class != LineageUniverseClass::BlizzardUiSource {
        return Err(invalid());
    }
    validate_fingerprint_partition(data, before, after, limits, stop)
}
pub(super) fn validate_fingerprint_partition(
    mut data: ProjectFingerprintPartition,
    before: &GraphPartitionRead<'_>,
    after: &GraphPartitionRead<'_>,
    limits: LineageLimits,
    stop: &AtomicBool,
) -> GraphResult<ValidatedLineageInput> {
    checkpoint(stop)?;
    limits.validate()?;
    let mut ledger = OperationBudget::resume(limits, LineageUsage::default(), stop)?;
    // Bound/cancel the full projection before traversing Source metadata.
    ledger.metadata(&data)?;
    let (profile, schema, producer) = super::source_model::selected_input_profile(&data)?;
    let ordinary = data.binding.universe_class == LineageUniverseClass::UserProject;
    if data.schema.as_ref() != schema
        || data.producer_id.as_ref() != producer
        || data.binding.profile_id.as_ref() != profile
        || data.coverage.as_ref() != "partial"
        || data.source_bodies.as_ref() != "excluded"
        || data.proof_ceiling != GraphConfidence::Candidate
        || data
            .records
            .len()
            .saturating_add(data.reference_records.len())
            > limits.max_entities
        || data.reference_omissions.len() > limits.max_entities
        || data.partitions.len() > crate::MAX_GRAPH_PRODUCER_PARTITIONS * 2
        || (data.binding.universe_class != LineageUniverseClass::ReferencePlatformContract
            && data.binding.before.store_epoch_id != data.binding.after.store_epoch_id)
        || data.binding.before.project_id != data.binding.after.project_id
        || data.binding.before.project_kind != data.binding.after.project_kind
        || (ordinary
            && (!matches!(
                data.binding.before.project_kind.as_ref(),
                "fixture" | "repository"
            ) || data.binding.before.selected_profile != data.binding.after.selected_profile
                || data.binding.before.configuration_digest
                    != data.binding.after.configuration_digest
                || data.binding.before.analyzer_pin_id != data.binding.after.analyzer_pin_id
                || data.binding.before.analyzer_configuration_digest
                    != data.binding.after.analyzer_configuration_digest
                || data.binding.before.reference_generation_id
                    != data.binding.after.reference_generation_id
                || data.binding.before.original_graph_scope.universe
                    != data.binding.after.original_graph_scope.universe))
    {
        return Err(invalid());
    }
    let reference = data.binding.universe_class == LineageUniverseClass::ReferencePlatformContract;
    if !reference
        && (data.binding.reference.is_some()
            || !data.reference_records.is_empty()
            || !data.reference_omissions.is_empty())
    {
        return Err(invalid());
    }
    if let Some(source) = &data.binding.source {
        ledger.work(
            source
                .before
                .inventory_entries
                .checked_add(source.after.inventory_entries)
                .ok_or_else(budget)?,
        )?;
    }
    for (index, (binding, graph)) in [(&data.binding.before, before), (&data.binding.after, after)]
        .into_iter()
        .enumerate()
    {
        checkpoint(stop)?;
        ledger.work(1)?;
        if &binding.published_graph_snapshot_id != graph.owner().snapshot().snapshot_id()
            || &binding.published_graph_generation_id != graph.owner().snapshot().generation()
            || binding.original_graph_scope.source_context_id != graph.owner().source_context_id()
        {
            return Err(invalid());
        }
        if let Some(source) = &data.binding.source {
            let metadata = if index == 0 {
                &source.before
            } else {
                &source.after
            };
            if metadata.native_registry_digest.as_ref()
                != graph.owner().registry().registry_digest()
                || &metadata.foundation_snapshot_id != graph.owner().foundation().snapshot_id()
            {
                return Err(invalid());
            }
        }
    }
    data.records
        .sort_by(|a, b| (a.role, a.node.node_id()).cmp(&(b.role, b.node.node_id())));
    if data
        .records
        .windows(2)
        .any(|p| p[0].role == p[1].role && p[0].node.node_id() == p[1].node.node_id())
    {
        return Err(invalid());
    }
    ledger.metadata(&(before.owner().snapshot(), after.owner().snapshot()))?;
    let left = before.owner().producer_lookup(stop)?;
    let right = after.owner().producer_lookup(stop)?;
    data.partitions
        .sort_by(|a, b| (a.role, a.partition_id.as_ref()).cmp(&(b.role, b.partition_id.as_ref())));
    if data.partitions.len() != before.owner().partitions().len() + after.owner().partitions().len()
        || data
            .partitions
            .windows(2)
            .any(|p| p[0].role == p[1].role && p[0].partition_id == p[1].partition_id)
    {
        return Err(invalid());
    }
    for row in &data.partitions {
        checkpoint(stop)?;
        ledger.work(1)?;
        let graph = match row.role {
            LineageRole::Before => before,
            LineageRole::After => after,
        };
        let part = graph
            .owner()
            .partition(&row.partition_id)
            .ok_or_else(invalid)?;
        if row.partition_digest.as_ref() != part.partition_digest()
            || row.producer_version.as_ref() != part.producer_version()
            || row.coverage.as_slice() != part.coverage()
            || row.conflicts.as_slice()
                != part
                    .batch()
                    .assertion_records()
                    .map_or(&[][..], |r| r.conflicts.as_slice())
        {
            return Err(invalid());
        }
    }
    for record in &data.records {
        checkpoint(stop)?;
        ledger.work(1)?;
        let (binding, lookup, graph) = match record.role {
            LineageRole::Before => (&data.binding.before, &left, before),
            LineageRole::After => (&data.binding.after, &right, after),
        };
        if &binding.original_graph_scope != lookup.scope() {
            return Err(invalid());
        }
        let entity = lookup.entity(
            &binding.original_graph_scope,
            &record.source_assertion,
            stop,
        )?;
        let ownership = lookup.relation(
            &binding.original_graph_scope,
            &record.ownership_assertion,
            stop,
        )?;
        let GraphAssertionRef::Producer {
            partition_id,
            assertion,
            ..
        } = &record.source_assertion
        else {
            return Err(invalid());
        };
        let node = graph.published_entity(partition_id, &assertion.proposal_id, stop)?;
        if node != &record.node
            || !matches!(
                record.function_kind,
                LineageFunctionKind::Chunk | LineageFunctionKind::Closure
            )
            || node.kind() != "lua_source_function"
            || entity.proposal().entity_kind_id() != "lua_source_function"
            || ownership.proposal().relation_kind_id() != "source_declaration_owns"
            || ownership.accepted().edge().to() != entity.accepted().node().node_id()
            || lookup
                .input_view()
                .node(ownership.accepted().edge().from())
                .is_none_or(|n| n.kind() != "source_file")
            || !entity
                .proposal()
                .source_handle_ids()
                .contains(&record.source_handle.handle_id())
            || !entity
                .proposal()
                .evidence_ids()
                .contains(&record.evidence.evidence_id())
            || !ownership
                .proposal()
                .source_handle_ids()
                .contains(&record.source_handle.handle_id())
            || !ownership
                .proposal()
                .evidence_ids()
                .contains(&record.evidence.evidence_id())
            || record.evidence.context_id() != binding.original_graph_scope.source_context_id
            || !record
                .evidence
                .source_handle_ids()
                .contains(&record.source_handle.handle_id())
        {
            return Err(invalid());
        }
        if !matches!(entity.proposal().semantic_key().get("function"),Some(crate::GraphProposalValue::String(id)) if id.as_ref()==record.function_id.as_ref())
        {
            return Err(invalid());
        }
        if let LineageFingerprintFeature::Available { source_span } = &record.fingerprint {
            if source_span.profile_id.as_ref() != SOURCE_SPAN_FEATURE_PROFILE {
                return Err(invalid());
            }
            if let (Some(start), Some(end)) = (
                record.source_handle.span().byte_start(),
                record.source_handle.span().byte_end(),
            ) && end.checked_sub(start) != Some(source_span.byte_length)
            {
                return Err(invalid());
            }
        }
    }
    if reference {
        super::reference_model::validate_records(&mut data, before, after, &mut ledger, stop)?;
    }
    checkpoint(stop)?;
    let partition_id = identity("wow-graph/lineage-input-partition/1", &data)?;
    Ok(ValidatedLineageInput {
        data,
        partition_id,
        limits,
        usage: ledger.usage,
    })
}
