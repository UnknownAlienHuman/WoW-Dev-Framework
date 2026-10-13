//! Structural rendering transport for caller-supplied Primary L0 semantic packs.
//!
//! Bounded byte admission and rendering belong to wow-context's single
//! `validate_and_render_primary_l0_pack` owner invocation. This adapter hashes
//! the exact input, debits that work from the owner allowance, and transports
//! the unchanged typed report and optional rendered artifact. It opens no
//! project host, store, or native custody.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_context::{
    PrimaryPackRenderFormat, PrimaryPackValidation, PrimaryPackValidationLimits,
    PrimaryPackValidationState, RenderedContextArtifact,
};

use crate::context::operation::{self, ContextMapStatus};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub const CONTEXT_RENDER_REQUEST_SCHEMA: &str = "wow-service/context-render-request/1";
pub const CONTEXT_RENDER_RESULT_SCHEMA: &str = "wow-service/context-render-result/1";
const DIGEST_CHUNK_BYTES: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRenderFormat {
    Json,
    Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ContextRenderArtifactKind {
    SemanticPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ContextRenderMediaType {
    ApplicationJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ContextRenderOriginClosure {
    StructuralOnly,
}

/// Transport request. The input artifact bytes and any filesystem path are
/// supplied separately and are never copied into the serialized request.
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextRenderRequest {
    schema: &'static str,
    artifact_kind: ContextRenderArtifactKind,
    media_type: ContextRenderMediaType,
    validation_profile_id: &'static str,
    origin_closure_level: ContextRenderOriginClosure,
    format: ContextRenderFormat,
    max_input_bytes: u64,
    max_metadata_bytes: u64,
    max_context_work: u64,
    max_depth: u32,
    max_tokens: u64,
}

impl ContextRenderRequest {
    pub fn primary_semantic_pack_structural(
        format: ContextRenderFormat,
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
            schema: CONTEXT_RENDER_REQUEST_SCHEMA,
            artifact_kind: ContextRenderArtifactKind::SemanticPack,
            media_type: ContextRenderMediaType::ApplicationJson,
            validation_profile_id: wow_context::PRIMARY_CONTEXT_PACK_PROFILE,
            origin_closure_level: ContextRenderOriginClosure::StructuralOnly,
            format,
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

/// Canonical result envelope bytes with typed operation and validation status.
pub struct ContextRenderResult {
    bytes: Box<[u8]>,
    operation_status: ContextMapStatus,
    validation_state: Option<PrimaryPackValidationState>,
    failure_code: Option<ServiceErrorCode>,
}

impl ContextRenderResult {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn operation_status(&self) -> ContextMapStatus {
        self.operation_status
    }

    #[must_use]
    pub const fn validation_state(&self) -> Option<PrimaryPackValidationState> {
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
    artifact_kind: ContextRenderArtifactKind,
    media_type: ContextRenderMediaType,
    validation_profile_id: &'static str,
    origin_closure_level: ContextRenderOriginClosure,
    format: ContextRenderFormat,
    max_input_bytes: u64,
    max_metadata_bytes: u64,
    max_context_work: u64,
    max_depth: u32,
    max_tokens: u64,
    artifact_digest: &'a str,
    artifact_byte_length: u64,
}

#[derive(Serialize)]
struct ContextRenderWorkUsage {
    service_input_hash_work: u64,
    owner_validation_and_render_work: u64,
    total_context_work: u64,
}

#[derive(Serialize)]
struct ContextRenderPayload<'a> {
    validation: &'a PrimaryPackValidation,
    #[serde(skip_serializing_if = "Option::is_none")]
    rendered_artifact: Option<&'a RenderedContextArtifact>,
    work_usage: ContextRenderWorkUsage,
}

pub fn execute_context_render(
    request: &ContextRenderRequest,
    artifact_bytes: &[u8],
    stop: &AtomicBool,
) -> ServiceResult<ContextRenderResult> {
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
        format: request.format,
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
        wow_core::domain_separated_digest(CONTEXT_RENDER_REQUEST_SCHEMA, &identity)
            .map_err(|_| error(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-render-request:{}",
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
    let format = match request.format {
        ContextRenderFormat::Json => PrimaryPackRenderFormat::Json,
        ContextRenderFormat::Markdown => PrimaryPackRenderFormat::Markdown,
    };

    // Exactly one owner call: validation and rendering share its admitted
    // typed decode and preserve its report/artifact without reconstruction.
    let rendered = match wow_context::validate_and_render_primary_l0_pack(
        artifact_bytes,
        limits,
        format,
        stop,
    ) {
        Ok(rendered) => rendered,
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

    let validation = rendered.validation();
    let rendered_artifact = rendered.artifact();
    let validation_state = validation.overall_state();
    let owner_work = rendered.total_work();
    let Some(total_context_work) = digest_work
        .checked_add(owner_work)
        .filter(|total| *total <= request.max_context_work)
    else {
        return failed(
            &identity,
            &request_id,
            ServiceErrorCode::InternalContractViolation,
            envelope_limit,
        );
    };
    if (validation_state == PrimaryPackValidationState::Valid) != rendered_artifact.is_some() {
        return failed(
            &identity,
            &request_id,
            ServiceErrorCode::InternalContractViolation,
            envelope_limit,
        );
    }
    let payload = ContextRenderPayload {
        validation,
        rendered_artifact,
        work_usage: ContextRenderWorkUsage {
            service_input_hash_work: digest_work,
            owner_validation_and_render_work: owner_work,
            total_context_work,
        },
    };
    let envelope = operation::Envelope {
        schema: CONTEXT_RENDER_RESULT_SCHEMA,
        operation: "context_render",
        request_id: &request_id,
        request: &identity,
        status: ContextMapStatus::Complete,
        resource_closure: "closed",
        failure_code: None,
        payload: Some(payload),
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
    Ok(ContextRenderResult {
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
) -> ServiceResult<ContextRenderResult> {
    let operation_status = if code == ServiceErrorCode::Cancelled {
        ContextMapStatus::Cancelled
    } else {
        ContextMapStatus::Failed
    };
    let envelope = operation::Envelope::<_, ()> {
        schema: CONTEXT_RENDER_RESULT_SCHEMA,
        operation: "context_render",
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
    Ok(ContextRenderResult {
        bytes: bytes.into_boxed_slice(),
        operation_status,
        validation_state: None,
        failure_code: Some(code),
    })
}

fn error(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(code, "context render operation could not finalize")
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
