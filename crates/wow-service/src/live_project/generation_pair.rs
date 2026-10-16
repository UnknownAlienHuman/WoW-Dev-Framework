//! One ordinary Store, two retained exact reads, one Project-owned pair.
use super::*;
use serde::Serialize;
use std::{
    cell::Cell,
    io::{self, Write},
    sync::atomic::Ordering,
};
use wow_project::replay::publication::{RETAINED_GENERATION_PAIR_PROFILE, RetainedGenerationPair};
use wow_store::project::StoreGenerationId;

pub const PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Serialize)]
pub struct ProjectGenerationPairRequest {
    schema: &'static str,
    profile_id: &'static str,
    before_generation: StoreGenerationId,
    after_generation: StoreGenerationId,
    max_output_bytes: usize,
}
impl ProjectGenerationPairRequest {
    pub fn new(before: &str, after: &str, max_output_bytes: usize) -> ServiceResult<Self> {
        if max_output_bytes == 0 || max_output_bytes > PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES {
            return Err(fail(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            schema: "wow-service/project-generation-pair-request/1",
            profile_id: RETAINED_GENERATION_PAIR_PROFILE,
            before_generation: StoreGenerationId::parse(before)
                .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?,
            after_generation: StoreGenerationId::parse(after)
                .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?,
            max_output_bytes,
        })
    }
}
pub struct ProjectGenerationPairResult {
    bytes: Box<[u8]>,
}
impl ProjectGenerationPairResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub fn read_live_project_generation_pair(
    root: &Path,
    request: &ProjectGenerationPairRequest,
    stop: &AtomicBool,
) -> ServiceResult<ProjectGenerationPairResult> {
    if !root.is_absolute() || root.as_os_str().len() > 32768 {
        return Err(fail(ServiceErrorCode::InvalidRequest));
    }
    let request_id =
        crate::identity::canonical_digest("project-generation-pair-request:sha256:", request)?;
    let bytes = with_pair(root, request, stop, |before, after| {
        let pair = RetainedGenerationPair::bind(
            before.pair(),
            after.pair(),
            before.read.custody(),
            after.read.custody(),
            stop,
        )
        .map_err(project_error)?;
        let value = PairEnvelope {
            schema: "wow-service/project-generation-pair-result/1",
            operation: "project_generation_pair",
            request_id: &request_id,
            resource_closure: "closed",
            pair: pair.descriptor(),
        };
        let mut count = Count {
            bytes: 0,
            limit: request.max_output_bytes,
            stop,
            exceeded: false,
        };
        if serde_json::to_writer(&mut count, &value).is_err() {
            publication_checkpoint(stop)?;
            return Err(fail(if count.exceeded {
                ServiceErrorCode::BudgetExceeded
            } else {
                ServiceErrorCode::CanonicalizationFailed
            }));
        }
        publication_checkpoint(stop)?;
        let bytes = wow_core::canonical_json_bytes(&value)
            .map_err(|_| fail(ServiceErrorCode::CanonicalizationFailed))?;
        if bytes.len() > request.max_output_bytes {
            return Err(fail(ServiceErrorCode::BudgetExceeded));
        }
        publication_checkpoint(stop)?;
        Ok(bytes.into_boxed_slice())
    })?;
    Ok(ProjectGenerationPairResult { bytes })
}

#[derive(Serialize)]
struct PairEnvelope<'a> {
    schema: &'static str,
    operation: &'static str,
    request_id: &'a str,
    resource_closure: &'static str,
    pair: wow_project::replay::publication::RetainedGenerationPairDescriptor<'a>,
}

pub(crate) fn with_pair<T>(
    root: &Path,
    request: &ProjectGenerationPairRequest,
    stop: &AtomicBool,
    build: impl FnOnce(&LiveProjectRead, &LiveProjectRead) -> ServiceResult<T>,
) -> ServiceResult<T> {
    publication_checkpoint(stop)?;
    let store = LiveProjectStore::open(root)?;
    let closure = Cell::new("pending");
    let draft = with_pair_in_owner(&store, request, stop, &closure, build);
    selected_read_scope::finish_store(store, draft, &closure, stop)
}

pub(crate) fn with_pair_in_owner<T>(
    store: &LiveProjectStore,
    request: &ProjectGenerationPairRequest,
    stop: &AtomicBool,
    closure: &Cell<&'static str>,
    build: impl FnOnce(&LiveProjectRead, &LiveProjectRead) -> ServiceResult<T>,
) -> ServiceResult<T> {
    publication_checkpoint(stop)?;
    if store.store.epoch().namespace().is_some() {
        Err(fail(ServiceErrorCode::InvalidRequest))
    } else {
        match store.read_checked(
            &ReadSelector::Exact(request.before_generation.clone()),
            stop,
        ) {
            Err(error) => {
                closure.set(closure_name(error.closure));
                Err(error.error)
            }
            Ok(before) => {
                let draft = match store
                    .read_checked(&ReadSelector::Exact(request.after_generation.clone()), stop)
                {
                    Err(error) => {
                        closure.set(closure_name(error.closure));
                        Err(error.error)
                    }
                    Ok(after) => {
                        let draft = build(&before, &after);
                        match after.close() {
                            Ok(()) => draft,
                            Err(_) => {
                                closure.set("failed");
                                Err(fail(ServiceErrorCode::InternalContractViolation))
                            }
                        }
                    }
                };
                match before.close() {
                    Ok(()) => draft,
                    Err(_) => {
                        closure.set("failed");
                        Err(fail(ServiceErrorCode::InternalContractViolation))
                    }
                }
            }
        }
    }
}
fn closure_name(closure: ReadFailureClosure) -> &'static str {
    match closure {
        ReadFailureClosure::Closed => "closed",
        ReadFailureClosure::Failed => "failed",
        ReadFailureClosure::NotEvaluated => "not_evaluated",
    }
}
struct Count<'a> {
    bytes: usize,
    limit: usize,
    stop: &'a AtomicBool,
    exceeded: bool,
}
impl Write for Count<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        let Some(total) = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
        else {
            self.exceeded = true;
            return Err(io::Error::other("generation pair output exceeds its bound"));
        };
        self.bytes = total;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
