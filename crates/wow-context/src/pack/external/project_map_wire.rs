//! Closed ProjectMap/support wire projection for the selected Primary L0
//! GraphEntity route. This is a private validator input model, never a native
//! ProjectMap and never authority/custody. Unsupported custody families fail
//! closed before a caller may report the enclosing map Valid.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{CoverageRecord, EvidenceRecord, GenerationContext, SourceHandle};
use wow_graph::{
    GraphAcceptedEntityProposal, GraphAcceptedRelationProposal, GraphConflictRecord,
    GraphCoverageRecord, GraphDerivationRecord, GraphDirection, GraphEdge,
    GraphEntityKindDefinition, GraphEntityProposal, GraphGenerationId, GraphLimits,
    GraphNeighborReadQuery, GraphNode, GraphNodeId, GraphPathConfidence,
    GraphRelationKindDefinition, GraphRelationProposal, GraphSnapshotId, GraphUniverseId,
};
use wow_project::{ProjectFileId, ProjectId, graph::ProjectGraphFile};
use wow_reference::{CoverageStatus, ReferenceConflict};

use crate::ContextError;
type EndpointProposalKey<'a> = (&'a str, &'a str, &'a str, &'a str, &'a str, &'a str);

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectMapWire {
    pub project_map_id: String,
    pub core: ProjectMapCoreWire,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectMapCoreWire {
    pub schema: String,
    pub profile: ProjectMapProfileWire,
    pub universe_set_id: String,
    pub binding: UniverseBindingWire,
    pub project_container: ProjectSourceOriginWire,
    pub navigation: MapNavigationWire,
    pub supports: Vec<ResolvedSupportWire>,
    pub source_custody: Vec<SourceCustodyWire>,
    pub load_custody: LoadCustodyWire,
    pub project_coverage: Vec<CoverageRecord>,
    pub deferred_project_capabilities: Vec<DeferredCapabilityWire>,
    pub reference_coverage: Vec<ReferenceCoverageWire>,
    pub reference_conflicts: Vec<ReferenceConflict>,
    pub omissions: Vec<ProjectMapOmissionWire>,
    pub state: ProjectMapStateWire,
    pub absence_authoritative: bool,
    pub budget: ProjectMapBudgetWire,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectMapProfileWire {
    pub confidence: String,
    pub direction: String,
    pub edges: Vec<wow_graph::GraphRelationKind>,
    pub nodes: String,
    pub ordering: String,
    pub platform: String,
    pub privacy: String,
    pub profile: String,
    pub reference: String,
    pub source: String,
    pub tokens: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UniverseBindingWire {
    pub schema: String,
    pub project_id: ProjectId,
    pub project_kind: ProjectKindWire,
    pub project_snapshot_id: String,
    pub project_generation: wow_core::ProjectGenerationId,
    pub configuration_digest: wow_core::ContentDigest<wow_core::CanonicalResult>,
    pub graph_universe: GraphUniverseId,
    pub graph_generation: GraphGenerationId,
    pub graph_snapshot_id: GraphSnapshotId,
    pub source_context_id: wow_core::GenerationContextId,
    pub source_registry_id: String,
    pub analyzer_snapshot_id: String,
    pub registry_digest: String,
    pub target_profile: wow_core::ProfileIdentity,
    pub reference_generation: wow_core::ReferenceGenerationId,
    pub reference_schema: String,
    pub reference_view_digest: String,
    pub compatibility_basis: String,
    pub platform_absence: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectKindWire {
    Fixture,
    Repository,
    BlizzardUiPlatformSource,
}

/// `ProjectSourceOrigin` is Serialize-only. This is its exact emitted record;
/// it is descriptive provenance and cannot recreate a ProjectView/read lease.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectSourceOriginWire {
    pub origin_id: wow_project::ProjectSourceOriginId,
    pub project_id: ProjectId,
    pub workspace_id: wow_project::ProjectWorkspaceId,
    pub origin_kind: ProjectSourceOriginKindWire,
    pub logical_root: wow_core::NormalizedSourcePath,
    pub revision_identity: String,
    pub project_generation: wow_core::ProjectGenerationId,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectSourceOriginKindWire {
    FixtureProject,
    RepositoryProject,
    BlizzardUiPlatformSource,
}

/// `MapNavigation` is `serde(untagged)`: neither variant has a `kind` key.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum MapNavigationWire {
    Direct(Box<NeighborViewWire>),
    Project(ProjectNavigationWire),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectNavigationWire {
    pub schema: String,
    pub project_id: ProjectId,
    pub source_file_roots: Vec<ProjectFileRootWire>,
    pub neighborhoods: Vec<NeighborViewWire>,
    pub root_enumeration: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectFileRootWire {
    pub file: ProjectGraphFile,
    pub node: GraphNode,
}

/// `GraphNeighborView` is Serialize-only; the query and all payload leaves use
/// the real graph/core owners' strict Deserialize contracts.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NeighborViewWire {
    pub schema: String,
    pub query: GraphNeighborReadQuery,
    pub query_digest: String,
    pub universe: GraphUniverseId,
    pub generation: GraphGenerationId,
    pub root: GraphNode,
    pub state: wow_graph::GraphQueryState,
    pub edges: Vec<GraphEdge>,
    pub adjacent_nodes: Vec<GraphNode>,
    pub coverage: Vec<GraphCoverageRecord>,
    pub missing_coverage: Vec<wow_graph::GraphRelationKind>,
    pub scanned_edges: u32,
    pub matching_edges: u32,
    pub omitted_edges: u32,
    pub truncations: Vec<NeighborTruncationWire>,
    pub no_new_evidence: bool,
    pub absence_authoritative: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NeighborTruncationWire {
    Edges,
    OutputBytes,
}

/// Recursive support receipt: every Serialize-only graph explanation/resolution
/// layer is mirrored, while assertion/evidence/source leaves reuse strict owner
/// records. No flattened/anonymous support data is admitted.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolvedSupportWire {
    pub schema: String,
    pub query_digest: String,
    pub explanation: ExplanationWire,
    pub evidence_resolution: EvidenceResolutionWire,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExplanationWire {
    pub schema: String,
    pub query: ExplainQueryWire,
    pub query_digest: String,
    pub universe: GraphUniverseId,
    pub generation: GraphGenerationId,
    pub foundation_snapshot_id: GraphSnapshotId,
    pub source_context_id: wow_core::GenerationContextId,
    pub registry: ExplanationRegistryWire,
    pub record: ExplainedRecordWire,
    pub supports: Vec<AssertionSupportWire>,
    pub assertion_supports: Vec<AssertionSupportWire>,
    pub derivations: Vec<DerivationObservationWire>,
    pub conflicts: Vec<ConflictObservationWire>,
    pub derivation_complete: bool,
    pub coverage: Vec<ExplanationCoverageWire>,
    pub scanned_assertions: u32,
    pub total_supports: u32,
    pub support_complete: bool,
    pub truncations: Vec<ExplanationTruncationWire>,
    pub boundaries: Vec<ExplanationBoundaryWire>,
    pub absence_authoritative: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExplainQueryWire {
    pub snapshot_id: GraphSnapshotId,
    pub subject: wow_graph::GraphExplainSubject,
    pub limits: ExplainLimitsWire,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExplanationRegistryWire {
    pub bundle_id: String,
    pub version: String,
    pub digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ExplainedRecordWire {
    Entity {
        node: GraphNode,
    },
    Relation {
        edge: GraphEdge,
        source: GraphNode,
        target: GraphNode,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "origin", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum AssertionSupportWire {
    FoundationEntity {
        node: GraphNode,
    },
    FoundationRelation {
        edge: GraphEdge,
    },
    ProducerEntity {
        producer: ProducerSupportOriginWire,
        proposal: GraphEntityProposal,
        accepted: GraphAcceptedEntityProposal,
        definition: GraphEntityKindDefinition,
    },
    ProducerRelation {
        producer: ProducerSupportOriginWire,
        proposal: GraphRelationProposal,
        accepted: GraphAcceptedRelationProposal,
        definition: GraphRelationKindDefinition,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProducerSupportOriginWire {
    pub partition_id: String,
    pub partition_digest: String,
    pub producer_version: String,
    pub batch_id: String,
    pub report_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DerivationObservationWire {
    pub record_id: String,
    pub producer: ProducerSupportOriginWire,
    pub record: GraphDerivationRecord,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConflictObservationWire {
    pub record_id: String,
    pub producer: ProducerSupportOriginWire,
    pub record: GraphConflictRecord,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExplanationCoverageWire {
    pub origin: CoverageOriginWire,
    pub observation: CoverageObservationWire,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "layer", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum CoverageOriginWire {
    Projection {
        snapshot_id: GraphSnapshotId,
    },
    Foundation {
        snapshot_id: GraphSnapshotId,
    },
    Producer {
        partition_id: String,
        partition_digest: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "state",
    content = "record",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(crate) enum CoverageObservationWire {
    Retained(GraphCoverageRecord),
    Missing,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExplanationTruncationWire {
    Supports,
    OutputBytes,
    DerivationDepth,
    AssertionWork,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExplanationBoundaryWire {
    EvidenceRecordsNotResolved,
    EvidenceResolutionIncomplete,
    DerivationRecordsNotRetained,
    ConflictAssessmentNotAvailable,
    FoundationProducerNotRetained,
    EntityCoverageNotModeled,
    DerivationSupportMissing,
    RetainedConflictsUnresolved,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceResolutionWire {
    pub catalog_digest: String,
    pub context: GenerationContext,
    pub limits: wow_graph::GraphEvidenceResolveLimits,
    pub evidence_records: Vec<EvidenceRecord>,
    pub source_handles: Vec<SourceHandle>,
    pub unresolved: Vec<UnresolvedEvidenceWire>,
    pub requested_evidence_roots: usize,
    pub requested_source_roots: usize,
    pub known_evidence: usize,
    pub known_sources: usize,
    pub known_pending_evidence: usize,
    pub known_pending_sources: usize,
    pub work: u32,
    pub closure_complete: bool,
    pub supports_complete: bool,
    pub truncations: Vec<EvidenceTruncationWire>,
    pub source_bytes_verified: bool,
    pub coverage_references_resolved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum UnresolvedEvidenceWire {
    Evidence {
        id: String,
        reason: UnresolvedReasonWire,
    },
    SourceHandle {
        id: wow_core::StableHandleId,
        reason: UnresolvedReasonWire,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum UnresolvedReasonWire {
    NotRetained,
    UnsupportedIdentifier,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EvidenceTruncationWire {
    EvidenceRecords,
    SourceHandles,
    DerivationDepth,
    Work,
    OutputBytes,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SourceCustodyWire {
    Primary {
        handle: SourceHandle,
        artifact_path: wow_core::NormalizedSourcePath,
        content_digest: wow_core::ContentDigest<wow_core::SourceContent>,
        byte_length: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<Box<ProjectFileRecordWire>>,
    },
    ReferenceContractNotEvaluated {
        handle: SourceHandle,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectFileRecordWire {
    pub manifest: ProjectFileManifestWire,
    pub source_origin_id: wow_project::ProjectSourceOriginId,
    pub workspace_id: wow_project::ProjectWorkspaceId,
    pub project_generation: wow_core::ProjectGenerationId,
    pub analyzer_file_id: String,
    pub source_handle_base: SourceHandle,
}

/// ProjectFileManifestEntry is Serialize-only; mirror its exact record.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectFileManifestWire {
    pub file_id: ProjectFileId,
    pub relative_path: wow_core::NormalizedSourcePath,
    pub language_kind: ProjectLanguageKindWire,
    pub role: ProjectFileRoleWire,
    pub content_digest: wow_core::ContentDigest<wow_core::SourceContent>,
    pub byte_length: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectLanguageKindWire {
    Lua,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectFileRoleWire {
    FirstPartyMain,
    Library,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeferredCapabilityWire {
    pub capability_id: wow_core::CapabilityId,
    pub record: wow_core::NotEvaluatedRecord,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReferenceCoverageWire {
    pub partition_id: String,
    pub state: CoverageStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectMapOmissionWire {
    pub scope: String,
    pub reason: String,
    pub affected_capability: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_omitted: Option<u64>,
    pub enumeration: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectMapStateWire {
    Partial,
    Truncated,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectMapBudgetWire {
    pub limits: PrimaryMapLimitsWire,
    pub context_work_reserved: u64,
    pub retained_rows_bytes_reserved: u64,
    pub native_receipt_bytes_upper_bound: u64,
    pub tokens: String,
    pub byte_accounting: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrimaryMapLimitsWire {
    pub neighbors: wow_graph::GraphNeighborReadLimits,
    pub explain: ExplainLimitsWire,
    pub max_direct_edges: u32,
    pub max_input_bytes: u64,
    pub max_metadata_bytes: u64,
    pub max_context_work: u64,
}

/// The native type defaults `max_derivation_depth` for legacy request
/// compatibility. The persisted ProjectMap writer always emits it, so this
/// wire mirror requires it and closes that omission loophole.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExplainLimitsWire {
    pub max_scanned_assertions: u32,
    pub max_supports: u32,
    pub max_output_bytes: u32,
    pub max_derivation_depth: u32,
}

/// The writer omits each absent family. Present custody payloads are retained
/// solely as canonical digest material until their separate closed wire owners
/// are integrated. The generic JSON value is never evidence of validity:
/// presence always yields `NotEvaluated` after the supported structural checks.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoadCustodyWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_toc: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_load: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_main: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectMapWireOutcome {
    SupportedPrimaryL0,
    NotEvaluated {
        project_navigation: bool,
        selected_toc: bool,
        package_load: bool,
        package_main: bool,
        explanation_extensions: bool,
    },
}

/// A structural rejection is distinct from a caller budget/cancellation error.
/// The driver supplies its existing `Budget::work(amount, stop)` hook, preserving
/// the native context error instead of hiding it behind a parser-local limit.
#[derive(Debug)]
pub(crate) enum ProjectMapWireCheckError<E> {
    Invalid(&'static str),
    Work(E),
}

/// Structural checks only. The caller supplies both its continuing work hook
/// and its stop-aware existing ProjectMap identity helper; no string-only ID or
/// decoded receipt creates native custody. The caller must additionally resolve
/// every reference against the one admitted ProjectMap wire graph and preserve
/// `NotEvaluated` state.
pub(crate) fn check_project_map_links<E: From<ContextError>>(
    map: &ProjectMapWire,
    requested_l0_root: &GraphNodeId,
    mut reserve_work: impl FnMut(usize) -> Result<(), E>,
    stop: &std::sync::atomic::AtomicBool,
) -> Result<ProjectMapWireOutcome, ProjectMapWireCheckError<E>> {
    reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
    super::owner_records::check(map, &mut reserve_work)?;
    if !matches!(
        map.core.schema.as_str(),
        "wow-context/project-map/primary-direct-navigation/1"
            | "wow-context/project-map/primary-project-navigation/1"
    ) || map.core.project_container.project_id != map.core.binding.project_id
        || map.core.project_container.project_generation != map.core.binding.project_generation
        || map.core.universe_set_id.is_empty()
        || map.core.absence_authoritative
        || map.core.profile.tokens != "unavailable"
    {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap schema, identity, authority, or root cardinality mismatch",
        ));
    }
    if !matches!(
        (
            &map.core.binding.project_kind,
            &map.core.project_container.origin_kind
        ),
        (
            ProjectKindWire::Fixture,
            ProjectSourceOriginKindWire::FixtureProject
        ) | (
            ProjectKindWire::Repository,
            ProjectSourceOriginKindWire::RepositoryProject
        )
    ) {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap project kind and project source-origin kind disagree",
        ));
    }
    let project_navigation =
        map.core.schema == "wow-context/project-map/primary-project-navigation/1";
    let expected_profile = if project_navigation {
        crate::PRIMARY_PROJECT_MAP_PROFILE
    } else {
        crate::PRIMARY_MAP_PROFILE
    };
    if map.core.profile.profile != expected_profile {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap profile identifier mismatch",
        ));
    }
    let profile = &map.core.profile;
    let expected_nodes = if project_navigation {
        "project_container_and_all_source_file_direct_endpoints"
    } else {
        "exact_root_and_direct_endpoints"
    };
    let expected_edges = [
        wow_graph::GraphRelationKind::Contains,
        wow_graph::GraphRelationKind::Defines,
        wow_graph::GraphRelationKind::DependsOn,
        wow_graph::GraphRelationKind::OptionalDependsOn,
        wow_graph::GraphRelationKind::Owns,
        wow_graph::GraphRelationKind::Loads,
        wow_graph::GraphRelationKind::LoadsBefore,
    ];
    if profile.confidence != "include_candidate"
        || profile.direction != "both"
        || profile.edges.as_slice() != expected_edges.as_slice()
        || profile.nodes != expected_nodes
        || profile.ordering != "node_id_then_edge_id"
        || profile.platform != "absent_profile_excluded"
        || profile.privacy != "local_metadata_only"
        || profile.reference != "binding_and_coverage_only"
        || profile.source != "handles_and_load_custody_only"
    {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap profile descriptor differs from the selected native profile",
        ));
    }
    // The caller verifies the exact MapCore identity in the bounded digest
    // phase before entering these structural joins.
    let mut roots = Vec::<&GraphNode>::new();
    let mut neighborhoods = Vec::<&NeighborViewWire>::new();
    match &map.core.navigation {
        MapNavigationWire::Direct(view) => {
            if map.core.schema != "wow-context/project-map/primary-direct-navigation/1" {
                return Err(ProjectMapWireCheckError::Invalid(
                    "direct navigation/schema mismatch",
                ));
            }
            if view.root.node_id() != requested_l0_root
                || !matches!(view.root.kind(), "source_file" | "source_package")
            {
                return Err(ProjectMapWireCheckError::Invalid(
                    "default Primary L0 requires its exact source_file/source_package map root",
                ));
            }
            roots.push(&view.root);
            neighborhoods.push(view);
        }
        MapNavigationWire::Project(project) => {
            if map.core.schema != "wow-context/project-map/primary-project-navigation/1"
                || project.schema != "wow-context/project-navigation/1"
                || project.root_enumeration != "complete_for_retained_primary_source_files"
                || project.project_id != map.core.binding.project_id
                || project.source_file_roots.len() != project.neighborhoods.len()
            {
                return Err(ProjectMapWireCheckError::Invalid(
                    "project navigation/schema/root cardinality mismatch",
                ));
            }
            let mut previous: Option<&GraphNodeId> = None;
            for (root, view) in project.source_file_roots.iter().zip(&project.neighborhoods) {
                reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                if root.node.node_id() != view.root.node_id()
                    || root.node.kind() != "source_file"
                    || previous.is_some_and(|id| id >= root.node.node_id())
                {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "project navigation roots are not exact and ordered",
                    ));
                }
                previous = Some(root.node.node_id());
                roots.push(&root.node);
                neighborhoods.push(view);
            }
        }
    }
    let mut node_records = std::collections::BTreeMap::new();
    let mut edge_records = std::collections::BTreeMap::new();
    for (root, view) in roots.iter().zip(&neighborhoods) {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        let query = view.query.query();
        let mut sorted_expected_edges = expected_edges;
        sorted_expected_edges.sort();
        let query_digest = graph_query_digest(
            wow_graph::GRAPH_NEIGHBOR_READ_SCHEMA,
            &view.query,
            "graph-direct-query:sha256:",
            &mut reserve_work,
            stop,
            map.core.budget.limits.max_metadata_bytes,
        )?;
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        let mut observed_coverage = Vec::new();
        let mut coverage_partial = false;
        let mut coverage_not_evaluated = !view.missing_coverage.is_empty();
        for record in &view.coverage {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            super::owner_records::check_graph_coverage(record, &mut reserve_work)?;
            observed_coverage.push(record.relation());
            coverage_partial |= matches!(
                record.state(),
                wow_graph::GraphCoverageState::Partial | wow_graph::GraphCoverageState::Failed
            );
            coverage_not_evaluated |= record.state() == wow_graph::GraphCoverageState::NotEvaluated;
        }
        let mut combined_coverage = observed_coverage.clone();
        combined_coverage.extend(view.missing_coverage.iter().copied());
        combined_coverage.sort();
        let coverage_is_exact = observed_coverage.windows(2).all(|pair| pair[0] < pair[1])
            && view
                .missing_coverage
                .windows(2)
                .all(|pair| pair[0] < pair[1])
            && combined_coverage == sorted_expected_edges;
        let expected_neighbor_state = if !view.truncations.is_empty() {
            wow_graph::GraphQueryState::Truncated
        } else if coverage_partial {
            wow_graph::GraphQueryState::Partial
        } else if coverage_not_evaluated {
            wow_graph::GraphQueryState::NotEvaluated
        } else {
            wow_graph::GraphQueryState::Complete
        };
        if view.schema != wow_graph::GRAPH_NEIGHBOR_READ_SCHEMA
            || view.query.validate().is_err()
            || !coverage_is_exact
            || root.node_id() != view.query.query().node_id()
            || view.query.snapshot_id() != &map.core.binding.graph_snapshot_id
            || query.direction() != GraphDirection::Both
            || query.relations() != sorted_expected_edges.as_slice()
            || query.max_edges() != map.core.budget.limits.max_direct_edges
            || view.query.confidence() != GraphPathConfidence::IncludeCandidate
            || view.query.limits() != map.core.budget.limits.neighbors
            || view.query_digest != query_digest
            || view.state != expected_neighbor_state
            || view.universe != map.core.binding.graph_universe
            || view.generation != map.core.binding.graph_generation
            || root.universe() != &map.core.binding.graph_universe
            || root.generation() != &map.core.binding.graph_generation
            || view.matching_edges < u32::try_from(view.edges.len()).unwrap_or(u32::MAX)
            || view.scanned_edges < view.matching_edges
            || view.scanned_edges > view.query.limits().max_scanned_edges
            || view.truncations.len() > 1
            || view.omitted_edges
                != view
                    .matching_edges
                    .saturating_sub(u32::try_from(view.edges.len()).unwrap_or(u32::MAX))
            || view.truncations.is_empty() != (view.omitted_edges == 0)
            || view.no_new_evidence != view.edges.is_empty()
            || view.absence_authoritative
        {
            return Err(ProjectMapWireCheckError::Invalid(
                "ProjectMap root/query/snapshot identities disagree",
            ));
        }
        insert_node_record(&mut node_records, root).map_err(ProjectMapWireCheckError::Invalid)?;
        let mut adjacent_ids = std::collections::BTreeSet::new();
        let mut previous_adjacent: Option<&GraphNodeId> = None;
        for node in &view.adjacent_nodes {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            if node.universe() != &map.core.binding.graph_universe
                || node.generation() != &map.core.binding.graph_generation
                || node.node_id() == root.node_id()
                || previous_adjacent.is_some_and(|id| id >= node.node_id())
            {
                return Err(ProjectMapWireCheckError::Invalid(
                    "neighbor nodes are mixed-generation or not canonically ordered",
                ));
            }
            previous_adjacent = Some(node.node_id());
            adjacent_ids.insert(node.node_id().clone());
            insert_node_record(&mut node_records, node)
                .map_err(ProjectMapWireCheckError::Invalid)?;
        }
        let mut expected_adjacent_ids = std::collections::BTreeSet::new();
        let mut previous_edge: Option<&wow_graph::GraphEdgeId> = None;
        for edge in &view.edges {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            edge_ids_for_view_insert(&mut edge_records, edge)
                .map_err(ProjectMapWireCheckError::Invalid)?;
            if previous_edge.is_some_and(|id| id >= edge.edge_id())
                || query.relations().binary_search(&edge.relation()).is_err()
                || edge.from() == edge.to()
                || edge.from() != root.node_id() && edge.to() != root.node_id()
                || (edge.from() != root.node_id() && !adjacent_ids.contains(edge.from()))
                || (edge.to() != root.node_id() && !adjacent_ids.contains(edge.to()))
            {
                return Err(ProjectMapWireCheckError::Invalid(
                    "neighbor edge is outside its exact query root or endpoint support",
                ));
            }
            previous_edge = Some(edge.edge_id());
            let adjacent_id = if edge.from() == root.node_id() {
                edge.to()
            } else {
                edge.from()
            };
            let mut adjacent_found = false;
            for node in &view.adjacent_nodes {
                reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                adjacent_found |= node.node_id() == adjacent_id;
            }
            if !adjacent_found {
                return Err(ProjectMapWireCheckError::Invalid(
                    "neighbor edge endpoint is absent from adjacent node set",
                ));
            }
            expected_adjacent_ids.insert(adjacent_id.clone());
        }
        // The owner only inserts adjacent nodes while retaining a matching
        // incident edge. Check both directions so an orphan node cannot enter
        // the selected entity set through serialized metadata.
        if adjacent_ids != expected_adjacent_ids {
            return Err(ProjectMapWireCheckError::Invalid(
                "neighbor node set differs from selected incident-edge endpoints",
            ));
        }
    }
    if map.core.state
        != if neighborhoods
            .iter()
            .any(|view| view.state == wow_graph::GraphQueryState::Truncated)
        {
            ProjectMapStateWire::Truncated
        } else {
            ProjectMapStateWire::Partial
        }
    {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap state differs from native query coverage/truncation",
        ));
    }
    let node_ids: std::collections::BTreeSet<_> = node_records.keys().cloned().collect();
    let edge_ids: std::collections::BTreeSet<_> = edge_records.keys().cloned().collect();
    let mut omitted_prefix = 0_u64;
    for view in &neighborhoods {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        omitted_prefix = omitted_prefix
            .checked_add(u64::from(view.omitted_edges))
            .ok_or(ProjectMapWireCheckError::Invalid(
                "ProjectMap omitted-edge total overflow",
            ))?;
    }
    let expected_enumeration = if project_navigation {
        "sum_of_complete_match_counts_under_native_queries"
    } else {
        "complete_match_count_under_native_query"
    };
    if map.core.omissions.len() != 5
        || map.core.omissions[0].scope != "optional_platform"
        || map.core.omissions[0].reason != "profile_excluded"
        || map.core.omissions[0].affected_capability != "platform_context"
        || map.core.omissions[0].known_omitted.is_some()
        || map.core.omissions[0].enumeration != "not_evaluated"
        || map.core.omissions[1].scope != "other_relations_and_transitive_graph"
        || map.core.omissions[1].reason != "profile_excluded"
        || map.core.omissions[1].affected_capability != "broader_graph_navigation"
        || map.core.omissions[1].known_omitted.is_some()
        || map.core.omissions[1].enumeration != "not_enumerated"
        || map.core.omissions[2].scope != "reference_enrichment_and_source_membership"
        || map.core.omissions[2].reason != "profile_excluded"
        || map.core.omissions[2].affected_capability != "reference_fact_enrichment"
        || map.core.omissions[2].known_omitted.is_some()
        || map.core.omissions[2].enumeration != "not_evaluated"
        || map.core.omissions[3].scope != "l0_l1_enrichment_excerpts_tokens_continuation"
        || map.core.omissions[3].reason != "unsupported_capability"
        || map.core.omissions[3].affected_capability != "broader_e3_b_context"
        || map.core.omissions[3].known_omitted.is_some()
        || map.core.omissions[3].enumeration != "not_evaluated"
        || map.core.omissions[4].scope != "direct_navigation_prefix"
        || map.core.omissions[4].reason != "native_budget_pruned"
        || map.core.omissions[4].affected_capability != "selected_direct_navigation"
        || map.core.omissions[4].known_omitted != Some(omitted_prefix)
        || map.core.omissions[4].enumeration != expected_enumeration
    {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap omission records differ from the native writer profile",
        ));
    }
    let selected_record_count =
        node_ids
            .len()
            .checked_add(edge_ids.len())
            .ok_or(ProjectMapWireCheckError::Invalid(
                "selected graph record count overflow",
            ))?;
    if map.core.supports.len() != selected_record_count {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap support receipt cardinality differs from selected graph records",
        ));
    }
    let mut supported_nodes = std::collections::BTreeSet::new();
    let mut supported_edges = std::collections::BTreeSet::new();
    let mut support_handles = std::collections::BTreeMap::new();
    let mut accepted_entity_proposals = std::collections::BTreeMap::new();
    // Native input_view unions evidence for assertions with one foundation
    // identity, while explain::select rebinds that identity to the published
    // graph generation. Build that exact structural join before checking edges.
    let mut projected_node_ids = std::collections::BTreeMap::new();
    for receipt in &map.core.supports {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        let ExplainedRecordWire::Entity { node } = &receipt.explanation.record else {
            continue;
        };
        let mut joined_evidence = std::collections::BTreeSet::new();
        for support in &receipt.explanation.supports {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            let original = match support {
                AssertionSupportWire::FoundationEntity { node } => node,
                AssertionSupportWire::ProducerEntity {
                    producer,
                    proposal,
                    accepted,
                    ..
                } => {
                    let address = (
                        producer.partition_id.as_str(),
                        producer.partition_digest.as_str(),
                        producer.producer_version.as_str(),
                        producer.batch_id.as_str(),
                        producer.report_id.as_str(),
                        proposal.proposal_id(),
                    );
                    if let Some(previous) = accepted_entity_proposals.get(&address) {
                        if previous != accepted.node().node_id() {
                            return Err(ProjectMapWireCheckError::Invalid(
                                "producer entity proposal ID maps to conflicting accepted nodes",
                            ));
                        }
                    } else {
                        accepted_entity_proposals
                            .insert(address, accepted.node().node_id().clone());
                    }
                    accepted.node()
                }
                _ => {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "entity explanation contains a non-entity assertion support",
                    ));
                }
            };
            reserve_work(original.evidence_ids().len().saturating_add(1))
                .map_err(ProjectMapWireCheckError::Work)?;
            let projected_id =
                project_node_id(original, map).map_err(ProjectMapWireCheckError::Invalid)?;
            if &projected_id != node.node_id() {
                return Err(ProjectMapWireCheckError::Invalid(
                    "entity assertion does not project to its explained graph node",
                ));
            }
            if let Some(previous) =
                projected_node_ids.insert(original.node_id().clone(), projected_id.clone())
                && previous != projected_id
            {
                return Err(ProjectMapWireCheckError::Invalid(
                    "one foundation node identity maps to conflicting projected nodes",
                ));
            }
            for evidence_id in original.evidence_ids() {
                reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                joined_evidence.insert(evidence_id.clone());
            }
        }
        let joined = GraphNode::new(
            map.core.binding.graph_universe.clone(),
            map.core.binding.graph_generation.clone(),
            node.kind(),
            node.owner_key(),
            joined_evidence.into_iter().collect(),
            PROJECT_MAP_PROJECTION_LIMITS,
        )
        .map_err(|_| {
            ProjectMapWireCheckError::Invalid(
                "entity support evidence cannot form its projected graph node",
            )
        })?;
        if joined != *node {
            return Err(ProjectMapWireCheckError::Invalid(
                "explained entity evidence differs from the native assertion union",
            ));
        }
    }
    for receipt in &map.core.supports {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        let explanation_digest = graph_query_digest(
            wow_graph::GRAPH_EXPLANATION_SCHEMA,
            &receipt.explanation.query,
            "graph-explanation-query:sha256:",
            &mut reserve_work,
            stop,
            map.core.budget.limits.max_metadata_bytes,
        )?;
        let resolved_material = (
            &receipt.explanation.query,
            &receipt.evidence_resolution.limits,
            receipt.evidence_resolution.catalog_digest.as_str(),
        );
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        let resolved_digest = graph_query_digest(
            "graph-resolved-explanation-query/2",
            &resolved_material,
            "graph-resolved-explanation-query/2:sha256:",
            &mut reserve_work,
            stop,
            map.core.budget.limits.max_metadata_bytes,
        )?;
        if receipt.schema != "wow-graph/resolved-explanation/e2-a/2"
            || receipt.explanation.schema != wow_graph::GRAPH_EXPLANATION_SCHEMA
            || receipt.query_digest != resolved_digest
            || receipt.explanation.query_digest != explanation_digest
            || receipt.explanation.query.snapshot_id != map.core.binding.graph_snapshot_id
            || receipt.explanation.query.limits != map.core.budget.limits.explain
            || !receipt.explanation.support_complete
            || !receipt.explanation.truncations.is_empty()
            || receipt.explanation.supports.is_empty()
            || !receipt.evidence_resolution.closure_complete
            || !receipt.evidence_resolution.truncations.is_empty()
            || !receipt.evidence_resolution.unresolved.is_empty()
        {
            return Err(ProjectMapWireCheckError::Invalid(
                "native support receipt is incomplete or has an unsupported schema",
            ));
        }
        if receipt.explanation.query.snapshot_id != map.core.binding.graph_snapshot_id
            || receipt.explanation.universe != map.core.binding.graph_universe
            || receipt.explanation.generation != map.core.binding.graph_generation
            || receipt.explanation.source_context_id != map.core.binding.source_context_id
            || receipt.evidence_resolution.context.context_id()
                != map.core.binding.source_context_id
        {
            return Err(ProjectMapWireCheckError::Invalid(
                "support receipt is bound to another graph or source context",
            ));
        }
        let mut receipt_evidence_ids = std::collections::BTreeSet::new();
        for evidence in &receipt.evidence_resolution.evidence_records {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            if evidence.context_id() != map.core.binding.source_context_id {
                return Err(ProjectMapWireCheckError::Invalid(
                    "support evidence belongs to another source context",
                ));
            }
            if !receipt_evidence_ids.insert(evidence.evidence_id().to_string()) {
                return Err(ProjectMapWireCheckError::Invalid(
                    "support receipt repeats an evidence record ID",
                ));
            }
        }
        let mut receipt_handle_ids = std::collections::BTreeSet::new();
        for handle in &receipt.evidence_resolution.source_handles {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            let id = handle.handle_id();
            if let Some(previous) = support_handles.get(&id) {
                // The native builder deduplicates shared handles; equal repeated
                // IDs are acceptable only when their complete typed records match.
                if *previous != handle {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "support source-handle ID has conflicting records",
                    ));
                }
            } else {
                support_handles.insert(id, handle);
            }
            receipt_handle_ids.insert(id);
        }
        for evidence in &receipt.evidence_resolution.evidence_records {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            for handle_id in evidence.source_handle_ids() {
                reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                if !receipt_handle_ids.contains(handle_id) {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "resolved evidence references a source handle absent from its receipt",
                    ));
                }
            }
        }
        let explained_evidence_ids = match &receipt.explanation.record {
            ExplainedRecordWire::Entity { node } => node.evidence_ids(),
            ExplainedRecordWire::Relation { edge, .. } => edge.evidence_ids(),
        };
        for evidence_id in explained_evidence_ids {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            if !receipt_evidence_ids.contains(evidence_id.as_ref()) {
                return Err(ProjectMapWireCheckError::Invalid(
                    "explained graph record evidence is absent from its resolved receipt",
                ));
            }
        }
        match &receipt.explanation.record {
            ExplainedRecordWire::Entity { node } => {
                if !node_ids.contains(node.node_id())
                    || node_records
                        .get(node.node_id())
                        .is_none_or(|selected| selected != node)
                    || !supported_nodes.insert(node.node_id().clone())
                    || !matches!(&receipt.explanation.query.subject, wow_graph::GraphExplainSubject::Entity(id) if id == node.node_id())
                {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "entity support is missing, duplicate, or outside selected graph",
                    ));
                }
            }
            ExplainedRecordWire::Relation {
                edge,
                source,
                target,
            } => {
                if !edge_ids.contains(edge.edge_id())
                    || edge_records
                        .get(edge.edge_id())
                        .is_none_or(|selected| selected != edge)
                    || node_records
                        .get(source.node_id())
                        .is_none_or(|selected| selected != source)
                    || node_records
                        .get(target.node_id())
                        .is_none_or(|selected| selected != target)
                    || !supported_edges.insert(edge.edge_id().clone())
                    || source.node_id() != edge.from()
                    || target.node_id() != edge.to()
                    || !matches!(&receipt.explanation.query.subject, wow_graph::GraphExplainSubject::Relation(id) if id == edge.edge_id())
                {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "relation support endpoint/identity closure mismatch",
                    ));
                }
            }
        }
        for support in &receipt.explanation.supports {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            let evidence_work = match support {
                AssertionSupportWire::ProducerEntity { accepted, .. } => {
                    accepted.node().evidence_ids().len()
                }
                AssertionSupportWire::ProducerRelation { accepted, .. } => {
                    accepted.edge().evidence_ids().len()
                }
                AssertionSupportWire::FoundationEntity { node } => node.evidence_ids().len(),
                AssertionSupportWire::FoundationRelation { edge } => edge.evidence_ids().len(),
            };
            reserve_work(evidence_work).map_err(ProjectMapWireCheckError::Work)?;
            let producer_links_resolved = match support {
                AssertionSupportWire::ProducerEntity { proposal, .. } => {
                    let mut resolved = true;
                    for id in proposal.evidence_ids() {
                        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                        resolved &= receipt_evidence_ids.contains(&id.to_string());
                    }
                    for id in proposal.source_handle_ids() {
                        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                        resolved &= receipt_handle_ids.contains(id);
                    }
                    resolved
                }
                AssertionSupportWire::ProducerRelation { proposal, .. } => {
                    let mut resolved = true;
                    for id in proposal.evidence_ids() {
                        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                        resolved &= receipt_evidence_ids.contains(&id.to_string());
                    }
                    for id in proposal.source_handle_ids() {
                        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                        resolved &= receipt_handle_ids.contains(id);
                    }
                    resolved
                }
                _ => true,
            };
            if !producer_links_resolved {
                return Err(ProjectMapWireCheckError::Invalid(
                    "producer proposal evidence/source links are outside its resolved receipt",
                ));
            }
            if matches!(
                support,
                AssertionSupportWire::ProducerEntity { .. }
                    | AssertionSupportWire::ProducerRelation { .. }
            ) && !super::support_record_links::validate_producer_record(
                support,
                map,
                &mut reserve_work,
                stop,
            )? {
                return Err(ProjectMapWireCheckError::Invalid(
                    "producer proposal, definition and accepted record disagree",
                ));
            }
            match (&receipt.explanation.record, support) {
                (
                    ExplainedRecordWire::Entity { node },
                    AssertionSupportWire::ProducerEntity {
                        proposal,
                        accepted,
                        definition,
                        ..
                    },
                ) if proposal.proposal_id() == accepted.proposal_id()
                    && proposal.entity_kind_id() == definition.kind_id()
                    && project_node_id(accepted.node(), map)
                        .is_ok_and(|projected| projected == *node.node_id()) => {}
                (
                    ExplainedRecordWire::Relation {
                        edge,
                        source,
                        target,
                    },
                    AssertionSupportWire::ProducerRelation {
                        proposal,
                        accepted,
                        definition,
                        ..
                    },
                ) if proposal.proposal_id() == accepted.proposal_id()
                    && definition.relation() == edge.relation()
                    && super::support_record_links::relation_endpoint_kinds_match(
                        definition, source, target,
                    )
                    && project_edge(accepted.edge(), &projected_node_ids)
                        .is_ok_and(|projected| projected == *edge) => {}
                (
                    ExplainedRecordWire::Entity { node },
                    AssertionSupportWire::FoundationEntity { node: original },
                ) if project_node_id(original, map)
                    .is_ok_and(|projected| projected == *node.node_id()) => {}
                (
                    ExplainedRecordWire::Relation { edge, .. },
                    AssertionSupportWire::FoundationRelation { edge: original },
                ) if project_edge(original, &projected_node_ids)
                    .is_ok_and(|projected| projected == *edge) => {}
                _ => {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "producer proposal/accepted record does not join its explained graph fact",
                    ));
                }
            }
        }
    }
    if supported_nodes != node_ids || supported_edges != edge_ids {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap omitted required native graph support",
        ));
    }
    for receipt in &map.core.supports {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        for support in &receipt.explanation.supports {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            let AssertionSupportWire::ProducerRelation {
                producer,
                proposal,
                accepted,
                ..
            } = support
            else {
                continue;
            };
            let [source, target] = proposal.endpoints();
            reserve_work(2).map_err(ProjectMapWireCheckError::Work)?;
            if !proposal_endpoint_matches(
                source,
                accepted.edge().from(),
                producer,
                &accepted_entity_proposals,
            ) || !proposal_endpoint_matches(
                target,
                accepted.edge().to(),
                producer,
                &accepted_entity_proposals,
            ) {
                return Err(ProjectMapWireCheckError::Invalid(
                    "producer relation endpoints do not join accepted graph edge endpoints",
                ));
            }
        }
    }
    for receipt in &map.core.supports {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        for evidence in &receipt.evidence_resolution.evidence_records {
            reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
            for id in evidence.source_handle_ids() {
                reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
                if !support_handles.contains_key(id) {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "support evidence references an unresolved source handle",
                    ));
                }
            }
        }
    }
    let mut custody_handles = std::collections::BTreeSet::new();
    for custody in &map.core.source_custody {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        let handle = match custody {
            SourceCustodyWire::Primary {
                handle,
                artifact_path,
                content_digest,
                byte_length,
                file,
            } => {
                if handle.project_generation() != Some(map.core.binding.project_generation)
                    || handle.reference_generation() != Some(map.core.binding.reference_generation)
                    || handle.origin_id() != map.core.project_container.origin_id.as_str()
                    || handle.path() != artifact_path
                    || handle.content_digest() != content_digest
                    || (handle.span().kind() == wow_core::SourceSpanKind::ByteRange
                        && handle
                            .span()
                            .byte_end()
                            .is_none_or(|end| end > *byte_length))
                {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "primary source custody disagrees with its handle or universe binding",
                    ));
                }
                if let Some(file) = file
                    && (file.manifest.relative_path != *artifact_path
                        || file.manifest.content_digest != *content_digest
                        || file.manifest.byte_length != *byte_length
                        || file.source_origin_id != map.core.project_container.origin_id
                        || file.project_generation != map.core.binding.project_generation)
                {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "project file receipt disagrees with source custody",
                    ));
                }
                handle
            }
            SourceCustodyWire::ReferenceContractNotEvaluated { handle } => {
                if handle.project_generation().is_some()
                    || handle.origin_kind() != wow_core::SourceOriginKind::ReferencePack
                    || handle.reference_generation() != Some(map.core.binding.reference_generation)
                {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "reference source receipt is not the bound Reference generation",
                    ));
                }
                handle
            }
        };
        if !custody_handles.insert(handle.handle_id())
            || support_handles
                .get(&handle.handle_id())
                .is_none_or(|support| *support != handle)
        {
            return Err(ProjectMapWireCheckError::Invalid(
                "source custody is duplicate or has no native support receipt",
            ));
        }
    }
    if custody_handles.len() != support_handles.len() {
        return Err(ProjectMapWireCheckError::Invalid(
            "ProjectMap source custody does not close support handles",
        ));
    }
    let selected_toc = map.core.load_custody.selected_toc.is_some();
    let package_load = map.core.load_custody.package_load.is_some();
    let package_main = map.core.load_custody.package_main.is_some();
    let project_navigation = matches!(map.core.navigation, MapNavigationWire::Project(_));
    let mut explanation_extensions = false;
    for receipt in &map.core.supports {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        explanation_extensions |=
            !super::assertion_links::check_local_chain(receipt, map, &mut reserve_work, stop)?;
    }
    Ok(
        if project_navigation
            || selected_toc
            || package_load
            || package_main
            || explanation_extensions
        {
            ProjectMapWireOutcome::NotEvaluated {
                project_navigation,
                selected_toc,
                package_load,
                package_main,
                explanation_extensions,
            }
        } else {
            ProjectMapWireOutcome::SupportedPrimaryL0
        },
    )
}

fn graph_query_digest<T: Serialize, E: From<ContextError>>(
    schema: &str,
    value: &T,
    prefix: &str,
    reserve_work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &std::sync::atomic::AtomicBool,
    max_bytes: u64,
) -> Result<String, ProjectMapWireCheckError<E>> {
    let material = (schema, value);
    // Bound canonical material before allocating it, then reserve proportional
    // work before serialization and for every bounded hash span.
    reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
    let encoded_length = crate::budget::encoded_len(&material, max_bytes, stop)
        .map_err(|error| ProjectMapWireCheckError::Work(E::from(error)))?;
    let encode_work = usize::try_from(encoded_length.div_ceil(4096))
        .map_err(|_| ProjectMapWireCheckError::Work(E::from(crate::budget::exhausted())))?
        .max(1);
    reserve_work(encode_work).map_err(ProjectMapWireCheckError::Work)?;
    let bytes = crate::budget::canonical(&material, max_bytes, stop)
        .map_err(|error| ProjectMapWireCheckError::Work(E::from(error)))?;
    let mut hasher = Sha256::new();
    for chunk in bytes.chunks(4096) {
        reserve_work(1).map_err(ProjectMapWireCheckError::Work)?;
        hasher.update(chunk);
    }
    let mut text = String::with_capacity(prefix.len() + 64);
    text.push_str(prefix);
    for byte in hasher.finalize() {
        use std::fmt::Write as _;
        write!(&mut text, "{byte:02x}").map_err(|_| {
            ProjectMapWireCheckError::Invalid("graph query digest formatting failed")
        })?;
    }
    Ok(text)
}

fn insert_node_record(
    records: &mut std::collections::BTreeMap<GraphNodeId, GraphNode>,
    node: &GraphNode,
) -> Result<(), &'static str> {
    match records.get(node.node_id()) {
        Some(previous) if previous != node => Err("graph node ID has conflicting selected records"),
        Some(_) => Ok(()),
        None => {
            records.insert(node.node_id().clone(), node.clone());
            Ok(())
        }
    }
}

fn edge_ids_for_view_insert(
    records: &mut std::collections::BTreeMap<wow_graph::GraphEdgeId, GraphEdge>,
    edge: &GraphEdge,
) -> Result<(), &'static str> {
    match records.get(edge.edge_id()) {
        Some(previous) if previous != edge => Err("graph edge ID has conflicting selected records"),
        Some(_) => Ok(()),
        None => {
            records.insert(edge.edge_id().clone(), edge.clone());
            Ok(())
        }
    }
}

fn proposal_endpoint_matches(
    endpoint: &wow_graph::GraphProposalEndpoint,
    expected: &GraphNodeId,
    producer: &ProducerSupportOriginWire,
    accepted_entities: &std::collections::BTreeMap<EndpointProposalKey<'_>, GraphNodeId>,
) -> bool {
    match endpoint {
        wow_graph::GraphProposalEndpoint::Existing(node_id) => node_id == expected,
        wow_graph::GraphProposalEndpoint::Proposed(proposal_id) => {
            accepted_entities.get(&(
                producer.partition_id.as_str(),
                producer.partition_digest.as_str(),
                producer.producer_version.as_str(),
                producer.batch_id.as_str(),
                producer.report_id.as_str(),
                proposal_id.as_ref(),
            )) == Some(expected)
        }
    }
}

// ProjectMap does not serialize the source snapshot's GraphLimits. These
// constructors use the schema's maximum admitted per-record evidence bound;
// this is structural identity checking, not reconstructed owner custody.
pub(super) const PROJECT_MAP_PROJECTION_LIMITS: GraphLimits = GraphLimits {
    max_nodes: 1,
    max_edges: 1,
    max_coverage_records: 1,
    max_evidence_per_record: 256,
    max_query_edges: 1,
};

pub(super) fn project_node_id(
    original: &GraphNode,
    map: &ProjectMapWire,
) -> Result<GraphNodeId, &'static str> {
    if original.universe() != &map.core.binding.graph_universe {
        return Err("source node belongs to another graph universe");
    }
    let retained = GraphNode::new(
        original.universe().clone(),
        original.generation().clone(),
        original.kind(),
        original.owner_key(),
        original.evidence_ids().to_vec(),
        PROJECT_MAP_PROJECTION_LIMITS,
    )
    .map_err(|_| "source node is invalid under the graph schema bounds")?;
    if retained != *original {
        return Err("source node identity or evidence does not match its original fields");
    }
    GraphNode::new(
        map.core.binding.graph_universe.clone(),
        map.core.binding.graph_generation.clone(),
        original.kind(),
        original.owner_key(),
        Vec::new(),
        PROJECT_MAP_PROJECTION_LIMITS,
    )
    .map(|projected| projected.node_id().clone())
    .map_err(|_| "source node cannot be projected under the graph schema bounds")
}

pub(super) fn project_edge(
    original: &GraphEdge,
    projected_nodes: &std::collections::BTreeMap<GraphNodeId, GraphNodeId>,
) -> Result<GraphEdge, &'static str> {
    let from = projected_nodes
        .get(original.from())
        .ok_or("source endpoint has no projected entity support")?;
    let to = projected_nodes
        .get(original.to())
        .ok_or("target endpoint has no projected entity support")?;
    let retained = GraphEdge::new(
        original.from().clone(),
        original.to().clone(),
        original.relation(),
        original.confidence(),
        original.evidence_ids().to_vec(),
        PROJECT_MAP_PROJECTION_LIMITS,
    )
    .map_err(|_| "source edge is invalid under the graph schema bounds")?;
    if retained != *original {
        return Err("source edge identity or evidence does not match its original fields");
    }
    GraphEdge::new(
        from.clone(),
        to.clone(),
        original.relation(),
        original.confidence(),
        original.evidence_ids().to_vec(),
        PROJECT_MAP_PROJECTION_LIMITS,
    )
    .map_err(|_| "source edge cannot be projected under the graph schema bounds")
}
