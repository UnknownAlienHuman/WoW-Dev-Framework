//! Project-owned coherent handoff to the manifested store. Semantic identities
//! precede logical membership, publication-set identity and store generation.
mod commitments;
mod direct;
mod namespace;
use super::{ProjectReplay, invalid};
use crate::graph::{
    PLATFORM_RAW_INVENTORY_PARTITION, SOURCE_GRAPH_PARTITION, build_source_graph_proposals,
};
use crate::{ProjectPhase, ProjectPublisher, ProjectResult, ProjectView};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
use wow_graph::{GraphPartitionSnapshot, ValidatedGraphPartitionSnapshot};
use wow_store::project::{
    EpochId, PartitionRecord, ProjectStoreNamespace, ReadSnapshot, StoreGenerationId,
};

pub const STORAGE_SCHEMAS: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    "wow-project.live-replay.v16",
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
/// Exact catalog before platform raw-source replay; retain these epoch identities.
pub const STORAGE_SCHEMAS_V4: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-pair.v1",
];
/// Exact catalog before opt-in same-session package XML binding replay.
pub const STORAGE_SCHEMAS_V5: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-pair.v1",
];
/// Exact catalog before explicitly selected package graph projection.
pub const STORAGE_SCHEMAS_V6: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-pair.v1",
];
/// Exact catalog before the independent platform raw inventory graph partition.
pub const STORAGE_SCHEMAS_V7: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-pair.v1",
];
/// Exact catalog before explicitly selected native direct platform producers.
pub const STORAGE_SCHEMAS_V8: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-pair.v1",
];
pub const STORAGE_CHECK: &str = "wow-project.live-pair-native-replay.v1";
/// Frozen catalog before exact independently versioned capability selection.
pub const STORAGE_SCHEMAS_V9: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-pair.v1",
];
/// Frozen catalog before native retained-package and function-budget replay.
pub const STORAGE_SCHEMAS_V10: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-pair.v1",
];
/// Frozen actual catalog before explicitly selected global-access work/text.
pub const STORAGE_SCHEMAS_V12: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-pair.v1",
];
/// Frozen actual catalog before explicitly selected graph metadata capacity.
pub const STORAGE_SCHEMAS_V13: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-pair.v1",
];
/// Frozen actual catalog before the optional native XML preflight count component.
pub const STORAGE_SCHEMAS_V14: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-pair.v1",
];
/// Exact complete catalog admitted before report10/replay16. This is the former
/// default catalog, including v15; V14 cannot reopen these retained epochs.
pub const STORAGE_SCHEMAS_V15: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    "wow-project.live-pair.v1",
];
/// New exact catalog for the report10 retained global-access replay recipe.
/// Earlier report9 catalogs remain unchanged and continue to reject v16 writes.
pub const STORAGE_SCHEMAS_V16: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    "wow-project.live-replay.v16",
    "wow-project.live-pair.v1",
];
/// Exact catalog for the named AceAddon local-result replay consumer. Earlier
/// catalogs remain frozen; this admits only their retained records plus the new
/// explicitly selected recipe.
pub const STORAGE_SCHEMAS_ACEADDON_LOCAL_RESULT_V1: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    "wow-project.live-replay.v16",
    super::ACEADDON_LOCAL_RESULT_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Explicit composed classification catalog. Keep ordinary/retained epoch bytes
/// unchanged; application selection must opt into this catalog before publishing.
pub const STORAGE_SCHEMAS_XML_CLASSIFICATION_V1: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    super::XML_CLASSIFICATION_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Explicit declaration-only catalog; prior catalogs keep their exact epoch bytes.
pub const STORAGE_SCHEMAS_PHYSICAL_FUNCTION_DECLARATIONS_V1: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    super::PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Explicit composed XML/declaration catalog. Neither old selected catalog is widened.
pub const STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_V1: &[&str] = &[
    "wow-project.live-replay.v1",
    "wow-project.live-replay.v2",
    "wow-project.live-replay.v3",
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v13",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    super::XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Exact body-shape recipe catalog. Old recipes retain their separate catalogs.
pub const STORAGE_SCHEMAS_PHYSICAL_FUNCTION_BODY_SHAPE_V1: &[&str] = &[
    super::PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Exact composed XML/body-shape catalog; no prior selected epoch is widened.
pub const STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_V1: &[&str] = &[
    super::XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Fixed selected-component envelope; exact recipes are retained in configuration.
/// Prior catalogs keep their exact schema membership and epoch identities.
pub const STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1: &[&str] = &[
    super::SELECTED_COMPONENTS_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Ordinary physical Primary callables; platform and prior catalogs stay frozen.
pub const STORAGE_SCHEMAS_PRIMARY_CALLABLES_V1: &[&str] = &[
    super::PRIMARY_CALLABLES_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Ordinary native Library producer-input custody; prior epoch catalogs are frozen.
pub const STORAGE_SCHEMAS_NATIVE_LIBRARY_INPUT_V1: &[&str] = &[
    super::NATIVE_LIBRARY_INPUT_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Ordinary selected native API-use recipe; prior epoch catalogs remain frozen.
pub const STORAGE_SCHEMAS_NATIVE_API_USES_V1: &[&str] = &[
    super::NATIVE_API_USES_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
/// Exact catalogs admitted by open/recovery/migration. Never widen a retained epoch.
pub const STORAGE_SCHEMA_CATALOGS: &[&[&str]] = &[
    STORAGE_SCHEMAS_NATIVE_API_USES_V1,
    STORAGE_SCHEMAS_NATIVE_LIBRARY_INPUT_V1,
    STORAGE_SCHEMAS_PRIMARY_CALLABLES_V1,
    STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1,
    STORAGE_SCHEMAS_PHYSICAL_FUNCTION_BODY_SHAPE_V1,
    STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_V1,
    STORAGE_SCHEMAS_PHYSICAL_FUNCTION_DECLARATIONS_V1,
    STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_V1,
    STORAGE_SCHEMAS_XML_CLASSIFICATION_V1,
    STORAGE_SCHEMAS,
    STORAGE_SCHEMAS_ACEADDON_LOCAL_RESULT_V1,
    STORAGE_SCHEMAS_V16,
    STORAGE_SCHEMAS_V15,
    STORAGE_SCHEMAS_V14,
    STORAGE_SCHEMAS_V13,
    STORAGE_SCHEMAS_V12,
    STORAGE_SCHEMAS_V10,
    STORAGE_SCHEMAS_V9,
    STORAGE_SCHEMAS_V8,
    STORAGE_SCHEMAS_V7,
    STORAGE_SCHEMAS_V6,
    STORAGE_SCHEMAS_V5,
    STORAGE_SCHEMAS_V4,
    STORAGE_SCHEMAS_V3,
    STORAGE_SCHEMAS_V2,
    STORAGE_SCHEMAS_V1,
];
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
    replay_storage_schema: &'static str,
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
    /// Publish into an explicitly selected stable platform namespace. The native
    /// semantic records and publication-set identity precede store bindings.
    pub fn build_in_namespace(
        publisher: &ProjectPublisher,
        graph: &GraphPartitionSnapshot,
        namespace: &ProjectStoreNamespace,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let replay = ProjectReplay::capture(publisher, stop)?;
        let project = publisher
            .current_snapshot()
            .ok_or_else(invalid)?
            .open_view();
        validate_pair(&project, graph, stop)?;
        plan_in_namespace(&replay, &project, graph, Some(namespace), stop)
    }
    /// Re-enter native owners once before validating the selected namespace.
    pub fn from_replay_in_namespace(
        replay: &ProjectReplay,
        graph: &GraphPartitionSnapshot,
        namespace: &ProjectStoreNamespace,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let project = replay.hydrate(stop)?;
        validate_pair(&project, graph, stop)?;
        plan_in_namespace(replay, &project, graph, Some(namespace), stop)
    }
    /// Use the already finished native proof without rebuilding its projection.
    /// This remains a generic project/graph bundle; service records are separate.
    pub fn build_with_native_parts(
        publisher: &ProjectPublisher,
        graph: &GraphPartitionSnapshot,
        native: &crate::graph::PlatformIndexSourceParts,
        namespace: Option<&ProjectStoreNamespace>,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let project = publisher.open_current()?;
        validate_native_parts(&project, graph, native, stop)?;
        let replay = ProjectReplay::capture(publisher, stop)?;
        plan_in_namespace(&replay, &project, graph, namespace, stop)
    }

    /// Exact schema already selected and validated for the retained replay record.
    /// Catalog dispatch borrows this native handoff without decoding record bytes.
    #[must_use]
    pub const fn replay_storage_schema(&self) -> &'static str {
        self.replay_storage_schema
    }

    pub fn into_parts(self) -> (Vec<PartitionRecord>, BTreeMap<String, String>) {
        (self.records, self.bindings)
    }
}

/// Executable immutable owners reconstructed under one held read snapshot.
/// This is distinct from serialized storage records and public result DTOs.
pub struct AcquiredProjectPair {
    read_epoch_id: EpochId,
    read_generation_id: StoreGenerationId,
    project: ProjectView,
    publisher: ProjectPublisher,
    supports_physical_update: bool,
    graph: wow_graph::ValidatedGraphPartitionSnapshot,
    source_provenance: Option<Box<crate::graph::ProjectGraphProvenance>>,
    publication_set_id: String,
}
impl AcquiredProjectPair {
    /// Match the original exact retained owner read without reconstructing its
    /// proof from a serialized publication or caller-supplied identity.
    pub fn is_bound_to_read(&self, read: &ReadSnapshot) -> bool {
        &self.read_epoch_id == read.epoch().epoch_id()
            && self.read_generation_id == read.manifest().generation_id
    }
    pub fn project(&self) -> &ProjectView {
        &self.project
    }
    pub fn graph(&self) -> &GraphPartitionSnapshot {
        self.graph.as_snapshot()
    }
    /// Borrow the retained graph owner's proof without a new rebuild or clone.
    pub fn graph_read(
        &self,
        stop: &AtomicBool,
    ) -> wow_graph::GraphResult<wow_graph::GraphPartitionRead<'_>> {
        self.graph.borrow_read(stop)
    }
    /// Exact source-owner provenance rebuilt while validating this acquired pair.
    /// Only the ordinary source branch retains it; direct/native acquisition
    /// returns None. This accessor does not construct an evidence catalog.
    #[must_use]
    pub fn source_provenance(&self) -> Option<&crate::graph::ProjectGraphProvenance> {
        self.source_provenance.as_deref()
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
    /// Consume this exact rebuilt native pair for a retained-platform update.
    /// The proof must own the same snapshot and validated native graph prefix.
    /// Ordinary physical/legacy admission remains in into_update_publisher.
    pub fn into_platform_update_publisher(
        self,
        native: &crate::graph::PlatformIndexSourceParts,
        stop: &AtomicBool,
    ) -> ProjectResult<ProjectPublisher> {
        crate::analyzer::checkpoint(stop)?;
        let configuration = self.project.configuration();
        if !matches!(
            configuration.platform_graph_profile(),
            Some(crate::PlatformGraphProfile::DirectCapabilitiesV1(_))
        ) || configuration.platform_packages().is_none()
            || configuration.package_load_plan().is_none()
            || configuration.package_main_plan().is_none()
            || self
                .project
                .snapshot()
                .generation_candidate()
                .project_generation_schema_version()
                != 2
            || self
                .project
                .snapshot()
                .analyzer_binding()
                .function_call_report()
                .is_none()
        {
            return Err(invalid());
        }
        validate_read_native_parts(&self.project, &self.graph, native, stop)?;
        crate::analyzer::checkpoint(stop)?;
        Ok(self.publisher)
    }

    pub fn read(read: &ReadSnapshot, stop: &AtomicBool) -> ProjectResult<Self> {
        Self::read_owner(
            read,
            stop,
            |_, _| Ok(((), Vec::new())),
            |project, graph, _, stop| validate_read_pair(project, graph, stop),
        )
        .map(|(pair, ())| pair)
    }

    /// A compiled companion owner rebuilds on this one hydrated native view.
    /// Its additional records must match exact manifested membership. This
    /// does not decode or attest the companion owner's semantic authority.
    pub fn read_with_native_owner_records<T, E>(
        read: &ReadSnapshot,
        stop: &AtomicBool,
        rebuild: impl FnOnce(
            &ProjectView,
            &GraphPartitionSnapshot,
        ) -> Result<(T, Vec<PartitionRecord>), E>,
    ) -> Result<(Self, T), E>
    where
        T: AsRef<crate::graph::PlatformIndexSourceParts>,
        E: From<crate::ProjectError>,
    {
        Self::read_owner(read, stop, rebuild, |project, graph, owner: &T, stop| {
            validate_read_native_parts(project, graph, owner.as_ref(), stop).map(|()| None)
        })
    }

    fn read_owner<T, E: From<crate::ProjectError>>(
        read: &ReadSnapshot,
        stop: &AtomicBool,
        rebuild: impl FnOnce(
            &ProjectView,
            &GraphPartitionSnapshot,
        ) -> Result<(T, Vec<PartitionRecord>), E>,
        validate: impl FnOnce(
            &ProjectView,
            &ValidatedGraphPartitionSnapshot,
            &T,
            &AtomicBool,
        ) -> ProjectResult<Option<crate::graph::ProjectGraphProvenance>>,
    ) -> Result<(Self, T), E> {
        crate::analyzer::checkpoint(stop)?;
        let header: PairHeader = load(read, HEADER_KEY, "wow-project.live-pair.v1", stop)?;
        if header.schema != HEADER_SCHEMA {
            return Err(invalid().into());
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
                | "wow-project.live-replay.v5"
                | "wow-project.live-replay.v6"
                | "wow-project.live-replay.v7"
                | "wow-project.live-replay.v8"
                | "wow-project.live-replay.v9"
                | "wow-project.live-replay.v10"
                | "wow-project.live-replay.v11"
                | "wow-project.live-replay.v12"
                | "wow-project.live-replay.v13"
                | "wow-project.live-replay.v14"
                | "wow-project.live-replay.v15"
                | "wow-project.live-replay.v16"
                | super::XML_CLASSIFICATION_STORAGE_SCHEMA
                | super::PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA
                | super::XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA
                | super::PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA
                | super::XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA
                | super::SELECTED_COMPONENTS_STORAGE_SCHEMA
                | super::PRIMARY_CALLABLES_STORAGE_SCHEMA
                | super::NATIVE_LIBRARY_INPUT_STORAGE_SCHEMA
                | super::NATIVE_API_USES_STORAGE_SCHEMA
        ) {
            return Err(invalid().into());
        }
        let replay: ProjectReplay = load(read, REPLAY_KEY, replay_schema, stop)?;
        if replay.storage_schema()? != replay_schema {
            return Err(invalid().into());
        }
        let graph =
            GraphPartitionSnapshot::read_stored_validated(read, stop).map_err(graph_error)?;
        let publisher = replay.hydrate_owner(stop)?;
        let project = publisher.open_current()?;
        let (owner, companion_records) = rebuild(&project, graph.as_snapshot())?;
        let source_provenance = validate(&project, &graph, &owner, stop)?;
        commitments::validate_read(
            read,
            &replay,
            &project,
            &graph,
            &header,
            companion_records,
            stop,
        )?;
        crate::analyzer::checkpoint(stop)?;
        Ok((
            Self {
                read_epoch_id: read.epoch().epoch_id().clone(),
                read_generation_id: read.manifest().generation_id.clone(),
                project,
                publisher,
                supports_physical_update: replay.supports_physical_update(),
                graph,
                source_provenance: source_provenance.map(Box::new),
                publication_set_id: header.publication_set_id,
            },
            owner,
        ))
    }
}

/// Only a genuine consumed native owner supplies these prefix commitments.
/// Pointer equality requires the same immutable snapshot, not matching text IDs.
fn validate_native_parts(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    native: &crate::graph::PlatformIndexSourceParts,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    graph.validate(stop).map_err(graph_error)?;
    validate_native_binding(project, graph, native, stop)
}

fn validate_native_binding(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    native: &crate::graph::PlatformIndexSourceParts,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let profile = match project.configuration().platform_graph_profile() {
        Some(crate::PlatformGraphProfile::DirectCapabilitiesV1(selected))
            if native.capabilities() == Some(selected) =>
        {
            crate::graph::PLATFORM_CAPABILITIES_GRAPH_PROFILE
        }
        Some(crate::PlatformGraphProfile::DirectPlatformProducersWithRawInventoryV1)
            if native.capabilities().is_none() =>
        {
            crate::graph::PLATFORM_DIRECT_GRAPH_PROFILE
        }
        _ => return Err(invalid()),
    };
    if !std::sync::Arc::ptr_eq(project.snapshot(), native.project().snapshot())
        || profile != native.profile()
        || graph.registry().registry_digest() != native.registry_digest()
        || graph.foundation().snapshot_id() != native.foundation_snapshot_id()
        || graph.source_context_id() != native.scope().source_context_id
        || graph.partition(SOURCE_GRAPH_PARTITION).is_some()
    {
        return Err(invalid());
    }
    for (id, digest) in native.native_prefix_partitions() {
        crate::analyzer::checkpoint(stop)?;
        if graph
            .partition(id)
            .is_none_or(|partition| partition.partition_digest() != digest)
        {
            return Err(invalid());
        }
    }
    crate::analyzer::checkpoint(stop)
}

fn validate_pair(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    validate_pair_with_source_provenance(project, graph, stop).map(|_| ())
}

fn validate_pair_with_source_provenance(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<Option<crate::graph::ProjectGraphProvenance>> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    graph.validate(stop).map_err(graph_error)?;
    validate_source_binding(project, graph, stop)
}

fn validate_read_pair(
    project: &ProjectView,
    graph: &ValidatedGraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<Option<crate::graph::ProjectGraphProvenance>> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    validate_source_binding(project, graph.as_snapshot(), stop)
}

fn validate_source_binding(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<Option<crate::graph::ProjectGraphProvenance>> {
    if project
        .configuration()
        .platform_graph_profile()
        .is_some_and(crate::PlatformGraphProfile::is_direct)
    {
        return direct::validate(project, graph, stop).map(|()| None);
    }
    if direct::has_direct_partitions(graph) {
        return Err(invalid());
    }
    // Reuse the source owner over the real replayed session. No metadata receipt
    // is relabeled as a live project or treated as an executable analyzer.
    let proposals = build_source_graph_proposals(project, stop)?;
    let inventory_batch = proposals.inventory_batch().cloned();
    let (registry, batch, mut coverage, provenance, limits) = proposals.into_parts();
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
    match (
        inventory_batch.as_ref(),
        graph.partition(PLATFORM_RAW_INVENTORY_PARTITION),
    ) {
        (None, None) => {}
        (Some(batch), Some(partition))
            if partition.batch() == batch
                && partition.coverage().is_empty()
                && partition.report().accepted_entities().len()
                    == batch.entity_proposals().len()
                && partition.report().accepted_relations().is_empty() => {}
        _ => return Err(invalid()),
    }
    Ok(Some(provenance))
}

fn plan_in_namespace(
    replay: &ProjectReplay,
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    namespace: Option<&ProjectStoreNamespace>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectPublicationBundle> {
    let mut plan = plan(replay, project, graph, stop)?;
    if let Some(namespace) = namespace {
        namespace::bind(&mut plan.bindings, namespace, project, graph)?;
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(plan)
}

fn plan(
    replay: &ProjectReplay,
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<ProjectPublicationBundle> {
    let mut records = graph.storage_records(stop).map_err(graph_error)?;
    let replay_storage_schema = replay.storage_schema()?;
    records.push(
        PartitionRecord::new(REPLAY_KEY, replay_storage_schema, replay)
            .map_err(|_| super::exhausted())?,
    );
    records.sort_by(|a, b| a.key().cmp(b.key()));
    // PartitionVersionId already binds key, schema, byte count and exact payload.
    let membership = records
        .iter()
        .map(|r| (r.key(), r.version()))
        .collect::<Vec<_>>();
    let (header, bindings) = commitments::metadata(project, graph, &membership)?;
    records.push(
        PartitionRecord::new(HEADER_KEY, "wow-project.live-pair.v1", &header)
            .map_err(|_| invalid())?,
    );
    records.sort_by(|a, b| a.key().cmp(b.key()));
    Ok(ProjectPublicationBundle {
        records,
        bindings,
        replay_storage_schema,
    })
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
        wow_graph::GraphErrorCode::StoreReadUnavailable => crate::ProjectError::new(
            crate::ProjectErrorCode::StoreReadUnavailable,
            ProjectPhase::Publication,
            "native project pair graph store read is unavailable",
        ),
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
        wow_store::StoreErrorCode::DatabaseUnavailable
        | wow_store::StoreErrorCode::WriterBusy
        | wow_store::StoreErrorCode::OutcomeUnknown => crate::ProjectError::new(
            crate::ProjectErrorCode::StoreReadUnavailable,
            ProjectPhase::Publication,
            "native project pair store read is unavailable",
        ),
        _ => invalid(),
    }
}
