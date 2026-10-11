use serde::Serialize;
use wow_graph::GraphRelationKind;

pub const PRIMARY_MAP_PROFILE: &str = "wow-context/primary-direct-navigation/1";
pub const PRIMARY_PROJECT_MAP_PROFILE: &str = "wow-context/primary-project-navigation/1";
pub const PRIMARY_PROJECT_MAP_PROFILE_BYTES: &[u8] = br#"{"confidence":"include_candidate","direction":"both","edges":["contains","defines","depends_on","optional_depends_on","owns","loads","loads_before"],"nodes":"project_container_and_all_source_file_direct_endpoints","ordering":"node_id_then_edge_id","platform":"absent_profile_excluded","privacy":"local_metadata_only","profile":"wow-context/primary-project-navigation/1","reference":"binding_and_coverage_only","source":"handles_and_load_custody_only","tokens":"unavailable"}"#;

/// Frozen canonical descriptor. Node ordering is exact node ID; direct parallel
/// edges retain native edge-ID order. No grouping, source-text labels, or paths.
pub const PRIMARY_MAP_PROFILE_BYTES: &[u8] = br#"{"confidence":"include_candidate","direction":"both","edges":["contains","defines","depends_on","optional_depends_on","owns","loads","loads_before"],"nodes":"exact_root_and_direct_endpoints","ordering":"node_id_then_edge_id","platform":"absent_profile_excluded","privacy":"local_metadata_only","profile":"wow-context/primary-direct-navigation/1","reference":"binding_and_coverage_only","source":"handles_and_load_custody_only","tokens":"unavailable"}"#;

pub(crate) const RELATIONS: [GraphRelationKind; 7] = [
    GraphRelationKind::Contains,
    GraphRelationKind::Defines,
    GraphRelationKind::DependsOn,
    GraphRelationKind::OptionalDependsOn,
    GraphRelationKind::Owns,
    GraphRelationKind::Loads,
    GraphRelationKind::LoadsBefore,
];

#[derive(Serialize)]
pub(crate) struct ProfileDescription {
    confidence: &'static str,
    direction: &'static str,
    edges: &'static [GraphRelationKind],
    nodes: &'static str,
    ordering: &'static str,
    platform: &'static str,
    privacy: &'static str,
    profile: &'static str,
    reference: &'static str,
    source: &'static str,
    tokens: &'static str,
}
pub(crate) const PROFILE: ProfileDescription = ProfileDescription {
    confidence: "include_candidate",
    direction: "both",
    edges: &RELATIONS,
    nodes: "exact_root_and_direct_endpoints",
    ordering: "node_id_then_edge_id",
    platform: "absent_profile_excluded",
    privacy: "local_metadata_only",
    profile: PRIMARY_MAP_PROFILE,
    reference: "binding_and_coverage_only",
    source: "handles_and_load_custody_only",
    tokens: "unavailable",
};
pub(crate) const PROJECT_PROFILE: ProfileDescription = ProfileDescription {
    nodes: "project_container_and_all_source_file_direct_endpoints",
    profile: PRIMARY_PROJECT_MAP_PROFILE,
    ..PROFILE
};

/// The complete reviewed native direct relation vocabulary. This does not assert
/// that every producer/capability was evaluated in the retained generation.
pub(crate) const DETAIL_RELATIONS: [GraphRelationKind; 27] = [
    GraphRelationKind::Calls,
    GraphRelationKind::ReadsState,
    GraphRelationKind::WritesState,
    GraphRelationKind::RegistersNativeEvent,
    GraphRelationKind::HandlesNativeEvent,
    GraphRelationKind::BridgesNativeEvent,
    GraphRelationKind::EmitsCustomSignal,
    GraphRelationKind::HandlesCustomSignal,
    GraphRelationKind::RegistersCvarCallback,
    GraphRelationKind::SetsScript,
    GraphRelationKind::HooksScript,
    GraphRelationKind::SecureHooksFunction,
    GraphRelationKind::Contains,
    GraphRelationKind::Defines,
    GraphRelationKind::DependsOn,
    GraphRelationKind::OptionalDependsOn,
    GraphRelationKind::Owns,
    GraphRelationKind::Loads,
    GraphRelationKind::LoadsBefore,
    GraphRelationKind::Inherits,
    GraphRelationKind::ParentOf,
    GraphRelationKind::ReferencesTemplate,
    GraphRelationKind::MixesIn,
    GraphRelationKind::Instantiates,
    GraphRelationKind::FactoryCreates,
    GraphRelationKind::UsesApi,
    GraphRelationKind::HasSchemaClass,
];
pub(crate) const DETAIL_PROFILE: ProfileDescription = ProfileDescription {
    profile: "wow-context/primary-entity-detail-origins/1",
    edges: &DETAIL_RELATIONS,
    ..PROFILE
};
pub(crate) const FRONTIER_PROFILE: ProfileDescription = ProfileDescription {
    profile: "wow-context/primary-frontier-origins/1",
    nodes: "initial_detail_and_explicit_frontier_direct_endpoints",
    edges: &DETAIL_RELATIONS,
    ..PROFILE
};
