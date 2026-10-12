//! Genuine composed native capture. No receipt/data constructor mints authority.
//! The selected caller replaces its existing producer block with capture_once;
//! running both blocks would be duplicate work and is not an admitted route.
use super::source_addresses::SourceGraphAddressCrosswalk;
use super::*;
use wow_graph::{GraphAssertionRecordScope, GraphGenerationId, GraphSnapshotId};
use wow_project::graph::{PlatformGraphProvenance, PlatformIndexSourceParts};
use wow_recognizers::{
    source_bridge::W2_PARTITION,
    source_calls::SOURCE_CALL_PARTITION,
    source_construction::SOURCE_CONSTRUCTION_PARTITION,
    source_mixins::{SOURCE_MIXIN_ASSIGNMENT_PARTITION, SOURCE_MIXIN_PARTITION},
    source_scripts::{SOURCE_SCRIPT_PARTITION, W5_HOOK_PARTITION},
    source_signals::{W1_SIGNAL_PARTITION, W3_SIGNAL_PARTITION, W4_PARTITION},
    source_state::{SOURCE_STATE_LIBRARY_PARTITION, SOURCE_STATE_PARTITION},
    source_state_core::{SourceStateCoreFamily, SourceStateCoreRecognition},
    source_toc::{SourceTocFamily, SourceTocRecognition},
    source_xml::{SourceXmlFamily, SourceXmlRecognition},
};

const CANDIDATE_SCHEMA: &str = "wow-service/blizzard-ui-index-candidate/1";
const CORE: [&str; 13] = [
    SOURCE_CALL_PARTITION,
    SOURCE_CONSTRUCTION_PARTITION,
    SOURCE_MIXIN_PARTITION,
    SOURCE_MIXIN_ASSIGNMENT_PARTITION,
    SOURCE_SCRIPT_PARTITION,
    SOURCE_STATE_PARTITION,
    W1_SIGNAL_PARTITION,
    W2_PARTITION,
    W3_SIGNAL_PARTITION,
    W4_PARTITION,
    W5_HOOK_PARTITION,
    SOURCE_STATE_LIBRARY_PARTITION,
    SourceTocFamily::SavedVariableRoot.partition_id(),
];
fn core_ids() -> impl Iterator<Item = &'static str> {
    CORE.into_iter()
        .chain(
            SourceTocFamily::ALL
                .into_iter()
                .map(|family| family.partition_id()),
        )
        .chain(
            SourceXmlFamily::ALL
                .into_iter()
                .map(|family| family.partition_id()),
        )
        .chain(
            SourceStateCoreFamily::ALL
                .into_iter()
                .map(|family| family.partition_id()),
        )
}

/// Read-only originals, including receipts, omissions and unconfirmed outcomes.
/// There is no capture constructor taking this aggregate from caller data.
#[derive(Serialize)]
pub struct CoreRecognizerReports {
    pub(super) call_recognition: SourceCallRecognition,
    pub(super) construction_recognition: SourceConstructionRecognition,
    pub(super) mixin_recognition: SourceMixinRecognition,
    pub(super) mixin_assignment_recognition: SourceMixinAssignmentRecognition,
    #[serde(serialize_with = "serialize_scripts")]
    pub(super) script_recognition: ScriptRecognition,
    #[serde(serialize_with = "serialize_state")]
    pub(super) state_recognition: StateRecognition,
    pub(super) signal_recognition: signals::W1Recognition,
    pub(super) bridge_recognition: signals::W2BridgeRecognition,
    pub(super) custom_recognition: signals::W3Recognition,
    pub(super) cvar_recognition: signals::W4Recognition,
    pub(super) hook_recognition: signals::W5HookRecognition,
    pub(super) library_recognition: signals::SourceLibraryRecognition,
    pub(super) toc_recognition: Vec<SourceTocRecognition>,
    pub(super) xml_recognition: Vec<SourceXmlRecognition>,
    pub(super) state_root_recognition: SourceTocRecognition,
    pub(super) state_core_recognition: Vec<SourceStateCoreRecognition>,
}
fn serialize_scripts<S: serde::Serializer>(
    value: &ScriptRecognition,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        ScriptRecognition::Assertions(value) => value.serialize(serializer),
        ScriptRecognition::Legacy(_) => {
            Err(serde::ser::Error::custom("native script proof required"))
        }
    }
}
fn serialize_state<S: serde::Serializer>(
    value: &StateRecognition,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        StateRecognition::Assertions(value) => value.serialize(serializer),
        StateRecognition::Legacy(_) => {
            Err(serde::ser::Error::custom("native state proof required"))
        }
    }
}

#[derive(Serialize)]
struct SelectedQueries {
    #[serde(skip_serializing_if = "Option::is_none")]
    functions: Option<FunctionFingerprintSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml: Option<XmlFingerprintSelection>,
}
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ComponentSelection {
    Selected,
    NotEvaluated,
}
impl ComponentSelection {
    const fn for_presence(present: bool) -> Self {
        if present {
            Self::Selected
        } else {
            Self::NotEvaluated
        }
    }
}

#[derive(Serialize)]
enum CandidatePublicationState {
    NotPublishedE3A,
}
/// Inert identity/query receipt. Serialization does not grant candidate authority.
/// Stored readers must rebuild through the genuine native/service owners.
#[derive(Serialize)]
pub struct BlizzardUiIndexManifest {
    schema: &'static str,
    candidate_id: Box<str>,
    candidate_digest: Box<str>,
    binding: NativeBinding,
    final_graph_snapshot_id: GraphSnapshotId,
    final_graph_generation: GraphGenerationId,
    queries: SelectedQueries,
    coverage: wow_graph::GraphCoverageState,
    publication_state: CandidatePublicationState,
    function_fingerprints: ComponentSelection,
    xml_fingerprints: ComponentSelection,
}
impl BlizzardUiIndexManifest {
    pub fn candidate_id(&self) -> &str {
        &self.candidate_id
    }
    pub fn candidate_digest(&self) -> &str {
        &self.candidate_digest
    }
}

/// One immutable native plus composed capability, before any store publication.
/// Never constructed by Deserialize, a raw ProjectView/graph pair or a receipt.
pub struct BlizzardUiIndexCandidate {
    // The aggregate is retained through nested native replay/readback. Keep
    // one owned body so moving its capability never copies a large stack value.
    body: Box<CandidateBody>,
}

struct CandidateBody {
    native: PlatformIndexSourceParts,
    snapshot: GraphPartitionSnapshot,
    reports: CoreRecognizerReports,
    function_fingerprints: Option<wow_project::graph::fingerprints::FunctionFingerprintManifest>,
    xml_fingerprints: Option<XmlFingerprintManifest>,
    manifest: BlizzardUiIndexManifest,
}
impl AsRef<PlatformIndexSourceParts> for BlizzardUiIndexCandidate {
    fn as_ref(&self) -> &PlatformIndexSourceParts {
        &self.body.native
    }
}
impl BlizzardUiIndexCandidate {
    pub(super) fn selected_queries(
        &self,
    ) -> (
        Option<&FunctionFingerprintSelection>,
        Option<&XmlFingerprintSelection>,
    ) {
        (
            self.body.manifest.queries.functions.as_ref(),
            self.body.manifest.queries.xml.as_ref(),
        )
    }
    pub const fn native(&self) -> &PlatformIndexSourceParts {
        &self.body.native
    }
    pub const fn graph(&self) -> &GraphPartitionSnapshot {
        &self.body.snapshot
    }
    pub const fn reports(&self) -> &CoreRecognizerReports {
        &self.body.reports
    }
    pub const fn manifest(&self) -> &BlizzardUiIndexManifest {
        &self.body.manifest
    }
    pub fn function_fingerprints(
        &self,
    ) -> Option<&wow_project::graph::fingerprints::FunctionFingerprintManifest> {
        self.body.function_fingerprints.as_ref()
    }
    pub fn xml_fingerprints(&self) -> Option<&XmlFingerprintManifest> {
        self.body.xml_fingerprints.as_ref()
    }
}

#[derive(Serialize)]
struct CandidateIdentity<'a> {
    schema: &'static str,
    binding: &'a NativeBinding,
    source: &'a ProjectGraphProvenance,
    evidence_catalog_digest: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    membership: Option<&'a XmlClassificationMembership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml_source_maps: Option<&'a wow_project::graph::XmlSourceMapSummary>,
    graph: &'a GraphPartitionSnapshot,
    reports: &'a CoreRecognizerReports,
    queries: &'a SelectedQueries,
    #[serde(skip_serializing_if = "Option::is_none")]
    functions: Option<&'a wow_project::graph::fingerprints::FunctionFingerprintManifest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml: Option<&'a XmlFingerprintManifest>,
}

/// Fresh process-local capture; its fields cannot be supplied by a reader.
pub(super) struct NativeCandidateCapture {
    snapshot: GraphPartitionSnapshot,
    reports: CoreRecognizerReports,
    function_fingerprints: Option<wow_project::graph::fingerprints::FunctionFingerprintManifest>,
    xml_fingerprints: Option<XmlFingerprintManifest>,
    queries: SelectedQueries,
}
pub(super) struct PreparedNativeCandidate {
    snapshot: GraphPartitionSnapshot,
    reports: CoreRecognizerReports,
    function_fingerprints: Option<wow_project::graph::fingerprints::FunctionFingerprintManifest>,
    xml_fingerprints: Option<XmlFingerprintManifest>,
    manifest: BlizzardUiIndexManifest,
}

/// Execute the EXISTING helpers once from the exact held native predecessor.
/// This owns composition provenance where opaque producer bindings lack getters.
pub(super) fn capture_once(
    native: &PlatformGraphProvenance<'_>,
    addresses: &SourceGraphAddressCrosswalk<'_, '_>,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<NativeCandidateCapture> {
    checkpoint(stop)?;
    let caps = native
        .capabilities()
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    if native.project().configuration().platform_graph_profile()
        != Some(wow_project::PlatformGraphProfile::DirectCapabilitiesV1(
            caps,
        ))
        || request.capabilities != Some(caps)
        || addresses
            .direct_provenance()
            .is_none_or(|held| !std::ptr::eq(held, native))
    {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    // Copy only the bounded query selection after charging it; no request/input
    // source or whole ProjectView clone is introduced by this child owner.
    addresses.reserve(
        "candidate_queries",
        &(&request.function_fingerprints, &request.xml_fingerprints),
        stop,
    )?;
    let queries = SelectedQueries {
        functions: request.function_fingerprints.clone(),
        xml: request.xml_fingerprints.clone(),
    };
    let xml_fingerprints = queries
        .xml
        .as_ref()
        .map(|selection| selection.build(native, stop))
        .transpose()?;
    if let Some(manifest) = &xml_fingerprints {
        addresses.reserve("xml_fingerprints", manifest, stop)?;
    }
    if let Some(inclusion) = native.xml_classification_inclusion() {
        addresses.reserve(
            "xml_classification_membership",
            inclusion.membership(),
            stop,
        )?;
    }
    let (snapshot, call_recognition) = calls::publish(native.graph(), addresses, None, stop)
        .map_err(|failure| stage_error("calls", failure))?;
    let (snapshot, construction_recognition) = construction::publish(&snapshot, addresses, stop)
        .map_err(|failure| stage_error("construction", failure))?;
    let (snapshot, mixin_recognition, mixin_assignment_recognition) =
        lua_mixins::publish(&snapshot, addresses, stop)
            .map_err(|failure| stage_error("mixins", failure))?;
    let (snapshot, script_recognition) = scripts::publish_bound(&snapshot, addresses, stop)
        .map_err(|failure| stage_error("scripts", failure))?;
    let (snapshot, state_recognition) = state::publish_bound(&snapshot, addresses, stop)
        .map_err(|failure| stage_error("state", failure))?;
    let (
        snapshot,
        signal_recognition,
        bridge_recognition,
        custom_recognition,
        cvar_recognition,
        hook_recognition,
        library_recognition,
    ) = signals::publish_signals(&snapshot, addresses, stop)
        .map_err(|failure| stage_error("signals", failure))?;
    let (snapshot, toc_recognition) = toc::publish_bound(&snapshot, addresses, stop)
        .map_err(|failure| stage_error("toc", failure))?;
    let (snapshot, xml_recognition) = xml::publish_bound(&snapshot, addresses, stop)
        .map_err(|failure| stage_error("xml", failure))?;
    let (snapshot, state_root_recognition) =
        toc::publish_state_root_bound(&snapshot, addresses, stop)
            .map_err(|failure| stage_error("state-roots", failure))?;
    let (snapshot, state_core_recognition) =
        state_core::publish_bound(&snapshot, addresses, &state_recognition, stop)
            .map_err(|failure| stage_error("state-core", failure))?;
    let reports = CoreRecognizerReports {
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
        hook_recognition,
        library_recognition,
        toc_recognition,
        xml_recognition,
        state_root_recognition,
        state_core_recognition,
    };
    validate_closure(native, &snapshot, &reports, stop)?;
    // ONE selected query build after the final core owner. For a composed
    // neighborhood, build_composed retains the parent's same live query ledger.
    let function_fingerprints = queries
        .functions
        .as_ref()
        .map(|selection| {
            build_function_query(native, &snapshot, &reports, selection, addresses, stop)
        })
        .transpose()?;
    if let Some(manifest) = &function_fingerprints {
        addresses.reserve("function_fingerprints", manifest, stop)?;
    }
    checkpoint(stop)?;
    Ok(NativeCandidateCapture {
        snapshot,
        reports,
        function_fingerprints,
        xml_fingerprints,
        queries,
    })
}

fn build_function_query(
    native: &PlatformGraphProvenance<'_>,
    graph: &GraphPartitionSnapshot,
    reports: &CoreRecognizerReports,
    selection: &FunctionFingerprintSelection,
    addresses: &SourceGraphAddressCrosswalk<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<wow_project::graph::fingerprints::FunctionFingerprintManifest> {
    if selection.direct_neighborhood().is_none() {
        return selection.build(native, stop);
    }
    use wow_project::graph::fingerprints::{
        ComposedCallOutcome, ComposedCallReceipt, ComposedScriptReceipt,
        FunctionNeighborhoodComposition,
    };
    use wow_recognizers::source_calls::SourceCallOutcome;
    checkpoint(stop)?;
    let lookup = graph.producer_lookup(stop).map_err(graph_error)?;
    let calls = &reports.call_recognition;
    addresses.reserve("neighborhood_call_scratch", &calls.receipts(), stop)?;
    let mut call_receipts = Vec::new();
    for receipt in calls.receipts() {
        checkpoint(stop)?;
        let outcome = match &receipt.outcome {
            SourceCallOutcome::Projected { proposal_id, .. } => {
                ComposedCallOutcome::Projected { proposal_id }
            }
            SourceCallOutcome::LibraryTarget | SourceCallOutcome::ApiUseProjected { .. } => {
                ComposedCallOutcome::LibraryTarget
            }
            SourceCallOutcome::SelfRecursionUnsupported => {
                ComposedCallOutcome::SelfRecursionUnsupported
            }
            SourceCallOutcome::SignatureNotCaptured
            | SourceCallOutcome::Unresolved
            | SourceCallOutcome::Indeterminate => ComposedCallOutcome::Unknown,
        };
        call_receipts.push(ComposedCallReceipt {
            call_id: &receipt.call_id,
            outcome,
        });
    }
    let ScriptRecognition::Assertions(scripts) = &reports.script_recognition else {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    };
    addresses.reserve("neighborhood_script_scratch", &scripts.receipts(), stop)?;
    let mut script_receipts = Vec::new();
    for receipt in scripts.receipts() {
        checkpoint(stop)?;
        script_receipts.push(ComposedScriptReceipt {
            binding_id: &receipt.binding_id,
            proposal_id: &receipt.proposal_id,
        });
    }
    let composition = FunctionNeighborhoodComposition {
        final_lookup: &lookup,
        call_input_snapshot_id: calls.recognition().source_snapshot_id(),
        calls: &call_receipts,
        script_scope: scripts.scope(),
        scripts: &script_receipts,
    };
    selection.build_composed(native, &composition, stop)
}
impl NativeCandidateCapture {
    pub(super) const fn graph(&self) -> &GraphPartitionSnapshot {
        &self.snapshot
    }
    pub(super) const fn reports(&self) -> &CoreRecognizerReports {
        &self.reports
    }

    /// Call after final topology/export consumers, still on the same crosswalk.
    /// Existing reservations remain charged; no aggregate ledger is restarted.
    pub(super) fn prepare_seal(
        self,
        addresses: &SourceGraphAddressCrosswalk<'_, '_>,
        stop: &AtomicBool,
    ) -> ServiceResult<PreparedNativeCandidate> {
        checkpoint(stop)?;
        let native = addresses
            .direct_provenance()
            .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
        validate_closure(native, &self.snapshot, &self.reports, stop)?;
        let binding = NativeBinding::capture(native, addresses, stop)?;
        let inclusion = native.xml_classification_inclusion();
        let identity = CandidateIdentity {
            schema: CANDIDATE_SCHEMA,
            binding: &binding,
            source: native.source(),
            evidence_catalog_digest: native.evidence_catalog().digest(),
            membership: inclusion.as_ref().map(|value| value.membership()),
            xml_source_maps: native.xml_source_maps(),
            graph: &self.snapshot,
            reports: &self.reports,
            queries: &self.queries,
            functions: self.function_fingerprints.as_ref(),
            xml: self.xml_fingerprints.as_ref(),
        };
        addresses.reserve("candidate_identity", &identity, stop)?;
        // Existing bounded/cancellable canonical serializer, not a receipt parser
        // or reconstructed owner. Its bytes are transient scratch, not retained.
        let bytes = bounded_cancellable(&identity, MAX_BUNDLE_BYTES, stop)?;
        let candidate_digest = super::super::hash(&bytes);
        drop(bytes);
        let manifest = BlizzardUiIndexManifest {
            schema: CANDIDATE_SCHEMA,
            candidate_id: format!("blizzard-ui-index-candidate:{candidate_digest}").into(),
            candidate_digest,
            binding,
            final_graph_snapshot_id: self.snapshot.snapshot().snapshot_id().clone(),
            final_graph_generation: self.snapshot.snapshot().generation().clone(),
            coverage: wow_graph::GraphCoverageState::Partial,
            publication_state: CandidatePublicationState::NotPublishedE3A,
            function_fingerprints: ComponentSelection::for_presence(
                self.function_fingerprints.is_some(),
            ),
            xml_fingerprints: ComponentSelection::for_presence(self.xml_fingerprints.is_some()),
            queries: self.queries,
        };
        addresses.reserve("candidate_manifest", &manifest, stop)?;
        checkpoint(stop)?;
        Ok(PreparedNativeCandidate {
            snapshot: self.snapshot,
            reports: self.reports,
            function_fingerprints: self.function_fingerprints,
            xml_fingerprints: self.xml_fingerprints,
            manifest,
        })
    }
}
impl PreparedNativeCandidate {
    /// Consumes the original owned native proof after crosswalk borrows end.
    /// A failed/cancelled seal returns no capability or publication bundle.
    pub(super) fn seal(
        self,
        native: PlatformIndexSourceParts,
        stop: &AtomicBool,
    ) -> ServiceResult<BlizzardUiIndexCandidate> {
        checkpoint(stop)?;
        self.manifest.binding.validate_owned(&native, stop)?;
        // Compare every held native support/coverage component, not just compact
        // IDs. This second borrowed hash retains no encoded receipt/body copy.
        let inclusion = native.xml_classification_inclusion();
        let identity = CandidateIdentity {
            schema: CANDIDATE_SCHEMA,
            binding: &self.manifest.binding,
            source: native.source(),
            evidence_catalog_digest: native.evidence_catalog().digest(),
            membership: inclusion.as_ref().map(|value| value.membership()),
            xml_source_maps: native.xml_source_maps(),
            graph: &self.snapshot,
            reports: &self.reports,
            queries: &self.manifest.queries,
            functions: self.function_fingerprints.as_ref(),
            xml: self.xml_fingerprints.as_ref(),
        };
        let bytes = bounded_cancellable(&identity, MAX_BUNDLE_BYTES, stop)?;
        if super::super::hash(&bytes) != self.manifest.candidate_digest {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        drop(bytes);
        checkpoint(stop)?;
        Ok(BlizzardUiIndexCandidate {
            body: Box::new(CandidateBody {
                native,
                snapshot: self.snapshot,
                reports: self.reports,
                function_fingerprints: self.function_fingerprints,
                xml_fingerprints: self.xml_fingerprints,
                manifest: self.manifest,
            }),
        })
    }
}

#[derive(Serialize)]
struct NativeBinding {
    project_snapshot_id: Box<str>,
    project_generation: wow_core::ProjectGenerationId,
    analyzer_snapshot_id: Box<str>,
    configuration_digest: ContentDigest<CanonicalResult>,
    direct_profile: &'static str,
    capabilities: wow_project::PlatformGraphCapabilities,
    scope: GraphAssertionRecordScope,
    native_prefix_snapshot_id: GraphSnapshotId,
    native_prefix_generation: GraphGenerationId,
    foundation_snapshot_id: GraphSnapshotId,
    registry_digest: Box<str>,
    partitions: Vec<(Box<str>, Box<str>)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inventory_span_omissions: Option<usize>,
    evidence_catalog_digest: Box<str>,
}
struct PrefixPartitions<'a>(&'a [wow_graph::GraphProducerPartition]);
impl Serialize for PrefixPartitions<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(
            self.0
                .iter()
                .map(|partition| (partition.partition_id(), partition.partition_digest())),
        )
    }
}
impl NativeBinding {
    fn capture(
        native: &PlatformGraphProvenance<'_>,
        addresses: &SourceGraphAddressCrosswalk<'_, '_>,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        let project = native.project();
        addresses.reserve(
            "candidate_native_binding",
            &(
                project.snapshot_id(),
                project.project_generation(),
                project.analyzer_snapshot_id(),
                project.configuration().configuration_digest(),
                native.profile(),
                native.capabilities(),
                native.scope(),
                native.graph().snapshot().snapshot_id(),
                native.graph().snapshot().generation(),
                native.graph().foundation().snapshot_id(),
                native.graph().registry().registry_digest(),
                PrefixPartitions(native.graph().partitions()),
                native.inventory_span_omissions(),
                native.evidence_catalog().digest(),
            ),
            stop,
        )?;
        let mut partitions = Vec::with_capacity(native.graph().partitions().len());
        for partition in native.graph().partitions() {
            checkpoint(stop)?;
            partitions.push((
                partition.partition_id().into(),
                partition.partition_digest().into(),
            ));
        }
        checkpoint(stop)?;
        Ok(Self {
            project_snapshot_id: project.snapshot_id().into(),
            project_generation: project.project_generation(),
            analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
            configuration_digest: project.configuration().configuration_digest(),
            direct_profile: native.profile(),
            capabilities: native
                .capabilities()
                .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?,
            scope: native.scope().clone(),
            native_prefix_snapshot_id: native.graph().snapshot().snapshot_id().clone(),
            native_prefix_generation: native.graph().snapshot().generation().clone(),
            foundation_snapshot_id: native.graph().foundation().snapshot_id().clone(),
            registry_digest: native.graph().registry().registry_digest().into(),
            partitions,
            inventory_span_omissions: native.inventory_span_omissions(),
            evidence_catalog_digest: native.evidence_catalog().digest().into(),
        })
    }
    fn validate_owned(
        &self,
        native: &PlatformIndexSourceParts,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        let project = native.project();
        if project.snapshot_id() != self.project_snapshot_id.as_ref()
            || project.project_generation() != self.project_generation
            || project.analyzer_snapshot_id() != self.analyzer_snapshot_id.as_ref()
            || project.configuration().configuration_digest() != self.configuration_digest
            || native.profile() != self.direct_profile
            || native.capabilities() != Some(self.capabilities)
            || native.scope() != &self.scope
            || native.native_prefix_snapshot_id() != &self.native_prefix_snapshot_id
            || native.native_prefix_generation() != &self.native_prefix_generation
            || native.foundation_snapshot_id() != &self.foundation_snapshot_id
            || native.registry_digest() != self.registry_digest.as_ref()
            || native.inventory_span_omissions() != self.inventory_span_omissions
            || native.evidence_catalog().digest() != self.evidence_catalog_digest.as_ref()
        {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        let mut actual = native.native_prefix_partitions();
        if actual.len() != self.partitions.len() {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        for (id, digest) in &self.partitions {
            checkpoint(stop)?;
            if actual.next() != Some((id.as_ref(), digest.as_ref())) {
                return Err(error(ServiceErrorCode::IdentityMismatch));
            }
        }
        checkpoint(stop)
    }
}

fn validate_closure(
    native: &PlatformGraphProvenance<'_>,
    graph: &GraphPartitionSnapshot,
    reports: &CoreRecognizerReports,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    checkpoint(stop)?;
    if graph.registry() != native.graph().registry()
        || graph.foundation() != native.graph().foundation()
        || graph.source_context_id() != native.scope().source_context_id
        || native
            .graph()
            .partitions()
            .len()
            .checked_add(core_ids().count())
            != Some(graph.partitions().len())
    {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    for partition in native.graph().partitions() {
        checkpoint(stop)?;
        if graph.partition(partition.partition_id()) != Some(partition) {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
    }
    for id in core_ids() {
        checkpoint(stop)?;
        if graph.partition(id).is_none() {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
    }
    for partition in graph.partitions() {
        checkpoint(stop)?;
        if native.graph().partition(partition.partition_id()).is_none()
            && !core_ids().any(|id| id == partition.partition_id())
        {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
    }
    let ScriptRecognition::Assertions(scripts) = &reports.script_recognition else {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    };
    let StateRecognition::Assertions(state) = &reports.state_recognition else {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    };
    if scripts.scope() != native.scope()
        || state.scope() != native.scope()
        || reports.state_root_recognition.family != SourceTocFamily::SavedVariableRoot
        || reports.toc_recognition.len() != SourceTocFamily::ALL.len()
        || reports.xml_recognition.len() != SourceXmlFamily::ALL.len()
        || reports.state_core_recognition.len() != SourceStateCoreFamily::ALL.len()
    {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    for (report, family) in reports.toc_recognition.iter().zip(SourceTocFamily::ALL) {
        checkpoint(stop)?;
        if report.family != family {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
    }
    for (report, family) in reports.xml_recognition.iter().zip(SourceXmlFamily::ALL) {
        checkpoint(stop)?;
        if report.family != family {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
    }
    let function_report = native
        .source()
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    let admission = graph
        .partition(SOURCE_STATE_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    for (report, family) in reports
        .state_core_recognition
        .iter()
        .zip(SourceStateCoreFamily::ALL)
    {
        checkpoint(stop)?;
        if report.family != family
            || report.analyzer_report_id != function_report.analysis_id()
            || report.admission_partition_digest != admission.partition_digest()
        {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
    }
    checkpoint(stop)
}
