//! Admission of explicit local platform-source bytes, before package or semantic
//! indexing. Materializer assertions remain separate from owner-observed bytes.
mod model;
mod package_binding;
mod packages;
mod profile;

pub use package_binding::PlatformPackageBinding;
pub use packages::PlatformPackageSpecialization;

pub use model::{
    PlatformEntryDisposition, PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope,
    PlatformLicenseRecord, PlatformLicenseState, PlatformMaterializer, PlatformRootInventory,
    PlatformSourceInventory, PlatformSourceOrigin, PlatformSourceRevision, PlatformSpecialEntry,
};
pub use profile::{
    BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformRootSpec, PlatformSourceClass,
    PlatformTarget, ProfileExclusion, SourceAdmissionLimits,
};

use std::{collections::BTreeMap, io::Write, sync::atomic::AtomicBool};

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, CoverageStatus};

use crate::{
    ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult,
    disk::{self, ProjectDiskFile, ProjectInputDirectory},
    identity,
};

/// Capabilities that byte admission does not evaluate or authorize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformUnevaluatedCapability {
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

/// Observed byte coverage is restricted to explicitly declared included members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformAdmissionCoverage {
    inventory: CoverageStatus,
    declared_included_bytes: CoverageStatus,
    verified_files: usize,
    verified_bytes: u64,
    unevaluated: Vec<PlatformUnevaluatedCapability>,
}
impl PlatformAdmissionCoverage {
    #[must_use]
    pub const fn inventory(&self) -> CoverageStatus {
        self.inventory
    }
    #[must_use]
    pub const fn declared_included_bytes(&self) -> CoverageStatus {
        self.declared_included_bytes
    }
    #[must_use]
    pub const fn verified_files(&self) -> usize {
        self.verified_files
    }
    #[must_use]
    pub const fn verified_bytes(&self) -> u64 {
        self.verified_bytes
    }
    #[must_use]
    pub fn unevaluated(&self) -> &[PlatformUnevaluatedCapability] {
        &self.unevaluated
    }
}

/// Serialize-only evidence. Decoded evidence cannot construct admitted bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformSourceAdmissionReceipt {
    schema: &'static str,
    profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    source_snapshot_id: Box<str>,
    admission_digest: ContentDigest<CanonicalResult>,
    coverage: PlatformAdmissionCoverage,
    inventory: PlatformSourceInventory,
}
impl PlatformSourceAdmissionReceipt {
    #[must_use]
    pub const fn profile_digest(&self) -> ContentDigest<CanonicalResult> {
        self.profile_digest
    }
    #[must_use]
    pub const fn content_manifest_digest(&self) -> ContentDigest<CanonicalResult> {
        self.content_manifest_digest
    }
    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        &self.source_snapshot_id
    }
    #[must_use]
    pub const fn admission_digest(&self) -> ContentDigest<CanonicalResult> {
        self.admission_digest
    }
    #[must_use]
    pub const fn coverage(&self) -> &PlatformAdmissionCoverage {
        &self.coverage
    }
    /// Full caller assertions, including omissions, license and exact provenance.
    #[must_use]
    pub const fn inventory(&self) -> &PlatformSourceInventory {
        &self.inventory
    }
}

/// Owner-held local bytes. This is not a complete platform project/snapshot or a
/// redistributable source artifact. Later consumers use these retained bytes.
pub struct AdmittedPlatformSource {
    profile: BlizzardUiSourceProfile,
    receipt: PlatformSourceAdmissionReceipt,
    files: BTreeMap<String, Vec<u8>>,
}
impl AdmittedPlatformSource {
    #[must_use]
    pub const fn profile(&self) -> &BlizzardUiSourceProfile {
        &self.profile
    }
    #[must_use]
    pub const fn receipt(&self) -> &PlatformSourceAdmissionReceipt {
        &self.receipt
    }
    /// Exact local input bytes; no disk reread, normalization or semantic claim.
    pub fn source_bytes(&self, selected: &str) -> ProjectResult<&[u8]> {
        path(selected)?;
        self.files.get(selected).map(Vec::as_slice).ok_or_else(|| {
            ProjectError::new(
                ProjectErrorCode::FileNotPresent,
                ProjectPhase::Inventory,
                "path is not a verified included platform member",
            )
            .with_relative_path(selected)
        })
    }
}

impl ProjectInputDirectory {
    /// Verify every declared included member using the existing confined raw
    /// owner. No source acquisition, enumeration, execution or publication occurs.
    pub fn admit_platform_source(
        &self,
        profile: &BlizzardUiSourceProfile,
        inventory: PlatformSourceInventory,
        stop: &AtomicBool,
    ) -> ProjectResult<AdmittedPlatformSource> {
        disk::checkpoint(stop)?;
        profile.validate()?;
        let inventory = inventory.canonicalize(profile, stop)?;
        let mut roots = BTreeMap::new();
        for root in profile.roots() {
            disk::checkpoint(stop)?;
            roots.insert(root.root.as_str(), self.subdirectory(&root.root)?);
        }
        let limits = profile.limits();
        let mut files = BTreeMap::new();
        let mut verified_bytes = 0u64;
        for entry in &inventory.entries {
            disk::checkpoint(stop)?;
            if let PlatformEntryDisposition::Included {
                digest,
                byte_length,
                ..
            } = &entry.disposition
            {
                let (root, directory) = roots
                    .iter()
                    .find(|(root, _)| under(&entry.path, root))
                    .ok_or_else(|| invalid("included member has no configured root"))?;
                let relative = entry
                    .path
                    .strip_prefix(*root)
                    .and_then(|value| value.strip_prefix('/'))
                    .ok_or_else(|| invalid("included member is not a root descendant"))?;
                let remaining = limits
                    .max_total_bytes
                    .checked_sub(verified_bytes)
                    .ok_or_else(|| budget("platform source total byte budget exhausted"))?;
                let limit = usize::try_from(remaining.min(limits.max_file_bytes))
                    .map_err(|_| budget("platform source byte budget is not representable"))?;
                let selected = ProjectDiskFile::new(relative).with_identity(*digest, *byte_length);
                let bytes = directory.read(&selected, limit, stop)?;
                if lfs_pointer(&bytes) {
                    return Err(invalid("unmaterialized LFS pointer is not source content"));
                }
                verified_bytes = verified_bytes
                    .checked_add(bytes.len() as u64)
                    .ok_or_else(|| budget("platform source byte accounting overflow"))?;
                files.insert(entry.path.clone(), bytes);
            }
        }
        disk::checkpoint(stop)?;
        let root_names: Vec<_> = inventory.roots.iter().map(|root| &root.root).collect();
        let content_manifest_digest = identity::canonical_digest(
            "wow-project/platform-source-content-manifest/1",
            &(&root_names, &inventory.entries),
            ProjectPhase::Inventory,
        )?;
        let source_snapshot_id = identity::canonical_id(
            "blizzard-ui-source-snapshot:",
            "wow-project/platform-source-snapshot/1",
            &(
                profile.digest(),
                &inventory.target,
                &inventory.origin.revision,
                content_manifest_digest,
            ),
            ProjectPhase::Inventory,
        )?;
        let admission_digest = identity::canonical_digest(
            "wow-project/platform-source-admission/1",
            &inventory,
            ProjectPhase::Inventory,
        )?;
        let coverage = PlatformAdmissionCoverage {
            inventory: CoverageStatus::Partial,
            declared_included_bytes: if files.is_empty() {
                CoverageStatus::NotApplicable
            } else {
                CoverageStatus::Complete
            },
            verified_files: files.len(),
            verified_bytes,
            unevaluated: vec![
                PlatformUnevaluatedCapability::RootCompleteness,
                PlatformUnevaluatedCapability::GitMembership,
                PlatformUnevaluatedCapability::MaterializerSecurity,
                PlatformUnevaluatedCapability::ClientCompatibility,
                PlatformUnevaluatedCapability::LicensePermission,
                PlatformUnevaluatedCapability::Decoding,
                PlatformUnevaluatedCapability::PackageLoad,
                PlatformUnevaluatedCapability::Analyzer,
                PlatformUnevaluatedCapability::Graph,
                PlatformUnevaluatedCapability::ApiContract,
                PlatformUnevaluatedCapability::Runtime,
            ],
        };
        let receipt = PlatformSourceAdmissionReceipt {
            schema: "wow-project/platform-source-admission/1",
            profile_digest: profile.digest(),
            content_manifest_digest,
            source_snapshot_id,
            admission_digest,
            coverage,
            inventory,
        };
        disk::checkpoint(stop)?;
        Ok(AdmittedPlatformSource {
            profile: profile.clone(),
            receipt,
            files,
        })
    }
}

fn lfs_pointer(bytes: &[u8]) -> bool {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    bytes
        .split(|byte| *byte == b'\n')
        .next()
        .is_some_and(|line| {
            line.strip_suffix(b"\r").unwrap_or(line)
                == b"version https://git-lfs.github.com/spec/v1"
        })
}

pub(super) fn invalid(message: &'static str) -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::InvalidInputInventory,
        ProjectPhase::Inventory,
        message,
    )
}
pub(super) fn budget(message: &'static str) -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::Inventory,
        message,
    )
}
pub(super) fn text(value: &str) -> ProjectResult<()> {
    if value.is_empty()
        || value.len() > 512
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.starts_with('/')
        || value.contains(':')
        || value.contains('\\')
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "main" | "master" | "live" | "latest" | "current" | "head" | "default" | "auto"
        )
    {
        return Err(invalid("platform metadata must be bounded exact text"));
    }
    Ok(())
}
pub(super) fn path(value: &str) -> ProjectResult<()> {
    disk::validate_path(value)?;
    if !value.is_ascii() {
        return Err(invalid(
            "platform path exceeds the supported ASCII case profile",
        ));
    }
    Ok(())
}
pub(super) fn under(selected: &str, root: &str) -> bool {
    selected
        .strip_prefix(root)
        .is_some_and(|tail| tail.starts_with('/'))
}
pub(super) fn serialized_size<T: Serialize>(value: &T, limit: usize) -> ProjectResult<()> {
    struct Counter {
        remaining: usize,
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.remaining = self
                .remaining
                .checked_sub(bytes.len())
                .ok_or_else(|| std::io::Error::other("bounded serialization exceeded"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { remaining: limit }, value)
        .map_err(|_| budget("platform metadata exceeds its serialization budget"))
}
