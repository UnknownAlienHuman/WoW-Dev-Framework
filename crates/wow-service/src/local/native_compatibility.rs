//! Explicit admission of external annotation parity and consumer-probe evidence.
//!
//! The host provides digest-pinned canonical JSON reports. `wow-annotations`
//! validates their semantics and exact artifact/profile/generation binding; this
//! service layer only performs bounded acquisition and retains a compact receipt.

use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_annotations::compatibility::{
    MAX_COMPATIBILITY_REPORT_BYTES, NativeCompatibilityBinding, NativeCompatibilityErrorCode,
    NativeCompatibilityEvidence,
};
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_reference::native::source_digest;

use super::cancelled;
use super::disk_input::acquisition_error;
use super::input::invalid;
use super::native_input::NativeFileIdentity;
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

const MAX_CONSUMER_REPORTS: usize = 8;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeCompatibilityInputs {
    parity_report: ProjectDiskFile,
    consumer_probe_results: Vec<ProjectDiskFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeConsumerProbeIdentity {
    pub consumer_kind: &'static str,
    pub result_id: String,
    pub file: NativeFileIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeCompatibilitySelection {
    pub schema: &'static str,
    pub evidence_id: String,
    pub parity_report_id: String,
    pub parity_report: NativeFileIdentity,
    pub consumer_probe_results: Vec<NativeConsumerProbeIdentity>,
    pub gate_status: &'static str,
    pub execution_authority: &'static str,
    pub review_authority: &'static str,
}

pub(super) struct LoadedNativeCompatibility {
    pub(super) evidence: NativeCompatibilityEvidence,
    pub(super) selection: NativeCompatibilitySelection,
}

impl NativeCompatibilityInputs {
    pub(super) fn read(
        &self,
        directory: &ProjectInputDirectory,
        binding: NativeCompatibilityBinding,
        stop: &AtomicBool,
    ) -> ServiceResult<LoadedNativeCompatibility> {
        cancelled(stop)?;
        if self.consumer_probe_results.is_empty()
            || self.consumer_probe_results.len() > MAX_CONSUMER_REPORTS
        {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "native consumer probe report count is outside the reviewed profile",
            ));
        }
        let mut seen = BTreeSet::new();
        for file in std::iter::once(&self.parity_report).chain(&self.consumer_probe_results) {
            if !seen.insert(file.path().to_ascii_lowercase()) {
                return Err(invalid(
                    "native compatibility evidence paths collide ignoring case",
                ));
            }
        }
        let parity_bytes = directory
            .read_json_artifact_with_limit(
                &self.parity_report,
                MAX_COMPATIBILITY_REPORT_BYTES,
                stop,
            )
            .map_err(acquisition_error)?;
        let mut consumer_bytes = Vec::with_capacity(self.consumer_probe_results.len());
        for file in &self.consumer_probe_results {
            cancelled(stop)?;
            consumer_bytes.push(
                directory
                    .read_json_artifact_with_limit(file, MAX_COMPATIBILITY_REPORT_BYTES, stop)
                    .map_err(acquisition_error)?,
            );
        }
        let consumer_slices = consumer_bytes.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let evidence = NativeCompatibilityEvidence::from_canonical_reports(
            binding,
            &parity_bytes,
            &consumer_slices,
        )
        .map_err(compatibility_error)?;
        let parity_report = NativeFileIdentity {
            path: self.parity_report.path().to_owned(),
            sha256: source_digest(&parity_bytes),
            byte_length: parity_bytes.len() as u64,
        };
        let consumer_probe_results = evidence
            .consumers()
            .iter()
            .map(|artifact| {
                let index = consumer_bytes
                    .iter()
                    .position(|bytes| bytes.as_slice() == artifact.bytes())
                    .ok_or_else(|| invalid("native consumer probe source identity was lost"))?;
                let file = &self.consumer_probe_results[index];
                let bytes = &consumer_bytes[index];
                Ok(NativeConsumerProbeIdentity {
                    consumer_kind: artifact.result().consumer_kind().as_str(),
                    result_id: artifact.result().result_id().to_owned(),
                    file: NativeFileIdentity {
                        path: file.path().to_owned(),
                        sha256: source_digest(bytes),
                        byte_length: bytes.len() as u64,
                    },
                })
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let selection = NativeCompatibilitySelection {
            schema: "wow-service/native-compatibility-selection/1",
            evidence_id: evidence.evidence_id().to_owned(),
            parity_report_id: evidence.parity_report().report_id().to_owned(),
            parity_report,
            consumer_probe_results,
            gate_status: evidence.status().as_str(),
            execution_authority: "external_adapter_exact_reports",
            review_authority: "not_authenticated_by_library",
        };
        cancelled(stop)?;
        Ok(LoadedNativeCompatibility {
            evidence,
            selection,
        })
    }
}

fn compatibility_error(
    error: wow_annotations::compatibility::NativeCompatibilityError,
) -> ServiceError {
    match error.code() {
        NativeCompatibilityErrorCode::InputLimit => ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "native compatibility evidence exceeds the reviewed budget",
        ),
        _ => invalid("native compatibility evidence failed owner validation"),
    }
}
