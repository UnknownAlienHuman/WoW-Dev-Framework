use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{CoverageId, EvidenceId, GenerationContextId, StableHandleId, canonical_json_bytes};

use crate::{
    GraphConfidence, GraphEdge, GraphError, GraphErrorCode, GraphGenerationId, GraphLimits,
    GraphNode, GraphNodeId, GraphRegistryBundle, GraphResult, GraphSnapshot, GraphUniverseId,
};

pub const GRAPH_PROPOSAL_BATCH_SCHEMA: &str = "wow-graph/proposal-batch/e2-a/1";
pub const GRAPH_PROPOSAL_REPORT_SCHEMA: &str = "wow-graph/proposal-report/e2-a/1";
const MAX_PROPOSALS: usize = 200_000;
const MAX_IDENTITY_FIELDS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum GraphProposalValue {
    Boolean(bool),
    Integer(i64),
    String(Box<str>),
    Identifier(Box<str>),
    Reference(Box<str>),
}

impl GraphProposalValue {
    fn validate(&self) -> GraphResult<()> {
        let value = match self {
            Self::Boolean(_) | Self::Integer(_) => return Ok(()),
            Self::String(value) | Self::Identifier(value) | Self::Reference(value) => value,
        };
        if value.is_empty()
            || value.len() > 4096
            || value.contains('\0')
            || value.chars().any(char::is_control)
        {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph proposal value is empty, too large, or contains control data",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEntityProposal {
    proposal_id: Box<str>,
    entity_kind_id: Box<str>,
    semantic_key: BTreeMap<Box<str>, GraphProposalValue>,
    confidence: GraphConfidence,
    source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    coverage_ids: Vec<CoverageId>,
}

impl GraphEntityProposal {
    pub fn new(
        proposal_id: impl Into<Box<str>>,
        entity_kind_id: impl Into<Box<str>>,
        semantic_key: BTreeMap<Box<str>, GraphProposalValue>,
        confidence: GraphConfidence,
        source_handle_ids: Vec<StableHandleId>,
        evidence_ids: Vec<EvidenceId>,
        coverage_ids: Vec<CoverageId>,
    ) -> GraphResult<Self> {
        let proposal_id = proposal_id.into();
        let entity_kind_id = entity_kind_id.into();
        validate_external_id(&proposal_id)?;
        crate::registry::validate_component(&entity_kind_id, "entity proposal kind")?;
        if semantic_key.is_empty() || semantic_key.len() > MAX_IDENTITY_FIELDS {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph entity proposal identity is empty or too large",
            ));
        }
        for (field, value) in &semantic_key {
            crate::registry::validate_component(field, "entity identity field")?;
            value.validate()?;
        }
        let source_handle_ids = normalize(source_handle_ids, "source handle")?;
        let evidence_ids = normalize(evidence_ids, "evidence")?;
        let coverage_ids = normalize(coverage_ids, "coverage")?;
        if source_handle_ids.is_empty() || evidence_ids.is_empty() {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph entity proposal requires source and evidence support",
            ));
        }
        Ok(Self {
            proposal_id,
            entity_kind_id,
            semantic_key,
            confidence,
            source_handle_ids,
            evidence_ids,
            coverage_ids,
        })
    }

    fn validate(&self) -> GraphResult<()> {
        if Self::new(
            self.proposal_id.clone(),
            self.entity_kind_id.clone(),
            self.semantic_key.clone(),
            self.confidence,
            self.source_handle_ids.clone(),
            self.evidence_ids.clone(),
            self.coverage_ids.clone(),
        )? != *self
        {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph entity proposal is not canonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum GraphProposalEndpoint {
    Existing(GraphNodeId),
    Proposed(Box<str>),
}

impl GraphProposalEndpoint {
    fn validate(&self) -> GraphResult<()> {
        match self {
            Self::Existing(_) => Ok(()),
            Self::Proposed(value) => validate_external_id(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphRelationProposalInput {
    pub source: GraphProposalEndpoint,
    pub target: GraphProposalEndpoint,
    pub confidence: GraphConfidence,
    pub source_handle_ids: Vec<StableHandleId>,
    pub evidence_ids: Vec<EvidenceId>,
    pub coverage_ids: Vec<CoverageId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphRelationProposal {
    proposal_id: Box<str>,
    relation_kind_id: Box<str>,
    source: GraphProposalEndpoint,
    target: GraphProposalEndpoint,
    confidence: GraphConfidence,
    source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    coverage_ids: Vec<CoverageId>,
}

impl GraphRelationProposal {
    pub fn new(
        proposal_id: impl Into<Box<str>>,
        relation_kind_id: impl Into<Box<str>>,
        input: GraphRelationProposalInput,
    ) -> GraphResult<Self> {
        let proposal_id = proposal_id.into();
        let relation_kind_id = relation_kind_id.into();
        let GraphRelationProposalInput {
            source,
            target,
            confidence,
            source_handle_ids,
            evidence_ids,
            coverage_ids,
        } = input;
        validate_external_id(&proposal_id)?;
        crate::registry::validate_component(&relation_kind_id, "relation proposal kind")?;
        source.validate()?;
        target.validate()?;
        let source_handle_ids = normalize(source_handle_ids, "source handle")?;
        let evidence_ids = normalize(evidence_ids, "evidence")?;
        let coverage_ids = normalize(coverage_ids, "coverage")?;
        if source_handle_ids.is_empty() || evidence_ids.is_empty() {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph relation proposal requires source and evidence support",
            ));
        }
        Ok(Self {
            proposal_id,
            relation_kind_id,
            source,
            target,
            confidence,
            source_handle_ids,
            evidence_ids,
            coverage_ids,
        })
    }

    fn validate(&self) -> GraphResult<()> {
        if Self::new(
            self.proposal_id.clone(),
            self.relation_kind_id.clone(),
            GraphRelationProposalInput {
                source: self.source.clone(),
                target: self.target.clone(),
                confidence: self.confidence,
                source_handle_ids: self.source_handle_ids.clone(),
                evidence_ids: self.evidence_ids.clone(),
                coverage_ids: self.coverage_ids.clone(),
            },
        )? != *self
        {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph relation proposal is not canonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphProposalBatch {
    schema: Box<str>,
    batch_id: Box<str>,
    registry_bundle_id: Box<str>,
    registry_digest: Box<str>,
    universe: GraphUniverseId,
    generation: GraphGenerationId,
    source_context_id: GenerationContextId,
    producer_partition_id: Box<str>,
    entity_proposals: Vec<GraphEntityProposal>,
    relation_proposals: Vec<GraphRelationProposal>,
}

impl GraphProposalBatch {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        registry_bundle_id: impl Into<Box<str>>,
        registry_digest: impl Into<Box<str>>,
        universe: GraphUniverseId,
        generation: GraphGenerationId,
        source_context_id: GenerationContextId,
        producer_partition_id: impl Into<Box<str>>,
        mut entity_proposals: Vec<GraphEntityProposal>,
        mut relation_proposals: Vec<GraphRelationProposal>,
    ) -> GraphResult<Self> {
        let registry_bundle_id = registry_bundle_id.into();
        let registry_digest = registry_digest.into();
        let producer_partition_id = producer_partition_id.into();
        crate::registry::validate_component(&registry_bundle_id, "proposal registry bundle id")?;
        validate_digest(&registry_digest, "graph-registry:sha256:")?;
        validate_external_id(&producer_partition_id)?;
        if entity_proposals
            .len()
            .saturating_add(relation_proposals.len())
            > MAX_PROPOSALS
        {
            return Err(GraphError::new(
                GraphErrorCode::BudgetExceeded,
                "graph proposal batch exceeds the hard proposal limit",
            ));
        }
        for proposal in &entity_proposals {
            proposal.validate()?;
        }
        for proposal in &relation_proposals {
            proposal.validate()?;
        }
        entity_proposals.sort_by(|left, right| left.proposal_id.cmp(&right.proposal_id));
        relation_proposals.sort_by(|left, right| left.proposal_id.cmp(&right.proposal_id));
        let mut ids = BTreeSet::new();
        if entity_proposals
            .iter()
            .map(|proposal| proposal.proposal_id.as_ref())
            .chain(
                relation_proposals
                    .iter()
                    .map(|proposal| proposal.proposal_id.as_ref()),
            )
            .any(|proposal_id| !ids.insert(proposal_id))
        {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph proposal batch contains duplicate proposal IDs",
            ));
        }
        let batch_id = derive_batch_id(BatchIdentity {
            registry_bundle_id: &registry_bundle_id,
            registry_digest: &registry_digest,
            universe: &universe,
            generation: &generation,
            source_context_id,
            producer_partition_id: &producer_partition_id,
            entity_proposals: &entity_proposals,
            relation_proposals: &relation_proposals,
        })?;
        Ok(Self {
            schema: GRAPH_PROPOSAL_BATCH_SCHEMA.into(),
            batch_id,
            registry_bundle_id,
            registry_digest,
            universe,
            generation,
            source_context_id,
            producer_partition_id,
            entity_proposals,
            relation_proposals,
        })
    }

    pub fn validate(&self) -> GraphResult<()> {
        if self.schema.as_ref() != GRAPH_PROPOSAL_BATCH_SCHEMA {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchInvalid,
                "graph proposal batch schema is unsupported",
            ));
        }
        let rebuilt = Self::build(
            self.registry_bundle_id.clone(),
            self.registry_digest.clone(),
            self.universe.clone(),
            self.generation.clone(),
            self.source_context_id,
            self.producer_partition_id.clone(),
            self.entity_proposals.clone(),
            self.relation_proposals.clone(),
        )?;
        if rebuilt != *self {
            return Err(GraphError::new(
                GraphErrorCode::ProposalBatchIdentityMismatch,
                "graph proposal batch identity or order does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    #[must_use]
    pub fn registry_bundle_id(&self) -> &str {
        &self.registry_bundle_id
    }

    #[must_use]
    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }

    #[must_use]
    pub fn universe(&self) -> &GraphUniverseId {
        &self.universe
    }

    #[must_use]
    pub fn generation(&self) -> &GraphGenerationId {
        &self.generation
    }

    #[must_use]
    pub const fn source_context_id(&self) -> GenerationContextId {
        self.source_context_id
    }

    #[must_use]
    pub fn producer_partition_id(&self) -> &str {
        &self.producer_partition_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphProposalRejectionCode {
    RegistryMismatch,
    UnknownEntityKind,
    UnknownRelationKind,
    UniverseNotAllowed,
    ConfidenceNotAllowed,
    IdentityFieldsMismatch,
    DuplicateSemanticEntity,
    EndpointMissing,
    EndpointKindMismatch,
    InvalidGraphRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphProposalRejection {
    proposal_id: Box<str>,
    code: GraphProposalRejectionCode,
}

impl GraphProposalRejection {
    #[must_use]
    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    #[must_use]
    pub const fn code(&self) -> GraphProposalRejectionCode {
        self.code
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAcceptedEntityProposal {
    proposal_id: Box<str>,
    node: GraphNode,
    source_handle_ids: Vec<StableHandleId>,
    coverage_ids: Vec<CoverageId>,
}

impl GraphAcceptedEntityProposal {
    #[must_use]
    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    #[must_use]
    pub const fn node(&self) -> &GraphNode {
        &self.node
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAcceptedRelationProposal {
    proposal_id: Box<str>,
    edge: GraphEdge,
    source_handle_ids: Vec<StableHandleId>,
    coverage_ids: Vec<CoverageId>,
}

impl GraphAcceptedRelationProposal {
    #[must_use]
    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    #[must_use]
    pub const fn edge(&self) -> &GraphEdge {
        &self.edge
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphProposalValidationReport {
    schema: Box<str>,
    report_id: Box<str>,
    batch_id: Box<str>,
    registry_bundle_id: Box<str>,
    registry_digest: Box<str>,
    accepted_entities: Vec<GraphAcceptedEntityProposal>,
    accepted_relations: Vec<GraphAcceptedRelationProposal>,
    rejections: Vec<GraphProposalRejection>,
}

impl GraphProposalValidationReport {
    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    #[must_use]
    pub fn accepted_entities(&self) -> &[GraphAcceptedEntityProposal] {
        &self.accepted_entities
    }

    #[must_use]
    pub fn accepted_relations(&self) -> &[GraphAcceptedRelationProposal] {
        &self.accepted_relations
    }

    #[must_use]
    pub fn rejections(&self) -> &[GraphProposalRejection] {
        &self.rejections
    }

    #[must_use]
    pub fn ready_for_publication(&self) -> bool {
        self.rejections.is_empty()
    }

    pub fn validate(&self, limits: GraphLimits) -> GraphResult<()> {
        if self.schema.as_ref() != GRAPH_PROPOSAL_REPORT_SCHEMA
            || self.batch_id.is_empty()
            || self.registry_bundle_id.is_empty()
            || self.registry_digest.is_empty()
            || self
                .accepted_entities
                .windows(2)
                .any(|pair| pair[0].proposal_id >= pair[1].proposal_id)
            || self
                .accepted_relations
                .windows(2)
                .any(|pair| pair[0].proposal_id >= pair[1].proposal_id)
            || self.rejections.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(GraphError::new(
                GraphErrorCode::ProposalReportInvalid,
                "graph proposal report header or order is invalid",
            ));
        }
        for item in &self.accepted_entities {
            item.node.validate(limits)?;
            validate_external_id(&item.proposal_id)?;
            if normalize(item.source_handle_ids.clone(), "source handle")? != item.source_handle_ids
                || normalize(item.coverage_ids.clone(), "coverage")? != item.coverage_ids
            {
                return Err(GraphError::new(
                    GraphErrorCode::ProposalReportInvalid,
                    "accepted entity proposal support is not canonical",
                ));
            }
        }
        for item in &self.accepted_relations {
            item.edge.validate(limits)?;
            validate_external_id(&item.proposal_id)?;
            if normalize(item.source_handle_ids.clone(), "source handle")? != item.source_handle_ids
                || normalize(item.coverage_ids.clone(), "coverage")? != item.coverage_ids
            {
                return Err(GraphError::new(
                    GraphErrorCode::ProposalReportInvalid,
                    "accepted relation proposal support is not canonical",
                ));
            }
        }
        let expected = derive_report_id(
            &self.batch_id,
            &self.registry_bundle_id,
            &self.registry_digest,
            &self.accepted_entities,
            &self.accepted_relations,
            &self.rejections,
        )?;
        if expected != self.report_id {
            return Err(GraphError::new(
                GraphErrorCode::ProposalReportIdentityMismatch,
                "graph proposal report identity does not match",
            ));
        }
        Ok(())
    }
}

/// Validates one immutable producer batch against one exact graph registry and optional base.
pub fn validate_graph_proposal_batch(
    registry: &GraphRegistryBundle,
    base: Option<&GraphSnapshot>,
    batch: &GraphProposalBatch,
    limits: GraphLimits,
) -> GraphResult<GraphProposalValidationReport> {
    registry.validate()?;
    batch.validate()?;
    limits.validate()?;
    if registry.bundle_id() != batch.registry_bundle_id()
        || registry.registry_digest() != batch.registry_digest()
    {
        return Err(GraphError::new(
            GraphErrorCode::RegistryIdentityMismatch,
            "graph proposal batch targets another registry bundle",
        ));
    }
    if let Some(base) = base {
        base.validate()?;
        if base.universe() != batch.universe() {
            return Err(GraphError::new(
                GraphErrorCode::UniverseMismatch,
                "graph proposal batch and base snapshot use different universes",
            ));
        }
        if base.generation() != batch.generation() {
            return Err(GraphError::new(
                GraphErrorCode::GenerationMismatch,
                "graph proposal batch and base snapshot use different generations",
            ));
        }
    }

    let mut accepted_entities = Vec::new();
    let mut accepted_relations = Vec::new();
    let mut rejections = Vec::new();
    let mut proposed_nodes = BTreeMap::<Box<str>, GraphNode>::new();
    let mut semantic_nodes = BTreeSet::<GraphNodeId>::new();

    for proposal in &batch.entity_proposals {
        let result = validate_entity_proposal(registry, batch, proposal, limits);
        match result {
            Ok(node) if semantic_nodes.insert(node.node_id().clone()) => {
                proposed_nodes.insert(proposal.proposal_id.clone(), node.clone());
                accepted_entities.push(GraphAcceptedEntityProposal {
                    proposal_id: proposal.proposal_id.clone(),
                    node,
                    source_handle_ids: proposal.source_handle_ids.clone(),
                    coverage_ids: proposal.coverage_ids.clone(),
                });
            }
            Ok(_) => rejections.push(GraphProposalRejection {
                proposal_id: proposal.proposal_id.clone(),
                code: GraphProposalRejectionCode::DuplicateSemanticEntity,
            }),
            Err(code) => rejections.push(GraphProposalRejection {
                proposal_id: proposal.proposal_id.clone(),
                code,
            }),
        }
    }

    for proposal in &batch.relation_proposals {
        match validate_relation_proposal(registry, base, &proposed_nodes, proposal, limits) {
            Ok(edge) => accepted_relations.push(GraphAcceptedRelationProposal {
                proposal_id: proposal.proposal_id.clone(),
                edge,
                source_handle_ids: proposal.source_handle_ids.clone(),
                coverage_ids: proposal.coverage_ids.clone(),
            }),
            Err(code) => rejections.push(GraphProposalRejection {
                proposal_id: proposal.proposal_id.clone(),
                code,
            }),
        }
    }

    accepted_entities.sort_by(|left, right| left.proposal_id.cmp(&right.proposal_id));
    accepted_relations.sort_by(|left, right| left.proposal_id.cmp(&right.proposal_id));
    rejections.sort();
    let report_id = derive_report_id(
        batch.batch_id(),
        registry.bundle_id(),
        registry.registry_digest(),
        &accepted_entities,
        &accepted_relations,
        &rejections,
    )?;
    let report = GraphProposalValidationReport {
        schema: GRAPH_PROPOSAL_REPORT_SCHEMA.into(),
        report_id,
        batch_id: batch.batch_id.clone(),
        registry_bundle_id: batch.registry_bundle_id.clone(),
        registry_digest: batch.registry_digest.clone(),
        accepted_entities,
        accepted_relations,
        rejections,
    };
    report.validate(limits)?;
    Ok(report)
}

fn validate_entity_proposal(
    registry: &GraphRegistryBundle,
    batch: &GraphProposalBatch,
    proposal: &GraphEntityProposal,
    limits: GraphLimits,
) -> Result<GraphNode, GraphProposalRejectionCode> {
    let Some(definition) = registry.entity_kind(&proposal.entity_kind_id) else {
        return Err(GraphProposalRejectionCode::UnknownEntityKind);
    };
    if !definition.allows_universe(batch.universe()) {
        return Err(GraphProposalRejectionCode::UniverseNotAllowed);
    }
    if !definition.allows_confidence(proposal.confidence) {
        return Err(GraphProposalRejectionCode::ConfidenceNotAllowed);
    }
    if proposal
        .semantic_key
        .keys()
        .map(Box::as_ref)
        .ne(definition.identity_fields().iter().map(Box::as_ref))
    {
        return Err(GraphProposalRejectionCode::IdentityFieldsMismatch);
    }
    let owner_key = derive_owner_key(&proposal.entity_kind_id, &proposal.semantic_key)
        .map_err(|_| GraphProposalRejectionCode::InvalidGraphRecord)?;
    GraphNode::new(
        batch.universe.clone(),
        batch.generation.clone(),
        proposal.entity_kind_id.clone(),
        owner_key,
        proposal
            .evidence_ids
            .iter()
            .map(EvidenceId::canonical)
            .map(String::into_boxed_str)
            .collect(),
        limits,
    )
    .map_err(|_| GraphProposalRejectionCode::InvalidGraphRecord)
}

fn validate_relation_proposal(
    registry: &GraphRegistryBundle,
    base: Option<&GraphSnapshot>,
    proposed_nodes: &BTreeMap<Box<str>, GraphNode>,
    proposal: &GraphRelationProposal,
    limits: GraphLimits,
) -> Result<GraphEdge, GraphProposalRejectionCode> {
    let Some(definition) = registry.relation_kind(&proposal.relation_kind_id) else {
        return Err(GraphProposalRejectionCode::UnknownRelationKind);
    };
    if !definition.allows_confidence(proposal.confidence) {
        return Err(GraphProposalRejectionCode::ConfidenceNotAllowed);
    }
    let source = resolve_endpoint(&proposal.source, base, proposed_nodes)
        .ok_or(GraphProposalRejectionCode::EndpointMissing)?;
    let target = resolve_endpoint(&proposal.target, base, proposed_nodes)
        .ok_or(GraphProposalRejectionCode::EndpointMissing)?;
    if !definition.allows_source_kind(source.kind())
        || !definition.allows_target_kind(target.kind())
    {
        return Err(GraphProposalRejectionCode::EndpointKindMismatch);
    }
    GraphEdge::new(
        source.node_id().clone(),
        target.node_id().clone(),
        definition.relation(),
        proposal.confidence,
        proposal
            .evidence_ids
            .iter()
            .map(EvidenceId::canonical)
            .map(String::into_boxed_str)
            .collect(),
        limits,
    )
    .map_err(|_| GraphProposalRejectionCode::InvalidGraphRecord)
}

fn resolve_endpoint<'a>(
    endpoint: &GraphProposalEndpoint,
    base: Option<&'a GraphSnapshot>,
    proposed_nodes: &'a BTreeMap<Box<str>, GraphNode>,
) -> Option<&'a GraphNode> {
    match endpoint {
        GraphProposalEndpoint::Existing(node_id) => {
            base.and_then(|snapshot| snapshot.node(node_id))
        }
        GraphProposalEndpoint::Proposed(proposal_id) => proposed_nodes.get(proposal_id),
    }
}

fn derive_owner_key(
    kind_id: &str,
    semantic_key: &BTreeMap<Box<str>, GraphProposalValue>,
) -> GraphResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        kind_id: &'a str,
        semantic_key: &'a BTreeMap<Box<str>, GraphProposalValue>,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: "wow-graph/entity-semantic-key/e2-a/1",
        kind_id,
        semantic_key,
    })
    .map_err(|_| {
        GraphError::new(
            GraphErrorCode::ProposalBatchInvalid,
            "graph entity semantic key cannot be canonicalized",
        )
    })?;
    Ok(format!("{kind_id}:sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str())
}

#[derive(Serialize)]
struct BatchIdentity<'a> {
    registry_bundle_id: &'a str,
    registry_digest: &'a str,
    universe: &'a GraphUniverseId,
    generation: &'a GraphGenerationId,
    source_context_id: GenerationContextId,
    producer_partition_id: &'a str,
    entity_proposals: &'a [GraphEntityProposal],
    relation_proposals: &'a [GraphRelationProposal],
}

fn derive_batch_id(identity: BatchIdentity<'_>) -> GraphResult<Box<str>> {
    let bytes = canonical_json_bytes(&identity).map_err(|_| {
        GraphError::new(
            GraphErrorCode::ProposalBatchIdentityMismatch,
            "graph proposal batch identity cannot be canonicalized",
        )
    })?;
    Ok(format!(
        "graph-proposal-batch:sha256:{}",
        hex(&Sha256::digest(bytes))
    )
    .into_boxed_str())
}

fn derive_report_id(
    batch_id: &str,
    registry_bundle_id: &str,
    registry_digest: &str,
    accepted_entities: &[GraphAcceptedEntityProposal],
    accepted_relations: &[GraphAcceptedRelationProposal],
    rejections: &[GraphProposalRejection],
) -> GraphResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        batch_id: &'a str,
        registry_bundle_id: &'a str,
        registry_digest: &'a str,
        accepted_entities: &'a [GraphAcceptedEntityProposal],
        accepted_relations: &'a [GraphAcceptedRelationProposal],
        rejections: &'a [GraphProposalRejection],
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: GRAPH_PROPOSAL_REPORT_SCHEMA,
        batch_id,
        registry_bundle_id,
        registry_digest,
        accepted_entities,
        accepted_relations,
        rejections,
    })
    .map_err(|_| {
        GraphError::new(
            GraphErrorCode::ProposalReportIdentityMismatch,
            "graph proposal report identity cannot be canonicalized",
        )
    })?;
    Ok(format!(
        "graph-proposal-report:sha256:{}",
        hex(&Sha256::digest(bytes))
    )
    .into_boxed_str())
}

fn validate_external_id(value: &str) -> GraphResult<()> {
    if value.is_empty()
        || value.len() > 1024
        || value.contains('\0')
        || value.chars().any(char::is_control)
    {
        return Err(GraphError::new(
            GraphErrorCode::ProposalBatchInvalid,
            "graph external proposal identity is invalid",
        ));
    }
    Ok(())
}

fn validate_digest(value: &str, prefix: &str) -> GraphResult<()> {
    let Some(hex) = value.strip_prefix(prefix) else {
        return Err(GraphError::new(
            GraphErrorCode::ProposalBatchInvalid,
            "graph proposal digest has an invalid prefix",
        ));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(GraphError::new(
            GraphErrorCode::ProposalBatchInvalid,
            "graph proposal digest is not canonical lowercase SHA-256",
        ));
    }
    Ok(())
}

fn normalize<T: Ord>(mut values: Vec<T>, label: &str) -> GraphResult<Vec<T>> {
    if values.len() > 256 {
        return Err(GraphError::new(
            GraphErrorCode::BudgetExceeded,
            format!("graph proposal {label} count exceeds the hard limit"),
        ));
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GraphError::new(
            GraphErrorCode::ProposalBatchInvalid,
            format!("graph proposal {label} identities must be unique"),
        ));
    }
    Ok(values)
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
