//! Project-owned coherent handoff to the manifested store. Semantic identities
//! precede logical membership, publication-set identity and store generation.
use super::{ProjectReplay, invalid};
use crate::graph::{SOURCE_GRAPH_PARTITION, build_source_graph_proposals};
use crate::{ProjectPhase, ProjectPublisher, ProjectResult, ProjectView};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
use wow_graph::GraphPartitionSnapshot;
use wow_store::project::{PartitionRecord, ReadSnapshot};

pub const STORAGE_SCHEMAS: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-pair.v1",
];
/// Exact catalog of already published physical-input epochs. It is never widened
/// in place; reopening it preserves its original epoch and membership identities.
pub const STORAGE_SCHEMAS_V1: &[&str] = &["wow-project.live-replay.v1", "wow-project.live-pair.v1"];
pub const STORAGE_SCHEMAS_V2: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-pair.v1",
];
pub const STORAGE_SCHEMAS_V3: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-pair.v1",
];
pub const STORAGE_CHECK: &str = "wow-project.live-pair-native-replay.v1";
const HEADER_KEY: &str = "live.project.header";
const REPLAY_KEY: &str = "live.project.replay";
const HEADER_SCHEMA: &str = "wow-project/live-project-graph-pair/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PairHeader {
    schema: String,
    project_snapshot_id: String,
    analyzer_snapshot_id: String,
    graph_snapshot_id: String,
    publication_set_id: String,
}

/// A handoff from actual live owners. It cannot be supplied by deserializing a
/// success flag; the underlying store request is assembled from validated records.
pub struct ProjectPublicationBundle {
    records: Vec<PartitionRecord>,
    bindings: BTreeMap<String, String>,
}
impl ProjectPublicationBundle {
    pub fn build(
        publisher: &ProjectPublisher,
        graph: &GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let replay = ProjectReplay::capture(publisher, stop)?;
        let project = publisher
            .current_snapshot()
            .ok_or_else(invalid)?
            .open_view();
        validate_pair(&project, graph, stop)?;
        plan(&replay, &project, graph, stop)
    }

    /// Restore an exact retained native archive through real owner analysis
    /// before admitting its original project/graph pair to publication.
    pub fn from_replay(
        replay: &ProjectReplay,
        graph: &GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let project = replay.hydrate(stop)?;
        validate_pair(&project, graph, stop)?;
        plan(replay, &project, graph, stop)
    }
    pub fn into_parts(self) -> (Vec<PartitionRecord>, BTreeMap<String, String>) {
        (self.records, self.bindings)
    }
}

/// Executable immutable owners reconstructed under one held read snapshot.
/// This is distinct from serialized storage records and public result DTOs.
pub struct AcquiredProjectPair {
    project: ProjectView,
    publisher: ProjectPublisher,
    supports_physical_update: bool,
    graph: GraphPartitionSnapshot,
    publication_set_id: String,
}
impl AcquiredProjectPair {
    pub fn project(&self) -> &ProjectView {
        &self.project
    }
    pub fn graph(&self) -> &GraphPartitionSnapshot {
        &self.graph
    }
    pub fn publication_set_id(&self) -> &str {
        &self.publication_set_id
    }
    /// Consume the validated owner; legacy and loader profiles cannot be updated
    /// through the physical-input protocol.
    pub fn into_update_publisher(self) -> ProjectResult<ProjectPublisher> {
        if !self.supports_physical_update {
            return Err(crate::ProjectError::new(
                crate::ProjectErrorCode::DeferredCapability,
                ProjectPhase::Update,
                "retained archive has no physical update capability",
            ));
        }
        Ok(self.publisher)
    }
    pub fn read(read: &ReadSnapshot, stop: &AtomicBool) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        let header: PairHeader = load(read, HEADER_KEY, "wow-project.live-pair.v1", stop)?;
        if header.schema != HEADER_SCHEMA {
            return Err(invalid());
        }
        let replay_schema = read
            .manifest()
            .members
            .iter()
            .find(|member| member.key == REPLAY_KEY)
            .map(|member| member.schema.as_str())
            .ok_or_else(invalid)?;
        if !matches!(
            replay_schema,
            "wow-project.live-replay.v1"
                | "wow-project.live-replay.v2"
                | "wow-project.live-replay.v3"
                | "wow-project.live-replay.v4"
        ) {
            return Err(invalid());
        }
        let replay: ProjectReplay = load(read, REPLAY_KEY, replay_schema, stop)?;
        if replay.storage_schema() != replay_schema {
            return Err(invalid());
        }
        let graph = GraphPartitionSnapshot::read_stored(read, stop).map_err(graph_error)?;
        let publisher = replay.hydrate_owner(stop)?;
        let project = publisher.open_current()?;
        validate_pair(&project, &graph, stop)?;
        let expected = plan(&replay, &project, &graph, stop)?;
        if expected.bindings != read.manifest().bindings
            || expected.records.len() != read.manifest().members.len()
            || read.manifest().owner != graph.snapshot().universe().as_str()
        {
            return Err(invalid());
        }
        for (record, member) in expected.records.iter().zip(&read.manifest().members) {
            crate::analyzer::checkpoint(stop)?;
            if record.key() != member.key || record.version() != &member.version {
                return Err(invalid());
            }
        }
        let expected_header: PairHeader = expected
            .records
            .iter()
            .find(|r| r.key() == HEADER_KEY)
            .ok_or_else(invalid)?
            .decode()
            .map_err(|_| invalid())?;
        if header != expected_header {
            return Err(invalid());
        }
        Ok(Self {
            project,
            publisher,
            supports_physical_update: replay.supports_physical_update(),
            graph,
            publication_set_id: header.publication_set_id,
        })
    }
}

fn validate_pair(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    graph.validate(stop).map_err(graph_error)?;
    // Reuse the source owner over the real replayed session. No metadata receipt
    // is relabeled as a live project or treated as an executable analyzer.
    let (registry, batch, mut coverage, _, limits) =
        build_source_graph_proposals(project, stop)?.into_parts();
    // Partition publication canonically orders this exact owner-provided set.
    coverage.sort_by_key(wow_graph::GraphCoverageRecord::relation);
    let source = graph
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or_else(invalid)?;
    if graph.registry() != &registry
        || source.batch() != &batch
        || source.coverage() != coverage
        || graph.source_context_id() != project.snapshot().generation_context().context_id()
    {
        return Err(invalid());
    }
    let foundation = wow_graph::GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage,
    )
    .map_err(graph_error)?;
    if graph.foundation() != &foundation {
        return Err(invalid());
    }
    Ok(())
}

fn plan(
    replay: &ProjectReplay,
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<ProjectPublicationBundle> {
    let mut records = graph.storage_records(stop).map_err(graph_error)?;
    records.push(
        PartitionRecord::new(REPLAY_KEY, replay.storage_schema(), replay)
            .map_err(|_| super::exhausted())?,
    );
    records.sort_by(|a, b| a.key().cmp(b.key()));
    // PartitionVersionId already binds key, schema, byte count and exact payload.
    let membership = records
        .iter()
        .map(|r| (r.key(), r.version()))
        .collect::<Vec<_>>();
    // All graph/project/analyzer IDs and logical member versions are already
    // stable. Neither current, epoch nor store generation enters this identity.
    let publication_set_id = crate::identity::canonical_id(
        "project-publication-set:sha256:",
        "wow-project/live-project-publication-set/1",
        &(
            project.snapshot_id(),
            project.analyzer_snapshot_id(),
            graph.snapshot().snapshot_id(),
            &membership,
        ),
        ProjectPhase::Publication,
    )?
    .into_string();
    let header = PairHeader {
        schema: HEADER_SCHEMA.into(),
        project_snapshot_id: project.snapshot_id().into(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        graph_snapshot_id: graph.snapshot().snapshot_id().as_str().into(),
        publication_set_id: publication_set_id.clone(),
    };
    records.push(
        PartitionRecord::new(HEADER_KEY, "wow-project.live-pair.v1", &header)
            .map_err(|_| invalid())?,
    );
    records.sort_by(|a, b| a.key().cmp(b.key()));
    let bindings = BTreeMap::from([
        ("scope".into(), "native-live-project-pair-v1".into()),
        ("project_publication_set_id".into(), publication_set_id),
        ("project_snapshot_id".into(), project.snapshot_id().into()),
        (
            "analyzer_snapshot_id".into(),
            project.analyzer_snapshot_id().into(),
        ),
        (
            "graph_snapshot_id".into(),
            graph.snapshot().snapshot_id().as_str().into(),
        ),
        (
            "project_generation_id".into(),
            project.project_generation().to_string(),
        ),
        (
            "source_context_id".into(),
            graph.source_context_id().to_string(),
        ),
        (
            "profile_id".into(),
            project
                .configuration()
                .selected_profile()
                .profile_id()
                .to_string(),
        ),
        (
            "reference_generation_id".into(),
            project.configuration().reference_generation().to_string(),
        ),
    ]);
    Ok(ProjectPublicationBundle { records, bindings })
}
fn load<T: serde::de::DeserializeOwned>(
    read: &ReadSnapshot,
    key: &str,
    schema: &str,
    stop: &AtomicBool,
) -> ProjectResult<T> {
    crate::analyzer::checkpoint(stop)?;
    if !read
        .manifest()
        .members
        .iter()
        .any(|m| m.key == key && m.schema == schema)
    {
        return Err(invalid());
    }
    read.record(key, stop)
        .map_err(store_error)?
        .ok_or_else(invalid)?
        .decode()
        .map_err(|_| invalid())
}
fn graph_error(error: wow_graph::GraphError) -> crate::ProjectError {
    match error.code() {
        wow_graph::GraphErrorCode::Cancelled => crate::ProjectError::new(
            crate::ProjectErrorCode::AnalysisCancelled,
            ProjectPhase::Publication,
            "native project pair validation cancelled",
        ),
        wow_graph::GraphErrorCode::BudgetExceeded => super::exhausted(),
        _ => invalid(),
    }
}
fn store_error(error: wow_store::StoreError) -> crate::ProjectError {
    match error.code() {
        wow_store::StoreErrorCode::Cancelled => crate::ProjectError::new(
            crate::ProjectErrorCode::AnalysisCancelled,
            ProjectPhase::Publication,
            "native project pair acquisition cancelled",
        ),
        wow_store::StoreErrorCode::BudgetExceeded | wow_store::StoreErrorCode::ObjectTooLarge => {
            super::exhausted()
        }
        _ => invalid(),
    }
}
