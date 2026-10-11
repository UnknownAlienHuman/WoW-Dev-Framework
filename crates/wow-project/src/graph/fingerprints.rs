//! First physical-function recipe: exact source bytes, never lineage or shape.
//! Construction borrows native authority; owned receipts cannot recreate it.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, DigestPurpose, EvidenceId, ProfileIdentity,
    ProjectGenerationId, ReferenceGenerationId, SourceContent, SourceSpan, StableHandleId,
};
use wow_emmy::function_calls::SourceFunctionKind;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence,
    GraphConflictRecord, GraphCoverageRecord, GraphGenerationId, GraphLocalAssertion, GraphNodeId,
    GraphSnapshotId,
};

use super::{PlatformGraphProvenance, producer_budget::ProducerBudget};
use crate::{PlatformGraphProfile, ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

mod declaration;
mod load_role;
pub use load_role::{
    FUNCTION_PACKAGE_LOAD_ROLE_RECIPE, FunctionPackageLoadRole, FunctionPackageLoadRoleComparable,
    FunctionPackageLoadRoleComponent, FunctionPackageLoadRoleCoverage,
    FunctionPackageLoadRoleFingerprint, FunctionPackageLoadRoleRecipe,
    FunctionPackageLoadRoleSelection, FunctionPackageLoadRoleUnavailable,
    FunctionPackageLoadRoleWitness, FunctionPackageLoadUnitRole,
};
mod shape;
pub use declaration::{
    FunctionDeclaredHeader, FunctionDeclaredHeaderComponent, FunctionDeclaredHeaderFingerprint,
    FunctionDeclaredHeaderProfile,
};
pub use shape::{
    DECLARATION_BODY_SHAPE_RECIPE, FunctionDeclarationShapeComponent,
    FunctionDeclarationShapeFingerprint,
};

pub const FUNCTION_FINGERPRINT_SCHEMA: &str = "wow-project/function-fingerprint-records/1";
pub const FUNCTION_FINGERPRINT_PROFILE: &str = "wow-project/physical-function-source-span/1";
pub const DECLARED_HEADER_FINGERPRINT_SCHEMA: &str =
    "wow-project/function-fingerprint-records/declared-header/1";
pub const DECLARED_HEADER_FINGERPRINT_PROFILE: &str =
    "wow-project/physical-function-components/declared-header/1";
const SOURCE_SPAN_DOMAIN: &str = "wow.project.fingerprint.physical-function.source-span.v1";
pub const BODY_SHAPE_FINGERPRINT_SCHEMA: &str =
    "wow-project/function-fingerprint-records/declared-syntax-and-body-shape/1";
pub const BODY_SHAPE_FINGERPRINT_PROFILE: &str =
    "wow-project/function-fingerprint/declaration-body-shape/1";
pub const SELECTED_COMPONENTS_FINGERPRINT_SCHEMA: &str =
    "wow-project/function-fingerprint-records/selected-components/1";
pub const SELECTED_COMPONENTS_FINGERPRINT_PROFILE: &str =
    "wow-project/physical-function-components/selected/1";

/// Local purpose: comparison component, not source/generation or manifest identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSourceSpanFingerprint;
impl DigestPurpose for FunctionSourceSpanFingerprint {
    const NAME: &'static str = "function_source_span_fingerprint";
}

/// No default or wire input is introduced by this leaf. Root selects finite limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FunctionFingerprintLimits {
    max_records: usize,
    max_source_bytes: usize,
    max_metadata_bytes: usize,
}
impl FunctionFingerprintLimits {
    pub fn new(
        max_records: usize,
        max_source_bytes: usize,
        max_metadata_bytes: usize,
    ) -> ProjectResult<Self> {
        if max_records == 0 || max_source_bytes == 0 || max_metadata_bytes == 0 {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Configuration,
                "function fingerprint limits must be positive",
            ));
        }
        Ok(Self {
            max_records,
            max_source_bytes,
            max_metadata_bytes,
        })
    }
    #[must_use]
    pub const fn max_records(self) -> usize {
        self.max_records
    }
    #[must_use]
    pub const fn max_source_bytes(self) -> usize {
        self.max_source_bytes
    }
    #[must_use]
    pub const fn max_metadata_bytes(self) -> usize {
        self.max_metadata_bytes
    }
}

/// Exact rooted key. Equality includes scope and node identity; it implies no continuity.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ExactFunctionOwnerKey {
    scope: GraphAssertionRecordScope,
    node_id: GraphNodeId,
    owner_key: Box<str>,
}
impl ExactFunctionOwnerKey {
    #[must_use]
    pub const fn scope(&self) -> &GraphAssertionRecordScope {
        &self.scope
    }
    #[must_use]
    pub const fn node_id(&self) -> &GraphNodeId {
        &self.node_id
    }
    #[must_use]
    pub fn owner_key(&self) -> &str {
        &self.owner_key
    }
}

/// Binding receipt only: none of these fields enters the comparable span digest.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FunctionFingerprintBinding {
    project_snapshot_id: Box<str>,
    project_generation: ProjectGenerationId,
    analyzer_snapshot_id: Box<str>,
    selected_profile: ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    configuration_digest: ContentDigest<CanonicalResult>,
    source_snapshot_id: Box<str>,
    source_profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    package_binding_digest: ContentDigest<CanonicalResult>,
    source_admission_coverage: crate::platform_source::PlatformAdmissionCoverage,
    direct_profile: &'static str,
    graph_scope: GraphAssertionRecordScope,
    published_graph_snapshot_id: GraphSnapshotId,
    published_graph_generation: GraphGenerationId,
    function_analysis_id: Box<str>,
}
impl FunctionFingerprintBinding {
    #[must_use]
    pub fn project_snapshot_id(&self) -> &str {
        &self.project_snapshot_id
    }
    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }
    #[must_use]
    pub fn analyzer_snapshot_id(&self) -> &str {
        &self.analyzer_snapshot_id
    }
    #[must_use]
    pub const fn selected_profile(&self) -> &ProfileIdentity {
        &self.selected_profile
    }
    #[must_use]
    pub const fn reference_generation(&self) -> ReferenceGenerationId {
        self.reference_generation
    }
    #[must_use]
    pub const fn configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.configuration_digest
    }
    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        &self.source_snapshot_id
    }
    #[must_use]
    pub const fn graph_scope(&self) -> &GraphAssertionRecordScope {
        &self.graph_scope
    }
    #[must_use]
    pub const fn published_graph_snapshot_id(&self) -> &GraphSnapshotId {
        &self.published_graph_snapshot_id
    }
    #[must_use]
    pub const fn published_graph_generation(&self) -> &GraphGenerationId {
        &self.published_graph_generation
    }
    #[must_use]
    pub fn function_analysis_id(&self) -> &str {
        &self.function_analysis_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionFingerprintUnavailable {
    MissingSignature,
    MissingShape,
    NotSelected,
}

/// Exact bytes: comments, whitespace, BOM and newlines are significant.
/// Available does not mean an available normalized declaration shape.
#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FunctionSourceSpanComponent {
    Available {
        digest: ContentDigest<FunctionSourceSpanFingerprint>,
        content_sha256: ContentDigest<SourceContent>,
        byte_length: u64,
    },
    MissingExactSpan,
}

#[derive(Debug, Serialize)]
pub struct FunctionFingerprintSupport {
    entity_assertion: GraphAssertionRef,
    incoming_ownership_assertion: GraphAssertionRef,
    incoming_ownership_definition: &'static str,
    source_handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: Box<str>,
    span: SourceSpan,
    file_content_digest: ContentDigest<SourceContent>,
}
impl FunctionFingerprintSupport {
    #[must_use]
    pub const fn entity_assertion(&self) -> &GraphAssertionRef {
        &self.entity_assertion
    }
    #[must_use]
    pub const fn incoming_ownership_assertion(&self) -> &GraphAssertionRef {
        &self.incoming_ownership_assertion
    }
    #[must_use]
    pub const fn source_handle_id(&self) -> StableHandleId {
        self.source_handle_id
    }
    #[must_use]
    pub const fn evidence_id(&self) -> EvidenceId {
        self.evidence_id
    }
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

#[derive(Debug, Serialize)]
pub struct FunctionFingerprintRecord {
    exact_owner: ExactFunctionOwnerKey,
    function_id: Box<str>,
    kind: SourceFunctionKind,
    source_span: FunctionSourceSpanComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    declared_header: Option<FunctionDeclaredHeader>,
    signature: FunctionFingerprintUnavailable,
    declaration_shape: FunctionDeclarationShapeComponent,
    direct_neighborhood: FunctionFingerprintUnavailable,
    package_load_role: FunctionPackageLoadRoleSelection,
    support: FunctionFingerprintSupport,
}
impl FunctionFingerprintRecord {
    #[must_use]
    pub fn function_id(&self) -> &str {
        &self.function_id
    }
    #[must_use]
    pub const fn exact_owner(&self) -> &ExactFunctionOwnerKey {
        &self.exact_owner
    }
    #[must_use]
    pub const fn source_span(&self) -> &FunctionSourceSpanComponent {
        &self.source_span
    }
    #[must_use]
    pub const fn declared_header(&self) -> Option<&FunctionDeclaredHeader> {
        self.declared_header.as_ref()
    }
    #[must_use]
    pub const fn signature(&self) -> FunctionFingerprintUnavailable {
        self.signature
    }
    #[must_use]
    pub const fn declaration_shape(&self) -> &FunctionDeclarationShapeComponent {
        &self.declaration_shape
    }
    #[must_use]
    pub const fn direct_neighborhood(&self) -> FunctionFingerprintUnavailable {
        self.direct_neighborhood
    }
    #[must_use]
    pub const fn package_load_role(&self) -> &FunctionPackageLoadRoleSelection {
        &self.package_load_role
    }
    #[must_use]
    pub const fn support(&self) -> &FunctionFingerprintSupport {
        &self.support
    }
}

#[derive(Debug, Serialize)]
pub struct FunctionFingerprintPartitionCoverage {
    partition_id: Box<str>,
    partition_digest: Box<str>,
    producer_version: Box<str>,
    records: Vec<GraphCoverageRecord>,
}
impl FunctionFingerprintPartitionCoverage {
    #[must_use]
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }
    #[must_use]
    pub fn records(&self) -> &[GraphCoverageRecord] {
        &self.records
    }
}
#[derive(Debug, Serialize)]
pub struct FunctionFingerprintScopeConflict {
    partition_id: Box<str>,
    batch_id: Box<str>,
    record: GraphConflictRecord,
}
impl FunctionFingerprintScopeConflict {
    #[must_use]
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }
    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }
    #[must_use]
    pub const fn record(&self) -> &GraphConflictRecord {
        &self.record
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionFingerprintCoverage {
    Partial,
}

/// Serialize-only owned receipt: it survives moving/dropping the native owners.
/// No composite, empty-neighborhood proof, removed-entity or equivalence output.
#[derive(Debug, Serialize)]
pub struct FunctionFingerprintComponentSelection {
    #[serde(skip_serializing_if = "Option::is_none")]
    physical_declarations: Option<wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe>,
    package_load_role: FunctionPackageLoadRoleRecipe,
}
impl FunctionFingerprintComponentSelection {
    pub const fn physical_declarations(
        &self,
    ) -> Option<wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe> {
        self.physical_declarations
    }
    pub const fn package_load_role(&self) -> FunctionPackageLoadRoleRecipe {
        self.package_load_role
    }
}

#[derive(Debug, Serialize)]
pub struct FunctionFingerprintManifest {
    schema: &'static str,
    profile: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_components: Option<FunctionFingerprintComponentSelection>,
    limits: FunctionFingerprintLimits,
    coverage: FunctionFingerprintCoverage,
    available_functions: usize,
    binding: FunctionFingerprintBinding,
    partition_coverage: Vec<FunctionFingerprintPartitionCoverage>,
    scope_conflicts: Vec<FunctionFingerprintScopeConflict>,
    records: Vec<FunctionFingerprintRecord>,
}
impl FunctionFingerprintManifest {
    pub const fn selected_components(&self) -> Option<&FunctionFingerprintComponentSelection> {
        self.selected_components.as_ref()
    }
    #[must_use]
    pub const fn limits(&self) -> FunctionFingerprintLimits {
        self.limits
    }
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }
    #[must_use]
    pub const fn coverage(&self) -> FunctionFingerprintCoverage {
        self.coverage
    }
    #[must_use]
    pub const fn is_partial(&self) -> bool {
        matches!(self.coverage, FunctionFingerprintCoverage::Partial)
    }
    #[must_use]
    pub const fn available_functions(&self) -> usize {
        self.available_functions
    }
    #[must_use]
    pub fn records(&self) -> &[FunctionFingerprintRecord] {
        &self.records
    }
    #[must_use]
    pub const fn binding(&self) -> &FunctionFingerprintBinding {
        &self.binding
    }
    #[must_use]
    pub fn partition_coverage(&self) -> &[FunctionFingerprintPartitionCoverage] {
        &self.partition_coverage
    }
    #[must_use]
    pub fn scope_conflicts(&self) -> &[FunctionFingerprintScopeConflict] {
        &self.scope_conflicts
    }
}

#[derive(Serialize)]
struct BorrowedExactFunctionOwnerKey<'a> {
    scope: &'a GraphAssertionRecordScope,
    node_id: &'a GraphNodeId,
    owner_key: &'a str,
}

#[derive(Serialize)]
struct BorrowedFunctionFingerprintBinding<'a> {
    project_snapshot_id: &'a str,
    project_generation: ProjectGenerationId,
    analyzer_snapshot_id: &'a str,
    selected_profile: &'a ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    configuration_digest: ContentDigest<CanonicalResult>,
    source_snapshot_id: &'a str,
    source_profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    package_binding_digest: ContentDigest<CanonicalResult>,
    source_admission_coverage: &'a crate::platform_source::PlatformAdmissionCoverage,
    direct_profile: &'static str,
    graph_scope: &'a GraphAssertionRecordScope,
    published_graph_snapshot_id: &'a GraphSnapshotId,
    published_graph_generation: &'a GraphGenerationId,
    function_analysis_id: &'a str,
}

#[derive(Serialize)]
struct BorrowedFunctionFingerprintSupport<'a> {
    entity_assertion: &'a GraphAssertionRef,
    incoming_ownership_assertion: &'a GraphAssertionRef,
    incoming_ownership_definition: &'static str,
    source_handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &'a str,
    span: SourceSpan,
    file_content_digest: ContentDigest<SourceContent>,
}

#[derive(Serialize)]
struct BorrowedFunctionFingerprintRecord<'a> {
    exact_owner: BorrowedExactFunctionOwnerKey<'a>,
    function_id: &'a str,
    kind: SourceFunctionKind,
    source_span: FunctionSourceSpanComponent,
    signature: FunctionFingerprintUnavailable,
    declaration_shape: FunctionDeclarationShapeComponent,
    direct_neighborhood: FunctionFingerprintUnavailable,
    package_load_role: FunctionPackageLoadRoleSelection,
    support: BorrowedFunctionFingerprintSupport<'a>,
}

#[derive(Serialize)]
struct BorrowedPartitionCoverage<'a> {
    partition_id: &'a str,
    partition_digest: &'a str,
    producer_version: &'a str,
    records: &'a [GraphCoverageRecord],
}
#[derive(Serialize)]
struct BorrowedScopeConflict<'a> {
    partition_id: &'a str,
    batch_id: &'a str,
    record: &'a GraphConflictRecord,
}
// Each conversion is private and called only after precharging its borrowed row.
impl BorrowedFunctionFingerprintBinding<'_> {
    fn into_owned(self, stop: &AtomicBool) -> ProjectResult<FunctionFingerprintBinding> {
        crate::analyzer::checkpoint(stop)?;
        let binding = FunctionFingerprintBinding {
            project_snapshot_id: self.project_snapshot_id.into(),
            project_generation: self.project_generation,
            analyzer_snapshot_id: self.analyzer_snapshot_id.into(),
            selected_profile: (*self.selected_profile).clone(),
            reference_generation: self.reference_generation,
            configuration_digest: self.configuration_digest,
            source_snapshot_id: self.source_snapshot_id.into(),
            source_profile_digest: self.source_profile_digest,
            content_manifest_digest: self.content_manifest_digest,
            admission_digest: self.admission_digest,
            package_binding_digest: self.package_binding_digest,
            source_admission_coverage: (*self.source_admission_coverage).clone(),
            direct_profile: self.direct_profile,
            graph_scope: (*self.graph_scope).clone(),
            published_graph_snapshot_id: (*self.published_graph_snapshot_id).clone(),
            published_graph_generation: (*self.published_graph_generation).clone(),
            function_analysis_id: self.function_analysis_id.into(),
        };
        crate::analyzer::checkpoint(stop)?;
        Ok(binding)
    }
}
impl BorrowedFunctionFingerprintRecord<'_> {
    fn into_owned(self, stop: &AtomicBool) -> ProjectResult<FunctionFingerprintRecord> {
        crate::analyzer::checkpoint(stop)?;
        let record = FunctionFingerprintRecord {
            exact_owner: ExactFunctionOwnerKey {
                scope: (*self.exact_owner.scope).clone(),
                node_id: (*self.exact_owner.node_id).clone(),
                owner_key: self.exact_owner.owner_key.into(),
            },
            function_id: self.function_id.into(),
            kind: self.kind,
            source_span: self.source_span,
            declared_header: None,
            signature: self.signature,
            declaration_shape: self.declaration_shape,
            direct_neighborhood: self.direct_neighborhood,
            package_load_role: self.package_load_role,
            support: FunctionFingerprintSupport {
                entity_assertion: (*self.support.entity_assertion).clone(),
                incoming_ownership_assertion: (*self.support.incoming_ownership_assertion).clone(),
                incoming_ownership_definition: self.support.incoming_ownership_definition,
                source_handle_id: self.support.source_handle_id,
                evidence_id: self.support.evidence_id,
                path: self.support.path.into(),
                span: self.support.span,
                file_content_digest: self.support.file_content_digest,
            },
        };
        crate::analyzer::checkpoint(stop)?;
        Ok(record)
    }
}
impl BorrowedPartitionCoverage<'_> {
    fn into_owned(self, stop: &AtomicBool) -> ProjectResult<FunctionFingerprintPartitionCoverage> {
        let mut records = Vec::new();
        for record in self.records {
            crate::analyzer::checkpoint(stop)?;
            // One admitted coverage record has bounded blocker count/text in wow-graph.
            let owned = record.clone();
            crate::analyzer::checkpoint(stop)?;
            records.push(owned);
        }
        crate::analyzer::checkpoint(stop)?;
        let coverage = FunctionFingerprintPartitionCoverage {
            partition_id: self.partition_id.into(),
            partition_digest: self.partition_digest.into(),
            producer_version: self.producer_version.into(),
            records,
        };
        crate::analyzer::checkpoint(stop)?;
        Ok(coverage)
    }
}
impl BorrowedScopeConflict<'_> {
    fn into_owned(self, stop: &AtomicBool) -> ProjectResult<FunctionFingerprintScopeConflict> {
        crate::analyzer::checkpoint(stop)?;
        let mut assertions = Vec::new();
        for reference in &self.record.assertions {
            crate::analyzer::checkpoint(stop)?;
            assertions.push(reference.clone());
        }
        let mut affected_capabilities = Vec::new();
        for capability in &self.record.affected_capabilities {
            crate::analyzer::checkpoint(stop)?;
            affected_capabilities.push(capability.clone());
        }
        let mut affected_axes = Vec::new();
        for axis in &self.record.affected_axes {
            crate::analyzer::checkpoint(stop)?;
            affected_axes.push(axis.clone());
        }
        crate::analyzer::checkpoint(stop)?;
        let conflict = FunctionFingerprintScopeConflict {
            partition_id: self.partition_id.into(),
            batch_id: self.batch_id.into(),
            record: GraphConflictRecord {
                kind: self.record.kind,
                subject: self.record.subject.clone(),
                assertions,
                affected_capabilities,
                affected_axes,
            },
        };
        crate::analyzer::checkpoint(stop)?;
        Ok(conflict)
    }
}

/// Select exact current native fact IDs in strictly increasing order.
/// Selection is not a claim that absent functions, relations or packages do not exist.
/// Refusal/cancellation returns no truncated successful manifest.
pub fn build_platform_function_fingerprints(
    direct: &PlatformGraphProvenance<'_>,
    selected_function_ids: &[&str],
    limits: FunctionFingerprintLimits,
    stop: &AtomicBool,
) -> ProjectResult<FunctionFingerprintManifest> {
    build_selected_function_fingerprints(direct, selected_function_ids, limits, None, stop)
}

/// Select genuine native static load roles without changing the source/graph recipe.
pub fn build_platform_function_fingerprints_with_load_roles(
    direct: &PlatformGraphProvenance<'_>,
    selected_function_ids: &[&str],
    limits: FunctionFingerprintLimits,
    recipe: FunctionPackageLoadRoleRecipe,
    stop: &AtomicBool,
) -> ProjectResult<FunctionFingerprintManifest> {
    build_selected_function_fingerprints(direct, selected_function_ids, limits, Some(recipe), stop)
}

fn build_selected_function_fingerprints(
    direct: &PlatformGraphProvenance<'_>,
    selected_function_ids: &[&str],
    limits: FunctionFingerprintLimits,
    load_roles: Option<FunctionPackageLoadRoleRecipe>,
    stop: &AtomicBool,
) -> ProjectResult<FunctionFingerprintManifest> {
    crate::analyzer::checkpoint(stop)?;
    if selected_function_ids.len() > limits.max_records {
        return Err(exhausted(
            "records",
            selected_function_ids.len(),
            limits.max_records,
        ));
    }
    let project = direct.project();
    let config = project.configuration();
    let Some(PlatformGraphProfile::DirectCapabilitiesV1(selected)) =
        config.platform_graph_profile()
    else {
        return Err(invalid());
    };
    if direct.capabilities() != Some(selected) {
        return Err(invalid());
    }
    let packages = config.platform_packages().ok_or_else(invalid)?;
    let receipt = packages.source().receipt();
    let report = direct.source().function_call_report().ok_or_else(invalid)?;
    let analyzer = project.snapshot().analyzer_binding();
    // finish() already validated report/library/budget/file identities through the native plan.
    if report.main_snapshot_id() != analyzer.main_workspace().snapshot_id()
        || report.functions().len() != direct.source().functions().len()
        || report.budget() != config.budget_policy().function_call_budget()
        || report.declaration_recipe() != config.physical_function_declarations()
    {
        return Err(invalid());
    }
    // Bound caller strings using this admitted native population before inspecting their bytes.
    // Native function IDs are source-owner canonical IDs, not arbitrary caller identity claims.
    let mut max_native_id_bytes = 0_usize;
    for fact in report.functions() {
        crate::analyzer::checkpoint(stop)?;
        max_native_id_bytes = max_native_id_bytes.max(fact.fact_id().len());
    }
    for &function_id in selected_function_ids {
        crate::analyzer::checkpoint(stop)?;
        // str::len is constant-time: reject oversized strings before any lexical comparison.
        if function_id.is_empty() || function_id.len() > max_native_id_bytes {
            return Err(invalid_selection());
        }
        // The native owner sorts by fact_id and validate rejects non-strict order.
        let matched = report
            .functions()
            .binary_search_by(|fact| fact.fact_id().cmp(function_id));
        crate::analyzer::checkpoint(stop)?;
        matched.map_err(|_| invalid_selection())?;
    }
    // Every caller ID now equals a bounded native ID; only now compare caller order.
    for pair in selected_function_ids.windows(2) {
        crate::analyzer::checkpoint(stop)?;
        if pair[0] >= pair[1] {
            return Err(invalid_selection());
        }
    }
    let lookup = direct
        .graph()
        .producer_lookup(stop)
        .map_err(super::platform_producers::graph_error)?;
    if lookup.scope() != direct.scope() {
        return Err(invalid());
    }
    let binding = BorrowedFunctionFingerprintBinding {
        project_snapshot_id: project.snapshot_id(),
        project_generation: project.project_generation(),
        analyzer_snapshot_id: project.analyzer_snapshot_id(),
        selected_profile: config.selected_profile(),
        reference_generation: config.reference_generation(),
        configuration_digest: config.configuration_digest(),
        source_snapshot_id: receipt.source_snapshot_id(),
        source_profile_digest: receipt.profile_digest(),
        content_manifest_digest: receipt.content_manifest_digest(),
        admission_digest: receipt.admission_digest(),
        package_binding_digest: packages.binding().binding_digest(),
        source_admission_coverage: receipt.coverage(),
        direct_profile: direct.profile(),
        graph_scope: direct.scope(),
        published_graph_snapshot_id: direct.graph().snapshot().snapshot_id(),
        published_graph_generation: direct.graph().snapshot().generation(),
        function_analysis_id: report.analysis_id(),
    };
    let mut retained = ProducerBudget::new(0, limits.max_metadata_bytes)?;
    retained.charge_serialized(&limits, stop)?;
    retained.charge_serialized(&binding, stop)?;
    let binding = binding.into_owned(stop)?;
    let mut partition_coverage = Vec::new();
    let mut scope_conflicts = Vec::new();
    for partition in direct.graph().partitions() {
        crate::analyzer::checkpoint(stop)?;
        let coverage = BorrowedPartitionCoverage {
            partition_id: partition.partition_id(),
            partition_digest: partition.partition_digest(),
            producer_version: partition.producer_version(),
            records: partition.coverage(),
        };
        retained.charge_serialized(&coverage, stop)?;
        partition_coverage.push(coverage.into_owned(stop)?);
        if let Some(assertions) = partition.batch().assertion_records() {
            for conflict in &assertions.conflicts {
                crate::analyzer::checkpoint(stop)?;
                let support = BorrowedScopeConflict {
                    partition_id: partition.partition_id(),
                    batch_id: partition.batch().batch_id(),
                    record: conflict,
                };
                retained.charge_serialized(&support, stop)?;
                scope_conflicts.push(support.into_owned(stop)?);
            }
        }
    }
    let mut source_bytes = 0_usize;
    let mut records = Vec::new();
    for &function_id in selected_function_ids {
        crate::analyzer::checkpoint(stop)?;
        // Native report sorts by fact_id (function_calls.rs:998); validate rejects non-strict order (345-348).
        let index = report
            .functions()
            .binary_search_by(|fact| fact.fact_id().cmp(function_id))
            .map_err(|_| invalid())?;
        let fact = &report.functions()[index];
        let function = direct.source().functions().get(index).ok_or_else(invalid)?;
        if function.function_id != fact.fact_id()
            || function.path != fact.path()
            || function.kind != fact.kind()
            || function.span != fact.span()
        {
            return Err(invalid());
        }
        let entity_key = GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: function.proposal_id.as_str().into(),
        };
        let ownership_key = GraphLocalAssertion {
            kind: GraphAssertionKind::Relation,
            proposal_id: function.ownership_proposal_id.as_str().into(),
        };
        let entity_ref = direct.assertion(&entity_key).ok_or_else(invalid)?;
        let ownership_ref = direct.assertion(&ownership_key).ok_or_else(invalid)?;
        let entity = lookup
            .entity(direct.scope(), entity_ref, stop)
            .map_err(super::platform_producers::graph_error)?;
        let ownership = lookup
            .relation(direct.scope(), ownership_ref, stop)
            .map_err(super::platform_producers::graph_error)?;
        let node = entity.accepted().node();
        let edge = ownership.accepted().edge();
        if entity.proposal().entity_kind_id() != "lua_source_function"
            || entity.proposal().confidence() != GraphConfidence::Derived
            || ownership.proposal().relation_kind_id() != "source_declaration_owns"
            || ownership.proposal().confidence() != GraphConfidence::Derived
            || edge.to() != node.node_id()
            || lookup
                .input_view()
                .node(edge.from())
                .is_none_or(|owner| owner.kind() != "source_file")
            || !entity
                .proposal()
                .source_handle_ids()
                .contains(&function.source_handle_id)
            || !entity
                .proposal()
                .evidence_ids()
                .contains(&function.evidence_id)
            || !ownership
                .proposal()
                .source_handle_ids()
                .contains(&function.source_handle_id)
            || !ownership
                .proposal()
                .evidence_ids()
                .contains(&function.evidence_id)
        {
            return Err(invalid());
        }
        let handle = direct
            .source()
            .source_handles()
            .get(&function.source_handle_id)
            .ok_or_else(invalid)?;
        let evidence = direct
            .source()
            .evidence()
            .get(&function.evidence_id)
            .ok_or_else(invalid)?;
        let file_digest =
            crate::identity::parse_source_digest(fact.content_digest(), ProjectPhase::View)?;
        if handle.path().as_str() != fact.path()
            || handle.span() != fact.span()
            || *handle.content_digest() != file_digest
            || evidence.context_id() != direct.scope().source_context_id
            || !evidence
                .source_handle_ids()
                .contains(&function.source_handle_id)
        {
            return Err(invalid());
        }
        // Existing path-sorted borrowed lookup: no per-function scan, clone or new index.
        let file = analyzer
            .main_workspace()
            .file(fact.path())
            .ok_or_else(invalid)?;
        if file.content_sha256() != fact.content_digest() {
            return Err(invalid());
        }
        let text = if fact.span() == SourceSpan::whole_file()
            && fact.kind() == SourceFunctionKind::Chunk
        {
            Some(file.text())
        } else if let (Some(start), Some(end)) = (fact.span().byte_start(), fact.span().byte_end())
        {
            let start = usize::try_from(start).map_err(|_| invalid())?;
            let end = usize::try_from(end).map_err(|_| invalid())?;
            Some(file.text().get(start..end).ok_or_else(invalid)?)
        } else {
            None
        };
        let source_span = match text {
            None => FunctionSourceSpanComponent::MissingExactSpan,
            Some(text) => span_component(text.as_bytes(), &mut source_bytes, limits, stop)?,
        };
        let record = BorrowedFunctionFingerprintRecord {
            exact_owner: BorrowedExactFunctionOwnerKey {
                scope: direct.scope(),
                node_id: node.node_id(),
                owner_key: node.owner_key(),
            },
            function_id: fact.fact_id(),
            kind: fact.kind(),
            source_span,
            signature: FunctionFingerprintUnavailable::MissingSignature,
            declaration_shape: FunctionDeclarationShapeComponent::MissingShape,
            direct_neighborhood: FunctionFingerprintUnavailable::NotSelected,
            package_load_role: FunctionPackageLoadRoleSelection::NotSelected,
            support: BorrowedFunctionFingerprintSupport {
                entity_assertion: entity_ref,
                incoming_ownership_assertion: ownership_ref,
                incoming_ownership_definition: "source_declaration_owns",
                source_handle_id: function.source_handle_id,
                evidence_id: function.evidence_id,
                path: fact.path(),
                span: fact.span(),
                file_content_digest: file_digest,
            },
        };
        retained.charge_serialized(&record, stop)?;
        records.push(record.into_owned(stop)?);
    }
    let declarations_selected = report.declaration_recipe().is_some();
    let body_shape_selected = report.declaration_recipe() == Some(wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1);
    let mut manifest = FunctionFingerprintManifest {
        schema: if load_roles.is_some() {
            SELECTED_COMPONENTS_FINGERPRINT_SCHEMA
        } else if body_shape_selected {
            BODY_SHAPE_FINGERPRINT_SCHEMA
        } else if declarations_selected {
            DECLARED_HEADER_FINGERPRINT_SCHEMA
        } else {
            FUNCTION_FINGERPRINT_SCHEMA
        },
        profile: if load_roles.is_some() {
            SELECTED_COMPONENTS_FINGERPRINT_PROFILE
        } else if body_shape_selected {
            BODY_SHAPE_FINGERPRINT_PROFILE
        } else if declarations_selected {
            DECLARED_HEADER_FINGERPRINT_PROFILE
        } else {
            FUNCTION_FINGERPRINT_PROFILE
        },
        limits,
        selected_components: load_roles.map(|recipe| FunctionFingerprintComponentSelection {
            physical_declarations: report.declaration_recipe(),
            package_load_role: recipe,
        }),
        coverage: FunctionFingerprintCoverage::Partial,
        available_functions: report.functions().len(),
        binding,
        partition_coverage,
        scope_conflicts,
        records,
    };
    if declarations_selected {
        for index in 0..manifest.records.len() {
            let header = declaration::build_declared_header_component(
                report,
                &manifest,
                index,
                &mut retained,
                stop,
            )?;
            manifest.records[index].declared_header = Some(header);
            if body_shape_selected {
                let shape =
                    shape::build_shape_component(report, &manifest, index, &mut retained, stop)?;
                manifest.records[index].declaration_shape = shape;
            }
        }
    }
    // Same cap, fresh exact output count including array/object framing, not a second allowance.
    if load_roles.is_some() {
        let rows =
            load_role::build_package_load_role_components(direct, &manifest, &mut retained, stop)?;
        if rows.len() != manifest.records.len() {
            return Err(invalid());
        }
        for (record, row) in manifest.records.iter_mut().zip(rows) {
            crate::analyzer::checkpoint(stop)?;
            // The private producer returns the same strict native record order.
            // The native producer precharged the complete owned row before this
            // fixed-size allocation; unselected records keep a small enum.
            record.package_load_role = FunctionPackageLoadRoleSelection::Selected(Box::new(row));
        }
    }
    if body_shape_selected || load_roles.is_some() {
        retained.charge_serialized(&manifest, stop)?;
    } else {
        // Preserve the admitted older recipe's exact output-ceiling check.
        ProducerBudget::new(0, limits.max_metadata_bytes)?.charge_serialized(&manifest, stop)?;
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(manifest)
}

fn span_component(
    bytes: &[u8],
    used: &mut usize,
    limits: FunctionFingerprintLimits,
    stop: &AtomicBool,
) -> ProjectResult<FunctionSourceSpanComponent> {
    let attempted = *used as u128 + bytes.len() as u128;
    if attempted > limits.max_source_bytes as u128 {
        return Err(ProjectError::new(
            ProjectErrorCode::SourceBudgetExceeded,
            ProjectPhase::View,
            format!(
                "function fingerprint source bytes exceed limit: {attempted}/{}",
                limits.max_source_bytes
            ),
        ));
    }
    *used = used.checked_add(bytes.len()).ok_or_else(invalid)?;
    let mut hash = Sha256::new();
    // Work checkpoint granularity only; this does not change any admission cap.
    for piece in bytes.chunks(64 * 1024) {
        crate::analyzer::checkpoint(stop)?;
        hash.update(piece);
    }
    crate::analyzer::checkpoint(stop)?;
    let content_sha256 = ContentDigest::<SourceContent>::from_bytes(hash.finalize().into());
    #[derive(Serialize)]
    struct SpanProjection {
        content_sha256: ContentDigest<SourceContent>,
        byte_length: u64,
    }
    let byte_length = u64::try_from(bytes.len()).map_err(|_| invalid())?;
    let value = SpanProjection {
        content_sha256,
        byte_length,
    };
    let digest = crate::identity::canonical_digest(SOURCE_SPAN_DOMAIN, &value, ProjectPhase::View)?;
    Ok(FunctionSourceSpanComponent::Available {
        digest: ContentDigest::from_bytes(*digest.as_bytes()),
        content_sha256,
        byte_length,
    })
}

fn exhausted(resource: &str, attempted: usize, limit: usize) -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::View,
        format!("function fingerprint {resource} exceed limit: {attempted}/{limit}"),
    )
}
fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SnapshotInvalid,
        ProjectPhase::View,
        "function fingerprint selection or native owner binding disagrees",
    )
}
fn invalid_selection() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::InvalidFingerprintSelection,
        ProjectPhase::View,
        "function fingerprint selection is invalid",
    )
}
