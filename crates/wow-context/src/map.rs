use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{CoverageRecord, SourceHandle, SourceOriginKind, SourceSpanKind, StableHandleId};
use wow_graph::{
    GraphDirection, GraphEdge, GraphEdgeId, GraphExplainQuery, GraphExplainSubject,
    GraphExplainedRecord, GraphNeighborQuery, GraphNeighborReadQuery, GraphNeighborView, GraphNode,
    GraphNodeId, GraphPathConfidence, GraphQueryState, GraphResolvedExplanation,
};
use wow_project::ProjectSourceOrigin;
use wow_project::load::{
    LoadIssue, LoadRecord, LoadSource, ProjectLoadPlan, ProjectPackageLoadPlan,
    ProjectPackageMainPlan, TocLoadContext,
};
use wow_reference::ReferenceConflict;

use crate::budget::{Budget, canonical, checkpoint, encoded_len, exhausted, identity};
use crate::error::graph_error;
use crate::profile::{PROFILE, RELATIONS};
use crate::universe::{ContextUniverseSet, ReferenceCoverage, UniverseBinding};
use crate::{ContextError, ContextErrorCode, ContextResult, PrimaryMapLimits};

pub const PROJECT_MAP_SCHEMA: &str = "wow-context/project-map/primary-direct-navigation/1";

/// Broader context capabilities are excluded; no navigation output is promoted
/// to a complete corpus, L0/L1 pack, token fit, or runtime claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMapState {
    Partial,
    Truncated,
}

#[derive(Serialize)]
struct SingleLoadCustody<'a> {
    digest: wow_core::ContentDigest<wow_core::CanonicalResult>,
    selected_toc: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_context: Option<&'a TocLoadContext>,
    sources: &'a [LoadSource],
    records: &'a [LoadRecord],
    issues: &'a [LoadIssue],
}
impl<'a> SingleLoadCustody<'a> {
    fn from_plan(plan: &'a ProjectLoadPlan) -> Self {
        Self {
            digest: plan.digest(),
            selected_toc: plan.selected_toc(),
            load_context: plan.load_context(),
            sources: plan.sources(),
            records: plan.records(),
            issues: plan.issues(),
        }
    }
}
#[derive(Serialize)]
struct LoadCustody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_toc: Option<SingleLoadCustody<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_load: Option<&'a ProjectPackageLoadPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_main: Option<&'a ProjectPackageMainPlan>,
}

#[derive(Serialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
enum SourceCustody<'a> {
    Primary {
        handle: &'a SourceHandle,
        artifact_path: &'a wow_core::NormalizedSourcePath,
        content_digest: wow_core::ContentDigest<wow_core::SourceContent>,
        byte_length: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<&'a wow_project::ProjectFileRecord>,
    },
    ReferenceContractNotEvaluated {
        handle: &'a SourceHandle,
    },
}

#[derive(Serialize)]
struct Omission {
    scope: &'static str,
    reason: &'static str,
    affected_capability: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    known_omitted: Option<u64>,
    enumeration: &'static str,
}
#[derive(Serialize)]
struct DeferredCapability<'a> {
    capability_id: &'a wow_core::CapabilityId,
    record: &'a wow_core::NotEvaluatedRecord,
}
#[derive(Serialize)]
struct BudgetReport {
    limits: PrimaryMapLimits,
    context_work_reserved: u64,
    retained_rows_bytes_reserved: u64,
    native_receipt_bytes_upper_bound: u64,
    tokens: &'static str,
    byte_accounting: &'static str,
}

/// Immutable prepared native selection. Exposes exactly the required native
/// support queries; a caller may not substitute transport-decoded assertions.
pub struct ProjectMapSelection<'a> {
    universe: &'a ContextUniverseSet<'a>,
    neighbors: GraphNeighborView<'a>,
    nodes: Vec<&'a GraphNode>,
    queries: Vec<GraphExplainQuery>,
    budget: Budget,
    native_receipt_bytes_upper_bound: u64,
}
impl ProjectMapSelection<'_> {
    #[must_use]
    pub fn explanation_queries(&self) -> &[GraphExplainQuery] {
        &self.queries
    }
    #[must_use]
    pub const fn native_receipt_bytes_upper_bound(&self) -> u64 {
        self.native_receipt_bytes_upper_bound
    }
}

/// Exact compiled native query for the selected root. The application obtains
/// a genuine GraphNeighborView, then passes it to prepare_project_map.
/// Execute through the acquired pair's graph-owned read proof to reuse its
/// immutable validation. No query revalidates or clones the full graph.
pub fn primary_navigation_query(
    universe: &ContextUniverseSet<'_>,
    root: GraphNodeId,
    stop: &AtomicBool,
) -> ContextResult<GraphNeighborReadQuery> {
    checkpoint(stop)?;
    let snapshot = universe.pair.graph().snapshot();
    if snapshot.node(&root).is_none() {
        return Err(root_invalid());
    }
    let left = universe
        .limits
        .max_input_bytes
        .checked_sub(universe.input_bytes_reserved)
        .ok_or_else(exhausted)?;
    encoded_len(snapshot, left, stop)?;
    let query = GraphNeighborQuery::new(
        root,
        GraphDirection::Both,
        RELATIONS.to_vec(),
        universe.limits.max_direct_edges,
    )
    .map_err(graph_error)?;
    GraphNeighborReadQuery::new(
        snapshot.snapshot_id().clone(),
        query,
        universe.limits.neighbors,
    )
    .map(|query| query.with_confidence(GraphPathConfidence::IncludeCandidate))
    .map_err(graph_error)
}

/// Validate a genuine native one-hop read against the exact held owner and
/// compiled profile. No query is re-executed; no graph or node is cloned.
pub fn prepare_project_map<'a>(
    universe: &'a ContextUniverseSet<'a>,
    root: GraphNodeId,
    neighbors: GraphNeighborView<'a>,
    stop: &AtomicBool,
) -> ContextResult<ProjectMapSelection<'a>> {
    checkpoint(stop)?;
    let snapshot = universe.pair.graph().snapshot();
    let limits = universe.limits;
    let expected = primary_navigation_query(universe, root, stop)?;
    if neighbors.query() != &expected
        || !std::ptr::eq(
            neighbors.root(),
            snapshot
                .node(expected.query().node_id())
                .ok_or_else(root_invalid)?,
        )
    {
        return Err(support_mismatch());
    }
    let mut budget = Budget::new(limits);
    budget.retain(&neighbors, stop)?;
    budget.work(neighbors.scanned_edges() as usize, stop)?;
    let mut nodes = Vec::new();
    for node in std::iter::once(neighbors.root()).chain(neighbors.adjacent_nodes().iter().copied())
    {
        budget.work(1, stop)?;
        let actual = snapshot.node(node.node_id()).ok_or_else(support_mismatch)?;
        if !std::ptr::eq(node, actual) {
            return Err(support_mismatch());
        }
        nodes.push(node);
    }
    nodes.sort_by(|left, right| left.node_id().cmp(right.node_id()));
    if nodes
        .windows(2)
        .any(|pair| pair[0].node_id() == pair[1].node_id())
    {
        return Err(support_mismatch());
    }
    let count = nodes
        .len()
        .checked_add(neighbors.edges().len())
        .ok_or_else(exhausted)?;
    budget.work(count, stop)?;
    // Each real resolved explanation obeys the original explain output limit.
    // Refuse before asking the application to materialize their whole batch.
    let native_receipt_bytes_upper_bound = u64::try_from(count)
        .map_err(|_| exhausted())?
        .checked_mul(u64::from(limits.explain.max_output_bytes))
        .ok_or_else(exhausted)?;
    if native_receipt_bytes_upper_bound > limits.max_metadata_bytes {
        return Err(exhausted());
    }
    let mut queries = Vec::new();
    for node in &nodes {
        charge_query(
            &mut budget,
            snapshot.snapshot_id(),
            QuerySubjectCost::Entity(node.node_id()),
            limits,
            stop,
        )?;
        queries.push(
            GraphExplainQuery::new(
                snapshot.snapshot_id().clone(),
                GraphExplainSubject::Entity(node.node_id().clone()),
                limits.explain,
            )
            .map_err(graph_error)?,
        );
    }
    for edge in neighbors.edges() {
        let actual = snapshot.edge(edge.edge_id()).ok_or_else(support_mismatch)?;
        if !std::ptr::eq(*edge, actual)
            || nodes
                .binary_search_by(|node| node.node_id().cmp(edge.from()))
                .is_err()
            || nodes
                .binary_search_by(|node| node.node_id().cmp(edge.to()))
                .is_err()
        {
            return Err(support_mismatch());
        }
        charge_query(
            &mut budget,
            snapshot.snapshot_id(),
            QuerySubjectCost::Relation(edge.edge_id()),
            limits,
            stop,
        )?;
        queries.push(
            GraphExplainQuery::new(
                snapshot.snapshot_id().clone(),
                GraphExplainSubject::Relation(edge.edge_id().clone()),
                limits.explain,
            )
            .map_err(graph_error)?,
        );
    }
    Ok(ProjectMapSelection {
        universe,
        neighbors,
        nodes,
        queries,
        budget,
        native_receipt_bytes_upper_bound,
    })
}

// Cost descriptor only: same serde fields as GraphExplainQuery, borrowing exact
// typed IDs. It never resolves/validates assertions or serves as query authority.
#[derive(Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
enum QuerySubjectCost<'a> {
    Entity(&'a GraphNodeId),
    Relation(&'a GraphEdgeId),
}
#[derive(Serialize)]
struct QueryCost<'a> {
    snapshot_id: &'a wow_graph::GraphSnapshotId,
    subject: QuerySubjectCost<'a>,
    limits: wow_graph::GraphExplainLimits,
}
fn charge_query(
    budget: &mut Budget,
    snapshot: &wow_graph::GraphSnapshotId,
    subject: QuerySubjectCost<'_>,
    limits: PrimaryMapLimits,
    stop: &AtomicBool,
) -> ContextResult<()> {
    budget.retain(
        &QueryCost {
            snapshot_id: snapshot,
            subject,
            limits: limits.explain,
        },
        stop,
    )
}

#[derive(Serialize)]
struct MapCore<'a> {
    schema: &'static str,
    profile: &'static crate::profile::ProfileDescription,
    universe_set_id: &'a str,
    binding: &'a UniverseBinding<'a>,
    project_container: &'a ProjectSourceOrigin,
    navigation: GraphNeighborView<'a>,
    supports: Vec<&'a GraphResolvedExplanation<'a>>,
    source_custody: Vec<SourceCustody<'a>>,
    load_custody: LoadCustody<'a>,
    project_coverage: &'a [CoverageRecord],
    deferred_project_capabilities: Vec<DeferredCapability<'a>>,
    reference_coverage: Vec<ReferenceCoverage<'a>>,
    reference_conflicts: &'a [ReferenceConflict],
    omissions: Vec<Omission>,
    state: ProjectMapState,
    absence_authoritative: bool,
    budget: BudgetReport,
}

/// Serialize-only artifact with borrowed native custody. Holding it holds the
/// exact owner/read lifetime; a stored JSON object cannot recreate authority.
#[derive(Serialize)]
pub struct ProjectMap<'a> {
    project_map_id: Box<str>,
    core: MapCore<'a>,
    #[serde(skip)]
    nodes: Vec<&'a GraphNode>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
}
impl ProjectMap<'_> {
    #[must_use]
    pub fn project_map_id(&self) -> &str {
        &self.project_map_id
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn canonical_byte_length(&self) -> usize {
        self.canonical_bytes.len()
    }
    #[must_use]
    pub const fn state(&self) -> ProjectMapState {
        self.core.state
    }
}

/// Required receipts are produced by the real graph owner using the prepared
/// queries. Evidence-resolution limits remain in each original native receipt;
/// context does not pretend to bind an unexposed native limits accessor.
/// All selected support is mandatory.
/// No receipt is decoded as authority; missing/truncated custody refuses.
pub fn build_project_map<'a>(
    mut selection: ProjectMapSelection<'a>,
    receipts: &'a [GraphResolvedExplanation<'a>],
    stop: &AtomicBool,
) -> ContextResult<ProjectMap<'a>> {
    checkpoint(stop)?;
    let universe = selection.universe;
    let project = universe.pair.project();
    let owner = universe.pair.graph();
    let limits = universe.limits;
    if receipts.len() != selection.queries.len() {
        return Err(support_mismatch());
    }
    selection.budget.work(receipts.len(), stop)?;
    let mut index = BTreeMap::new();
    for receipt in receipts {
        selection.budget.retain(receipt, stop)?;
        let explanation = receipt.explanation();
        let key = subject_key(explanation.query().subject());
        if index.insert(key, receipt).is_some() {
            return Err(support_mismatch());
        }
    }
    let mut supports = Vec::new();
    let mut sources: BTreeMap<StableHandleId, &'a SourceHandle> = BTreeMap::new();
    for query in &selection.queries {
        selection.budget.work(1, stop)?;
        let receipt = index
            .remove(&subject_key(query.subject()))
            .ok_or_else(support_mismatch)?;
        let explanation = receipt.explanation();
        if explanation.query() != query
            || !explanation.support_complete()
            || !explanation.truncations().is_empty()
            || explanation.supports().is_empty()
            || !receipt.evidence_resolution().closure_complete()
            || !receipt.evidence_resolution().truncations().is_empty()
            || !receipt.evidence_resolution().unresolved().is_empty()
        {
            return Err(custody_missing());
        }
        // Exact borrowed-owner admission is noncanonical. Native explanation
        // fields are private; these pointers prove that this receipt was read
        // from this same acquired graph, rather than an equal-looking DTO.
        match explanation.record() {
            GraphExplainedRecord::Entity { node } => {
                let actual = owner
                    .snapshot()
                    .node(node.node_id())
                    .ok_or_else(support_mismatch)?;
                if !std::ptr::eq(*node, actual) {
                    return Err(support_mismatch());
                }
            }
            GraphExplainedRecord::Relation {
                edge,
                source,
                target,
            } => {
                let actual = owner
                    .snapshot()
                    .edge(edge.edge_id())
                    .ok_or_else(support_mismatch)?;
                let from = owner
                    .snapshot()
                    .node(edge.from())
                    .ok_or_else(support_mismatch)?;
                let to = owner
                    .snapshot()
                    .node(edge.to())
                    .ok_or_else(support_mismatch)?;
                if !std::ptr::eq(*edge, actual)
                    || !std::ptr::eq(*source, from)
                    || !std::ptr::eq(*target, to)
                {
                    return Err(support_mismatch());
                }
            }
        }
        for evidence in receipt.evidence_resolution().evidence_records() {
            selection.budget.work(1, stop)?;
            if evidence.context_id() != universe.binding.source_context_id {
                return Err(support_mismatch());
            }
        }
        for handle in receipt.evidence_resolution().source_handles() {
            selection.budget.work(1, stop)?;
            if let Some(previous) = sources.get(&handle.handle_id()) {
                if *previous != *handle {
                    return Err(support_mismatch());
                }
            } else {
                // Entire receipt already charged before adding this borrowed index.
                sources.insert(handle.handle_id(), *handle);
            }
        }
        supports.push(receipt);
    }
    if !index.is_empty() {
        return Err(support_mismatch());
    }
    let configuration = project.configuration();
    if configuration.package_load_plan().is_some() != configuration.package_main_plan().is_some() {
        return Err(custody_missing());
    }
    let load_custody = LoadCustody {
        selected_toc: configuration.load_plan().map(SingleLoadCustody::from_plan),
        package_load: configuration.package_load_plan(),
        package_main: configuration.package_main_plan(),
    };
    selection.budget.retain(&load_custody, stop)?;
    let lookup_work = source_lookup_work(project, &mut selection.budget, stop)?;
    let mut source_custody = Vec::new();
    for handle in sources.into_values() {
        selection.budget.work(lookup_work, stop)?;
        let custody = if handle.project_generation() == Some(project.project_generation()) {
            if handle.reference_generation() != Some(configuration.reference_generation())
                || handle.origin_id() != configuration.source_origin_id().as_str()
            {
                return Err(support_mismatch());
            }
            let lookup = project.source_artifact(handle.path().as_str());
            checkpoint(stop)?;
            let artifact = lookup
                .map_err(|_| custody_missing())?
                .ok_or_else(custody_missing)?;
            if artifact.content_digest() != *handle.content_digest()
                || (handle.span().kind() == SourceSpanKind::ByteRange
                    && handle
                        .span()
                        .byte_end()
                        .is_none_or(|end| end > artifact.byte_length()))
            {
                return Err(custody_missing());
            }
            let file = project.file_by_path(handle.path().as_str());
            checkpoint(stop)?;
            let file = file.map_err(|_| custody_missing())?;
            SourceCustody::Primary {
                handle,
                artifact_path: handle.path(),
                content_digest: artifact.content_digest(),
                byte_length: artifact.byte_length(),
                file,
            }
        } else if handle.project_generation().is_none()
            && handle.origin_kind() == SourceOriginKind::ReferencePack
            && handle.reference_generation() == Some(configuration.reference_generation())
        {
            // Exact retained source handle only: ReferenceView association and
            // bytes/source membership are deliberately not manufactured.
            SourceCustody::ReferenceContractNotEvaluated { handle }
        } else {
            return Err(custody_missing());
        };
        selection.budget.retain(&custody, stop)?;
        source_custody.push(custody);
    }
    let mut reference_coverage = Vec::new();
    for partition in universe.reference.partitions() {
        selection.budget.work(1, stop)?;
        let row = ReferenceCoverage {
            partition_id: partition.id(),
            state: partition.coverage(),
        };
        selection.budget.retain(&row, stop)?;
        reference_coverage.push(row);
    }
    let project_coverage = project.project_coverage_records();
    let mut deferred_project_capabilities = Vec::new();
    for capability in project.deferred_capabilities() {
        selection.budget.work(1, stop)?;
        let row = DeferredCapability {
            capability_id: capability.capability_id(),
            record: capability.record(),
        };
        selection.budget.retain(&row, stop)?;
        deferred_project_capabilities.push(row);
    }
    selection.budget.work(project_coverage.len(), stop)?;
    selection
        .budget
        .work(deferred_project_capabilities.len(), stop)?;
    selection
        .budget
        .work(universe.reference.conflicts().len(), stop)?;
    selection.budget.retain(project_coverage, stop)?;
    selection
        .budget
        .retain(&deferred_project_capabilities, stop)?;
    selection
        .budget
        .retain(universe.reference.conflicts(), stop)?;
    let state = if selection.neighbors.state() == GraphQueryState::Truncated {
        ProjectMapState::Truncated
    } else {
        ProjectMapState::Partial
    };
    let omissions = vec![
        Omission {
            scope: "optional_platform",
            reason: "profile_excluded",
            affected_capability: "platform_context",
            known_omitted: None,
            enumeration: "not_evaluated",
        },
        Omission {
            scope: "other_relations_and_transitive_graph",
            reason: "profile_excluded",
            affected_capability: "broader_graph_navigation",
            known_omitted: None,
            enumeration: "not_enumerated",
        },
        Omission {
            scope: "reference_enrichment_and_source_membership",
            reason: "profile_excluded",
            affected_capability: "reference_fact_enrichment",
            known_omitted: None,
            enumeration: "not_evaluated",
        },
        Omission {
            scope: "l0_l1_enrichment_excerpts_tokens_continuation",
            reason: "unsupported_capability",
            affected_capability: "broader_e3_b_context",
            known_omitted: None,
            enumeration: "not_evaluated",
        },
        Omission {
            scope: "direct_navigation_prefix",
            reason: "native_budget_pruned",
            affected_capability: "selected_direct_navigation",
            known_omitted: Some(u64::from(selection.neighbors.omitted_edges())),
            enumeration: "complete_match_count_under_native_query",
        },
    ];
    selection.budget.retain(&omissions, stop)?;
    let core = MapCore {
        schema: PROJECT_MAP_SCHEMA,
        profile: &PROFILE,
        universe_set_id: universe.universe_set_id(),
        binding: &universe.binding,
        project_container: project.snapshot().source_registry().source_origin(),
        navigation: selection.neighbors,
        supports,
        source_custody,
        load_custody,
        project_coverage,
        deferred_project_capabilities,
        reference_coverage,
        reference_conflicts: universe.reference.conflicts(),
        omissions,
        state,
        absence_authoritative: false,
        budget: BudgetReport {
            limits,
            context_work_reserved: selection.budget.work,
            retained_rows_bytes_reserved: selection.budget.retained_bytes,
            native_receipt_bytes_upper_bound: selection.native_receipt_bytes_upper_bound,
            tokens: "unavailable",
            byte_accounting: "exact_final_envelope_accessor",
        },
    };
    let project_map_id = identity(
        PROJECT_MAP_SCHEMA,
        "project-map:sha256:",
        &core,
        limits.max_metadata_bytes,
        stop,
    )?;
    let mut map = ProjectMap {
        project_map_id,
        core,
        nodes: selection.nodes,
        canonical_bytes: Box::new([]),
    };
    // ID excludes itself and the final-byte accessor. The whole ID+core envelope
    // is checked, then retained exactly; no circular byte-count/hash field.
    map.canonical_bytes = canonical(&map, limits.max_metadata_bytes, stop)?.into_boxed_slice();
    Ok(map)
}

fn subject_key(subject: &GraphExplainSubject) -> (u8, &str) {
    match subject {
        GraphExplainSubject::Entity(id) => (0, id.as_str()),
        GraphExplainSubject::Relation(id) => (1, id.as_str()),
    }
}

/// Reserve every bounded native source_artifact lookup loop, including the
/// package source_path membership scan nested inside each selected source.
/// This is an upper bound, never reported as actual visited-source census.
fn source_lookup_work(
    project: &wow_project::ProjectView,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<usize> {
    let mut count = project
        .file_manifest()
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .ok_or_else(exhausted)?;
    if let Some(plan) = project.configuration().load_plan() {
        count = count
            .checked_add(plan.sources().len())
            .ok_or_else(exhausted)?;
    }
    if let Some(packages) = project.configuration().package_load_plan() {
        for package in packages.packages() {
            budget.work(1, stop)?;
            let plan = packages
                .package_plan(&package.package)
                .ok_or_else(custody_missing)?;
            let sources = plan.sources().len();
            let work = sources
                .checked_mul(sources)
                .and_then(|n| n.checked_add(sources))
                .and_then(|n| n.checked_add(1))
                .ok_or_else(exhausted)?;
            count = count.checked_add(work).ok_or_else(exhausted)?;
        }
    }
    Ok(count)
}

/// Read view over an artifact that can only be constructed by the owner.
/// No decode, mutable augmentation, source read, or lazy owner call.
pub struct ProjectMapView<'m, 'a> {
    map: &'m ProjectMap<'a>,
}
pub fn open_project_map_view<'m, 'a>(
    map: &'m ProjectMap<'a>,
    stop: &AtomicBool,
) -> ContextResult<ProjectMapView<'m, 'a>> {
    checkpoint(stop)?;
    Ok(ProjectMapView { map })
}
impl<'a> ProjectMapView<'_, 'a> {
    #[must_use]
    pub fn node(&self, id: &GraphNodeId) -> Option<&'a GraphNode> {
        self.map
            .nodes
            .binary_search_by(|node| node.node_id().cmp(id))
            .ok()
            .map(|index| self.map.nodes[index])
    }
    #[must_use]
    pub fn edge(&self, id: &GraphEdgeId) -> Option<&'a GraphEdge> {
        self.map
            .core
            .navigation
            .edges()
            .binary_search_by(|edge| edge.edge_id().cmp(id))
            .ok()
            .map(|index| self.map.core.navigation.edges()[index])
    }
    #[must_use]
    pub fn project_container(&self) -> &'a ProjectSourceOrigin {
        self.map.core.project_container
    }
    #[must_use]
    pub fn project_map_id(&self) -> &str {
        self.map.project_map_id()
    }
}
fn support_mismatch() -> ContextError {
    ContextError::new(ContextErrorCode::SupportMismatch)
}
fn custody_missing() -> ContextError {
    ContextError::new(ContextErrorCode::MissingRequiredCustody)
}
fn root_invalid() -> ContextError {
    ContextError::new(ContextErrorCode::RootInvalid)
}
