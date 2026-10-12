//! One-shot composition of real project, analyzer, reference and rule owners.
//! Explicit disk inputs use the project acquisition port; no discovery, source execution or persistent current pointer.
mod backend;
mod disk_input;
mod input;
pub mod native_artifact;
mod native_compatibility;
mod native_distribution;
mod native_input;
pub(crate) mod native_library_input;
mod native_reference;
mod native_resources;
mod native_source;
mod native_target;
mod platform_input;
mod projection;
mod response;
mod xml_bindings;
mod xml_lua;
mod xml_references;

pub use backend::LocalProjectBackend;
pub use disk_input::{
    LOCAL_FILES_SCHEMA, LOCAL_PACKAGES_SCHEMA, LOCAL_PRIMARY_CALLABLES_FILES_SCHEMA,
    LOCAL_TOC_SCHEMA,
};
pub use input::{LOCAL_INPUT_MAX_BYTES, LOCAL_INPUT_SCHEMA, LocalProjectInput};
pub use native_artifact::NativeArtifactReceipt;
pub use native_compatibility::{NativeCompatibilitySelection, NativeConsumerProbeIdentity};
pub use native_distribution::NativeDistributionSelection;
pub use native_input::{
    LOCAL_NATIVE_SCHEMA, NativeAnnotationFile, NativeFileIdentity, NativeInputReceipt,
    NativeManifestIdentity, NativeRestrictionIdentity,
};
pub use native_library_input::RequalifiedNativeLibrary;
pub use native_reference::{
    SOURCE_REFERENCE_INPUT_SCHEMA, SourceReferenceResult, read_local_source_reference,
};
pub use native_resources::{NativeAnnotationInputsReceipt, NativeAnnotationSelection};
pub use native_target::{
    NativeApiTargetAssociation, NativeApiTargetOmission, NativeApiTargetResolution,
    NativeApiTargetResolver,
};
pub use platform_input::{LOCAL_PLATFORM_BUDGET_SCHEMA, LOCAL_PLATFORM_SCHEMA};
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

/// The source-producing route and retained-artifact route never claim the same
/// provenance. Keep one parameter at the owner composition boundary.
#[derive(Clone, Copy)]
pub(super) enum NativeEvidenceReceipt<'a> {
    Source(&'a NativeInputReceipt),
    Artifact(&'a NativeArtifactReceipt),
}
