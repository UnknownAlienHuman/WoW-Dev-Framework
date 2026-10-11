//! Exact primary-map orchestration over retained native owners.
//! Host registration and filesystem locations are separate from semantic requests.
mod host;
mod inspect;
mod operation;
mod request;
#[cfg(test)]
mod tests;

pub use host::{CONTEXT_MAP_HOST_INPUT_MAX_BYTES, CONTEXT_MAP_HOST_SCHEMA, ContextMapHostInput};
pub use inspect::{ContextInspectRequest, execute_context_inspect};
pub use operation::{ContextMapResult, ContextMapStatus, execute_context_map};
pub use wow_context::SkeletonLevel as ContextInspectLevel;
pub const PRIMARY_L0_PROFILE_ID: &str = wow_context::PRIMARY_L0_PROFILE;
pub const PRIMARY_L1_PROFILE_ID: &str = wow_context::PRIMARY_L1_PROFILE;
pub use request::{
    ContextMapCurrentGuard, ContextMapPublication, ContextMapReferenceGuard, ContextMapRequest,
    ContextMapRoot,
};

pub const PRIMARY_NAVIGATION_MAP_PROFILE_ID: &str = wow_context::PRIMARY_MAP_PROFILE;
pub const PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID: &str =
    wow_context::PRIMARY_PROJECT_MAP_PROFILE;
pub const CONTEXT_MAP_ID_MAX_BYTES: usize = 512;
pub const CONTEXT_MAP_RESULT_MAX_BYTES: usize = 16 * 1024 * 1024;
pub const CONTEXT_MAP_RESULT_SCHEMA: &str = "wow-service/context-map-result/1";

use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use std::sync::atomic::{AtomicBool, Ordering};

fn failure(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(code, "context map operation refused")
}
fn checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(ServiceErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn graph_error(error: wow_graph::GraphError) -> ServiceError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InternalContractViolation,
    })
}
fn context_error(error: wow_context::ContextError) -> ServiceError {
    use wow_context::ContextErrorCode;
    failure(match error.code() {
        ContextErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        ContextErrorCode::MinimumRequiredContentExceedsBudget => ServiceErrorCode::BudgetExceeded,
        ContextErrorCode::UniverseMismatch | ContextErrorCode::SupportMismatch => {
            ServiceErrorCode::IdentityMismatch
        }
        ContextErrorCode::RootInvalid => ServiceErrorCode::InvalidRequest,
        ContextErrorCode::MissingRequiredCustody | ContextErrorCode::NativeOwnerRefused => {
            ServiceErrorCode::ComponentUnavailable
        }
        ContextErrorCode::InvalidBudget => ServiceErrorCode::InvalidConfiguration,
        ContextErrorCode::EncodingInvalid => ServiceErrorCode::CanonicalizationFailed,
    })
}
