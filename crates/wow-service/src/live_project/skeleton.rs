//! Candidate-required, metadata/handles-only Skeleton read under one live lease.
use super::{PlatformProjectSelection, fail, publication_checkpoint};
use crate::{ServiceErrorCode, ServiceResult};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path, sync::atomic::AtomicBool};
use wow_graph::GraphQueryState;
use wow_project::skeleton_input::{
    OpenSkeletonInputRequest, SkeletonInputPage, open_skeleton_input_view,
};

pub const LIVE_SKELETON_REQUEST_SCHEMA: &str = "wow-service/live-skeleton-read-request/1";
pub const LIVE_SKELETON_RESULT_SCHEMA: &str = "wow-service/live-skeleton-read-result/1";
pub const LIVE_SKELETON_REQUEST_MAX_BYTES: usize = crate::graph::GRAPH_REQUEST_MAX_BYTES;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkeletonRequest {
    schema: String,
    candidate_id: Box<str>,
    candidate_digest: Box<str>,
    input: OpenSkeletonInputRequest,
}

/// Inert observation bytes only. No owner authority can be decoded from them.
pub struct LiveSkeletonReadResult {
    bytes: Vec<u8>,
    state: GraphQueryState,
}
impl LiveSkeletonReadResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn state(&self) -> GraphQueryState {
        self.state
    }
    pub const fn exit_code(&self) -> u8 {
        2
    }
}
pub fn execute_live_skeleton_read(
    root: &Path,
    selection: &PlatformProjectSelection,
    generation: &str,
    request_bytes: &[u8],
    stop: &AtomicBool,
) -> ServiceResult<LiveSkeletonReadResult> {
    publication_checkpoint(stop)?;
    // Catalog intent is checked BEFORE acquisition/read, never inferred from a
    // native-looking raw pair or a receipt. Actual epoch validation remains exact.
    selection.require_candidate()?;
    let request: SkeletonRequest = crate::graph::input::decode(
        request_bytes,
        LIVE_SKELETON_REQUEST_MAX_BYTES,
        65_536,
        crate::graph::GraphReadStage::Request,
        stop,
    )
    .map_err(|failure| fail(failure.code))?;
    if request.schema != LIVE_SKELETON_REQUEST_SCHEMA {
        return Err(fail(ServiceErrorCode::InvalidRequest));
    }
    request.input.validate().map_err(super::project_error)?;
    let selector = super::operations::read_selector(generation)?;
    super::selected_read_scope::with_checked_selected_read(
        root,
        selection.owner(),
        &selector,
        stop,
        |read| {
            // No read primitive invocation before the genuine rebuilt candidate exists.
            let candidate = read
                .candidate()
                .ok_or_else(|| fail(ServiceErrorCode::IdentityMismatch))?;
            if request.candidate_id.as_ref() != candidate.manifest().candidate_id()
                || request.candidate_digest.as_ref() != candidate.manifest().candidate_digest()
            {
                return Err(fail(ServiceErrorCode::IdentityMismatch));
            }
            let limit = request.input.limits().max_output_bytes;
            let view = open_skeleton_input_view(
                candidate.native(),
                candidate.graph(),
                request.input,
                stop,
            )
            .map_err(super::project_error)?;
            let page = view.read(stop).map_err(super::project_error)?;
            let state = page.state();
            // Preserve the exact candidate identity and requested selector separately
            // from the source/project/final-graph binding already retained by the page.
            #[derive(Serialize)]
            struct Body<'a, 'page, S: Serialize> {
                schema: &'static str,
                selection: &'a S,
                requested_store_generation: &'a str,
                publication_set_id: &'a str,
                store_generation_id: &'a str,
                candidate_id: &'a str,
                candidate_digest: &'a str,
                page: &'a SkeletonInputPage<'page>,
            }
            let body = Body {
                schema: LIVE_SKELETON_RESULT_SCHEMA,
                selection: selection.intent(),
                requested_store_generation: generation,
                publication_set_id: read.publication_set_id(),
                store_generation_id: read.store_generation_id().as_str(),
                candidate_id: candidate.manifest().candidate_id(),
                candidate_digest: candidate.manifest().candidate_digest(),
                page: &page,
            };
            let bytes = encode(
                &body,
                usize::try_from(limit).map_err(|_| fail(ServiceErrorCode::BudgetExceeded))?,
                stop,
            )?;
            publication_checkpoint(stop)?;
            Ok(LiveSkeletonReadResult { bytes, state })
        },
    )
    .0
}

/// Count the whole selected result before canonical allocation. The same bound
/// includes service framing, not only the owner's page. Cancelled/refused reads
/// return Err with no successful payload; original store/source owners stay held.
pub(super) fn preflight(
    value: &impl Serialize,
    limit: usize,
    stop: &AtomicBool,
) -> ServiceResult<usize> {
    struct Counter<'a> {
        used: usize,
        limit: usize,
        exhausted: bool,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            publication_checkpoint(self.stop)
                .map_err(|_| std::io::Error::other("Skeleton output stopped"))?;
            self.used = self
                .used
                .checked_add(bytes.len())
                .filter(|used| *used <= self.limit)
                .ok_or_else(|| {
                    self.exhausted = true;
                    std::io::Error::other("Skeleton result byte limit")
                })?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    publication_checkpoint(stop)?;
    let mut counter = Counter {
        used: 0,
        limit,
        exhausted: false,
        stop,
    };
    let counted = serde_json::to_writer(&mut counter, value);
    publication_checkpoint(stop)?;
    if counter.exhausted {
        return Err(fail(ServiceErrorCode::BudgetExceeded));
    }
    counted.map_err(|_| fail(ServiceErrorCode::CanonicalizationFailed))?;
    Ok(counter.used)
}

pub(super) fn encode(
    value: &impl Serialize,
    limit: usize,
    stop: &AtomicBool,
) -> ServiceResult<Vec<u8>> {
    let used = preflight(value, limit, stop)?;
    let bytes = wow_core::canonical_json_bytes(value)
        .map_err(|_| fail(ServiceErrorCode::CanonicalizationFailed))?;
    publication_checkpoint(stop)?;
    if bytes.len() > limit || bytes.len() != used {
        return Err(fail(ServiceErrorCode::BudgetExceeded));
    }
    Ok(bytes)
}
