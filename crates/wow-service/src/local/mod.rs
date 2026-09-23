//! One-shot composition of real project, analyzer, reference and rule owners.
//! No source discovery, filesystem access, source execution or persistent current pointer.
mod backend;
mod input;
mod projection;
mod response;

pub use backend::LocalProjectBackend;
pub use input::{LOCAL_INPUT_MAX_BYTES, LOCAL_INPUT_SCHEMA, LocalProjectInput};
pub use projection::OwnerAnalysis;
pub use response::{
    LocalCommand, LocalOperationResult, LocalOutcome, OperationFailure, execute_local,
};

use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use std::sync::atomic::{AtomicBool, Ordering};

fn cancelled(flag: &AtomicBool) -> ServiceResult<()> {
    if flag.load(Ordering::Acquire) {
        Err(ServiceError::new(
            ServiceErrorCode::Cancelled,
            "operation cancelled",
        ))
    } else {
        Ok(())
    }
}

fn owner_error(owner: &'static str) -> ServiceError {
    ServiceError::new(ServiceErrorCode::InternalContractViolation, owner)
}
