use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::atomic::AtomicBool};
use wow_core::ProfileIdentity;
use wow_graph::{GraphEvidenceResolveLimits, GraphExplainLimits, GraphNeighborReadLimits};
use wow_project::ProjectId;
use wow_reference::ReferenceView;

use super::{CONTEXT_MAP_RESULT_MAX_BYTES, checkpoint, failure, request};
use crate::{ServiceErrorCode, ServiceResult};

pub const CONTEXT_MAP_HOST_SCHEMA: &str = "wow-service/context-map-native-host/1";
pub const CONTEXT_MAP_CONTINUATION_HOST_SCHEMA: &str = "wow-service/context-map-native-host/2";
pub const CONTEXT_MAP_HOST_INPUT_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Strict inert host registration. Never serialized into a semantic request/result.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextMapHostInput {
    schema: Box<str>,
    pub(super) project_store_id: Box<str>,
    pub(super) project_store_root: PathBuf,
    #[serde(default)]
    pub(super) continuation_store_file: Option<PathBuf>,
    pub(super) project_id: ProjectId,
    pub(super) reference_profile: ProfileIdentity,
    pub(super) reference: ReferenceView,
    pub(super) limits: Limits,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Limits {
    pub neighbors: GraphNeighborReadLimits,
    pub explain: GraphExplainLimits,
    pub evidence: GraphEvidenceResolveLimits,
    pub max_direct_edges: u32,
    pub max_input_bytes: u64,
    pub max_metadata_bytes: u64,
    pub max_context_work: u64,
}
impl Limits {
    pub fn owner_limits(&self) -> ServiceResult<wow_context::PrimaryMapLimits> {
        if self.max_metadata_bytes > CONTEXT_MAP_RESULT_MAX_BYTES as u64
            || self.max_input_bytes > 64 * 1024 * 1024
        {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        self.evidence.validate().map_err(super::graph_error)?;
        wow_context::PrimaryMapLimits::new(
            self.neighbors,
            self.explain,
            self.max_direct_edges,
            self.max_input_bytes,
            self.max_metadata_bytes,
            self.max_context_work,
        )
        .map_err(super::context_error)
    }
}
impl ContextMapHostInput {
    /// Pure bounded transport admission. No filesystem/store/source acquisition.
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let host: Self = crate::graph::input::decode(
            bytes,
            CONTEXT_MAP_HOST_INPUT_MAX_BYTES,
            2_000_000,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        host.validate(stop)?;
        Ok(host)
    }
    pub(super) fn validate(&self, stop: &AtomicBool) -> ServiceResult<()> {
        checkpoint(stop)?;
        let schema_valid = matches!(
            self.schema.as_ref(),
            CONTEXT_MAP_HOST_SCHEMA | CONTEXT_MAP_CONTINUATION_HOST_SCHEMA
        ) && (self.continuation_store_file.is_none()
            || self.schema.as_ref() == CONTEXT_MAP_CONTINUATION_HOST_SCHEMA);
        if !schema_valid
            || !self.project_store_root.is_absolute()
            || self.project_store_root.as_os_str().len() > 32768
            || self.continuation_store_file.as_ref().is_some_and(|path| {
                !path.is_absolute()
                    || path.as_os_str().len() > 32768
                    || path == &self.project_store_root
            })
        {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        request::registration_id(&self.project_store_id)?;
        self.reference_profile
            .validate()
            .map_err(|_| failure(ServiceErrorCode::InvalidConfiguration))?;
        self.limits.owner_limits()?;
        checkpoint(stop)
    }

    pub(super) fn continuation_store_file(&self) -> ServiceResult<&std::path::Path> {
        self.continuation_store_file
            .as_deref()
            .ok_or_else(|| failure(ServiceErrorCode::InvalidConfiguration))
    }
}
