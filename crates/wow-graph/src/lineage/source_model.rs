//! Owner-neutral exact Source provenance. These are data, never admitted source
//! bytes or a deserializable native owner/candidate/read capability.
use super::*;
use serde::{Deserialize, Serialize};
use wow_core::{
    CanonicalResult, ContentDigest, CoverageStatus, ProducerId, ProfileId, ProfileIdentity,
    ReferenceGenerationId, SourceContent, ToolVersion,
};
pub const SOURCE_SPAN_LINEAGE_PROFILE: &str =
    "wow-graph/lineage/blizzard-ui-source-span-candidates/1";
pub const SOURCE_SPAN_INPUT_SCHEMA: &str = "wow-graph/lineage-source-span-input/1";
pub const SOURCE_SPAN_PRODUCER: &str = "wow.project.lineage.blizzard_ui_source_fingerprint";
pub const SOURCE_SPAN_COMPARISON_POLICY: &str = "wow-project/blizzard-ui-source-span-comparison/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageSourceClass {
    VendorUiSourceMirror,
    SyntheticFixture,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LineageSourceRevision {
    Git {
        object_format: Box<str>,
        commit: Box<str>,
        tree: Box<str>,
    },
    Archive {
        digest: ContentDigest<SourceContent>,
    },
    Fixture {
        digest: ContentDigest<SourceContent>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceOrigin {
    pub provider: Box<str>,
    pub repository: Box<str>,
    pub revision: LineageSourceRevision,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceMaterializer {
    pub producer: ProducerId,
    pub version: ToolVersion,
    pub configuration_digest: ContentDigest<CanonicalResult>,
    pub report_digest: ContentDigest<SourceContent>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageSourceLicenseState {
    Known,
    Unknown,
    Conflict,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceLicense {
    pub state: LineageSourceLicenseState,
    pub attribution: Box<str>,
    pub evidence_digest: ContentDigest<SourceContent>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageSourceUnevaluated {
    RootCompleteness,
    GitMembership,
    MaterializerSecurity,
    ClientCompatibility,
    LicensePermission,
    Decoding,
    PackageLoad,
    Analyzer,
    Graph,
    ApiContract,
    Runtime,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceAdmissionCoverage {
    pub inventory: CoverageStatus,
    pub declared_included_bytes: CoverageStatus,
    pub verified_files: usize,
    pub verified_bytes: u64,
    pub unevaluated: Vec<LineageSourceUnevaluated>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceRoot {
    pub root: Box<str>,
    pub selected_tocs: Vec<Box<str>>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageSourceInventoryScope {
    DeclaredComplete,
    DeclaredPartial,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceRootInventory {
    pub root: Box<str>,
    pub declared_entries: u64,
    pub scope: LineageSourceInventoryScope,
    pub evidence_digest: ContentDigest<SourceContent>,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceOmissions {
    pub excluded: usize,
    pub unsupported: usize,
    pub external: usize,
    pub conflict: usize,
    pub failed: usize,
    pub unknown_kind: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceGenerationMetadata {
    pub source_profile_id: ProfileId,
    pub source_profile_digest: ContentDigest<CanonicalResult>,
    pub source_snapshot_id: Box<str>,
    pub content_manifest_digest: ContentDigest<CanonicalResult>,
    pub admission_digest: ContentDigest<CanonicalResult>,
    pub source_class: LineageSourceClass,
    pub product: Box<str>,
    pub channel: Box<str>,
    pub target_reference_profile: ProfileIdentity,
    pub target_reference_generation: ReferenceGenerationId,
    pub origin: LineageSourceOrigin,
    pub materializer: LineageSourceMaterializer,
    pub license: LineageSourceLicense,
    pub admission_coverage: LineageSourceAdmissionCoverage,
    pub roots: Vec<LineageSourceRoot>,
    pub exclusions: Vec<Box<str>>,
    pub root_inventory: Vec<LineageSourceRootInventory>,
    pub omissions: LineageSourceOmissions,
    pub inventory_entries: usize,
    pub compatibility_evidence: ContentDigest<SourceContent>,
    pub native_profile_id: Box<str>,
    pub native_prefix_snapshot_id: crate::GraphSnapshotId,
    pub native_prefix_generation: crate::GraphGenerationId,
    pub foundation_snapshot_id: crate::GraphSnapshotId,
    pub native_registry_digest: Box<str>,
    pub source_universe: crate::GraphUniverseId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSourceComparisonBinding {
    pub namespace_id: Box<str>,
    pub logical_namespace: Box<str>,
    pub owner_project_id: Box<str>,
    pub compatibility_profile_id: Box<str>,
    pub before: LineageSourceGenerationMetadata,
    pub after: LineageSourceGenerationMetadata,
}

/// Fixed same-product/channel/flavor source-span comparison. It deliberately
/// does not interpret target build order, API compatibility or source authority.
pub(super) fn validate_source_binding(binding: &LineageComparisonBinding) -> GraphResult<()> {
    let source = binding.source.as_ref().ok_or_else(invalid)?;
    let (a, b) = (&source.before, &source.after);
    let namespace = wow_store::project::ProjectStoreNamespace::new(
        wow_store::project::ProjectStoreNamespaceRequest {
            logical_namespace: source.logical_namespace.to_string(),
            owner_project_id: source.owner_project_id.to_string(),
        },
    )?;
    if binding.universe_class != LineageUniverseClass::BlizzardUiSource
        || binding.profile_id.as_ref() != SOURCE_SPAN_LINEAGE_PROFILE
        || source.compatibility_profile_id.as_ref() != SOURCE_SPAN_COMPARISON_POLICY
        || source.namespace_id.as_ref() != namespace.id().as_str()
        || source.owner_project_id != binding.before.project_id
        || source.owner_project_id != binding.after.project_id
        || source.logical_namespace.as_ref() != a.source_profile_id.as_str()
        || a.source_profile_id != b.source_profile_id
        || a.source_class != b.source_class
        || a.product != b.product
        || a.channel != b.channel
        || a.target_reference_profile.flavor_id() != b.target_reference_profile.flavor_id()
        || a.target_reference_profile.edition_id() != b.target_reference_profile.edition_id()
        || a.native_profile_id != b.native_profile_id
        || a.native_registry_digest != b.native_registry_digest
        || binding.before.project_kind.as_ref() != "blizzard_ui_source"
        || binding.after.project_kind.as_ref() != "blizzard_ui_source"
        || binding.before.project_generation_schema_version
            != binding.after.project_generation_schema_version
        || binding.before.analyzer_pin_id != binding.after.analyzer_pin_id
    {
        return Err(invalid());
    }
    for (metadata, generation) in [(a, &binding.before), (b, &binding.after)] {
        if metadata.target_reference_profile != generation.selected_profile
            || metadata.target_reference_generation.to_string().as_str()
                != generation.reference_generation_id.as_ref()
            || metadata.source_universe != generation.original_graph_scope.universe
            || metadata.source_snapshot_id.is_empty()
            || metadata.source_snapshot_id.len() > 256
            || metadata.roots.len() > 64
            || metadata.root_inventory.len() > 64
            || metadata.exclusions.len() > 1024
            || metadata.inventory_entries > 200_000
            || metadata.admission_coverage.unevaluated.len() > 16
            || !matches!(
                (
                    metadata.source_class,
                    metadata.target_reference_profile.profile_kind()
                ),
                (
                    LineageSourceClass::SyntheticFixture,
                    wow_core::ProfileKind::Fixture
                ) | (
                    LineageSourceClass::VendorUiSourceMirror,
                    wow_core::ProfileKind::Release
                )
            )
        {
            return Err(invalid());
        }
        for root in &metadata.roots {
            if root.selected_tocs.len() > 1024 {
                return Err(invalid());
            }
            canonical_path(&root.root)?;
            for toc in &root.selected_tocs {
                canonical_path(toc)?;
            }
        }
        for root in &metadata.root_inventory {
            canonical_path(&root.root)?;
        }
        for excluded in &metadata.exclusions {
            canonical_path(excluded)?;
        }
    }
    Ok(())
}
fn canonical_path(path: &str) -> GraphResult<()> {
    let parsed = wow_core::NormalizedSourcePath::parse(path).map_err(|_| invalid())?;
    if parsed.value().as_str() != path {
        return Err(invalid());
    }
    Ok(())
}

pub(super) fn selected_input_profile(
    data: &ProjectFingerprintPartition,
) -> GraphResult<(&'static str, &'static str, &'static str)> {
    match data.binding.universe_class {
        LineageUniverseClass::UserProject
            if data.binding.source.is_none() && data.binding.reference.is_none() =>
        {
            Ok((
                PROJECT_SPAN_LINEAGE_PROFILE,
                PROJECT_SPAN_INPUT_SCHEMA,
                PROJECT_SPAN_PRODUCER,
            ))
        }
        LineageUniverseClass::BlizzardUiSource if data.binding.reference.is_none() => {
            validate_source_binding(&data.binding)?;
            Ok((
                SOURCE_SPAN_LINEAGE_PROFILE,
                SOURCE_SPAN_INPUT_SCHEMA,
                SOURCE_SPAN_PRODUCER,
            ))
        }
        LineageUniverseClass::ReferencePlatformContract => {
            super::reference_model::validate_binding(data)?;
            Ok((
                REFERENCE_KEY_LINEAGE_PROFILE,
                REFERENCE_KEY_INPUT_SCHEMA,
                REFERENCE_KEY_PRODUCER,
            ))
        }
        _ => Err(invalid()),
    }
}
