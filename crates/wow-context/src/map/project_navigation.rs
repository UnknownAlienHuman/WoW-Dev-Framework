//! Exact admitted file-root navigation for the original project container.
use super::*;
use crate::profile::{PRIMARY_PROJECT_MAP_PROFILE_BYTES, PROJECT_PROFILE};
use wow_graph::{GraphNeighborBatchLimits, GraphPartitionRead};
use wow_project::{
    ProjectId,
    graph::{ProjectGraphFile, SOURCE_GRAPH_PARTITION},
};

#[derive(Serialize)]
pub(super) struct FileNavigationRoot<'a> {
    file: &'a ProjectGraphFile,
    node: &'a GraphNode,
}

#[derive(Serialize)]
pub(super) struct ProjectNavigation<'a> {
    schema: &'static str,
    project_id: &'a ProjectId,
    source_file_roots: Vec<FileNavigationRoot<'a>>,
    pub(super) neighborhoods: Vec<GraphNeighborView<'a>>,
    root_enumeration: &'static str,
}

/// One sealed enumeration and its bounded native queries. No caller/decoded
/// file list, path lookup, synthetic graph node or root prefix can construct it.
pub struct ProjectNavigationRequest<'a> {
    universe: &'a ContextUniverseSet<'a>,
    project_id: &'a ProjectId,
    roots: Vec<FileNavigationRoot<'a>>,
    queries: Vec<GraphNeighborReadQuery>,
    neighborhoods: Vec<GraphNeighborView<'a>>,
    budget: Budget,
}
impl<'a> ProjectNavigationRequest<'a> {
    #[must_use]
    pub fn navigation_queries(&self) -> &[GraphNeighborReadQuery] {
        &self.queries
    }
    /// Pass the actual remaining ledger to graph before materializing the batch.
    /// Full-scope logical scan work was already reserved during root enumeration;
    /// this ceiling verifies that reservation without charging it a second time.
    pub fn navigation_batch_limits(&self) -> ContextResult<GraphNeighborBatchLimits> {
        let work = self
            .queries
            .len()
            .checked_mul(self.universe.pair.graph().snapshot().edges().len())
            .ok_or_else(exhausted)?;
        Ok(GraphNeighborBatchLimits {
            max_logical_scanned_edges: u64::try_from(work).map_err(|_| exhausted())?,
            max_output_bytes: self.budget.remaining_metadata_bytes()?,
        })
    }
    /// Validate and charge one genuine native view before retaining it. The
    /// complete root batch must fit cumulatively, not just one view at a time.
    pub fn admit_neighborhood(
        &mut self,
        view: GraphNeighborView<'a>,
        stop: &AtomicBool,
    ) -> ContextResult<()> {
        checkpoint(stop)?;
        let index = self.neighborhoods.len();
        let query = self.queries.get(index).ok_or_else(support_mismatch)?;
        let root = self.roots.get(index).ok_or_else(support_mismatch)?;
        if view.query() != query || !std::ptr::eq(view.root(), root.node) {
            return Err(support_mismatch());
        }
        self.budget.retain(&view, stop)?;
        self.neighborhoods.push(view);
        Ok(())
    }
}

pub fn primary_project_navigation_queries<'a>(
    universe: &'a ContextUniverseSet<'a>,
    project_id: &ProjectId,
    graph: &GraphPartitionRead<'a>,
    stop: &AtomicBool,
) -> ContextResult<ProjectNavigationRequest<'a>> {
    checkpoint(stop)?;
    let project = universe.pair.project();
    let owner = universe.pair.graph();
    if project.configuration().project_id() != project_id || !std::ptr::eq(owner, graph.owner()) {
        return Err(root_invalid());
    }
    validate_navigation_input(universe, stop)?;
    if canonical(&PROJECT_PROFILE, universe.limits.max_metadata_bytes, stop)?.as_slice()
        != PRIMARY_PROJECT_MAP_PROFILE_BYTES
    {
        return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
    }
    let provenance = universe
        .pair
        .source_provenance()
        .ok_or_else(custody_missing)?;
    let partition = owner
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or_else(custody_missing)?;
    if provenance.files().is_empty() {
        return Err(custody_missing());
    }
    let mut budget = Budget::new(universe.limits);
    let mut roots = Vec::new();
    // All admitted roots are mandatory. Reserve every native adjacency scan
    // before handing the batch to service. Query guards still run independently.
    budget.work(
        provenance
            .files()
            .len()
            .checked_mul(owner.snapshot().edges().len())
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let lookup_work = partition.batch().entity_proposals().len().max(1).ilog2() as usize
        + partition.report().accepted_entities().len().max(1).ilog2() as usize
        + owner.snapshot().nodes().len().max(1).ilog2() as usize
        + 4;
    for file in provenance.files() {
        budget.work(lookup_work, stop)?;
        let proposal = partition
            .batch()
            .entity_proposal(&file.proposal_id)
            .ok_or_else(custody_missing)?;
        if proposal.entity_kind_id() != "source_file"
            || !proposal
                .source_handle_ids()
                .contains(&file.source_handle_id)
            || !proposal.evidence_ids().contains(&file.evidence_id)
        {
            return Err(custody_missing());
        }
        let node = graph
            .published_entity(SOURCE_GRAPH_PARTITION, &file.proposal_id, stop)
            .map_err(graph_error)?;
        let root = FileNavigationRoot { file, node };
        budget.retain(&root, stop)?;
        roots.push(root);
    }
    budget.work(
        roots
            .len()
            .checked_mul(roots.len().max(1).ilog2() as usize + 1)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    roots.sort_by(|a, b| a.node.node_id().cmp(b.node.node_id()));
    if roots
        .windows(2)
        .any(|pair| pair[0].node.node_id() == pair[1].node.node_id())
    {
        return Err(support_mismatch());
    }
    let mut queries = Vec::new();
    for root in &roots {
        budget.work(1, stop)?;
        let query = navigation_query(universe, root.node.node_id().clone(), stop)?;
        budget.retain(&query, stop)?;
        queries.push(query);
    }
    Ok(ProjectNavigationRequest {
        universe,
        project_id: project.configuration().project_id(),
        roots,
        queries,
        neighborhoods: Vec::new(),
        budget,
    })
}

/// Require one original native view for every enumerated file root, in exact
/// query order. Deduplicate native nodes/edges for support, retaining each
/// original neighborhood and its own coverage/truncation count in the map.
pub fn prepare_primary_project_map<'a>(
    mut request: ProjectNavigationRequest<'a>,
    stop: &AtomicBool,
) -> ContextResult<ProjectMapSelection<'a>> {
    checkpoint(stop)?;
    if request.neighborhoods.len() != request.queries.len() {
        return Err(support_mismatch());
    }
    let snapshot = request.universe.pair.graph().snapshot();
    let mut nodes = BTreeMap::new();
    let mut edges = BTreeMap::new();
    for ((view, query), root) in request
        .neighborhoods
        .iter()
        .zip(&request.queries)
        .zip(&request.roots)
    {
        request.budget.work(1, stop)?;
        if view.query() != query || !std::ptr::eq(view.root(), root.node) {
            return Err(support_mismatch());
        }
        // Native scan work was reserved before execution. Retain actual output
        // before adding its references to the cumulative native indexes.
        for node in std::iter::once(view.root()).chain(view.adjacent_nodes().iter().copied()) {
            request.budget.work(1, stop)?;
            if !std::ptr::eq(
                node,
                snapshot.node(node.node_id()).ok_or_else(support_mismatch)?,
            ) {
                return Err(support_mismatch());
            }
            nodes.insert(node.node_id(), node);
        }
        for edge in view.edges() {
            request.budget.work(1, stop)?;
            if !std::ptr::eq(
                *edge,
                snapshot.edge(edge.edge_id()).ok_or_else(support_mismatch)?,
            ) {
                return Err(support_mismatch());
            }
            edges.insert(edge.edge_id(), *edge);
        }
    }
    let navigation = MapNavigation::Project(ProjectNavigation {
        schema: "wow-context/project-navigation/1",
        project_id: request.project_id,
        source_file_roots: request.roots,
        neighborhoods: request.neighborhoods,
        root_enumeration: "complete_for_retained_primary_source_files",
    });
    finish_selection(
        request.universe,
        navigation,
        nodes.into_values().collect(),
        edges.into_values().collect(),
        request.budget,
        stop,
    )
}
