//! Explicit bounded observation selection; it cannot reconstruct native authority.
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use wow_project::graph::{
    PlatformGraphProvenance,
    fingerprints::{
        FunctionFingerprintLimits, FunctionFingerprintManifest,
        build_platform_function_fingerprints,
    },
};

use super::{checkpoint, error, project_error};
use crate::{ServiceErrorCode, ServiceResult};

const SELECTION_SCHEMA: &str = "wow-service/function-fingerprint-selection/1";

/// Exact current physical function IDs and cumulative observation limits.
/// Selection says nothing about functions omitted from this request.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionFingerprintSelection {
    schema: &'static str,
    function_ids: Vec<Box<str>>,
    max_records: usize,
    max_source_bytes: usize,
    max_metadata_bytes: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSelection {
    schema: String,
    function_ids: Vec<Box<str>>,
    max_records: usize,
    max_source_bytes: usize,
    max_metadata_bytes: usize,
}

impl FunctionFingerprintSelection {
    pub fn new(
        function_ids: Vec<Box<str>>,
        max_records: usize,
        max_source_bytes: usize,
        max_metadata_bytes: usize,
    ) -> ServiceResult<Self> {
        if max_records == 0 || max_source_bytes == 0 || max_metadata_bytes == 0 {
            return Err(error(ServiceErrorCode::InvalidRequest));
        }
        if function_ids.len() > max_records {
            return Err(error(ServiceErrorCode::BudgetExceeded));
        }
        let selection = Self {
            schema: SELECTION_SCHEMA,
            function_ids,
            max_records,
            max_source_bytes,
            max_metadata_bytes,
        };
        // Bound the entire selection before any later request clone or native lookup.
        super::preflight_cancellable(
            &selection,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            &AtomicBool::new(false),
        )?;
        Ok(selection)
    }

    pub fn from_json_slice(bytes: &[u8]) -> ServiceResult<Self> {
        Self::from_json_slice_cancellable(bytes, &AtomicBool::new(false))
    }

    pub fn from_json_slice_cancellable(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        checkpoint(stop)?;
        let wire: WireSelection = super::super::input::decode(
            bytes,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            super::super::GraphReadStage::Request,
            stop,
        )
        .map_err(|failure| error(failure.code))?;
        if wire.schema != SELECTION_SCHEMA {
            return Err(error(ServiceErrorCode::InvalidRequest));
        }
        let selection = Self::new(
            wire.function_ids,
            wire.max_records,
            wire.max_source_bytes,
            wire.max_metadata_bytes,
        )?;
        checkpoint(stop)?;
        Ok(selection)
    }

    pub(super) fn build(
        &self,
        direct: &PlatformGraphProvenance<'_>,
        stop: &AtomicBool,
    ) -> ServiceResult<FunctionFingerprintManifest> {
        checkpoint(stop)?;
        let ids = self
            .function_ids
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();
        let limits = FunctionFingerprintLimits::new(
            self.max_records,
            self.max_source_bytes,
            self.max_metadata_bytes,
        )
        .map_err(project_error)?;
        build_platform_function_fingerprints(direct, &ids, limits, stop).map_err(|failure| {
            if failure.code() == wow_project::ProjectErrorCode::InvalidFingerprintSelection {
                error(ServiceErrorCode::InvalidRequest)
            } else {
                project_error(failure)
            }
        })
    }
}
