#![forbid(unsafe_code)]

//! Immutable exact-generation graph records and bounded evidence-aware queries.

mod error;
mod identity;
mod model;
mod partition;
mod partition_session;
mod paths;
mod persistent;
mod proposal;
mod query;
mod registry;

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
    GRAPH_PARTITION_SNAPSHOT_SCHEMA, GraphPartitionChange, GraphPartitionReplacement,
    GraphPartitionReplacementPlan, GraphPartitionSnapshot, GraphProducerPartition,
    MAX_GRAPH_PRODUCER_PARTITIONS,
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
    GRAPH_PROPOSAL_BATCH_SCHEMA, GRAPH_PROPOSAL_REPORT_SCHEMA, GraphAcceptedEntityProposal,
    GraphAcceptedRelationProposal, GraphEntityProposal, GraphProposalBatch, GraphProposalEndpoint,
    GraphProposalRejection, GraphProposalRejectionCode, GraphProposalValidationReport,
    GraphProposalValue, GraphRelationProposal, GraphRelationProposalInput,
    validate_graph_proposal_batch,
};
pub use query::{GraphDirection, GraphNeighborQuery, GraphNeighborResult, GraphQueryState};
pub use registry::{
    GRAPH_REGISTRY_SCHEMA, GraphEntityKindDefinition, GraphRegistryBundle,
    GraphRelationKindDefinition,
};
