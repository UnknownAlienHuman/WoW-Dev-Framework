//! Transport-neutral E1-D Reference Pack candidate assembly, validation and rebuild comparison.
//!
//! The executable layout is deliberately a local native candidate profile. It packages
//! exact owner outputs plus a detached SQLite ReferenceStore that is independently reopened
//! read-only. Parity/consumer, license and filesystem-finalization gates still block
//! `validated-local`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_annotations::artifact::AnnotationArtifact;
use wow_core::{ProfileIdentity, canonical_json_bytes};
use wow_reference::{
    ReferenceView,
    persistent::{PersistentReferenceStore, ReferencePublicationKey, SealedReferenceStore},
};
use wow_store::{CatalogExpectation, SealedStore, Store, StoreConfiguration, StoreLimits};

use crate::local::{LocalProjectInput, NativeAnnotationFile, NativeInputReceipt};

pub const REFERENCE_PACK_SCHEMA: &str = "wow-service/reference-pack/e1-d/2";
pub const LOCAL_NATIVE_PACK_LAYOUT: &str = "wow-reference-pack/layout/local-native-candidate/2";
pub const LOCAL_NATIVE_VALIDATION_PROFILE: &str =
    "wow-reference-pack/validation/local-native-candidate/2";

const MANIFEST_PATH: &str = "manifest.json";
const CHECKSUMS_PATH: &str = "checksums.json";
const REFERENCE_VIEW_PATH: &str = "reference/reference-view.json";
const REFERENCE_STORE_PATH: &str = "reference/reference-store.sqlite3";
const REFERENCE_STORE_PROFILE: &str = "wow-reference-pack-store-local-native-v1";
const REFERENCE_STORE_CHANNEL: &str = "pack";
const ANNOTATION_MANIFEST_PATH: &str = "annotations/artifact-manifest.json";
const PROVENANCE_PATH: &str = "provenance/native-input.json";
const MAX_REVIEWED_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
const MAX_REVIEWED_MEMBER_BYTES: u64 = 64 * 1024 * 1024;
const MAX_REVIEWED_MEMBERS: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackErrorCode {
    InvalidRequest,
    UnsupportedLayout,
    SourceInputUnavailable,
    IdentityMismatch,
    MemberInvalid,
    BudgetExceeded,
    Cancelled,
    ManifestInvalid,
    ValidationFailed,
    SerializationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePackError {
    code: ReferencePackErrorCode,
    message: Box<str>,
}

impl ReferencePackError {
    fn new(code: ReferencePackErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ReferencePackErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ReferencePackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReferencePackError {}

pub type ReferencePackResult<T> = Result<T, ReferencePackError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackEligibilityTarget {
    Candidate,
    ValidatedLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackBudgets {
    max_members: u32,
    max_member_bytes: u64,
    max_total_bytes: u64,
}

impl ReferencePackBudgets {
    pub fn new(
        max_members: u32,
        max_member_bytes: u64,
        max_total_bytes: u64,
    ) -> ReferencePackResult<Self> {
        if max_members == 0
            || max_members > MAX_REVIEWED_MEMBERS
            || max_member_bytes == 0
            || max_member_bytes > MAX_REVIEWED_MEMBER_BYTES
            || max_total_bytes == 0
            || max_total_bytes > MAX_REVIEWED_TOTAL_BYTES
            || max_member_bytes > max_total_bytes
        {
            return Err(error(
                ReferencePackErrorCode::InvalidRequest,
                "reference pack budgets are outside the reviewed profile",
            ));
        }
        Ok(Self {
            max_members,
            max_member_bytes,
            max_total_bytes,
        })
    }

    #[must_use]
    pub const fn max_members(self) -> u32 {
        self.max_members
    }

    #[must_use]
    pub const fn max_member_bytes(self) -> u64 {
        self.max_member_bytes
    }

    #[must_use]
    pub const fn max_total_bytes(self) -> u64 {
        self.max_total_bytes
    }
}

impl Default for ReferencePackBudgets {
    fn default() -> Self {
        Self {
            max_members: 2048,
            max_member_bytes: 16 * 1024 * 1024,
            max_total_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackBuildRequest {
    request_id: Box<str>,
    expected_profile_id: Box<str>,
    expected_reference_generation_id: Box<str>,
    layout_profile_id: Box<str>,
    eligibility_target: ReferencePackEligibilityTarget,
    execution_profile_id: Box<str>,
    budgets: ReferencePackBudgets,
}

impl ReferencePackBuildRequest {
    pub fn new(
        request_id: impl Into<Box<str>>,
        expected_profile_id: impl Into<Box<str>>,
        expected_reference_generation_id: impl Into<Box<str>>,
        eligibility_target: ReferencePackEligibilityTarget,
        execution_profile_id: impl Into<Box<str>>,
        budgets: ReferencePackBudgets,
    ) -> ReferencePackResult<Self> {
        let request = Self {
            request_id: request_id.into(),
            expected_profile_id: expected_profile_id.into(),
            expected_reference_generation_id: expected_reference_generation_id.into(),
            layout_profile_id: LOCAL_NATIVE_PACK_LAYOUT.into(),
            eligibility_target,
            execution_profile_id: execution_profile_id.into(),
            budgets,
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> ReferencePackResult<()> {
        if !valid_identity(&self.request_id)
            || !valid_identity(&self.expected_profile_id)
            || !valid_identity(&self.expected_reference_generation_id)
            || !valid_identity(&self.execution_profile_id)
        {
            return Err(error(
                ReferencePackErrorCode::InvalidRequest,
                "reference pack request contains an invalid identity",
            ));
        }
        if self.layout_profile_id.as_ref() != LOCAL_NATIVE_PACK_LAYOUT {
            return Err(error(
                ReferencePackErrorCode::UnsupportedLayout,
                "reference pack layout profile is unsupported",
            ));
        }
        ReferencePackBudgets::new(
            self.budgets.max_members,
            self.budgets.max_member_bytes,
            self.budgets.max_total_bytes,
        )?;
        Ok(())
    }

    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    #[must_use]
    pub fn expected_profile_id(&self) -> &str {
        &self.expected_profile_id
    }

    #[must_use]
    pub fn expected_reference_generation_id(&self) -> &str {
        &self.expected_reference_generation_id
    }

    #[must_use]
    pub fn execution_profile_id(&self) -> &str {
        &self.execution_profile_id
    }

    #[must_use]
    pub const fn eligibility_target(&self) -> ReferencePackEligibilityTarget {
        self.eligibility_target
    }

    #[must_use]
    pub const fn budgets(&self) -> ReferencePackBudgets {
        self.budgets
    }

    fn with_execution_profile(&self, execution_profile_id: Box<str>) -> Self {
        let mut cloned = self.clone();
        cloned.execution_profile_id = execution_profile_id;
        cloned
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackGateStatus {
    Passed,
    Failed,
    NotEvaluated,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackGateRecord {
    gate_id: Box<str>,
    required_for_candidate: bool,
    required_for_validated_local: bool,
    status: PackGateStatus,
    reason_code: Box<str>,
}

impl PackGateRecord {
    fn new(
        gate_id: &'static str,
        required_for_candidate: bool,
        required_for_validated_local: bool,
        status: PackGateStatus,
        reason_code: &'static str,
    ) -> Self {
        Self {
            gate_id: gate_id.into(),
            required_for_candidate,
            required_for_validated_local,
            status,
            reason_code: reason_code.into(),
        }
    }

    #[must_use]
    pub fn gate_id(&self) -> &str {
        &self.gate_id
    }

    #[must_use]
    pub const fn status(&self) -> PackGateStatus {
        self.status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackMemberKind {
    ReferenceView,
    ReferenceStore,
    AnnotationArtifactManifest,
    AnnotationFile,
    ProvenanceManifest,
    ChecksumManifest,
    PackManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackMember {
    path: Box<str>,
    kind: ReferencePackMemberKind,
    logical_id: Box<str>,
    byte_length: u64,
    sha256: Box<str>,
}

impl ReferencePackMember {
    fn from_bytes(
        path: impl Into<Box<str>>,
        kind: ReferencePackMemberKind,
        logical_id: impl Into<Box<str>>,
        bytes: &[u8],
    ) -> ReferencePackResult<Self> {
        let path = path.into();
        validate_member_path(&path)?;
        let logical_id = logical_id.into();
        if !valid_identity(&logical_id) {
            return Err(error(
                ReferencePackErrorCode::MemberInvalid,
                "reference pack member logical identity is invalid",
            ));
        }
        Ok(Self {
            path,
            kind,
            logical_id,
            byte_length: bytes.len() as u64,
            sha256: sha256(bytes).into_boxed_str(),
        })
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn kind(&self) -> ReferencePackMemberKind {
        self.kind
    }

    #[must_use]
    pub fn logical_id(&self) -> &str {
        &self.logical_id
    }

    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackMaterializationEntry {
    member: ReferencePackMember,
    bytes: Box<[u8]>,
}

impl PackMaterializationEntry {
    fn new(member: ReferencePackMember, bytes: Vec<u8>) -> Self {
        Self {
            member,
            bytes: bytes.into_boxed_slice(),
        }
    }

    #[must_use]
    pub fn member(&self) -> &ReferencePackMember {
        &self.member
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackMaterializationPlan {
    plan_id: Box<str>,
    pack_id: Box<str>,
    layout_profile_id: Box<str>,
    directories: Box<[Box<str>]>,
    entries: Box<[PackMaterializationEntry]>,
}

impl PackMaterializationPlan {
    #[must_use]
    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }

    #[must_use]
    pub fn pack_id(&self) -> &str {
        &self.pack_id
    }

    #[must_use]
    pub fn layout_profile_id(&self) -> &str {
        &self.layout_profile_id
    }

    #[must_use]
    pub fn directories(&self) -> &[Box<str>] {
        &self.directories
    }

    #[must_use]
    pub fn entries(&self) -> &[PackMaterializationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn image(&self) -> ReferencePackImage {
        ReferencePackImage {
            files: self
                .entries
                .iter()
                .map(|entry| PackImageFile {
                    path: entry.member.path.clone(),
                    bytes: entry.bytes.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackImageFile {
    path: Box<str>,
    bytes: Box<[u8]>,
}

impl PackImageFile {
    pub fn new(path: impl Into<Box<str>>, bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackImage {
    files: Box<[PackImageFile]>,
}

impl ReferencePackImage {
    pub fn new(files: Vec<PackImageFile>) -> Self {
        Self {
            files: files.into_boxed_slice(),
        }
    }

    #[must_use]
    pub fn files(&self) -> &[PackImageFile] {
        &self.files
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackEligibilityState {
    Candidate,
    Blocked,
    ValidatedLocal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackManifest {
    schema: Box<str>,
    pack_id: Box<str>,
    layout_profile_id: Box<str>,
    profile: ProfileIdentity,
    reference_generation_id: Box<str>,
    reference_view_digest: Box<str>,
    reference_store_profile_id: Box<str>,
    reference_store_configuration_id: Box<str>,
    reference_store_manifest_id: Box<str>,
    reference_store_object_id: Box<str>,
    reference_store_publication_channel: Box<str>,
    annotation_artifact_id: Box<str>,
    annotation_payload_sha256: Box<str>,
    source_manifest_sha256: Box<str>,
    payload_members: Box<[ReferencePackMember]>,
    checksum_member: ReferencePackMember,
    gate_records: Box<[PackGateRecord]>,
    eligibility_state: ReferencePackEligibilityState,
    deferred_capabilities: Box<[Box<str>]>,
}

impl ReferencePackManifest {
    #[must_use]
    pub fn pack_id(&self) -> &str {
        &self.pack_id
    }

    #[must_use]
    pub fn profile(&self) -> &ProfileIdentity {
        &self.profile
    }

    #[must_use]
    pub fn reference_generation_id(&self) -> &str {
        &self.reference_generation_id
    }

    #[must_use]
    pub fn payload_members(&self) -> &[ReferencePackMember] {
        &self.payload_members
    }

    #[must_use]
    pub fn gate_records(&self) -> &[PackGateRecord] {
        &self.gate_records
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackChecksumEntry {
    path: Box<str>,
    byte_length: u64,
    sha256: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackChecksums {
    schema: Box<str>,
    entries: Box<[PackChecksumEntry]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackAnnotationFileDescriptor {
    source_path: Box<str>,
    pack_path: Box<str>,
    byte_length: u64,
    sha256: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackAnnotationArtifactManifest {
    schema: Box<str>,
    artifact_id: Box<str>,
    producer_id: Box<str>,
    producer_version: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    payload_sha256: Box<str>,
    files: Box<[PackAnnotationFileDescriptor]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackNativeProvenance {
    schema: Box<str>,
    profile: ProfileIdentity,
    revision: Box<str>,
    environment: Box<str>,
    source_binding: Box<str>,
    source_manifest_sha256: Box<str>,
    selected_toc_sha256: Box<str>,
    generated_api_closure_complete: bool,
    report_sha256: Box<str>,
    report_bytes: u64,
    annotation_projection: Box<str>,
    input_failures: u64,
    reference_issues: u64,
    reference_conflicts: u64,
    annotation_issues: u64,
    semantic_consumer_acceptance: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackValidationRequest {
    expected_pack_id: Box<str>,
    expected_profile_id: Box<str>,
    expected_reference_generation_id: Box<str>,
    validation_profile_id: Box<str>,
    budgets: ReferencePackBudgets,
}

impl ReferencePackValidationRequest {
    pub fn new(
        expected_pack_id: impl Into<Box<str>>,
        expected_profile_id: impl Into<Box<str>>,
        expected_reference_generation_id: impl Into<Box<str>>,
        budgets: ReferencePackBudgets,
    ) -> ReferencePackResult<Self> {
        let request = Self {
            expected_pack_id: expected_pack_id.into(),
            expected_profile_id: expected_profile_id.into(),
            expected_reference_generation_id: expected_reference_generation_id.into(),
            validation_profile_id: LOCAL_NATIVE_VALIDATION_PROFILE.into(),
            budgets,
        };
        if !valid_identity(&request.expected_pack_id)
            || !valid_identity(&request.expected_profile_id)
            || !valid_identity(&request.expected_reference_generation_id)
        {
            return Err(error(
                ReferencePackErrorCode::InvalidRequest,
                "reference pack validation request contains an invalid identity",
            ));
        }
        ReferencePackBudgets::new(
            budgets.max_members,
            budgets.max_member_bytes,
            budgets.max_total_bytes,
        )?;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackValidationReport {
    schema: Box<str>,
    validation_profile_id: Box<str>,
    pack_id: Box<str>,
    report_id: Box<str>,
    gate_records: Box<[PackGateRecord]>,
    candidate_eligible: bool,
    validated_local_eligible: bool,
    mutation_count: u32,
}

impl ReferencePackValidationReport {
    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    #[must_use]
    pub fn pack_id(&self) -> &str {
        &self.pack_id
    }

    #[must_use]
    pub const fn candidate_eligible(&self) -> bool {
        self.candidate_eligible
    }

    #[must_use]
    pub const fn validated_local_eligible(&self) -> bool {
        self.validated_local_eligible
    }

    #[must_use]
    pub fn gate_records(&self) -> &[PackGateRecord] {
        &self.gate_records
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackBuildStatus {
    CandidateReady,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePackBuildOutcome {
    request_id: Box<str>,
    execution_profile_id: Box<str>,
    status: ReferencePackBuildStatus,
    plan: PackMaterializationPlan,
    validation_report: ReferencePackValidationReport,
}

impl ReferencePackBuildOutcome {
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    #[must_use]
    pub fn execution_profile_id(&self) -> &str {
        &self.execution_profile_id
    }

    #[must_use]
    pub const fn status(&self) -> ReferencePackBuildStatus {
        self.status
    }

    #[must_use]
    pub fn plan(&self) -> &PackMaterializationPlan {
        &self.plan
    }

    #[must_use]
    pub fn validation_report(&self) -> &ReferencePackValidationReport {
        &self.validation_report
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackRebuildComparisonRequest {
    build_request: ReferencePackBuildRequest,
    left_execution_profile_id: Box<str>,
    right_execution_profile_id: Box<str>,
}

impl ReferencePackRebuildComparisonRequest {
    pub fn new(
        build_request: ReferencePackBuildRequest,
        left_execution_profile_id: impl Into<Box<str>>,
        right_execution_profile_id: impl Into<Box<str>>,
    ) -> ReferencePackResult<Self> {
        let request = Self {
            build_request,
            left_execution_profile_id: left_execution_profile_id.into(),
            right_execution_profile_id: right_execution_profile_id.into(),
        };
        if !valid_identity(&request.left_execution_profile_id)
            || !valid_identity(&request.right_execution_profile_id)
            || request.left_execution_profile_id == request.right_execution_profile_id
        {
            return Err(error(
                ReferencePackErrorCode::InvalidRequest,
                "rebuild comparison requires two distinct execution profile identities",
            ));
        }
        Ok(request)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RebuildComparisonStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebuildDifference {
    comparison_class: Box<str>,
    subject: Box<str>,
    left_identity: Box<str>,
    right_identity: Box<str>,
    allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackRebuildComparisonReport {
    schema: Box<str>,
    report_id: Box<str>,
    left_pack_id: Box<str>,
    right_pack_id: Box<str>,
    semantic_identity_equal: bool,
    canonical_member_bytes_equal: bool,
    sqlite_physical_classification: Box<str>,
    differences: Box<[RebuildDifference]>,
    status: RebuildComparisonStatus,
}

impl ReferencePackRebuildComparisonReport {
    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    #[must_use]
    pub const fn status(&self) -> RebuildComparisonStatus {
        self.status
    }

    #[must_use]
    pub fn differences(&self) -> &[RebuildDifference] {
        &self.differences
    }
}

pub struct ReferencePackService;

impl ReferencePackService {
    pub fn reference_pack_build(
        request: &ReferencePackBuildRequest,
        input: &LocalProjectInput,
        stop: &AtomicBool,
    ) -> ReferencePackResult<ReferencePackBuildOutcome> {
        request.validate()?;
        checkpoint(stop)?;
        let parts = admitted_native_parts(request, input)?;
        let payload = build_payload_entries(request, &parts, stop)?;
        let gates = build_gate_records(parts.receipt);
        let plan = finalize_plan(request, &parts, payload, gates, stop)?;
        checkpoint(stop)?;
        let validation_request = ReferencePackValidationRequest::new(
            plan.pack_id(),
            request.expected_profile_id(),
            request.expected_reference_generation_id(),
            request.budgets(),
        )?;
        let validation_report =
            Self::reference_pack_validate(&validation_request, &plan.image(), stop)?;
        let status = match request.eligibility_target() {
            ReferencePackEligibilityTarget::Candidate if validation_report.candidate_eligible() => {
                ReferencePackBuildStatus::CandidateReady
            }
            ReferencePackEligibilityTarget::ValidatedLocal
                if validation_report.validated_local_eligible() =>
            {
                ReferencePackBuildStatus::CandidateReady
            }
            ReferencePackEligibilityTarget::Candidate
            | ReferencePackEligibilityTarget::ValidatedLocal => ReferencePackBuildStatus::Blocked,
        };
        Ok(ReferencePackBuildOutcome {
            request_id: request.request_id.clone(),
            execution_profile_id: request.execution_profile_id.clone(),
            status,
            plan,
            validation_report,
        })
    }

    pub fn reference_pack_validate(
        request: &ReferencePackValidationRequest,
        image: &ReferencePackImage,
        stop: &AtomicBool,
    ) -> ReferencePackResult<ReferencePackValidationReport> {
        checkpoint(stop)?;
        if request.validation_profile_id.as_ref() != LOCAL_NATIVE_VALIDATION_PROFILE {
            return Err(error(
                ReferencePackErrorCode::UnsupportedLayout,
                "reference pack validation profile is unsupported",
            ));
        }
        let files = admit_image(image, request.budgets, stop)?;
        let manifest_bytes = required_bytes(&files, MANIFEST_PATH)?;
        let manifest: ReferencePackManifest = strict_json(manifest_bytes, "pack manifest")?;
        validate_manifest_identity(&manifest)?;
        if manifest.pack_id.as_ref() != request.expected_pack_id.as_ref()
            || manifest.profile.profile_id().as_str() != request.expected_profile_id.as_ref()
            || manifest.reference_generation_id.as_ref()
                != request.expected_reference_generation_id.as_ref()
        {
            return Err(error(
                ReferencePackErrorCode::IdentityMismatch,
                "reference pack does not match the requested exact identities",
            ));
        }
        let gates = validate_materialized_members(&files, &manifest, stop)?;
        if gates != manifest.gate_records.as_ref() {
            return Err(error(
                ReferencePackErrorCode::ValidationFailed,
                "reference pack gate claims differ from independently recomputed gates",
            ));
        }
        let candidate_eligible = gates
            .iter()
            .all(|gate| !gate.required_for_candidate || gate.status == PackGateStatus::Passed);
        let validated_local_eligible = gates.iter().all(|gate| {
            !gate.required_for_validated_local || gate.status == PackGateStatus::Passed
        });
        let report_id = validation_report_id(
            &manifest.pack_id,
            &gates,
            candidate_eligible,
            validated_local_eligible,
        )?;
        Ok(ReferencePackValidationReport {
            schema: REFERENCE_PACK_SCHEMA.into(),
            validation_profile_id: LOCAL_NATIVE_VALIDATION_PROFILE.into(),
            pack_id: manifest.pack_id,
            report_id,
            gate_records: gates.into_boxed_slice(),
            candidate_eligible,
            validated_local_eligible,
            mutation_count: 0,
        })
    }

    pub fn reference_pack_rebuild_compare(
        request: &ReferencePackRebuildComparisonRequest,
        input: &LocalProjectInput,
        stop: &AtomicBool,
    ) -> ReferencePackResult<ReferencePackRebuildComparisonReport> {
        checkpoint(stop)?;
        let left_request = request
            .build_request
            .with_execution_profile(request.left_execution_profile_id.clone());
        let right_request = request
            .build_request
            .with_execution_profile(request.right_execution_profile_id.clone());
        let left = Self::reference_pack_build(&left_request, input, stop)?;
        checkpoint(stop)?;
        let right = Self::reference_pack_build(&right_request, input, stop)?;
        compare_plans(left.plan(), right.plan())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReferenceStoreIdentity {
    profile_id: Box<str>,
    configuration_id: Box<str>,
    manifest_id: Box<str>,
    object_id: Box<str>,
    publication_channel: Box<str>,
}

struct BuiltPackPayload {
    entries: Vec<PackMaterializationEntry>,
    reference_store: ReferenceStoreIdentity,
}

struct NativePackParts<'a> {
    receipt: &'a NativeInputReceipt,
    profile: &'a ProfileIdentity,
    reference: &'a ReferenceView,
    artifact: &'a AnnotationArtifact,
    annotation_files: &'a [NativeAnnotationFile],
    source_manifest_sha256: &'a str,
}

fn admitted_native_parts<'a>(
    request: &ReferencePackBuildRequest,
    input: &'a LocalProjectInput,
) -> ReferencePackResult<NativePackParts<'a>> {
    let receipt = input.native_input_receipt().ok_or_else(|| {
        error(
            ReferencePackErrorCode::SourceInputUnavailable,
            "reference pack build requires source-backed native input",
        )
    })?;
    let source_manifest = receipt.source_manifest.as_ref().ok_or_else(|| {
        error(
            ReferencePackErrorCode::SourceInputUnavailable,
            "reference pack build requires an exact admitted source manifest and TOC",
        )
    })?;
    let reference = input.reference_view();
    reference.validate().map_err(|_| {
        error(
            ReferencePackErrorCode::IdentityMismatch,
            "reference pack input ReferenceView is invalid",
        )
    })?;
    let artifact = input.native_annotation_artifact().ok_or_else(|| {
        error(
            ReferencePackErrorCode::SourceInputUnavailable,
            "reference pack input lacks its native annotation artifact",
        )
    })?;
    artifact.validate().map_err(|_| {
        error(
            ReferencePackErrorCode::IdentityMismatch,
            "reference pack annotation artifact is invalid",
        )
    })?;
    let annotation_files = input.native_annotation_files().ok_or_else(|| {
        error(
            ReferencePackErrorCode::SourceInputUnavailable,
            "reference pack input lacks generated annotation files",
        )
    })?;
    if receipt.profile.profile_id().as_str() != request.expected_profile_id()
        || reference.generation_id() != request.expected_reference_generation_id()
        || artifact.profile_id() != request.expected_profile_id()
        || artifact.source_generation_id() != request.expected_reference_generation_id()
        || annotation_files.is_empty()
    {
        return Err(error(
            ReferencePackErrorCode::IdentityMismatch,
            "reference pack component identities do not close",
        ));
    }
    Ok(NativePackParts {
        receipt,
        profile: &receipt.profile,
        reference,
        artifact,
        annotation_files,
        source_manifest_sha256: &source_manifest.manifest_sha256,
    })
}

fn build_payload_entries(
    request: &ReferencePackBuildRequest,
    parts: &NativePackParts<'_>,
    stop: &AtomicBool,
) -> ReferencePackResult<BuiltPackPayload> {
    let mut entries = Vec::new();
    let reference_bytes = parts.reference.canonical_bytes().map_err(|_| {
        error(
            ReferencePackErrorCode::SerializationFailed,
            "ReferenceView cannot be encoded canonically",
        )
    })?;
    entries.push(entry(
        REFERENCE_VIEW_PATH,
        ReferencePackMemberKind::ReferenceView,
        parts.reference.self_digest(),
        reference_bytes,
    )?);
    let (reference_store_entry, reference_store) = build_reference_store_member(request, parts)?;
    entries.push(reference_store_entry);

    let mut annotation_descriptors = Vec::new();
    let mut seen_paths = BTreeSet::new();
    for file in parts.annotation_files {
        checkpoint(stop)?;
        let pack_path = format!("annotations/files/{}", file.path());
        validate_member_path(&pack_path)?;
        let folded = pack_path.to_ascii_lowercase();
        if !seen_paths.insert(folded) || sha256(file.bytes()) != file.sha256() {
            return Err(error(
                ReferencePackErrorCode::MemberInvalid,
                "generated annotation file identity or path is invalid",
            ));
        }
        annotation_descriptors.push(PackAnnotationFileDescriptor {
            source_path: file.path().into(),
            pack_path: pack_path.clone().into_boxed_str(),
            byte_length: file.bytes().len() as u64,
            sha256: file.sha256().into(),
        });
        entries.push(entry(
            pack_path,
            ReferencePackMemberKind::AnnotationFile,
            file.sha256(),
            file.bytes().to_vec(),
        )?);
    }
    annotation_descriptors.sort_by(|left, right| left.pack_path.cmp(&right.pack_path));
    let annotation_manifest = PackAnnotationArtifactManifest {
        schema: "wow-service/reference-pack/annotation-artifact-manifest/1".into(),
        artifact_id: parts.artifact.artifact_id().into(),
        producer_id: parts.artifact.producer_id().into(),
        producer_version: parts.artifact.producer_version().into(),
        profile_id: parts.artifact.profile_id().into(),
        source_generation_id: parts.artifact.source_generation_id().into(),
        payload_sha256: parts.artifact.payload_sha256().into(),
        files: annotation_descriptors.into_boxed_slice(),
    };
    let annotation_manifest_bytes = canonical(&annotation_manifest, "annotation manifest")?;
    entries.push(entry(
        ANNOTATION_MANIFEST_PATH,
        ReferencePackMemberKind::AnnotationArtifactManifest,
        parts.artifact.artifact_id(),
        annotation_manifest_bytes,
    )?);

    let source_manifest = parts.receipt.source_manifest.as_ref().ok_or_else(|| {
        error(
            ReferencePackErrorCode::SourceInputUnavailable,
            "native source manifest receipt is unavailable",
        )
    })?;
    let provenance = PackNativeProvenance {
        schema: "wow-service/reference-pack/native-provenance/1".into(),
        profile: parts.profile.clone(),
        revision: parts.receipt.revision.clone().into_boxed_str(),
        environment: parts.receipt.environment.clone().into_boxed_str(),
        source_binding: parts.receipt.source_binding.into(),
        source_manifest_sha256: source_manifest.manifest_sha256.clone().into_boxed_str(),
        selected_toc_sha256: source_manifest.toc.sha256.clone().into_boxed_str(),
        generated_api_closure_complete: source_manifest.generated_api_closure_complete,
        report_sha256: parts.receipt.report_sha256.clone().into_boxed_str(),
        report_bytes: parts.receipt.report_bytes as u64,
        annotation_projection: parts.receipt.annotation_projection.into(),
        input_failures: parts.receipt.input_failures as u64,
        reference_issues: parts.receipt.reference_issues as u64,
        reference_conflicts: parts.receipt.reference_conflicts as u64,
        annotation_issues: parts.receipt.annotation_issues as u64,
        semantic_consumer_acceptance: parts.receipt.semantic_consumer_acceptance.into(),
    };
    let provenance_bytes = canonical(&provenance, "native provenance")?;
    entries.push(entry(
        PROVENANCE_PATH,
        ReferencePackMemberKind::ProvenanceManifest,
        provenance.report_sha256.clone(),
        provenance_bytes,
    )?);
    enforce_entry_budgets(&entries, request.budgets())?;
    entries.sort_by(|left, right| left.member.path.cmp(&right.member.path));
    Ok(BuiltPackPayload {
        entries,
        reference_store,
    })
}

fn build_reference_store_member(
    request: &ReferencePackBuildRequest,
    parts: &NativePackParts<'_>,
) -> ReferencePackResult<(PackMaterializationEntry, ReferenceStoreIdentity)> {
    let configuration = reference_store_configuration()?;
    let publication_key =
        ReferencePublicationKey::new(parts.profile.profile_id().as_str(), REFERENCE_STORE_CHANNEL)
            .map_err(|_| {
                error(
                    ReferencePackErrorCode::IdentityMismatch,
                    "reference store publication key is invalid",
                )
            })?;
    let mut store = Store::open_in_memory(configuration.clone()).map_err(|_| {
        error(
            ReferencePackErrorCode::ValidationFailed,
            "reference store staging database could not be opened",
        )
    })?;
    let stored = PersistentReferenceStore::new(&mut store)
        .publish_current(
            publication_key.clone(),
            parts.reference,
            CatalogExpectation::Absent,
        )
        .map_err(|_| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reference store staging publication failed",
            )
        })?;
    let integrity = PersistentReferenceStore::new(&mut store)
        .validate_integrity(configuration.limits().max_manifest_records)
        .map_err(|_| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reference store staging integrity validation failed",
            )
        })?;
    if !integrity.complete() {
        return Err(error(
            ReferencePackErrorCode::ValidationFailed,
            "reference store staging integrity validation was incomplete",
        ));
    }
    let logical_manifest = PersistentReferenceStore::new(&mut store)
        .logical_manifest()
        .map_err(|_| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reference store logical manifest could not be produced",
            )
        })?;
    let bytes = store
        .serialize_database(request.budgets().max_member_bytes())
        .map_err(|_| {
            error(
                ReferencePackErrorCode::BudgetExceeded,
                "reference store serialized image exceeds the member budget",
            )
        })?;
    drop(store);

    let reopened = SealedStore::open_serialized(
        &bytes,
        configuration.clone(),
        request.budgets().max_member_bytes(),
    )
    .map_err(|_| {
        error(
            ReferencePackErrorCode::ValidationFailed,
            "serialized reference store did not reopen read-only",
        )
    })?;
    let reader = SealedReferenceStore::new(&reopened);
    let reopened_manifest = reader.logical_manifest().map_err(|_| {
        error(
            ReferencePackErrorCode::ValidationFailed,
            "reopened reference store logical manifest is unavailable",
        )
    })?;
    let published = reader
        .read_current(&publication_key)
        .map_err(|_| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reopened reference store current view failed validation",
            )
        })?
        .ok_or_else(|| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reopened reference store current view is missing",
            )
        })?;
    if reopened_manifest.manifest_id() != logical_manifest.manifest_id()
        || published.object_id() != stored.object_id()
        || published.view() != parts.reference
    {
        return Err(error(
            ReferencePackErrorCode::IdentityMismatch,
            "reopened reference store does not match the staged owner output",
        ));
    }

    let identity = ReferenceStoreIdentity {
        profile_id: configuration.profile_id().into(),
        configuration_id: configuration.configuration_id().into(),
        manifest_id: logical_manifest.manifest_id().into(),
        object_id: stored.object_id().as_str().into(),
        publication_channel: REFERENCE_STORE_CHANNEL.into(),
    };
    let member = entry(
        REFERENCE_STORE_PATH,
        ReferencePackMemberKind::ReferenceStore,
        identity.manifest_id.clone(),
        bytes.into_vec(),
    )?;
    Ok((member, identity))
}

fn reference_store_configuration() -> ReferencePackResult<StoreConfiguration> {
    let limits = StoreLimits::new(64 * 1024 * 1024, 16, 16, 128, 16).map_err(|_| {
        error(
            ReferencePackErrorCode::InvalidRequest,
            "reference store limits are outside the reviewed profile",
        )
    })?;
    StoreConfiguration::new(REFERENCE_STORE_PROFILE, limits).map_err(|_| {
        error(
            ReferencePackErrorCode::InvalidRequest,
            "reference store configuration is invalid",
        )
    })
}

fn build_gate_records(receipt: &NativeInputReceipt) -> Vec<PackGateRecord> {
    let source_complete = receipt
        .source_manifest
        .as_ref()
        .is_some_and(|manifest| manifest.generated_api_closure_complete);
    let projection_clean = receipt.input_failures == 0
        && receipt.reference_issues == 0
        && receipt.reference_conflicts == 0
        && receipt.annotation_issues == 0;
    let consumer_passed = receipt.semantic_consumer_acceptance == "passed";
    vec![
        PackGateRecord::new(
            "pack.structure",
            true,
            true,
            PackGateStatus::Passed,
            "planned_member_set_closed",
        ),
        PackGateRecord::new(
            "pack.source_manifest_closure",
            true,
            true,
            if source_complete {
                PackGateStatus::Passed
            } else {
                PackGateStatus::Failed
            },
            if source_complete {
                "exact_manifest_toc_closure"
            } else {
                "source_manifest_incomplete"
            },
        ),
        PackGateRecord::new(
            "pack.reference_view",
            true,
            true,
            PackGateStatus::Passed,
            "reference_view_valid",
        ),
        PackGateRecord::new(
            "pack.annotation_artifact",
            true,
            true,
            PackGateStatus::Passed,
            "annotation_artifact_valid",
        ),
        PackGateRecord::new(
            "pack.projection_coverage",
            false,
            true,
            if projection_clean {
                PackGateStatus::Passed
            } else {
                PackGateStatus::Failed
            },
            if projection_clean {
                "projection_has_no_reported_loss_or_conflict"
            } else {
                "projection_is_partial_or_conflicted"
            },
        ),
        PackGateRecord::new(
            "pack.reference_store",
            false,
            true,
            PackGateStatus::Passed,
            "sealed_reference_store_reopened_read_only",
        ),
        PackGateRecord::new(
            "pack.source_map_loss",
            false,
            true,
            PackGateStatus::NotEvaluated,
            "standalone_source_map_and_loss_members_not_materialized",
        ),
        PackGateRecord::new(
            "pack.parity_consumer",
            false,
            true,
            if consumer_passed {
                PackGateStatus::Passed
            } else {
                PackGateStatus::NotEvaluated
            },
            if consumer_passed {
                "semantic_consumer_acceptance_passed"
            } else {
                "semantic_consumer_acceptance_not_evaluated"
            },
        ),
        PackGateRecord::new(
            "pack.license_provenance",
            false,
            true,
            PackGateStatus::NotEvaluated,
            "redistribution_and_notice_closure_not_evaluated",
        ),
        PackGateRecord::new(
            "pack.deterministic_rebuild",
            false,
            true,
            PackGateStatus::NotEvaluated,
            "rebuild_comparison_not_bound_into_candidate",
        ),
    ]
}

fn finalize_plan(
    request: &ReferencePackBuildRequest,
    parts: &NativePackParts<'_>,
    payload: BuiltPackPayload,
    gates: Vec<PackGateRecord>,
    stop: &AtomicBool,
) -> ReferencePackResult<PackMaterializationPlan> {
    checkpoint(stop)?;
    let BuiltPackPayload {
        entries: payload_entries,
        reference_store,
    } = payload;
    let payload_members = payload_entries
        .iter()
        .map(|entry| entry.member.clone())
        .collect::<Vec<_>>();
    let checksums = PackChecksums {
        schema: "wow-service/reference-pack/checksums/1".into(),
        entries: payload_members
            .iter()
            .map(|member| PackChecksumEntry {
                path: member.path.clone(),
                byte_length: member.byte_length,
                sha256: member.sha256.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    };
    let checksum_bytes = canonical(&checksums, "checksum manifest")?;
    let checksum_member = ReferencePackMember::from_bytes(
        CHECKSUMS_PATH,
        ReferencePackMemberKind::ChecksumManifest,
        sha256(&checksum_bytes),
        &checksum_bytes,
    )?;
    let eligibility_state = if gates
        .iter()
        .all(|gate| !gate.required_for_candidate || gate.status == PackGateStatus::Passed)
    {
        ReferencePackEligibilityState::Candidate
    } else {
        ReferencePackEligibilityState::Blocked
    };
    let deferred_capabilities = vec![
        "standalone_source_map_and_loss".into(),
        "parity_and_consumer_evidence".into(),
        "license_and_redistribution_closure".into(),
        "filesystem_atomic_finalization".into(),
    ]
    .into_boxed_slice();
    let unsigned = UnsignedPackManifest {
        schema: REFERENCE_PACK_SCHEMA,
        layout_profile_id: LOCAL_NATIVE_PACK_LAYOUT,
        profile: parts.profile,
        reference_generation_id: parts.reference.generation_id(),
        reference_view_digest: parts.reference.self_digest(),
        reference_store_profile_id: &reference_store.profile_id,
        reference_store_configuration_id: &reference_store.configuration_id,
        reference_store_manifest_id: &reference_store.manifest_id,
        reference_store_object_id: &reference_store.object_id,
        reference_store_publication_channel: &reference_store.publication_channel,
        annotation_artifact_id: parts.artifact.artifact_id(),
        annotation_payload_sha256: parts.artifact.payload_sha256(),
        source_manifest_sha256: parts.source_manifest_sha256,
        payload_members: &payload_members,
        checksum_member: &checksum_member,
        gate_records: &gates,
        eligibility_state,
        deferred_capabilities: &deferred_capabilities,
    };
    let pack_id = format!(
        "reference-pack:sha256:{}",
        hex(&Sha256::digest(canonical(&unsigned, "pack identity")?))
    )
    .into_boxed_str();
    let manifest = ReferencePackManifest {
        schema: REFERENCE_PACK_SCHEMA.into(),
        pack_id: pack_id.clone(),
        layout_profile_id: LOCAL_NATIVE_PACK_LAYOUT.into(),
        profile: parts.profile.clone(),
        reference_generation_id: parts.reference.generation_id().into(),
        reference_view_digest: parts.reference.self_digest().into(),
        reference_store_profile_id: reference_store.profile_id,
        reference_store_configuration_id: reference_store.configuration_id,
        reference_store_manifest_id: reference_store.manifest_id,
        reference_store_object_id: reference_store.object_id,
        reference_store_publication_channel: reference_store.publication_channel,
        annotation_artifact_id: parts.artifact.artifact_id().into(),
        annotation_payload_sha256: parts.artifact.payload_sha256().into(),
        source_manifest_sha256: parts.source_manifest_sha256.into(),
        payload_members: payload_members.into_boxed_slice(),
        checksum_member: checksum_member.clone(),
        gate_records: gates.into_boxed_slice(),
        eligibility_state,
        deferred_capabilities,
    };
    let manifest_bytes = canonical(&manifest, "pack manifest")?;
    let manifest_member = ReferencePackMember::from_bytes(
        MANIFEST_PATH,
        ReferencePackMemberKind::PackManifest,
        pack_id.clone(),
        &manifest_bytes,
    )?;
    let mut entries = payload_entries;
    entries.push(PackMaterializationEntry::new(
        checksum_member,
        checksum_bytes,
    ));
    entries.push(PackMaterializationEntry::new(
        manifest_member,
        manifest_bytes,
    ));
    entries.sort_by(|left, right| left.member.path.cmp(&right.member.path));
    enforce_entry_budgets(&entries, request.budgets())?;
    let directories = derive_directories(&entries)?;
    let plan_id = plan_id(&pack_id, &entries, &directories)?;
    Ok(PackMaterializationPlan {
        plan_id,
        pack_id,
        layout_profile_id: LOCAL_NATIVE_PACK_LAYOUT.into(),
        directories,
        entries: entries.into_boxed_slice(),
    })
}

#[derive(Serialize)]
struct UnsignedPackManifest<'a> {
    schema: &'static str,
    layout_profile_id: &'static str,
    profile: &'a ProfileIdentity,
    reference_generation_id: &'a str,
    reference_view_digest: &'a str,
    reference_store_profile_id: &'a str,
    reference_store_configuration_id: &'a str,
    reference_store_manifest_id: &'a str,
    reference_store_object_id: &'a str,
    reference_store_publication_channel: &'a str,
    annotation_artifact_id: &'a str,
    annotation_payload_sha256: &'a str,
    source_manifest_sha256: &'a str,
    payload_members: &'a [ReferencePackMember],
    checksum_member: &'a ReferencePackMember,
    gate_records: &'a [PackGateRecord],
    eligibility_state: ReferencePackEligibilityState,
    deferred_capabilities: &'a [Box<str>],
}

fn validate_manifest_identity(manifest: &ReferencePackManifest) -> ReferencePackResult<()> {
    if manifest.schema.as_ref() != REFERENCE_PACK_SCHEMA
        || manifest.layout_profile_id.as_ref() != LOCAL_NATIVE_PACK_LAYOUT
    {
        return Err(error(
            ReferencePackErrorCode::ManifestInvalid,
            "reference pack manifest schema or layout is unsupported",
        ));
    }
    manifest.profile.validate().map_err(|_| {
        error(
            ReferencePackErrorCode::ManifestInvalid,
            "reference pack profile is invalid",
        )
    })?;
    let expected_store_configuration = reference_store_configuration()?;
    if manifest.reference_store_profile_id.as_ref() != REFERENCE_STORE_PROFILE
        || manifest.reference_store_configuration_id.as_ref()
            != expected_store_configuration.configuration_id()
        || manifest.reference_store_publication_channel.as_ref() != REFERENCE_STORE_CHANNEL
        || !valid_identity(&manifest.reference_store_manifest_id)
        || !valid_identity(&manifest.reference_store_object_id)
    {
        return Err(error(
            ReferencePackErrorCode::ManifestInvalid,
            "reference pack store identity is unsupported or invalid",
        ));
    }
    let unsigned = UnsignedPackManifest {
        schema: REFERENCE_PACK_SCHEMA,
        layout_profile_id: LOCAL_NATIVE_PACK_LAYOUT,
        profile: &manifest.profile,
        reference_generation_id: &manifest.reference_generation_id,
        reference_view_digest: &manifest.reference_view_digest,
        reference_store_profile_id: &manifest.reference_store_profile_id,
        reference_store_configuration_id: &manifest.reference_store_configuration_id,
        reference_store_manifest_id: &manifest.reference_store_manifest_id,
        reference_store_object_id: &manifest.reference_store_object_id,
        reference_store_publication_channel: &manifest.reference_store_publication_channel,
        annotation_artifact_id: &manifest.annotation_artifact_id,
        annotation_payload_sha256: &manifest.annotation_payload_sha256,
        source_manifest_sha256: &manifest.source_manifest_sha256,
        payload_members: &manifest.payload_members,
        checksum_member: &manifest.checksum_member,
        gate_records: &manifest.gate_records,
        eligibility_state: manifest.eligibility_state,
        deferred_capabilities: &manifest.deferred_capabilities,
    };
    let expected = format!(
        "reference-pack:sha256:{}",
        hex(&Sha256::digest(canonical(&unsigned, "pack identity")?))
    );
    if manifest.pack_id.as_ref() != expected {
        return Err(error(
            ReferencePackErrorCode::ManifestInvalid,
            "reference pack manifest identity does not match its contents",
        ));
    }
    validate_member_order(&manifest.payload_members)?;
    Ok(())
}

fn validate_materialized_members(
    files: &BTreeMap<Box<str>, Box<[u8]>>,
    manifest: &ReferencePackManifest,
    stop: &AtomicBool,
) -> ReferencePackResult<Vec<PackGateRecord>> {
    checkpoint(stop)?;
    let checksum_bytes = required_bytes(files, CHECKSUMS_PATH)?;
    if checksum_bytes.len() as u64 != manifest.checksum_member.byte_length
        || sha256(checksum_bytes) != manifest.checksum_member.sha256.as_ref()
    {
        return Err(error(
            ReferencePackErrorCode::ValidationFailed,
            "reference pack checksum manifest identity does not match",
        ));
    }
    let checksums: PackChecksums = strict_json(checksum_bytes, "checksum manifest")?;
    if checksums.schema.as_ref() != "wow-service/reference-pack/checksums/1" {
        return Err(error(
            ReferencePackErrorCode::ManifestInvalid,
            "reference pack checksum schema is unsupported",
        ));
    }
    let expected_paths = manifest
        .payload_members
        .iter()
        .map(|member| member.path.as_ref())
        .chain([CHECKSUMS_PATH, MANIFEST_PATH])
        .collect::<BTreeSet<_>>();
    let actual_paths = files.keys().map(AsRef::as_ref).collect::<BTreeSet<_>>();
    if expected_paths != actual_paths || checksums.entries.len() != manifest.payload_members.len() {
        return Err(error(
            ReferencePackErrorCode::ValidationFailed,
            "reference pack member set is incomplete or contains undeclared files",
        ));
    }
    for (member, checksum) in manifest
        .payload_members
        .iter()
        .zip(checksums.entries.iter())
    {
        checkpoint(stop)?;
        if member.path != checksum.path
            || member.byte_length != checksum.byte_length
            || member.sha256 != checksum.sha256
        {
            return Err(error(
                ReferencePackErrorCode::ValidationFailed,
                "reference pack checksum entries do not close over manifest members",
            ));
        }
        let bytes = required_bytes(files, member.path())?;
        if bytes.len() as u64 != member.byte_length || sha256(bytes) != member.sha256.as_ref() {
            return Err(error(
                ReferencePackErrorCode::ValidationFailed,
                "reference pack materialized member digest does not match",
            ));
        }
    }
    let reference: ReferenceView = strict_json(
        required_member_bytes(files, manifest, ReferencePackMemberKind::ReferenceView)?,
        "ReferenceView member",
    )?;
    reference.validate().map_err(|_| {
        error(
            ReferencePackErrorCode::ValidationFailed,
            "reference pack ReferenceView failed owner validation",
        )
    })?;
    if reference.generation_id() != manifest.reference_generation_id.as_ref()
        || reference.self_digest() != manifest.reference_view_digest.as_ref()
    {
        return Err(error(
            ReferencePackErrorCode::IdentityMismatch,
            "reference pack ReferenceView identity does not close",
        ));
    }
    validate_reference_store_member(files, manifest, &reference)?;
    let annotation_manifest: PackAnnotationArtifactManifest = strict_json(
        required_member_bytes(
            files,
            manifest,
            ReferencePackMemberKind::AnnotationArtifactManifest,
        )?,
        "annotation artifact manifest",
    )?;
    validate_annotation_members(files, manifest, &annotation_manifest)?;
    let provenance: PackNativeProvenance = strict_json(
        required_member_bytes(files, manifest, ReferencePackMemberKind::ProvenanceManifest)?,
        "native provenance",
    )?;
    if provenance.profile != manifest.profile
        || provenance.source_manifest_sha256 != manifest.source_manifest_sha256
        || annotation_manifest.profile_id.as_ref() != manifest.profile.profile_id().as_str()
        || annotation_manifest.source_generation_id != manifest.reference_generation_id
        || annotation_manifest.artifact_id != manifest.annotation_artifact_id
        || annotation_manifest.payload_sha256 != manifest.annotation_payload_sha256
    {
        return Err(error(
            ReferencePackErrorCode::IdentityMismatch,
            "reference pack component identities do not close during validation",
        ));
    }
    Ok(recomputed_gates(manifest, &provenance))
}

fn validate_reference_store_member(
    files: &BTreeMap<Box<str>, Box<[u8]>>,
    manifest: &ReferencePackManifest,
    reference: &ReferenceView,
) -> ReferencePackResult<()> {
    let bytes = required_member_bytes(files, manifest, ReferencePackMemberKind::ReferenceStore)?;
    let configuration = reference_store_configuration()?;
    if manifest.reference_store_profile_id.as_ref() != configuration.profile_id()
        || manifest.reference_store_configuration_id.as_ref() != configuration.configuration_id()
    {
        return Err(error(
            ReferencePackErrorCode::IdentityMismatch,
            "reference store configuration identity does not close",
        ));
    }
    let sealed = SealedStore::open_serialized(bytes, configuration.clone(), bytes.len() as u64)
        .map_err(|_| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reference store member failed independent read-only reopen",
            )
        })?;
    let reader = SealedReferenceStore::new(&sealed);
    let integrity = reader
        .validate_integrity(configuration.limits().max_manifest_records)
        .map_err(|_| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reopened reference store failed integrity validation",
            )
        })?;
    if !integrity.complete() {
        return Err(error(
            ReferencePackErrorCode::ValidationFailed,
            "reopened reference store integrity validation was incomplete",
        ));
    }
    let logical_manifest = reader.logical_manifest().map_err(|_| {
        error(
            ReferencePackErrorCode::ValidationFailed,
            "reopened reference store logical manifest is unavailable",
        )
    })?;
    let publication_key = ReferencePublicationKey::new(
        manifest.profile.profile_id().as_str(),
        manifest.reference_store_publication_channel.clone(),
    )
    .map_err(|_| {
        error(
            ReferencePackErrorCode::ManifestInvalid,
            "reference store publication key is invalid",
        )
    })?;
    let published = reader
        .read_current(&publication_key)
        .map_err(|_| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reopened reference store current view failed owner validation",
            )
        })?
        .ok_or_else(|| {
            error(
                ReferencePackErrorCode::ValidationFailed,
                "reopened reference store current view is missing",
            )
        })?;
    if logical_manifest.manifest_id() != manifest.reference_store_manifest_id.as_ref()
        || published.object_id().as_str() != manifest.reference_store_object_id.as_ref()
        || published.view() != reference
        || logical_manifest.objects().len() != 1
        || logical_manifest.catalog_entries().len() != 1
        || !logical_manifest.operations().is_empty()
        || !logical_manifest.leases().is_empty()
    {
        return Err(error(
            ReferencePackErrorCode::IdentityMismatch,
            "reopened reference store does not close over the declared exact view",
        ));
    }
    Ok(())
}

fn validate_annotation_members(
    files: &BTreeMap<Box<str>, Box<[u8]>>,
    manifest: &ReferencePackManifest,
    annotation_manifest: &PackAnnotationArtifactManifest,
) -> ReferencePackResult<()> {
    if annotation_manifest.schema.as_ref()
        != "wow-service/reference-pack/annotation-artifact-manifest/1"
        || annotation_manifest.files.is_empty()
    {
        return Err(error(
            ReferencePackErrorCode::ManifestInvalid,
            "annotation artifact manifest is unsupported or empty",
        ));
    }
    let declared = annotation_manifest
        .files
        .iter()
        .map(|file| file.pack_path.as_ref())
        .collect::<BTreeSet<_>>();
    let members = manifest
        .payload_members
        .iter()
        .filter(|member| member.kind == ReferencePackMemberKind::AnnotationFile)
        .map(|member| member.path.as_ref())
        .collect::<BTreeSet<_>>();
    if declared != members || declared.len() != annotation_manifest.files.len() {
        return Err(error(
            ReferencePackErrorCode::ValidationFailed,
            "annotation file manifest does not close over materialized files",
        ));
    }
    for file in &annotation_manifest.files {
        let bytes = required_bytes(files, &file.pack_path)?;
        if bytes.len() as u64 != file.byte_length || sha256(bytes) != file.sha256.as_ref() {
            return Err(error(
                ReferencePackErrorCode::ValidationFailed,
                "annotation file digest does not match its owner manifest",
            ));
        }
    }
    Ok(())
}

fn recomputed_gates(
    _manifest: &ReferencePackManifest,
    provenance: &PackNativeProvenance,
) -> Vec<PackGateRecord> {
    let projection_clean = provenance.input_failures == 0
        && provenance.reference_issues == 0
        && provenance.reference_conflicts == 0
        && provenance.annotation_issues == 0
        && provenance.annotation_projection.as_ref() != "partial";
    let projection_gate = PackGateRecord::new(
        "pack.projection_coverage",
        false,
        true,
        if projection_clean {
            PackGateStatus::Passed
        } else {
            PackGateStatus::Failed
        },
        if projection_clean {
            "projection_has_no_reported_loss_or_conflict"
        } else {
            "projection_is_partial_or_conflicted"
        },
    );
    vec![
        PackGateRecord::new(
            "pack.structure",
            true,
            true,
            PackGateStatus::Passed,
            "planned_member_set_closed",
        ),
        PackGateRecord::new(
            "pack.source_manifest_closure",
            true,
            true,
            if provenance.generated_api_closure_complete {
                PackGateStatus::Passed
            } else {
                PackGateStatus::Failed
            },
            if provenance.generated_api_closure_complete {
                "exact_manifest_toc_closure"
            } else {
                "source_manifest_incomplete"
            },
        ),
        PackGateRecord::new(
            "pack.reference_view",
            true,
            true,
            PackGateStatus::Passed,
            "reference_view_valid",
        ),
        PackGateRecord::new(
            "pack.annotation_artifact",
            true,
            true,
            PackGateStatus::Passed,
            "annotation_artifact_valid",
        ),
        projection_gate,
        PackGateRecord::new(
            "pack.reference_store",
            false,
            true,
            PackGateStatus::Passed,
            "sealed_reference_store_reopened_read_only",
        ),
        PackGateRecord::new(
            "pack.source_map_loss",
            false,
            true,
            PackGateStatus::NotEvaluated,
            "standalone_source_map_and_loss_members_not_materialized",
        ),
        PackGateRecord::new(
            "pack.parity_consumer",
            false,
            true,
            if provenance.semantic_consumer_acceptance.as_ref() == "passed" {
                PackGateStatus::Passed
            } else {
                PackGateStatus::NotEvaluated
            },
            if provenance.semantic_consumer_acceptance.as_ref() == "passed" {
                "semantic_consumer_acceptance_passed"
            } else {
                "semantic_consumer_acceptance_not_evaluated"
            },
        ),
        PackGateRecord::new(
            "pack.license_provenance",
            false,
            true,
            PackGateStatus::NotEvaluated,
            "redistribution_and_notice_closure_not_evaluated",
        ),
        PackGateRecord::new(
            "pack.deterministic_rebuild",
            false,
            true,
            PackGateStatus::NotEvaluated,
            "rebuild_comparison_not_bound_into_candidate",
        ),
    ]
}

fn semantic_pack_identity(plan: &PackMaterializationPlan) -> ReferencePackResult<Box<str>> {
    let manifest_entry = plan
        .entries
        .iter()
        .find(|entry| entry.member.kind == ReferencePackMemberKind::PackManifest)
        .ok_or_else(|| {
            error(
                ReferencePackErrorCode::ManifestInvalid,
                "rebuild input is missing its pack manifest",
            )
        })?;
    let manifest: ReferencePackManifest = strict_json(&manifest_entry.bytes, "pack manifest")?;
    validate_manifest_identity(&manifest)?;

    #[derive(Serialize)]
    struct SemanticMember<'a> {
        path: &'a str,
        kind: ReferencePackMemberKind,
        logical_id: &'a str,
    }

    #[derive(Serialize)]
    struct SemanticIdentity<'a> {
        schema: &'static str,
        layout_profile_id: &'a str,
        profile: &'a ProfileIdentity,
        reference_generation_id: &'a str,
        reference_view_digest: &'a str,
        reference_store_profile_id: &'a str,
        reference_store_configuration_id: &'a str,
        reference_store_manifest_id: &'a str,
        reference_store_object_id: &'a str,
        reference_store_publication_channel: &'a str,
        annotation_artifact_id: &'a str,
        annotation_payload_sha256: &'a str,
        source_manifest_sha256: &'a str,
        payload_members: Vec<SemanticMember<'a>>,
        gate_records: &'a [PackGateRecord],
        eligibility_state: ReferencePackEligibilityState,
        deferred_capabilities: &'a [Box<str>],
    }

    let payload_members = manifest
        .payload_members
        .iter()
        .map(|member| SemanticMember {
            path: &member.path,
            kind: member.kind,
            logical_id: &member.logical_id,
        })
        .collect::<Vec<_>>();
    let projection = SemanticIdentity {
        schema: REFERENCE_PACK_SCHEMA,
        layout_profile_id: &manifest.layout_profile_id,
        profile: &manifest.profile,
        reference_generation_id: &manifest.reference_generation_id,
        reference_view_digest: &manifest.reference_view_digest,
        reference_store_profile_id: &manifest.reference_store_profile_id,
        reference_store_configuration_id: &manifest.reference_store_configuration_id,
        reference_store_manifest_id: &manifest.reference_store_manifest_id,
        reference_store_object_id: &manifest.reference_store_object_id,
        reference_store_publication_channel: &manifest.reference_store_publication_channel,
        annotation_artifact_id: &manifest.annotation_artifact_id,
        annotation_payload_sha256: &manifest.annotation_payload_sha256,
        source_manifest_sha256: &manifest.source_manifest_sha256,
        payload_members,
        gate_records: &manifest.gate_records,
        eligibility_state: manifest.eligibility_state,
        deferred_capabilities: &manifest.deferred_capabilities,
    };
    Ok(format!(
        "reference-pack-semantic:sha256:{}",
        hex(&Sha256::digest(canonical(
            &projection,
            "semantic pack identity"
        )?))
    )
    .into_boxed_str())
}

fn compare_plans(
    left: &PackMaterializationPlan,
    right: &PackMaterializationPlan,
) -> ReferencePackResult<ReferencePackRebuildComparisonReport> {
    let left_semantic_id = semantic_pack_identity(left)?;
    let right_semantic_id = semantic_pack_identity(right)?;
    let semantic_identity_equal = left_semantic_id == right_semantic_id;
    let left_files = left
        .entries
        .iter()
        .map(|entry| (entry.member.path.as_ref(), entry))
        .collect::<BTreeMap<_, _>>();
    let right_files = right
        .entries
        .iter()
        .map(|entry| (entry.member.path.as_ref(), entry))
        .collect::<BTreeMap<_, _>>();
    let all_paths = left_files
        .keys()
        .chain(right_files.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut differences = Vec::new();
    for path in all_paths {
        match (left_files.get(path), right_files.get(path)) {
            (Some(left_entry), Some(right_entry)) if left_entry.bytes == right_entry.bytes => {}
            (Some(left_entry), Some(right_entry))
                if left_entry.member.kind == ReferencePackMemberKind::ReferenceStore
                    && right_entry.member.kind == ReferencePackMemberKind::ReferenceStore
                    && left_entry.member.logical_id == right_entry.member.logical_id =>
            {
                differences.push(RebuildDifference {
                    comparison_class: "sqlite_physical_bytes".into(),
                    subject: path.into(),
                    left_identity: left_entry.member.sha256.clone(),
                    right_identity: right_entry.member.sha256.clone(),
                    allowed: true,
                });
            }
            (Some(left_entry), Some(right_entry)) => differences.push(RebuildDifference {
                comparison_class: "canonical_bytes".into(),
                subject: path.into(),
                left_identity: left_entry.member.sha256.clone(),
                right_identity: right_entry.member.sha256.clone(),
                allowed: false,
            }),
            (Some(left_entry), None) => differences.push(RebuildDifference {
                comparison_class: "member_set".into(),
                subject: path.into(),
                left_identity: left_entry.member.sha256.clone(),
                right_identity: "missing".into(),
                allowed: false,
            }),
            (None, Some(right_entry)) => differences.push(RebuildDifference {
                comparison_class: "member_set".into(),
                subject: path.into(),
                left_identity: "missing".into(),
                right_identity: right_entry.member.sha256.clone(),
                allowed: false,
            }),
            (None, None) => {}
        }
    }
    if !semantic_identity_equal {
        differences.push(RebuildDifference {
            comparison_class: "semantic_identity".into(),
            subject: "logical_pack".into(),
            left_identity: left_semantic_id,
            right_identity: right_semantic_id,
            allowed: false,
        });
    }
    let canonical_member_bytes_equal = differences.iter().all(|difference| {
        !matches!(
            difference.comparison_class.as_ref(),
            "canonical_bytes" | "member_set"
        )
    });
    let status = if semantic_identity_equal
        && canonical_member_bytes_equal
        && differences.iter().all(|difference| difference.allowed)
    {
        RebuildComparisonStatus::Passed
    } else {
        RebuildComparisonStatus::Failed
    };
    let left_store = left
        .entries
        .iter()
        .find(|entry| entry.member.kind == ReferencePackMemberKind::ReferenceStore);
    let right_store = right
        .entries
        .iter()
        .find(|entry| entry.member.kind == ReferencePackMemberKind::ReferenceStore);
    let sqlite_physical_classification = match (left_store, right_store) {
        (Some(left_store), Some(right_store)) if left_store.bytes == right_store.bytes => {
            "observed_equal_not_contractual"
        }
        (Some(left_store), Some(right_store))
            if left_store.member.logical_id == right_store.member.logical_id =>
        {
            "different_but_logically_equivalent"
        }
        (Some(_), Some(_)) => "different_and_logically_incompatible",
        _ => "missing_reference_store_member",
    };
    let report_id = rebuild_report_id(
        &left.pack_id,
        &right.pack_id,
        semantic_identity_equal,
        canonical_member_bytes_equal,
        sqlite_physical_classification,
        &differences,
    )?;
    Ok(ReferencePackRebuildComparisonReport {
        schema: REFERENCE_PACK_SCHEMA.into(),
        report_id,
        left_pack_id: left.pack_id.clone(),
        right_pack_id: right.pack_id.clone(),
        semantic_identity_equal,
        canonical_member_bytes_equal,
        sqlite_physical_classification: sqlite_physical_classification.into(),
        differences: differences.into_boxed_slice(),
        status,
    })
}

fn admit_image(
    image: &ReferencePackImage,
    budgets: ReferencePackBudgets,
    stop: &AtomicBool,
) -> ReferencePackResult<BTreeMap<Box<str>, Box<[u8]>>> {
    if image.files.is_empty() || image.files.len() > budgets.max_members as usize {
        return Err(error(
            ReferencePackErrorCode::BudgetExceeded,
            "reference pack image member count exceeds budget",
        ));
    }
    let mut total = 0u64;
    let mut files = BTreeMap::new();
    let mut folded = BTreeSet::new();
    for file in &image.files {
        checkpoint(stop)?;
        validate_member_path(&file.path)?;
        let length = file.bytes.len() as u64;
        total = total.checked_add(length).ok_or_else(|| {
            error(
                ReferencePackErrorCode::BudgetExceeded,
                "reference pack image byte count overflow",
            )
        })?;
        if length > budgets.max_member_bytes
            || total > budgets.max_total_bytes
            || !folded.insert(file.path.to_ascii_lowercase())
            || files
                .insert(file.path.clone(), file.bytes.clone())
                .is_some()
        {
            return Err(error(
                ReferencePackErrorCode::BudgetExceeded,
                "reference pack image violates path or byte budgets",
            ));
        }
    }
    Ok(files)
}

fn required_member_bytes<'a>(
    files: &'a BTreeMap<Box<str>, Box<[u8]>>,
    manifest: &ReferencePackManifest,
    kind: ReferencePackMemberKind,
) -> ReferencePackResult<&'a [u8]> {
    let matches = manifest
        .payload_members
        .iter()
        .filter(|member| member.kind == kind)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(error(
            ReferencePackErrorCode::ValidationFailed,
            "reference pack requires exactly one member of this kind",
        ));
    }
    required_bytes(files, matches[0].path())
}

fn required_bytes<'a>(
    files: &'a BTreeMap<Box<str>, Box<[u8]>>,
    path: &str,
) -> ReferencePackResult<&'a [u8]> {
    files.get(path).map(AsRef::as_ref).ok_or_else(|| {
        error(
            ReferencePackErrorCode::ValidationFailed,
            "required reference pack member is missing",
        )
    })
}

fn strict_json<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    label: &'static str,
) -> ReferencePackResult<T> {
    serde_json::from_slice(bytes).map_err(|_| {
        error(
            ReferencePackErrorCode::ManifestInvalid,
            format!("{label} failed strict decoding"),
        )
    })
}

fn entry(
    path: impl Into<Box<str>>,
    kind: ReferencePackMemberKind,
    logical_id: impl Into<Box<str>>,
    bytes: Vec<u8>,
) -> ReferencePackResult<PackMaterializationEntry> {
    let member = ReferencePackMember::from_bytes(path, kind, logical_id, &bytes)?;
    Ok(PackMaterializationEntry::new(member, bytes))
}

fn enforce_entry_budgets(
    entries: &[PackMaterializationEntry],
    budgets: ReferencePackBudgets,
) -> ReferencePackResult<()> {
    if entries.len() > budgets.max_members as usize {
        return Err(error(
            ReferencePackErrorCode::BudgetExceeded,
            "reference pack member count exceeds budget",
        ));
    }
    let mut total = 0u64;
    let mut folded = BTreeSet::new();
    for entry in entries {
        let length = entry.bytes.len() as u64;
        total = total.checked_add(length).ok_or_else(|| {
            error(
                ReferencePackErrorCode::BudgetExceeded,
                "reference pack byte count overflow",
            )
        })?;
        if length > budgets.max_member_bytes
            || total > budgets.max_total_bytes
            || !folded.insert(entry.member.path.to_ascii_lowercase())
        {
            return Err(error(
                ReferencePackErrorCode::BudgetExceeded,
                "reference pack member violates path or byte budgets",
            ));
        }
    }
    Ok(())
}

fn derive_directories(
    entries: &[PackMaterializationEntry],
) -> ReferencePackResult<Box<[Box<str>]>> {
    let mut directories = BTreeSet::new();
    for entry in entries {
        let mut path = entry.member.path.as_ref();
        while let Some((parent, _)) = path.rsplit_once('/') {
            validate_member_path(parent)?;
            directories.insert(parent.to_owned().into_boxed_str());
            path = parent;
        }
    }
    Ok(directories
        .into_iter()
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn plan_id(
    pack_id: &str,
    entries: &[PackMaterializationEntry],
    directories: &[Box<str>],
) -> ReferencePackResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        pack_id: &'a str,
        layout_profile_id: &'static str,
        members: Vec<&'a ReferencePackMember>,
        directories: &'a [Box<str>],
    }
    let identity = Identity {
        schema: "wow-service/reference-pack/materialization-plan/1",
        pack_id,
        layout_profile_id: LOCAL_NATIVE_PACK_LAYOUT,
        members: entries.iter().map(|entry| &entry.member).collect(),
        directories,
    };
    Ok(format!(
        "pack-plan:sha256:{}",
        hex(&Sha256::digest(canonical(
            &identity,
            "materialization plan"
        )?))
    )
    .into())
}

fn validation_report_id(
    pack_id: &str,
    gates: &[PackGateRecord],
    candidate_eligible: bool,
    validated_local_eligible: bool,
) -> ReferencePackResult<Box<str>> {
    Ok(format!(
        "pack-validation:sha256:{}",
        hex(&Sha256::digest(canonical(
            &(
                LOCAL_NATIVE_VALIDATION_PROFILE,
                pack_id,
                gates,
                candidate_eligible,
                validated_local_eligible,
            ),
            "validation report",
        )?))
    )
    .into())
}

fn rebuild_report_id(
    left_pack_id: &str,
    right_pack_id: &str,
    semantic_identity_equal: bool,
    canonical_member_bytes_equal: bool,
    sqlite_physical_classification: &str,
    differences: &[RebuildDifference],
) -> ReferencePackResult<Box<str>> {
    Ok(format!(
        "pack-rebuild:sha256:{}",
        hex(&Sha256::digest(canonical(
            &(
                left_pack_id,
                right_pack_id,
                semantic_identity_equal,
                canonical_member_bytes_equal,
                sqlite_physical_classification,
                differences,
            ),
            "rebuild report",
        )?))
    )
    .into())
}

fn validate_member_order(members: &[ReferencePackMember]) -> ReferencePackResult<()> {
    if members
        .windows(2)
        .any(|pair| pair[0].path.as_ref() >= pair[1].path.as_ref())
    {
        return Err(error(
            ReferencePackErrorCode::ManifestInvalid,
            "reference pack members are not in canonical path order",
        ));
    }
    for member in members {
        validate_member_path(&member.path)?;
        if member.byte_length == 0 || !valid_sha256(&member.sha256) {
            return Err(error(
                ReferencePackErrorCode::ManifestInvalid,
                "reference pack member identity is invalid",
            ));
        }
    }
    Ok(())
}

fn validate_member_path(path: &str) -> ReferencePackResult<()> {
    if path.is_empty()
        || path.len() > 512
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
    {
        return Err(error(
            ReferencePackErrorCode::MemberInvalid,
            "reference pack member path is unsafe",
        ));
    }
    for segment in path.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") || reserved_windows_name(segment) {
            return Err(error(
                ReferencePackErrorCode::MemberInvalid,
                "reference pack member path contains a forbidden segment",
            ));
        }
    }
    Ok(())
}

fn reserved_windows_name(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn canonical(value: &impl Serialize, label: &'static str) -> ReferencePackResult<Vec<u8>> {
    canonical_json_bytes(value).map_err(|_| {
        error(
            ReferencePackErrorCode::SerializationFailed,
            format!("{label} cannot be canonicalized"),
        )
    })
}

fn checkpoint(stop: &AtomicBool) -> ReferencePackResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(error(
            ReferencePackErrorCode::Cancelled,
            "reference pack operation cancelled",
        ))
    } else {
        Ok(())
    }
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn error(code: ReferencePackErrorCode, message: impl Into<Box<str>>) -> ReferencePackError {
    ReferencePackError::new(code, message)
}
