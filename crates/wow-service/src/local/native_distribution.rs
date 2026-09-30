//! Explicit admission of pack-wide license, notice and redistribution evidence.
//!
//! The selected JSON is digest-pinned and canonical. Legal review remains external;
//! this layer performs bounded acquisition and retains a compact nonsemantic receipt.

use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_reference::native::source_digest;

use super::cancelled;
use super::disk_input::acquisition_error;
use super::input::invalid;
use super::native_input::NativeFileIdentity;
use crate::reference_pack_license::{
    MAX_DISTRIBUTION_MANIFEST_BYTES, NativeDistributionBinding, NativeDistributionErrorCode,
    NativeDistributionManifest,
};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeDistributionInputs {
    manifest: ProjectDiskFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeDistributionSelection {
    pub schema: &'static str,
    pub manifest_id: String,
    pub manifest: NativeFileIdentity,
    pub gate_status: &'static str,
    pub review_authority: &'static str,
}

pub(super) struct LoadedNativeDistribution {
    pub(super) evidence: NativeDistributionManifest,
    pub(super) selection: NativeDistributionSelection,
}

impl NativeDistributionInputs {
    pub(super) fn read(
        &self,
        directory: &ProjectInputDirectory,
        binding: NativeDistributionBinding,
        stop: &AtomicBool,
    ) -> ServiceResult<LoadedNativeDistribution> {
        cancelled(stop)?;
        let bytes = directory
            .read_json_artifact_with_limit(&self.manifest, MAX_DISTRIBUTION_MANIFEST_BYTES, stop)
            .map_err(acquisition_error)?;
        let evidence = NativeDistributionManifest::from_canonical_slice(binding, &bytes)
            .map_err(distribution_error)?;
        let selection = NativeDistributionSelection {
            schema: "wow-service/native-distribution-selection/1",
            manifest_id: evidence.manifest_id().to_owned(),
            manifest: NativeFileIdentity {
                path: self.manifest.path().to_owned(),
                sha256: source_digest(&bytes),
                byte_length: bytes.len() as u64,
            },
            gate_status: evidence.status().as_str(),
            review_authority: "external_explicit_review_not_authenticated_by_library",
        };
        cancelled(stop)?;
        Ok(LoadedNativeDistribution {
            evidence,
            selection,
        })
    }
}

fn distribution_error(
    error: crate::reference_pack_license::NativeDistributionError,
) -> ServiceError {
    match error.code() {
        NativeDistributionErrorCode::InputLimit => ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "native distribution evidence exceeds the reviewed budget",
        ),
        _ => invalid("native distribution evidence failed owner validation"),
    }
}
