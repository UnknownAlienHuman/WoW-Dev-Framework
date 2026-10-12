//! Explicit composed candidate persistence adapter. Stored selections are
//! inert input; only a fresh genuine native/core capture can return authority.
use super::candidate::{BlizzardUiIndexCandidate, CoreRecognizerReports};
use super::*;
use serde::{Deserialize, de::IgnoredAny};
use wow_project::ProjectView;
use wow_project::replay::publication::AcquiredProjectPair;
use wow_store::project::{PartitionRecord, ReadSnapshot};

pub(crate) const CANDIDATE_RECORD_KEY: &str = "service:blizzard-ui-index-candidate";
pub(crate) const CANDIDATE_STORAGE_SCHEMA: &str = "wow-service.blizzard-ui-index-candidate.v1";
pub(crate) const CANDIDATE_STORAGE_CHECK: &str =
    "wow-service.blizzard-ui-index-candidate-rebuild.v1";
const RECEIPT_SCHEMA: &str = "wow-service/blizzard-ui-index-candidate-storage/1";

#[derive(Serialize)]
struct StoredReceipt<'a> {
    schema: &'static str,
    manifest: &'a candidate::BlizzardUiIndexManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_query: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml_query: Option<Vec<u8>>,
}
/// Ignored manifest DATA never constructs or updates any live owner.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InertReceipt {
    schema: String,
    manifest: IgnoredAny,
    #[serde(default, deserialize_with = "non_null_query")]
    function_query: Option<QueryBytes>,
    #[serde(default, deserialize_with = "non_null_query")]
    xml_query: Option<QueryBytes>,
}
struct QueryBytes(Vec<u8>);
impl<'de> Deserialize<'de> for QueryBytes {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct Bytes;
        impl<'de> serde::de::Visitor<'de> for Bytes {
            type Value = QueryBytes;
            fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.write_str("bounded inert query bytes")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<QueryBytes, A::Error> {
                let mut bytes = Vec::new();
                while let Some(byte) = seq.next_element::<u8>()? {
                    if bytes.len() >= super::super::GRAPH_REQUEST_MAX_BYTES {
                        return Err(serde::de::Error::custom("candidate query byte bound"));
                    }
                    bytes.push(byte);
                }
                Ok(QueryBytes(bytes))
            }
        }
        decoder.deserialize_seq(Bytes)
    }
}
fn non_null_query<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<Option<QueryBytes>, D::Error> {
    QueryBytes::deserialize(decoder).map(Some)
}

pub(crate) fn candidate_storage_record(
    owner: &BlizzardUiIndexCandidate,
    stop: &AtomicBool,
) -> ServiceResult<PartitionRecord> {
    checkpoint(stop)?;
    let (functions, xml) = owner.selected_queries();
    let receipt = StoredReceipt {
        schema: RECEIPT_SCHEMA,
        manifest: owner.manifest(),
        function_query: functions
            .map(|selection| {
                bounded_cancellable(selection, super::super::GRAPH_REQUEST_MAX_BYTES, stop)
            })
            .transpose()?,
        xml_query: xml
            .map(|selection| {
                bounded_cancellable(selection, super::super::GRAPH_REQUEST_MAX_BYTES, stop)
            })
            .transpose()?,
    };
    // Same store record ceiling; no source/report body clone or new cap.
    preflight_cancellable(&receipt, wow_store::project::MAX_RECORD_BYTES, stop)?;
    let record = PartitionRecord::new(CANDIDATE_RECORD_KEY, CANDIDATE_STORAGE_SCHEMA, &receipt)
        .map_err(crate::live_project::store_error)?;
    checkpoint(stop)?;
    Ok(record)
}

fn request_from_held_project(
    project: &ProjectView,
    receipt: InertReceipt,
    stop: &AtomicBool,
) -> ServiceResult<GraphBuildRequest> {
    checkpoint(stop)?;
    if receipt.schema != RECEIPT_SCHEMA {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let _manifest_is_only_inert_data = receipt.manifest;
    let configuration = project.configuration();
    let profile = configuration
        .platform_graph_profile()
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    if !matches!(
        profile,
        wow_project::PlatformGraphProfile::DirectCapabilitiesV1(_)
    ) {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let mut request = GraphBuildRequest::new(
        configuration.project_id().as_str().into(),
        project.project_generation().to_string(),
    )?
    .with_platform_graph_profile(profile);
    request.xml_classification_profile_digest = configuration
        .xml_classification_profile()
        .map(|profile| profile.digest());
    request.physical_function_declarations = configuration.physical_function_declarations();
    request.physical_function_signatures = configuration.physical_function_signatures();
    request.select_component_schema();
    if let Some(query) = receipt.function_query {
        request = request.with_function_fingerprints(
            FunctionFingerprintSelection::from_json_slice_cancellable(&query.0, stop)?,
        )?;
    }
    if let Some(query) = receipt.xml_query {
        request = request.with_xml_fingerprints(
            XmlFingerprintSelection::from_json_slice_cancellable(&query.0, stop)?,
        )?;
    }
    checkpoint(stop)?;
    Ok(request)
}

pub(crate) fn capture_candidate_from_backend(
    backend: &LocalProjectBackend,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<BlizzardUiIndexCandidate> {
    let project = request.acquire_project(backend, stop)?;
    compose_candidate_from_project(&project, request, stop)
}

/// Reuse one genuine view (the exact reopen hydrate or admitted backend view).
/// Existing producer methods and crosswalk/export guards execute once here.
fn compose_candidate_from_project(
    project: &ProjectView,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<BlizzardUiIndexCandidate> {
    checkpoint(stop)?;
    let source_prefix::SourcePrefix {
        owner: source_owner,
        projection,
    } = source_prefix::prepare(project, stop, &mut SourceGraphDiagnostics::default())?;
    let source_prefix::SourceProjection::Direct(plan) = projection else {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    };
    let direct = (*plan).finish(&source_owner, stop).map_err(project_error)?;
    let addresses = SourceGraphAddressCrosswalk::direct(&direct, stop)?;
    addresses.reserve("request", request, stop)?;
    let capture = candidate::capture_once(&direct, &addresses, request, stop)?;
    let snapshot = capture.graph();
    let CoreRecognizerReports {
        call_recognition,
        construction_recognition,
        mixin_recognition,
        mixin_assignment_recognition,
        script_recognition,
        state_recognition,
        signal_recognition,
        bridge_recognition,
        custom_recognition,
        cvar_recognition,
        hook_recognition: _,
        library_recognition: _,
        toc_recognition,
        xml_recognition,
        state_root_recognition,
        state_core_recognition,
    } = capture.reports();
    let provenance = addresses.source();
    let toc_topology = toc::maps(snapshot, &addresses, toc_recognition, stop)
        .map_err(|e| stage_error("toc-crosswalk", e))?;
    let xml_topology = xml::maps(snapshot, &addresses, xml_recognition, stop)
        .map_err(|e| stage_error("xml-crosswalk", e))?;
    let state_root_topology = toc::maps(
        snapshot,
        &addresses,
        std::slice::from_ref(state_root_recognition),
        stop,
    )
    .map_err(|e| stage_error("state-root-crosswalk", e))?;
    let state_core_topology = state_core::maps(snapshot, &addresses, state_core_recognition, stop)
        .map_err(|e| stage_error("state-core-crosswalk", e))?;
    let (state_nodes, state_edges) =
        state::maps_bound(snapshot, &addresses, state_recognition, stop)
            .map_err(|e| stage_error("state-crosswalk", e))?;
    let signal_topology = signals::maps(
        snapshot,
        &addresses,
        signal_recognition,
        bridge_recognition,
        custom_recognition,
        cvar_recognition,
        stop,
    )
    .map_err(|e| stage_error("signal-crosswalk", e))?;
    let signal_nodes = signal_topology.nodes;
    let signal_edges = signal_topology.edges;
    let (handler_nodes, script_edges) =
        scripts::maps_bound(snapshot, &addresses, script_recognition, stop)
            .map_err(|e| stage_error("script-crosswalk", e))?;
    let (function_nodes, call_edges) =
        calls::maps(snapshot, &addresses, call_recognition, None, stop)
            .map_err(|e| stage_error("call-crosswalk", e))?;
    let (frame_nodes, creation_edges) =
        construction::maps(snapshot, &addresses, construction_recognition, stop)
            .map_err(|e| stage_error("construction-crosswalk", e))?;
    let (
        mixin_instance_nodes,
        instantiation_edges,
        construction_mixin_edges,
        assignment_mixin_edges,
    ) = lua_mixins::maps(
        snapshot,
        &addresses,
        mixin_recognition,
        mixin_assignment_recognition,
        stop,
    )
    .map_err(|e| stage_error("mixin-crosswalk", e))?;
    checkpoint(stop)?;
    let source_nodes = materialized::nodes(snapshot, stop)?;
    let source_lookup = snapshot.producer_lookup(stop).map_err(graph_error)?;
    let mut file_nodes = Vec::new();
    for file in provenance.files() {
        checkpoint(stop)?;
        addresses.append(
            "file_nodes",
            &mut file_nodes,
            FileNode {
                path: file.path.clone(),
                node_id: addresses.node_id(snapshot, &source_lookup, &file.proposal_id, stop)?,
            },
            stop,
        )?;
    }
    let mut package_nodes = Vec::new();
    for package in provenance.packages() {
        checkpoint(stop)?;
        addresses.append(
            "package_nodes",
            &mut package_nodes,
            PackageNode {
                package: package.package.clone(),
                order_group: package.order_group,
                reachability: package.reachability,
                phase: package.phase,
                node_id: addresses.node_id(snapshot, &source_lookup, &package.proposal_id, stop)?,
            },
            stop,
        )?;
    }
    let mut package_file_edges = Vec::new();
    for receipt in provenance.package_files() {
        checkpoint(stop)?;
        addresses.append(
            "package_file_edges",
            &mut package_file_edges,
            PackageFileEdge {
                package: receipt.package.clone(),
                path: receipt.path.clone(),
                edge_id: addresses.edge_id(
                    snapshot,
                    &source_lookup,
                    &receipt.proposal_id,
                    &source_nodes,
                    stop,
                )?,
            },
            stop,
        )?;
    }
    let mut package_dependency_edges = Vec::new();
    for dependency in provenance.package_dependencies() {
        checkpoint(stop)?;
        let ProjectGraphPackageDependencyOutcome::Projected {
            proposal_id,
            confidence,
        } = &dependency.outcome
        else {
            continue;
        };
        addresses.append(
            "package_dependency_edges",
            &mut package_dependency_edges,
            PackageDependencyEdge {
                ordinal: dependency.ordinal,
                package: dependency.package.clone(),
                dependency: dependency.dependency.clone(),
                kind: dependency.kind,
                confidence: *confidence,
                edge_id: addresses.edge_id(
                    snapshot,
                    &source_lookup,
                    proposal_id,
                    &source_nodes,
                    stop,
                )?,
            },
            stop,
        )?;
    }
    let mut package_load_edges = Vec::new();
    for load in provenance.package_loads() {
        checkpoint(stop)?;
        let ProjectGraphPackageLoadOutcome::Projected {
            proposal_id,
            confidence,
        } = &load.outcome
        else {
            continue;
        };
        addresses.append(
            "package_load_edges",
            &mut package_load_edges,
            PackageLoadEdge {
                unit_digest: load.unit_digest,
                package: load.package.clone(),
                target: load.target.clone(),
                confidence: *confidence,
                edge_id: addresses.edge_id(
                    snapshot,
                    &source_lookup,
                    proposal_id,
                    &source_nodes,
                    stop,
                )?,
            },
            stop,
        )?;
    }
    let mut xml_nodes = Vec::new();
    for declaration in provenance.xml_declarations() {
        checkpoint(stop)?;
        addresses.append(
            "xml_nodes",
            &mut xml_nodes,
            XmlNode {
                occurrence_id: declaration.occurrence_id.clone(),
                path: declaration.path.clone(),
                node_id: addresses.node_id(
                    snapshot,
                    &source_lookup,
                    &declaration.proposal_id,
                    stop,
                )?,
            },
            stop,
        )?;
    }
    let mut lua_nodes = Vec::new();
    for declaration in provenance.lua_declarations() {
        checkpoint(stop)?;
        addresses.append(
            "lua_nodes",
            &mut lua_nodes,
            LuaNode {
                declaration_id: declaration.declaration_id.clone(),
                path: declaration.path.clone(),
                span: declaration.span,
                node_id: addresses.node_id(
                    snapshot,
                    &source_lookup,
                    &declaration.proposal_id,
                    stop,
                )?,
            },
            stop,
        )?;
    }
    addresses.reserve("snapshot", snapshot, stop)?;
    let bytes = bounded_cancellable(snapshot, GRAPH_INPUT_MAX_BYTES, stop)?;
    // Ensure export and existing import share byte/token/depth/string limits.
    // This is runtime admission, not a test or a second project/analyzer pass.
    let admitted: GraphPartitionSnapshot = super::super::input::decode(
        &bytes,
        GRAPH_INPUT_MAX_BYTES,
        1_000_000,
        GraphReadStage::Snapshot,
        stop,
    )
    .map_err(|e| stage_error("final-graph-decode", error(e.code)))?;
    if &admitted != snapshot {
        return Err(ServiceError::new(
            ServiceErrorCode::InternalContractViolation,
            "final graph export differs from its admitted owner snapshot",
        ));
    }
    checkpoint(stop)?;

    // Topology/source-export results above are bounded scratch, not a second
    // owned report aggregate. All original debits remain in the same ledger.
    drop(source_lookup);
    drop((
        toc_topology,
        xml_topology,
        state_root_topology,
        state_core_topology,
        state_nodes,
        state_edges,
        signal_nodes,
        signal_edges,
        handler_nodes,
        script_edges,
        function_nodes,
        call_edges,
        frame_nodes,
        creation_edges,
        mixin_instance_nodes,
        instantiation_edges,
        construction_mixin_edges,
        assignment_mixin_edges,
        file_nodes,
        package_nodes,
        package_file_edges,
        package_dependency_edges,
        package_load_edges,
        xml_nodes,
        lua_nodes,
        source_nodes,
        bytes,
        admitted,
    ));
    let prepared = capture.prepare_seal(&addresses, stop)?;
    drop(addresses);
    let parts = direct
        .into_index_source_parts(stop)
        .map_err(project_error)?;
    prepared.seal(parts, stop)
}

/// Returns no candidate until the final graph AND exact inert record version
/// match, then the generic reader checks the entire manifest on the same lease.
pub(crate) fn read_candidate_pair(
    read: &ReadSnapshot,
    stop: &AtomicBool,
) -> ServiceResult<(AcquiredProjectPair, BlizzardUiIndexCandidate)> {
    checkpoint(stop)?;
    let member = read
        .manifest()
        .members
        .iter()
        .find(|member| member.key == CANDIDATE_RECORD_KEY)
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    if member.schema != CANDIDATE_STORAGE_SCHEMA {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let record = read
        .record(CANDIDATE_RECORD_KEY, stop)
        .map_err(crate::live_project::store_error)?
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    let inert: InertReceipt = record
        .decode()
        .map_err(|_| error(ServiceErrorCode::IdentityMismatch))?;
    checkpoint(stop)?;
    let result =
        AcquiredProjectPair::read_with_native_owner_records(read, stop, |project, graph| {
            (|| {
                let request = request_from_held_project(project, inert, stop)?;
                let candidate = compose_candidate_from_project(project, &request, stop)?;
                if candidate.graph() != graph {
                    return Err(error(ServiceErrorCode::IdentityMismatch));
                }
                let expected = candidate_storage_record(&candidate, stop)?;
                if expected.version() != record.version() {
                    return Err(error(ServiceErrorCode::IdentityMismatch));
                }
                checkpoint(stop)?;
                Ok((candidate, vec![expected]))
            })()
            .map_err(CandidateReadFailure::Service)
        })
        .map_err(|failure| match failure {
            CandidateReadFailure::Service(error) => error,
            CandidateReadFailure::Project(error) => project_error(error),
        })?;
    checkpoint(stop)?;
    Ok(result)
}

// Private error funnel preserves the original service cancellation/refusal.
// No ProjectError constructor, public wire/type, or synthesized success.
enum CandidateReadFailure {
    Project(wow_project::ProjectError),
    Service(ServiceError),
}
impl From<wow_project::ProjectError> for CandidateReadFailure {
    fn from(error: wow_project::ProjectError) -> Self {
        Self::Project(error)
    }
}
