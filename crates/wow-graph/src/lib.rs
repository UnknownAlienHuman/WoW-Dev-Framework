#![forbid(unsafe_code)]

//! Immutable exact-generation graph records and bounded evidence-aware queries.

mod error;
mod identity;
mod model;
mod persistent;
mod query;

pub use error::{GraphError, GraphErrorCode, GraphResult};
pub use identity::{
    GraphEdgeId, GraphGenerationId, GraphNodeId, GraphPublicationKey, GraphSnapshotId,
    GraphUniverseId,
};
pub use model::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphEdge, GraphLimits, GraphNode,
    GraphRelationKind, GraphSnapshot,
};
pub use persistent::{
    GRAPH_SNAPSHOT_OBJECT_KIND, GRAPH_SNAPSHOT_OBJECT_SCHEMA_VERSION, GRAPH_STORE_SCHEMA,
    PersistentGraphStore, PublishedGraphSnapshot, StoredGraphSnapshot,
};
pub use query::{GraphDirection, GraphNeighborQuery, GraphNeighborResult, GraphQueryState};
