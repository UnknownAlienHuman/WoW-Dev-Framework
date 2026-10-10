//! Retained raw bytes re-enter the same admission owner as confined disk reads.
//! Serialized receipts never construct an admitted source.
use std::{collections::BTreeMap, sync::atomic::AtomicBool};

use serde::{Deserialize, Serialize};
use wow_core::CoverageStatus;

use super::{
    AdmittedPlatformSource, BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest,
    PlatformAdmissionCoverage, PlatformEntryDisposition, PlatformSourceAdmissionReceipt,
    PlatformSourceInventory, PlatformUnevaluatedCapability, budget, invalid, lfs_pointer, path,
};
use crate::{ProjectPhase, ProjectResult, disk, disk::ProjectDiskFile, identity};

/// Binary records retain otherwise unconsumed or undecodable Included members.
/// A sequence preserves duplicate paths so admission can reject them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedPlatformFile {
    pub(crate) path: String,
    pub(crate) bytes: Vec<u8>,
}

impl AdmittedPlatformSource {
    pub(crate) fn retained_files(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
    }

    pub(crate) fn readmit(
        request: BlizzardUiSourceProfileRequest,
        inventory: PlatformSourceInventory,
        files: &[RetainedPlatformFile],
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        disk::checkpoint(stop)?;
        let profile = BlizzardUiSourceProfile::new(request)?;
        let inventory = inventory.canonicalize(&profile, stop)?;
        // Validate the complete borrowed corpus before copying any raw byte.
        validate_members(
            &profile,
            &inventory,
            files
                .iter()
                .map(|file| (file.path.as_str(), file.bytes.as_slice())),
            stop,
        )?;
        let mut retained = BTreeMap::new();
        for file in files {
            disk::checkpoint(stop)?;
            retained.insert(file.path.clone(), file.bytes.clone());
        }
        finish_admission(&profile, inventory, retained, stop)
    }
}

fn validate_members<'a>(
    profile: &BlizzardUiSourceProfile,
    inventory: &PlatformSourceInventory,
    files: impl Iterator<Item = (&'a str, &'a [u8])>,
    stop: &AtomicBool,
) -> ProjectResult<u64> {
    let mut expected = inventory.entries.iter().filter_map(|entry| {
        if let PlatformEntryDisposition::Included {
            digest,
            byte_length,
            ..
        } = entry.disposition
        {
            Some((entry.path.as_str(), digest, byte_length))
        } else {
            None
        }
    });
    let mut total = 0u64;
    let mut count = 0usize;
    for (selected, bytes) in files {
        disk::checkpoint(stop)?;
        path(selected)?;
        let (expected_path, digest, length) = expected
            .next()
            .ok_or_else(|| invalid("retained platform bytes contain a surplus member"))?;
        if selected != expected_path {
            return Err(invalid(
                "retained platform bytes differ from the exact Included set",
            ));
        }
        count = count
            .checked_add(1)
            .filter(|count| *count <= profile.limits().max_entries)
            .ok_or_else(|| budget("retained platform member count exceeds its limit"))?;
        total = total
            .checked_add(bytes.len() as u64)
            .filter(|total| *total <= profile.limits().max_total_bytes)
            .ok_or_else(|| budget("retained platform bytes exceed their total limit"))?;
        if bytes.len() as u64 > profile.limits().max_file_bytes {
            return Err(budget("retained platform member exceeds its byte limit"));
        }
        ProjectDiskFile::new(selected)
            .with_identity(digest, length)
            .verify(bytes)?;
        if lfs_pointer(bytes) {
            return Err(invalid("unmaterialized LFS pointer is not source content"));
        }
    }
    if expected.next().is_some() {
        return Err(invalid("retained platform bytes omit an Included member"));
    }
    disk::checkpoint(stop)?;
    Ok(total)
}

/// Private shared finalizer: both paths derive the original /1 identities from
/// validated requests, inventory and actually observed bytes.
pub(super) fn finish_admission(
    profile: &BlizzardUiSourceProfile,
    inventory: PlatformSourceInventory,
    files: BTreeMap<String, Vec<u8>>,
    stop: &AtomicBool,
) -> ProjectResult<AdmittedPlatformSource> {
    let inventory = inventory.canonicalize(profile, stop)?;
    let verified_bytes = validate_members(
        profile,
        &inventory,
        files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice())),
        stop,
    )?;
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
