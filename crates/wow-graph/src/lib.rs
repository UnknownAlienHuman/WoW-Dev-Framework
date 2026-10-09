#![forbid(unsafe_code)]

//! Immutable exact-generation graph records and bounded evidence-aware queries.

mod assertion_records;
mod assertion_validation;
mod axes;
mod direct;
mod error;
mod evidence;
mod explain;
mod identity;
mod model;
mod partition;
mod partition_session;
mod paths;
mod persistent;
mod proposal;
mod query;
mod registry;
mod subgraph;

pub use assertion_records::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRecords, GraphAssertionRef,
    GraphConflictKind, GraphConflictRecord, GraphDerivationRecord, GraphLocalAssertion,
};
pub use error::{GraphError, GraphErrorCode, GraphResult};
pub use identity::{
    GraphEdgeId, GraphGenerationId, GraphNodeId, GraphPublicationKey, GraphSnapshotId,
    GraphUniverseId,
};
pub use model::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphEdge, GraphLimits, GraphNode,
    GraphRelationKind, GraphSnapshot,
};
pub use partition::{
    GRAPH_PARTITION_SNAPSHOT_SCHEMA, GRAPH_PARTITION_SNAPSHOT_SCHEMA_V2, GraphPartitionChange,
    GraphPartitionReplacement, GraphPartitionReplacementPlan, GraphPartitionSnapshot,
    GraphProducerPartition, MAX_GRAPH_PRODUCER_PARTITIONS,
};
pub use partition_session::GraphPartitionSession;
pub use paths::{
    GRAPH_PATH_QUERY_SCHEMA, GraphPath, GraphPathConfidence, GraphPathCursor, GraphPathLimits,
    GraphPathQuery, GraphPathResult, GraphPathTruncation,
};
pub use persistent::{
    GRAPH_SNAPSHOT_OBJECT_KIND, GRAPH_SNAPSHOT_OBJECT_SCHEMA_VERSION, GRAPH_STORE_SCHEMA,
    PersistentGraphStore, PublishedGraphSnapshot, StoredGraphSnapshot,
};
pub use proposal::{
    GRAPH_PROPOSAL_BATCH_SCHEMA, GRAPH_PROPOSAL_BATCH_SCHEMA_V2, GRAPH_PROPOSAL_REPORT_SCHEMA,
    GraphAcceptedEntityProposal, GraphAcceptedRelationProposal, GraphEntityProposal,
    GraphProposalBatch, GraphProposalEndpoint, GraphProposalRejection, GraphProposalRejectionCode,
    GraphProposalValidationReport, GraphProposalValue, GraphRelationProposal,
    GraphRelationProposalInput, validate_graph_proposal_batch,
};
pub use query::{GraphDirection, GraphNeighborQuery, GraphNeighborResult, GraphQueryState};
pub use registry::{
    GRAPH_REGISTRY_SCHEMA, GraphEntityKindDefinition, GraphRegistryBundle,
    GraphRelationKindDefinition,
};

pub use subgraph::{
    GRAPH_DIRECTED_SUBGRAPH_QUERY_SCHEMA, GRAPH_SUBGRAPH_QUERY_SCHEMA, GraphRelationDirection,
    GraphSubgraphConfidence, GraphSubgraphLimits, GraphSubgraphNode, GraphSubgraphQuery,
    GraphSubgraphResult, GraphSubgraphTruncation,
};

pub use explain::{
    GRAPH_EXPLANATION_SCHEMA, GraphAssertionSupport, GraphConflictObservation,
    GraphCoverageObservation, GraphCoverageOrigin, GraphDerivationObservation, GraphExplainLimits,
    GraphExplainQuery, GraphExplainSubject, GraphExplainedRecord, GraphExplanation,
    GraphExplanationBoundary, GraphExplanationCoverage, GraphExplanationRegistry,
    GraphExplanationTruncation, GraphProducerSupportOrigin,
};

pub use axes::{
    GRAPH_AXIS_PROFILE_SCHEMA, GRAPH_AXIS_PROFILE_SCHEMA_V2, GRAPH_AXIS_PROFILE_SCHEMA_V3,
    GRAPH_AXIS_QUERY_SCHEMA, GraphAxis, GraphAxisBoundary, GraphAxisProfile, GraphAxisQuery,
    GraphAxisRelation, GraphAxisResult, GraphAxisShape, GraphAxisTraversal,
};

pub use direct::{
    GRAPH_ENTITY_QUERY_SCHEMA, GRAPH_NEIGHBOR_READ_SCHEMA, GraphEntityLookup, GraphEntityQuery,
    GraphEntityResult, GraphNeighborReadLimits, GraphNeighborReadQuery, GraphNeighborTruncation,
    GraphNeighborView,
};

pub use evidence::{
    GraphEvidenceCatalog, GraphEvidenceResolution, GraphEvidenceResolveLimits,
    GraphEvidenceTruncation, GraphResolvedExplanation, GraphUnresolvedEvidence,
    GraphUnresolvedEvidenceReason,
};
