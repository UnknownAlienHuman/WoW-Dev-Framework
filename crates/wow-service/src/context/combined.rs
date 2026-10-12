//! One selected native operation retaining distinct Primary/platform authority.
use std::{cell::Cell, sync::atomic::AtomicBool};

use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest};
use wow_store::project::{ReadSelector, StoreGenerationId};

use super::{
    ContextMapHostInput, ContextMapPublication, ContextMapRequest, ContextMapResult,
    ContextMapRoot, ContextMapStatus, ContextPlatformHostInput, ContextPlatformOperation,
    ContextPlatformRequest, PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID, checkpoint, context_error,
    failure,
    operation::{
        Envelope, encode, failed_operation, preflight, with_checked_context_read,
        with_primary_composition,
    },
    platform::{self, with_checked_platform_read, with_qualified_platform_input},
    request,
};
use crate::{ServiceErrorCode, ServiceResult, live_project::LiveProjectRead};

pub const CONTEXT_COMBINED_HOST_SCHEMA: &str = "wow-service/context-combined-native-host/1";
pub const CONTEXT_COMBINED_REQUEST_SCHEMA: &str = "wow-service/context-combined-request/1";
pub const CONTEXT_COMBINED_RESULT_SCHEMA: &str = "wow-service/context-combined-result/1";
pub const CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES: usize = super::CONTEXT_MAP_HOST_INPUT_MAX_BYTES;
pub const CONTEXT_COMBINED_REQUEST_MAX_BYTES: usize = super::CONTEXT_PLATFORM_REQUEST_MAX_BYTES;
pub const COMBINED_NATIVE_MAP_PROFILE_ID: &str = wow_context::COMBINED_NATIVE_ONE_HOP_PROFILE;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostDocument {
    schema: Box<str>,
    primary: ContextMapHostInput,
    platform: platform::HostDocument,
}

/// Strict inert registrations; paths are never semantic identity or output.
pub struct ContextCombinedHostInput {
    primary: ContextMapHostInput,
    platform: ContextPlatformHostInput,
}
impl ContextCombinedHostInput {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: HostDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
            2_000_000,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        if document.schema.as_ref() != CONTEXT_COMBINED_HOST_SCHEMA {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        let host = Self {
            primary: document.primary,
            platform: ContextPlatformHostInput::from_document(document.platform, stop)?,
        };
        host.validate(stop)?;
        Ok(host)
    }
    fn validate(&self, stop: &AtomicBool) -> ServiceResult<()> {
        self.primary.validate(stop)?;
        self.platform.validate(stop)?;
        if self.primary.limits.owner_limits()? != self.platform.limits.owner_limits()?
            || self.primary.limits.evidence != self.platform.limits.evidence
            || self.primary.reference_profile != self.platform.reference_profile
            || self.primary.reference.schema() != self.platform.reference.schema()
            || self.primary.reference.generation_id() != self.platform.reference.generation_id()
            || self.primary.reference.self_digest() != self.platform.reference.self_digest()
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        checkpoint(stop)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PrimarySelection {
    project_store_id: Box<str>,
    store_generation_id: StoreGenerationId,
    publication_set_id: Box<str>,
    map_profile_id: Box<str>,
    root: ContextMapRoot,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestDocument {
    schema: Box<str>,
    operation: Box<str>,
    profile_id: Box<str>,
    primary: PrimarySelection,
    platform: platform::RequestDocument,
}

/// Exact inert selections; decoded records cannot create retained owner views.
#[derive(Serialize)]
pub struct ContextCombinedRequest {
    schema: &'static str,
    operation: &'static str,
    profile_id: &'static str,
    primary: PrimarySelection,
    platform: ContextPlatformRequest,
    #[serde(skip)]
    primary_request: ContextMapRequest,
}
impl ContextCombinedRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: RequestDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_COMBINED_REQUEST_MAX_BYTES,
            65_536,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        if document.schema.as_ref() != CONTEXT_COMBINED_REQUEST_SCHEMA
            || document.operation.as_ref() != "map"
            || document.profile_id.as_ref() != COMBINED_NATIVE_MAP_PROFILE_ID
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        request::identifier(&document.primary.publication_set_id)?;
        // Reuse the original root/request admission, rather than accepting DTO authority.
        let root = match &document.primary.root {
            ContextMapRoot::Project(id) => ContextMapRoot::project(id.as_str())?,
            ContextMapRoot::GraphEntity(id) => ContextMapRoot::graph_entity(id.as_str())?,
        };
        let primary_request = ContextMapRequest::primary(
            &document.primary.project_store_id,
            ContextMapPublication::StoreGeneration(document.primary.store_generation_id.clone()),
            &document.primary.map_profile_id,
            Some(root),
            None,
            None,
        )?;
        let platform = ContextPlatformRequest::from_document(document.platform, stop)?;
        if platform.operation() != ContextPlatformOperation::Map {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        checkpoint(stop)?;
        Ok(Self {
            schema: CONTEXT_COMBINED_REQUEST_SCHEMA,
            operation: "map",
            profile_id: COMBINED_NATIVE_MAP_PROFILE_ID,
            primary: document.primary,
            platform,
            primary_request,
        })
    }
}

pub fn execute_context_combined(
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let digest = wow_core::domain_separated_digest(CONTEXT_COMBINED_REQUEST_SCHEMA, request)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-combined-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let platform_closure = Cell::new("not_acquired");
    let outcome: ServiceResult<()> = (|| {
        host.validate(stop)?;
        let limit = usize::try_from(host.primary.limits.max_input_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        // Aggregate semantic-input reservation before either expensive acquisition.
        preflight(
            &(
                request,
                &host.primary.reference_profile,
                &host.primary.reference,
                &host.platform.reference_profile,
                &host.platform.reference,
                host.platform.selection.intent(),
                &host.primary.limits,
            ),
            limit,
            stop,
        )?;
        Ok(())
    })();
    if let Err(error) = outcome {
        return failed_operation(
            request,
            &request_id,
            error.code(),
            "not_acquired",
            CONTEXT_COMBINED_RESULT_SCHEMA,
            "context_combined_map",
        );
    }
    let selector = ReadSelector::Exact(request.primary.store_generation_id.clone());
    let (outcome, primary_closure) = with_checked_context_read(
        &host.primary,
        &request.primary_request.project_store_id,
        &selector,
        stop,
        |primary_read| {
            let root = qualify_primary(primary_read, host, request)?;
            let universe = wow_context::bind_context_universe_set(
                primary_read.pair(),
                &host.primary.reference,
                &host.primary.reference_profile,
                host.primary.limits.owner_limits()?,
                stop,
            )
            .map_err(context_error)?;
            with_primary_composition(
                &universe,
                primary_read,
                &host.primary,
                root,
                stop,
                |primary| {
                    let (outcome, closure) = with_checked_platform_read(
                        &host.platform,
                        &request.platform,
                        stop,
                        |platform_read| {
                            with_qualified_platform_input(
                                platform_read,
                                &request.platform,
                                stop,
                                |view| {
                                    let map = wow_context::build_combined_context_map(
                                        primary,
                                        view,
                                        &host.platform.reference,
                                        &host.platform.reference_profile,
                                        stop,
                                    )
                                    .map_err(context_error)?;
                                    encode_result(
                                        host,
                                        request,
                                        &request_id,
                                        primary_read,
                                        platform_read,
                                        &map,
                                        stop,
                                    )
                                },
                            )
                        },
                    );
                    platform_closure.set(closure);
                    outcome
                },
            )
        },
    );
    let closure = combined_closure(primary_closure, platform_closure.get());
    match outcome {
        Ok((bytes, status)) if closure == "closed" => Ok(ContextMapResult {
            bytes: bytes.into_boxed_slice(),
            status,
            code: None,
        }),
        Ok(_) => failed_operation(
            request,
            &request_id,
            ServiceErrorCode::InternalContractViolation,
            closure,
            CONTEXT_COMBINED_RESULT_SCHEMA,
            "context_combined_map",
        ),
        Err(error) => failed_operation(
            request,
            &request_id,
            error.code(),
            closure,
            CONTEXT_COMBINED_RESULT_SCHEMA,
            "context_combined_map",
        ),
    }
}

fn combined_closure(primary: &'static str, platform: &'static str) -> &'static str {
    if primary == "failed" || platform == "failed" {
        "failed"
    } else if matches!(primary, "pending" | "not_evaluated")
        || matches!(platform, "pending" | "not_evaluated")
    {
        "not_evaluated"
    } else if primary == "closed" && matches!(platform, "closed" | "not_acquired") {
        "closed"
    } else {
        "not_acquired"
    }
}

fn qualify_primary(
    read: &LiveProjectRead,
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
) -> ServiceResult<ContextMapRoot> {
    if read.project().configuration().project_id() != &host.primary.project_id
        || read.store_generation_id() != &request.primary.store_generation_id
        || read.publication_set_id() != request.primary.publication_set_id.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    match &request.primary.root {
        ContextMapRoot::GraphEntity(id) => Ok(ContextMapRoot::GraphEntity(id.clone())),
        ContextMapRoot::Project(id)
            if request.primary_request.map_profile_id
                == PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID
                && id == read.project().configuration().project_id() =>
        {
            Ok(ContextMapRoot::Project(id.clone()))
        }
        _ => Err(failure(ServiceErrorCode::InvalidRequest)),
    }
}

#[derive(Serialize)]
struct ReadBinding<'a> {
    configured_registration_id: &'a str,
    store_generation_id: &'a str,
    publication_set_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_project_store_id: Option<&'a str>,
}
fn read_binding<'a>(read: &'a LiveProjectRead, registration: &'a str) -> ReadBinding<'a> {
    ReadBinding {
        configured_registration_id: registration,
        store_generation_id: read.store_generation_id().as_str(),
        publication_set_id: read.publication_set_id(),
        native_project_store_id: read.namespace().map(|namespace| namespace.id().as_str()),
    }
}
#[derive(Serialize)]
struct Payload<'a, S: Serialize, T: Serialize> {
    resolved_request_id: &'a str,
    primary: ReadBinding<'a>,
    platform: ReadBinding<'a>,
    platform_selection: &'a S,
    combined_map_byte_length: usize,
    combined_map: &'a T,
}
fn encode_result(
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
    request_id: &str,
    primary_read: &LiveProjectRead,
    platform_read: &LiveProjectRead,
    map: &wow_context::CombinedContextMap<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus)> {
    let primary = read_binding(primary_read, &host.primary.project_store_id);
    let platform = read_binding(platform_read, &host.platform.project_store_id);
    let digest = wow_core::domain_separated_digest(
        "wow-service/context-combined-resolved-request/1",
        &(
            request_id,
            &primary,
            &platform,
            host.platform.selection.intent(),
            &host.primary.reference_profile,
            host.primary.reference.schema(),
            host.primary.reference.generation_id(),
            host.primary.reference.self_digest(),
            &host.primary.limits,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-combined-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let status = match map.state() {
        wow_context::ProjectMapState::Partial => ContextMapStatus::Partial,
        wow_context::ProjectMapState::Truncated => ContextMapStatus::Truncated,
    };
    let payload = Payload {
        resolved_request_id: &resolved_request_id,
        primary,
        platform,
        platform_selection: host.platform.selection.intent(),
        combined_map_byte_length: map.canonical_bytes().len(),
        combined_map: map,
    };
    let envelope = Envelope {
        schema: CONTEXT_COMBINED_RESULT_SCHEMA,
        operation: "context_combined_map",
        request_id,
        request,
        status,
        resource_closure: "closed",
        failure_code: None,
        payload: Some(payload),
    };
    let limit = usize::try_from(map.remaining_metadata_bytes())
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    Ok((encode(&envelope, limit, stop)?, status))
}
