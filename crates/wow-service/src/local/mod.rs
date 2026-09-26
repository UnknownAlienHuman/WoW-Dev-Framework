//! One-shot composition of real project, analyzer, reference and rule owners.
//! Explicit disk inputs use the project acquisition port; no discovery, source execution or persistent current pointer.
mod backend;
mod disk_input;
mod input;
mod native_input;
mod native_source;
mod projection;
mod response;
mod xml_bindings;
mod xml_lua;
mod xml_references;

pub use backend::LocalProjectBackend;
pub use disk_input::{LOCAL_FILES_SCHEMA, LOCAL_TOC_SCHEMA};
pub use input::{LOCAL_INPUT_MAX_BYTES, LOCAL_INPUT_SCHEMA, LocalProjectInput};
pub use native_input::{
    LOCAL_NATIVE_SCHEMA, NativeFileIdentity, NativeInputReceipt, NativeManifestIdentity,
};
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
