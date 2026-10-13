//! Private E3-C external artifact validation transport adapter.
//!
//! All semantic parsing and validation belongs to wow-context's
//! `validate_primary_l0_pack` owner function. This adapter only bounds the
//! request, computes exact input identity, wraps the typed report, and maps
//! operation outcomes.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_context::PrimaryPackValidationLimits;

use crate::context::operation::{self, ContextMapStatus};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub use wow_context::PrimaryPackValidationState as ValidationState;

pub const CONTEXT_VALIDATE_REQUEST_SCHEMA: &str = "wow-service/context-validate-request/1";
pub const CONTEXT_VALIDATE_RESULT_SCHEMA: &str = "wow-service/context-validate-result/1";
const DIGEST_CHUNK_BYTES: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextValidateArtifactKind {
    SemanticPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextValidateMediaType {
    ApplicationJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextValidateOriginClosure {
    StructuralOnly,
}

/// Transport request. Raw artifact bytes and input paths are never serialized.
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextValidateRequest {
    schema: &'static str,
    artifact_kind: ContextValidateArtifactKind,
    media_type: ContextValidateMediaType,
    validation_profile_id: &'static str,
    origin_closure_level: ContextValidateOriginClosure,
    max_input_bytes: u64,
    max_metadata_bytes: u64,
    max_context_work: u64,
    max_depth: u32,
    max_tokens: u64,
}

impl ContextValidateRequest {
    pub fn primary_semantic_pack_structural(
        max_input_bytes: u64,
        max_metadata_bytes: u64,
        max_context_work: u64,
        max_depth: u32,
        max_tokens: u64,
    ) -> ServiceResult<Self> {
        if max_input_bytes > 64 * 1024 * 1024
            || max_metadata_bytes > crate::context::CONTEXT_MAP_RESULT_MAX_BYTES as u64
        {
            return Err(error(ServiceErrorCode::InvalidConfiguration));
        }
        PrimaryPackValidationLimits::new(
            max_input_bytes,
            max_metadata_bytes,
            max_context_work,
            max_depth,
            max_tokens,
        )
        .map_err(crate::context::context_error)?;
        Ok(Self {
            schema: CONTEXT_VALIDATE_REQUEST_SCHEMA,
            artifact_kind: ContextValidateArtifactKind::SemanticPack,
            media_type: ContextValidateMediaType::ApplicationJson,
            validation_profile_id: wow_context::PRIMARY_CONTEXT_PACK_PROFILE,
            origin_closure_level: ContextValidateOriginClosure::StructuralOnly,
            max_input_bytes,
            max_metadata_bytes,
            max_context_work,
            max_depth,
            max_tokens,
        })
    }

    #[must_use]
    pub const fn max_input_bytes(&self) -> u64 {
        self.max_input_bytes
    }
}

/// Canonical finalized envelope bytes plus the typed owner state for CLI exit
/// projection. The serialized envelope contains the owner's report unchanged.
pub struct ContextValidateResult {
    bytes: Box<[u8]>,
    operation_status: ContextMapStatus,
    validation_state: Option<ValidationState>,
    failure_code: Option<ServiceErrorCode>,
}

impl ContextValidateResult {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn operation_status(&self) -> ContextMapStatus {
        self.operation_status
    }

    #[must_use]
    pub const fn validation_state(&self) -> Option<ValidationState> {
        self.validation_state
    }

    #[must_use]
    pub const fn failure_code(&self) -> Option<ServiceErrorCode> {
        self.failure_code
    }
}

#[derive(Serialize)]
struct RequestIdentity<'a> {
    schema: &'static str,
    artifact_kind: ContextValidateArtifactKind,
    media_type: ContextValidateMediaType,
    validation_profile_id: &'static str,
    origin_closure_level: ContextValidateOriginClosure,
    max_input_bytes: u64,
    max_metadata_bytes: u64,
    max_context_work: u64,
    max_depth: u32,
    max_tokens: u64,
    artifact_digest: &'a str,
    artifact_byte_length: u64,
}

pub fn execute_context_validate(
    request: &ContextValidateRequest,
    artifact_bytes: &[u8],
    stop: &AtomicBool,
) -> ServiceResult<ContextValidateResult> {
    let envelope_limit = usize::try_from(request.max_metadata_bytes)
        .map_err(|_| error(ServiceErrorCode::BudgetExceeded))?
        .min(crate::context::CONTEXT_MAP_RESULT_MAX_BYTES);
    if stop.load(Ordering::Acquire) {
        return Err(error(ServiceErrorCode::Cancelled));
    }
    if artifact_bytes.len() as u128 > u128::from(request.max_input_bytes) {
        return Err(error(ServiceErrorCode::BudgetExceeded));
    }
    let digest_work = u64::try_from(artifact_bytes.len().div_ceil(DIGEST_CHUNK_BYTES))
        .map_err(|_| error(ServiceErrorCode::BudgetExceeded))?;
    let owner_max_context_work = request
        .max_context_work
        .checked_sub(digest_work)
        .filter(|remaining| *remaining > 0)
        .ok_or_else(|| error(ServiceErrorCode::BudgetExceeded))?;
    let artifact_digest = digest_input(artifact_bytes, stop)?;
    let identity = RequestIdentity {
        schema: request.schema,
        artifact_kind: request.artifact_kind,
        media_type: request.media_type,
        validation_profile_id: request.validation_profile_id,
        origin_closure_level: request.origin_closure_level,
        max_input_bytes: request.max_input_bytes,
        max_metadata_bytes: request.max_metadata_bytes,
        max_context_work: request.max_context_work,
        max_depth: request.max_depth,
        max_tokens: request.max_tokens,
        artifact_digest: &artifact_digest,
        artifact_byte_length: u64::try_from(artifact_bytes.len())
            .map_err(|_| error(ServiceErrorCode::BudgetExceeded))?,
    };
    let request_digest =
        wow_core::domain_separated_digest(CONTEXT_VALIDATE_REQUEST_SCHEMA, &identity)
            .map_err(|_| error(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-validate-request:{}",
        wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(request_digest)
    );

    let limits = PrimaryPackValidationLimits::new(
        request.max_input_bytes,
        request.max_metadata_bytes,
        owner_max_context_work,
        request.max_depth,
        request.max_tokens,
    )
    .map_err(crate::context::context_error)?;

    // One call to the owner parser; no report fields are converted or rebuilt.
    let validation = match wow_context::validate_primary_l0_pack(artifact_bytes, limits, stop) {
        Ok(validation) => validation,
        Err(owner_error) => {
            let code = crate::context::context_error(owner_error).code();
            return failed(&identity, &request_id, code, envelope_limit);
        }
    };
    if stop.load(Ordering::Acquire) {
        return failed(
            &identity,
            &request_id,
            ServiceErrorCode::Cancelled,
            envelope_limit,
        );
    }

    let validation_state = validation.overall_state();
    let envelope = operation::Envelope {
        schema: CONTEXT_VALIDATE_RESULT_SCHEMA,
        operation: "context_validate",
        request_id: &request_id,
        request: &identity,
        status: ContextMapStatus::Complete,
        resource_closure: "closed",
        failure_code: None,
        payload: Some(validation),
    };
    let bytes = match operation::encode(&envelope, envelope_limit, stop) {
        Ok(bytes) => bytes,
        Err(encode_error) => {
            let code = if stop.load(Ordering::Acquire)
                || encode_error.code() == ServiceErrorCode::Cancelled
            {
                ServiceErrorCode::Cancelled
            } else {
                encode_error.code()
            };
            return failed(&identity, &request_id, code, envelope_limit);
        }
    };
    if stop.load(Ordering::Acquire) {
        return failed(
            &identity,
            &request_id,
            ServiceErrorCode::Cancelled,
            envelope_limit,
        );
    }
    Ok(ContextValidateResult {
        bytes: bytes.into_boxed_slice(),
        operation_status: ContextMapStatus::Complete,
        validation_state: Some(validation_state),
        failure_code: None,
    })
}

fn failed(
    request: &RequestIdentity<'_>,
    request_id: &str,
    code: ServiceErrorCode,
    envelope_limit: usize,
) -> ServiceResult<ContextValidateResult> {
    let operation_status = if code == ServiceErrorCode::Cancelled {
        ContextMapStatus::Cancelled
    } else {
        ContextMapStatus::Failed
    };
    let envelope = operation::Envelope::<_, ()> {
        schema: CONTEXT_VALIDATE_RESULT_SCHEMA,
        operation: "context_validate",
        request_id,
        request,
        status: operation_status,
        resource_closure: "closed",
        failure_code: Some(code),
        payload: None,
    };
    let bytes = match operation::encode(&envelope, envelope_limit, &AtomicBool::new(false)) {
        Ok(bytes) => bytes,
        Err(_) if code == ServiceErrorCode::Cancelled => {
            return Err(error(ServiceErrorCode::Cancelled));
        }
        Err(encode_error) => return Err(encode_error),
    };
    Ok(ContextValidateResult {
        bytes: bytes.into_boxed_slice(),
        operation_status,
        validation_state: None,
        failure_code: Some(code),
    })
}

fn error(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(code, "context validation operation could not finalize")
}

fn digest_input(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<String> {
    let mut hasher = Sha256::new();
    for chunk in bytes.chunks(DIGEST_CHUNK_BYTES) {
        if stop.load(Ordering::Acquire) {
            return Err(error(ServiceErrorCode::Cancelled));
        }
        hasher.update(chunk);
    }
    if stop.load(Ordering::Acquire) {
        return Err(error(ServiceErrorCode::Cancelled));
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
