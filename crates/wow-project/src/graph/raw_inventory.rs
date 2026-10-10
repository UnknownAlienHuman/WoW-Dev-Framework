//! Exact admitted byte membership, separate from load and analyzer membership.
use super::*;
use crate::platform_source::{
    PlatformAdmissionCoverage, PlatformFileKind, PlatformLicenseRecord, PlatformRawMember,
    PlatformSourceInventory,
};
use std::{io::Write, ops::Range};
use wow_core::{CanonicalResult, ContentDigest, SourceContent};
use wow_graph::{GraphAssertionRef, GraphPartitionSnapshot};

pub const PLATFORM_RAW_INVENTORY_PARTITION: &str = "wow-project.platform-source-inventory";
pub const PLATFORM_RAW_MEMBER_KIND: &str = "platform_raw_member";
const MAX_MEMBERS: usize = 4096;
const MAX_LOCAL_READ_BYTES: usize = 64 * 1024;

/// Serialize-only metadata. It does not construct source bytes or a native graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectRawInventoryMember {
    pub path: String,
    pub kind: PlatformFileKind,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
    pub proposal_id: String,
    pub source_handle: SourceHandle,
    pub evidence: EvidenceRecord,
}

/// Original admission assertions remain distinct from observed Included bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRawInventoryManifest {
    schema: &'static str,
    source_snapshot_id: String,
    profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    coverage: PlatformAdmissionCoverage,
    inventory: PlatformSourceInventory,
    members: Vec<ProjectRawInventoryMember>,
}

#[derive(Serialize)]
struct ManifestMetadata<'a> {
    schema: &'static str,
    source_snapshot_id: &'a str,
    profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    coverage: &'a PlatformAdmissionCoverage,
    inventory: &'a PlatformSourceInventory,
    members: &'a [ProjectRawInventoryMember],
}

impl Serialize for ProjectRawInventoryManifest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ManifestMetadata {
            schema: self.schema,
            source_snapshot_id: &self.source_snapshot_id,
            profile_digest: self.profile_digest,
            content_manifest_digest: self.content_manifest_digest,
            admission_digest: self.admission_digest,
            coverage: &self.coverage,
            inventory: &self.inventory,
            members: &self.members,
        }
        .serialize(serializer)
    }
}
impl ProjectRawInventoryManifest {
    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        &self.source_snapshot_id
    }
    #[must_use]
    pub fn inventory(&self) -> &PlatformSourceInventory {
        &self.inventory
    }
    #[must_use]
    pub fn members(&self) -> &[ProjectRawInventoryMember] {
        &self.members
    }
    #[must_use]
    pub const fn admission_digest(&self) -> ContentDigest<CanonicalResult> {
        self.admission_digest
    }
    #[must_use]
    pub const fn profile_digest(&self) -> ContentDigest<CanonicalResult> {
        self.profile_digest
    }
    #[must_use]
    pub const fn content_manifest_digest(&self) -> ContentDigest<CanonicalResult> {
        self.content_manifest_digest
    }
    #[must_use]
    pub const fn coverage(&self) -> &PlatformAdmissionCoverage {
        &self.coverage
    }
}

pub(super) fn selected(config: &crate::ProjectConfiguration) -> bool {
    matches!(
        config.platform_graph_profile(),
        Some(
            crate::PlatformGraphProfile::PackageProjectionWithRawInventoryV1
                | crate::PlatformGraphProfile::DirectPlatformProducersWithRawInventoryV1
        )
    )
}

pub(super) fn extend_registry(entities: &mut Vec<GraphEntityKindDefinition>) -> ProjectResult<()> {
    entities.push(
        GraphEntityKindDefinition::new(
            PLATFORM_RAW_MEMBER_KIND,
            vec!["blizzard_ui_source".into()],
            vec![
                "source_snapshot".into(),
                "path".into(),
                "kind".into(),
                "content_digest".into(),
                "byte_length".into(),
            ],
            vec![GraphConfidence::Proven],
        )
        .map_err(|_| invalid())?,
    );
    Ok(())
}

pub(super) fn project(
    project: &ProjectView,
    registry: &GraphRegistryBundle,
    universe: &GraphUniverseId,
    generation: &GraphGenerationId,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<(GraphProposalBatch, ProjectRawInventoryManifest)> {
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    if !selected(config) {
        return Err(invalid());
    }
    let packages = config.platform_packages().ok_or_else(invalid)?;
    let source = packages.source();
    let receipt = source.receipt();
    if universe.as_str() != packages.binding().universe_id()
        || *generation != input_generation(project, registry)?
        || receipt.coverage().verified_files() > MAX_MEMBERS
    {
        return Err(if receipt.coverage().verified_files() > MAX_MEMBERS {
            exhausted()
        } else {
            invalid()
        });
    }
    // Count the complete borrowed envelope before retaining its metadata copy.
    let metadata = ManifestMetadata {
        schema: "wow-project/platform-raw-inventory-graph/1",
        source_snapshot_id: receipt.source_snapshot_id(),
        profile_digest: receipt.profile_digest(),
        content_manifest_digest: receipt.content_manifest_digest(),
        admission_digest: receipt.admission_digest(),
        coverage: receipt.coverage(),
        inventory: receipt.inventory(),
        members: &[],
    };
    charge_serialized(text_bytes, &metadata, stop)?;
    let context = project.snapshot().generation_context();
    // Batch IDs have a fixed SHA-256 encoding; an empty batch has the exact
    // envelope length of the final batch. Member bodies and commas follow below.
    let empty_batch = GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe.clone(),
        generation.clone(),
        context.context_id(),
        PLATFORM_RAW_INVENTORY_PARTITION,
        Vec::new(),
        Vec::new(),
    )
    .map_err(|_| invalid())?;
    charge_serialized(text_bytes, &empty_batch, stop)?;
    let (origin, revision) =
        crate::registry::source_handle_identity(config, project.project_generation())?;
    let mut cursor = source.raw_inventory(stop)?;
    let mut members = Vec::new();
    let mut entities = Vec::new();
    while let Some(member) = cursor.next(stop)? {
        crate::analyzer::checkpoint(stop)?;
        if members.len() >= MAX_MEMBERS {
            return Err(exhausted());
        }
        let digest = crate::identity::canonical_digest(
            "wow-project/platform-raw-member/1",
            &(receipt.source_snapshot_id(), member.path()),
            ProjectPhase::View,
        )?;
        let proposal_id = format!("raw-member:{digest}");
        // No Main file lookup or LoadSource is fabricated for this member.
        let handle = SourceHandleBuilder::new(
            origin,
            config.source_origin_id().as_str(),
            revision.as_ref(),
            member.path(),
            SourceSpan::whole_file(),
            member.content_digest(),
        )
        .reference_generation(config.reference_generation())
        .project_generation(project.project_generation())
        .build()
        .map_err(|_| invalid())?;
        let evidence = EvidenceRecord::new(
            context.context_id(),
            ProvenanceClass::ProjectSource,
            EvidenceConfidence::Proven,
            ClaimScope::SourceObservation,
            "wow.project".parse::<ProducerId>().map_err(|_| invalid())?,
            ToolVersion::parse(env!("CARGO_PKG_VERSION")).map_err(|_| invalid())?,
            vec![handle.handle_id()],
            Vec::new(),
            Vec::new(),
        )
        .map_err(|_| invalid())?;
        let record = ProjectRawInventoryMember {
            path: member.path().to_owned(),
            kind: member.kind(),
            content_digest: member.content_digest(),
            byte_length: member.byte_length(),
            proposal_id: proposal_id.clone(),
            source_handle: handle,
            evidence,
        };
        let kind = match member.kind() {
            PlatformFileKind::Lua => "lua",
            PlatformFileKind::Toc => "toc",
            PlatformFileKind::Xml => "xml",
            PlatformFileKind::Schema => "schema",
            PlatformFileKind::Unknown => "unknown",
        };
        let entity = GraphEntityProposal::new(
            proposal_id,
            PLATFORM_RAW_MEMBER_KIND,
            BTreeMap::from([
                (
                    "source_snapshot".into(),
                    GraphProposalValue::String(receipt.source_snapshot_id().into()),
                ),
                (
                    "path".into(),
                    GraphProposalValue::String(record.path.clone().into()),
                ),
                ("kind".into(), GraphProposalValue::Identifier(kind.into())),
                (
                    "content_digest".into(),
                    GraphProposalValue::String(record.content_digest.to_string().into()),
                ),
                (
                    "byte_length".into(),
                    GraphProposalValue::Integer(
                        i64::try_from(record.byte_length).map_err(|_| invalid())?,
                    ),
                ),
            ]),
            GraphConfidence::Proven,
            vec![record.source_handle.handle_id()],
            vec![record.evidence.evidence_id()],
            Vec::new(),
        )
        .map_err(|_| invalid())?;
        charge_serialized(text_bytes, &record, stop)?;
        charge_serialized(text_bytes, &entity, stop)?;
        if !members.is_empty() {
            // One separator in each of the manifest and batch member arrays.
            charge(text_bytes, 2)?;
        }
        members.push(record);
        entities.push(entity);
    }
    // Successful exhaustion is required; the native cursor checks file/byte closure.
    let manifest = ProjectRawInventoryManifest {
        schema: metadata.schema,
        source_snapshot_id: receipt.source_snapshot_id().to_owned(),
        profile_digest: receipt.profile_digest(),
        content_manifest_digest: receipt.content_manifest_digest(),
        admission_digest: receipt.admission_digest(),
        coverage: receipt.coverage().clone(),
        inventory: receipt.inventory().clone(),
        members,
    };
    let batch = GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe.clone(),
        generation.clone(),
        context.context_id(),
        PLATFORM_RAW_INVENTORY_PARTITION,
        entities,
        Vec::new(),
    )
    .map_err(|_| invalid())?;
    crate::analyzer::checkpoint(stop)?;
    Ok((batch, manifest))
}

pub(super) fn charge_serialized<T: Serialize>(
    used: &mut usize,
    value: &T,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    struct Counter<'a> {
        bytes: usize,
        limit: usize,
        exceeded: bool,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.stop.load(std::sync::atomic::Ordering::Acquire) {
                return Err(std::io::Error::other("cancelled"));
            }
            let next = self
                .bytes
                .checked_add(bytes.len())
                .filter(|count| *count <= self.limit);
            let Some(next) = next else {
                self.exceeded = true;
                return Err(std::io::Error::other("metadata budget"));
            };
            self.bytes = next;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    crate::analyzer::checkpoint(stop)?;
    let mut counter = Counter {
        bytes: 0,
        limit: MAX_TEXT_BYTES.checked_sub(*used).ok_or_else(exhausted)?,
        exceeded: false,
        stop,
    };
    let result = serde_json::to_writer(&mut counter, value);
    crate::analyzer::checkpoint(stop)?;
    if result.is_err() {
        return Err(if counter.exceeded {
            exhausted()
        } else {
            invalid()
        });
    }
    charge(used, counter.bytes)
}

/// Native byte capability tied to the held immutable project source. It cannot
/// be decoded from metadata. Byte ranges never imply UTF-8 or analyzer spans.
pub struct ProjectRawMemberReadBinding<'a> {
    member: PlatformRawMember<'a>,
    source_handle: SourceHandle,
    source_snapshot_id: &'a str,
    admission_digest: ContentDigest<CanonicalResult>,
    license: &'a PlatformLicenseRecord,
    reference: GraphAssertionRef,
}
impl ProjectRawMemberReadBinding<'_> {
    #[must_use]
    pub fn path(&self) -> &str {
        self.member.path()
    }
    #[must_use]
    pub fn kind(&self) -> PlatformFileKind {
        self.member.kind()
    }
    #[must_use]
    pub fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.member.content_digest()
    }
    #[must_use]
    pub fn byte_length(&self) -> u64 {
        self.member.byte_length()
    }
    #[must_use]
    pub fn source_handle(&self) -> &SourceHandle {
        &self.source_handle
    }
    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        self.source_snapshot_id
    }
    #[must_use]
    pub const fn admission_digest(&self) -> ContentDigest<CanonicalResult> {
        self.admission_digest
    }
    #[must_use]
    pub const fn license(&self) -> &PlatformLicenseRecord {
        self.license
    }
    #[must_use]
    pub const fn reference(&self) -> &GraphAssertionRef {
        &self.reference
    }
    pub fn read_bytes(
        &self,
        range: Range<u64>,
        max_bytes: usize,
        stop: &AtomicBool,
    ) -> ProjectResult<&[u8]> {
        crate::analyzer::checkpoint(stop)?;
        if !(1..=MAX_LOCAL_READ_BYTES).contains(&max_bytes) {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::View,
                "invalid raw read byte limit",
            ));
        }
        let length = range.end.checked_sub(range.start).ok_or_else(invalid)?;
        if range.end > self.member.byte_length() {
            return Err(invalid());
        }
        if length > max_bytes as u64 {
            return Err(exhausted());
        }
        let start = usize::try_from(range.start).map_err(|_| invalid())?;
        let end = usize::try_from(range.end).map_err(|_| invalid())?;
        let bytes = self.member.bytes().get(start..end).ok_or_else(invalid)?;
        crate::analyzer::checkpoint(stop)?;
        Ok(bytes)
    }
}

fn expected(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<(GraphProposalBatch, ProjectRawInventoryManifest)> {
    project.snapshot().validate()?;
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    if !selected(config) {
        return Err(ProjectError::new(
            ProjectErrorCode::DeferredCapability,
            ProjectPhase::View,
            "native raw graph selection is required for raw byte binding",
        ));
    }
    let registry = registry(config.project_kind(), true)?;
    if graph.registry() != &registry
        || graph.source_context_id() != project.snapshot().generation_context().context_id()
    {
        return Err(invalid());
    }
    let packages = config.platform_packages().ok_or_else(invalid)?;
    let universe = GraphUniverseId::new(packages.binding().universe_id()).map_err(|_| invalid())?;
    let generation = input_generation(project, &registry)?;
    self::project(project, &registry, &universe, &generation, &mut 0, stop)
}

pub fn bind_platform_raw_member<'a>(
    project: &'a ProjectView,
    graph: &GraphPartitionSnapshot,
    reference: &GraphAssertionRef,
    stop: &AtomicBool,
) -> ProjectResult<ProjectRawMemberReadBinding<'a>> {
    crate::analyzer::checkpoint(stop)?;
    graph.validate(stop).map_err(graph_error)?;
    let (batch, manifest) = expected(project, graph, stop)?;
    let partition = graph
        .partition(PLATFORM_RAW_INVENTORY_PARTITION)
        .ok_or_else(invalid)?;
    if partition.batch() != &batch
        || !partition.coverage().is_empty()
        || partition.report().accepted_entities().len() != manifest.members.len()
    {
        return Err(invalid());
    }
    let lookup = graph.producer_lookup(stop).map_err(graph_error)?;
    let resolved = lookup
        .entity(lookup.scope(), reference, stop)
        .map_err(graph_error)?;
    if resolved.partition().partition_id() != PLATFORM_RAW_INVENTORY_PARTITION
        || resolved.proposal().entity_kind_id() != PLATFORM_RAW_MEMBER_KIND
    {
        return Err(invalid());
    }
    let record = manifest
        .members
        .into_iter()
        .find(|member| member.proposal_id == resolved.proposal().proposal_id())
        .ok_or_else(invalid)?;
    let source = project
        .configuration()
        .platform_packages()
        .ok_or_else(invalid)?
        .source();
    let member = source.raw_member(&record.path, stop)?;
    if member.kind() != record.kind
        || member.content_digest() != record.content_digest
        || member.byte_length() != record.byte_length
    {
        return Err(invalid());
    }
    Ok(ProjectRawMemberReadBinding {
        member,
        source_handle: record.source_handle,
        source_snapshot_id: source.receipt().source_snapshot_id(),
        admission_digest: source.receipt().admission_digest(),
        license: &source.receipt().inventory().license,
        reference: resolved.reference(),
    })
}

fn graph_error(error: wow_graph::GraphError) -> ProjectError {
    ProjectError::new(
        match error.code() {
            wow_graph::GraphErrorCode::Cancelled => ProjectErrorCode::AnalysisCancelled,
            wow_graph::GraphErrorCode::BudgetExceeded => ProjectErrorCode::SourceBudgetExceeded,
            _ => ProjectErrorCode::SnapshotInvalid,
        },
        ProjectPhase::View,
        "native raw inventory graph validation failed",
    )
}
