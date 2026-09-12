use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{
    GraphEdgeId, GraphError, GraphErrorCode, GraphGenerationId, GraphNodeId, GraphResult,
    GraphSnapshotId, GraphUniverseId,
};

pub const GRAPH_SNAPSHOT_SCHEMA: &str = "wow-graph/snapshot/e2-a/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphLimits {
    pub max_nodes: u32,
    pub max_edges: u32,
    pub max_coverage_records: u32,
    pub max_evidence_per_record: u32,
    pub max_query_edges: u32,
}

impl GraphLimits {
    pub fn new(
        max_nodes: u32,
        max_edges: u32,
        max_coverage_records: u32,
        max_evidence_per_record: u32,
        max_query_edges: u32,
    ) -> GraphResult<Self> {
        let limits = Self {
            max_nodes,
            max_edges,
            max_coverage_records,
            max_evidence_per_record,
            max_query_edges,
        };
        limits.validate()?;
        Ok(limits)
    }

    pub(crate) fn validate(self) -> GraphResult<()> {
        if self.max_nodes == 0
            || self.max_nodes > 1_000_000
            || self.max_edges == 0
            || self.max_edges > 4_000_000
            || self.max_coverage_records == 0
            || self.max_coverage_records > 1024
            || self.max_evidence_per_record == 0
            || self.max_evidence_per_record > 256
            || self.max_query_edges == 0
            || self.max_query_edges > 100_000
        {
            return Err(GraphError::new(
                GraphErrorCode::LimitsInvalid,
                "graph limits are outside the reviewed profile",
            ));
        }
        Ok(())
    }
}

impl Default for GraphLimits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_edges: 500_000,
            max_coverage_records: 128,
            max_evidence_per_record: 64,
            max_query_edges: 10_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphConfidence {
    Proven,
    Derived,
    Possible,
    Candidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphRelationKind {
    Calls,
    ReadsState,
    WritesState,
    RegistersNativeEvent,
    HandlesNativeEvent,
    BridgesNativeEvent,
    EmitsCustomSignal,
    HandlesCustomSignal,
    RegistersCvarCallback,
    SetsScript,
    HooksScript,
    SecureHooksFunction,
    DependsOn,
    Owns,
    Loads,
    Inherits,
    MixesIn,
    FactoryCreates,
    UsesApi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphCoverageState {
    Complete,
    Partial,
    NotEvaluated,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNode {
    node_id: GraphNodeId,
    universe: GraphUniverseId,
    generation: GraphGenerationId,
    kind: Box<str>,
    owner_key: Box<str>,
    evidence_ids: Vec<Box<str>>,
}

impl GraphNode {
    pub fn new(
        universe: GraphUniverseId,
        generation: GraphGenerationId,
        kind: impl Into<Box<str>>,
        owner_key: impl Into<Box<str>>,
        evidence_ids: Vec<Box<str>>,
        limits: GraphLimits,
    ) -> GraphResult<Self> {
        limits.validate()?;
        let kind = kind.into();
        let owner_key = owner_key.into();
        validate_component(&kind, 128, "node kind")?;
        validate_owner_key(&owner_key)?;
        let evidence_ids = normalize_evidence(evidence_ids, limits)?;
        let node_id = derive_node_id(&universe, &generation, &kind, &owner_key)?;
        Ok(Self {
            node_id,
            universe,
            generation,
            kind,
            owner_key,
            evidence_ids,
        })
    }

    pub fn validate(&self, limits: GraphLimits) -> GraphResult<()> {
        limits.validate()?;
        validate_component(&self.kind, 128, "node kind")?;
        validate_owner_key(&self.owner_key)?;
        let evidence = normalize_evidence(self.evidence_ids.clone(), limits)?;
        if evidence != self.evidence_ids
            || self.node_id
                != derive_node_id(
                    &self.universe,
                    &self.generation,
                    &self.kind,
                    &self.owner_key,
                )?
        {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotInvalid,
                "graph node identity or evidence order is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn node_id(&self) -> &GraphNodeId {
        &self.node_id
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
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub fn owner_key(&self) -> &str {
        &self.owner_key
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[Box<str>] {
        &self.evidence_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEdge {
    edge_id: GraphEdgeId,
    from: GraphNodeId,
    to: GraphNodeId,
    relation: GraphRelationKind,
    confidence: GraphConfidence,
    evidence_ids: Vec<Box<str>>,
}

impl GraphEdge {
    pub fn new(
        from: GraphNodeId,
        to: GraphNodeId,
        relation: GraphRelationKind,
        confidence: GraphConfidence,
        evidence_ids: Vec<Box<str>>,
        limits: GraphLimits,
    ) -> GraphResult<Self> {
        limits.validate()?;
        if from == to {
            return Err(GraphError::new(
                GraphErrorCode::SelfEdgeInvalid,
                "graph self-edge is not valid in the E2-A profile",
            ));
        }
        let evidence_ids = normalize_evidence(evidence_ids, limits)?;
        let edge_id = derive_edge_id(&from, &to, relation, confidence, &evidence_ids)?;
        Ok(Self {
            edge_id,
            from,
            to,
            relation,
            confidence,
            evidence_ids,
        })
    }

    pub fn validate(&self, limits: GraphLimits) -> GraphResult<()> {
        limits.validate()?;
        if self.from == self.to {
            return Err(GraphError::new(
                GraphErrorCode::SelfEdgeInvalid,
                "graph self-edge is not valid in the E2-A profile",
            ));
        }
        let evidence = normalize_evidence(self.evidence_ids.clone(), limits)?;
        if evidence != self.evidence_ids
            || self.edge_id
                != derive_edge_id(
                    &self.from,
                    &self.to,
                    self.relation,
                    self.confidence,
                    &self.evidence_ids,
                )?
        {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotInvalid,
                "graph edge identity or evidence order is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn edge_id(&self) -> &GraphEdgeId {
        &self.edge_id
    }

    #[must_use]
    pub fn from(&self) -> &GraphNodeId {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &GraphNodeId {
        &self.to
    }

    #[must_use]
    pub const fn relation(&self) -> GraphRelationKind {
        self.relation
    }

    #[must_use]
    pub const fn confidence(&self) -> GraphConfidence {
        self.confidence
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[Box<str>] {
        &self.evidence_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphCoverageRecord {
    relation: GraphRelationKind,
    state: GraphCoverageState,
    negative_authority: bool,
    blocker_ids: Vec<Box<str>>,
}

impl GraphCoverageRecord {
    pub fn new(
        relation: GraphRelationKind,
        state: GraphCoverageState,
        negative_authority: bool,
        blocker_ids: Vec<Box<str>>,
        limits: GraphLimits,
    ) -> GraphResult<Self> {
        limits.validate()?;
        let blocker_ids = normalize_evidence(blocker_ids, limits)?;
        if negative_authority && state != GraphCoverageState::Complete {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotInvalid,
                "negative authority requires complete graph coverage",
            ));
        }
        if state == GraphCoverageState::Failed && blocker_ids.is_empty() {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotInvalid,
                "failed graph coverage requires an explicit blocker",
            ));
        }
        Ok(Self {
            relation,
            state,
            negative_authority,
            blocker_ids,
        })
    }

    pub fn validate(&self, limits: GraphLimits) -> GraphResult<()> {
        let rebuilt = Self::new(
            self.relation,
            self.state,
            self.negative_authority,
            self.blocker_ids.clone(),
            limits,
        )?;
        if rebuilt != *self {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotInvalid,
                "graph coverage record is not canonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn relation(&self) -> GraphRelationKind {
        self.relation
    }

    #[must_use]
    pub const fn state(&self) -> GraphCoverageState {
        self.state
    }

    #[must_use]
    pub const fn negative_authority(&self) -> bool {
        self.negative_authority
    }

    #[must_use]
    pub fn blocker_ids(&self) -> &[Box<str>] {
        &self.blocker_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphSnapshot {
    schema: Box<str>,
    snapshot_id: GraphSnapshotId,
    universe: GraphUniverseId,
    generation: GraphGenerationId,
    limits: GraphLimits,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    coverage: Vec<GraphCoverageRecord>,
}

impl GraphSnapshot {
    pub fn build(
        universe: GraphUniverseId,
        generation: GraphGenerationId,
        limits: GraphLimits,
        mut nodes: Vec<GraphNode>,
        mut edges: Vec<GraphEdge>,
        mut coverage: Vec<GraphCoverageRecord>,
    ) -> GraphResult<Self> {
        limits.validate()?;
        if nodes.len() > limits.max_nodes as usize
            || edges.len() > limits.max_edges as usize
            || coverage.len() > limits.max_coverage_records as usize
        {
            return Err(GraphError::new(
                GraphErrorCode::BudgetExceeded,
                "graph snapshot exceeds configured limits",
            ));
        }
        for node in &nodes {
            node.validate(limits)?;
            if node.universe() != &universe {
                return Err(GraphError::new(
                    GraphErrorCode::UniverseMismatch,
                    "graph node belongs to another universe",
                ));
            }
            if node.generation() != &generation {
                return Err(GraphError::new(
                    GraphErrorCode::GenerationMismatch,
                    "graph node belongs to another generation",
                ));
            }
        }
        nodes.sort_by(|left, right| left.node_id().cmp(right.node_id()));
        if nodes
            .windows(2)
            .any(|pair| pair[0].node_id() == pair[1].node_id())
        {
            return Err(GraphError::new(
                GraphErrorCode::NodeDuplicate,
                "graph snapshot contains a duplicate node",
            ));
        }
        let node_ids = nodes
            .iter()
            .map(|node| node.node_id().clone())
            .collect::<BTreeSet<_>>();
        for edge in &edges {
            edge.validate(limits)?;
            if !node_ids.contains(edge.from()) || !node_ids.contains(edge.to()) {
                return Err(GraphError::new(
                    GraphErrorCode::EndpointMissing,
                    "graph edge references a missing endpoint",
                ));
            }
        }
        edges.sort_by(|left, right| left.edge_id().cmp(right.edge_id()));
        if edges
            .windows(2)
            .any(|pair| pair[0].edge_id() == pair[1].edge_id())
        {
            return Err(GraphError::new(
                GraphErrorCode::EdgeDuplicate,
                "graph snapshot contains a duplicate edge",
            ));
        }
        for record in &coverage {
            record.validate(limits)?;
        }
        coverage.sort_by_key(GraphCoverageRecord::relation);
        if coverage
            .windows(2)
            .any(|pair| pair[0].relation() == pair[1].relation())
        {
            return Err(GraphError::new(
                GraphErrorCode::CoverageDuplicate,
                "graph snapshot contains duplicate relation coverage",
            ));
        }
        let snapshot_id =
            derive_snapshot_id(&universe, &generation, limits, &nodes, &edges, &coverage)?;
        Ok(Self {
            schema: GRAPH_SNAPSHOT_SCHEMA.into(),
            snapshot_id,
            universe,
            generation,
            limits,
            nodes,
            edges,
            coverage,
        })
    }

    pub fn validate(&self) -> GraphResult<()> {
        if self.schema.as_ref() != GRAPH_SNAPSHOT_SCHEMA {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotInvalid,
                "graph snapshot schema is unsupported",
            ));
        }
        let rebuilt = Self::build(
            self.universe.clone(),
            self.generation.clone(),
            self.limits,
            self.nodes.clone(),
            self.edges.clone(),
            self.coverage.clone(),
        )?;
        if rebuilt.snapshot_id != self.snapshot_id
            || rebuilt.nodes != self.nodes
            || rebuilt.edges != self.edges
            || rebuilt.coverage != self.coverage
        {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotIdentityMismatch,
                "graph snapshot identity or canonical order does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.snapshot_id
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
    pub const fn limits(&self) -> GraphLimits {
        self.limits
    }

    #[must_use]
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }

    #[must_use]
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    #[must_use]
    pub fn coverage(&self) -> &[GraphCoverageRecord] {
        &self.coverage
    }

    #[must_use]
    pub fn node(&self, node_id: &GraphNodeId) -> Option<&GraphNode> {
        self.nodes
            .binary_search_by(|node| node.node_id().cmp(node_id))
            .ok()
            .map(|index| &self.nodes[index])
    }

    #[must_use]
    pub(crate) fn coverage_for(&self, relation: GraphRelationKind) -> Option<&GraphCoverageRecord> {
        self.coverage
            .binary_search_by_key(&relation, GraphCoverageRecord::relation)
            .ok()
            .map(|index| &self.coverage[index])
    }
}

fn derive_node_id(
    universe: &GraphUniverseId,
    generation: &GraphGenerationId,
    kind: &str,
    owner_key: &str,
) -> GraphResult<GraphNodeId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        universe: &'a GraphUniverseId,
        generation: &'a GraphGenerationId,
        kind: &'a str,
        owner_key: &'a str,
    }
    let bytes = canonical_json_bytes(&Identity {
        universe,
        generation,
        kind,
        owner_key,
    })
    .map_err(|_| GraphError::new(GraphErrorCode::SnapshotInvalid, "node identity failed"))?;
    GraphNodeId::new(format!("graph-node:sha256:{}", hex(&Sha256::digest(bytes))))
}

fn derive_edge_id(
    from: &GraphNodeId,
    to: &GraphNodeId,
    relation: GraphRelationKind,
    confidence: GraphConfidence,
    evidence_ids: &[Box<str>],
) -> GraphResult<GraphEdgeId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        from: &'a GraphNodeId,
        to: &'a GraphNodeId,
        relation: GraphRelationKind,
        confidence: GraphConfidence,
        evidence_ids: &'a [Box<str>],
    }
    let bytes = canonical_json_bytes(&Identity {
        from,
        to,
        relation,
        confidence,
        evidence_ids,
    })
    .map_err(|_| GraphError::new(GraphErrorCode::SnapshotInvalid, "edge identity failed"))?;
    GraphEdgeId::new(format!("graph-edge:sha256:{}", hex(&Sha256::digest(bytes))))
}

fn derive_snapshot_id(
    universe: &GraphUniverseId,
    generation: &GraphGenerationId,
    limits: GraphLimits,
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    coverage: &[GraphCoverageRecord],
) -> GraphResult<GraphSnapshotId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        universe: &'a GraphUniverseId,
        generation: &'a GraphGenerationId,
        limits: GraphLimits,
        nodes: &'a [GraphNode],
        edges: &'a [GraphEdge],
        coverage: &'a [GraphCoverageRecord],
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: GRAPH_SNAPSHOT_SCHEMA,
        universe,
        generation,
        limits,
        nodes,
        edges,
        coverage,
    })
    .map_err(|_| GraphError::new(GraphErrorCode::SnapshotInvalid, "snapshot identity failed"))?;
    GraphSnapshotId::new(format!(
        "graph-snapshot:sha256:{}",
        hex(&Sha256::digest(bytes))
    ))
}

fn normalize_evidence(
    mut values: Vec<Box<str>>,
    limits: GraphLimits,
) -> GraphResult<Vec<Box<str>>> {
    if values.len() > limits.max_evidence_per_record as usize {
        return Err(GraphError::new(
            GraphErrorCode::BudgetExceeded,
            "graph evidence count exceeds configured limits",
        ));
    }
    for value in &values {
        validate_component(value, 512, "evidence id")?;
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GraphError::new(
            GraphErrorCode::EvidenceInvalid,
            "graph evidence identities must be unique",
        ));
    }
    Ok(values)
}

fn validate_component(value: &str, max: usize, label: &str) -> GraphResult<()> {
    if value.is_empty()
        || value.len() > max
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
    {
        return Err(GraphError::new(
            GraphErrorCode::IdentifierInvalid,
            format!("invalid {label}"),
        ));
    }
    Ok(())
}

fn validate_owner_key(value: &str) -> GraphResult<()> {
    if value.is_empty()
        || value.len() > 2048
        || value
            .chars()
            .any(|character| character.is_control() || matches!(character, '"' | '\\'))
    {
        return Err(GraphError::new(
            GraphErrorCode::IdentifierInvalid,
            "invalid graph owner key",
        ));
    }
    Ok(())
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
