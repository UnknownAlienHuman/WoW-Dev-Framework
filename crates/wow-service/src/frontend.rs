//! Bounded, transport-neutral frontend registry and in-memory session owner.
//!
//! This module owns only the first MCP stdio profile's exact capability and
//! session state. It does not wrap project, search, context, or analysis work.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::identity::canonical_digest;
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub const FRONTEND_REGISTRY_ID: &str = "wow-service/e7-a/frontend-session-operation-registry";
pub const MCP_PROTOCOL_REVISION: &str = "2025-11-25";
pub const MCP_STDIO_PROFILE_ID: &str = "mcp-2025-11-25-stdio-v1";
pub const MCP_PROFILE_RESOURCE_URI: &str = "wow://profiles/mcp-2025-11-25-stdio-v1";
pub const MCP_STATUS_TOOL_NAME: &str = "wow.status";

pub const MCP_MAX_FRAME_BYTES: usize = 64 * 1024;
pub const MCP_MAX_OUTPUT_BYTES: usize = 64 * 1024;
pub const MCP_MAX_JSON_DEPTH: usize = 32;
pub const MCP_MAX_CLIENT_NAME_BYTES: usize = 128;
pub const MCP_MAX_CLIENT_VERSION_BYTES: usize = 64;
pub const MCP_MAX_ACTIVE_SESSIONS: usize = 1;
pub const MCP_MAX_ACTIVE_REQUESTS: usize = 1;
pub const MCP_MAX_QUEUED_REQUESTS: usize = 0;

const REGISTRY_SCHEMA: &str = "wow-service/frontend-operation-registry/1";
const CAPABILITIES_SCHEMA: &str = "wow-service/frontend-capabilities/1";
const SESSION_SCHEMA: &str = "wow-service/frontend-session/1";
const STATUS_SCHEMA: &str = "wow-service/frontend-status/1";
const CLOSE_SCHEMA: &str = "wow-service/frontend-session-close/1";
const ERROR_SCHEMA: &str = "wow-service/frontend-error/1";
const TOOL_INPUT_SCHEMA_ID: &str = "wow-service/mcp/wow.status/request/1";
const TOOL_OUTPUT_SCHEMA_ID: &str = "wow-service/mcp/wow.status/result/1";

const EMPTY_OBJECT_SCHEMA: &str =
    r#"{"type":"object","properties":{},"additionalProperties":false}"#;
const STATUS_OUTPUT_SCHEMA: &str = r#"{"type":"object","properties":{"schema":{"const":"wow-service/frontend-status/1"},"sessionId":{"type":"integer","minimum":1},"state":{"const":"open"},"client":{"type":"object","properties":{"name":{"type":"string"},"version":{"type":"string"}},"required":["name","version"],"additionalProperties":false},"profileId":{"const":"mcp-2025-11-25-stdio-v1"},"registryDigest":{"type":"string","pattern":"^frontend-operation-registry:sha256:[0-9a-f]{64}$"},"profileDigest":{"type":"string","pattern":"^frontend-profile:sha256:[0-9a-f]{64}$"}},"required":["schema","sessionId","state","client","profileId","registryDigest","profileDigest"],"additionalProperties":false}"#;
const COMMON_ERROR_SCHEMA: &str = r#"{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"},"operationId":{"type":["string","null"]}},"required":["code","message","operationId"],"additionalProperties":false}"#;
const SESSION_OPEN_REQUEST_SCHEMA: &str = r#"{"type":"object","properties":{"protocolRevision":{"const":"2025-11-25"},"profileId":{"const":"mcp-2025-11-25-stdio-v1"},"transport":{"const":"stdio"},"client":{"type":"object","properties":{"name":{"type":"string","minLength":1,"maxLength":128},"version":{"type":"string","minLength":1,"maxLength":64}},"required":["name","version"],"additionalProperties":false}},"required":["protocolRevision","profileId","transport","client"],"additionalProperties":false}"#;
const SESSION_ID_REQUEST_SCHEMA: &str = r#"{"type":"object","properties":{"sessionId":{"type":"integer","minimum":1}},"required":["sessionId"],"additionalProperties":false}"#;
const CAPABILITIES_REQUEST_SCHEMA: &str = EMPTY_OBJECT_SCHEMA;
const CAPABILITIES_RESULT_SCHEMA: &str = r#"{"type":"object","properties":{"schema":{"const":"wow-service/frontend-capabilities/1"},"registryId":{"const":"wow-service/e7-a/frontend-session-operation-registry"},"registryDigest":{"type":"string"},"profile":{"type":"object"},"profileDigest":{"type":"string"},"operations":{"type":"array"},"tools":{"type":"array"},"resources":{"type":"array"}},"required":["schema","registryId","registryDigest","profile","profileDigest","operations","tools","resources"],"additionalProperties":false}"#;
const SESSION_RESULT_SCHEMA: &str = r#"{"type":"object","properties":{"schema":{"const":"wow-service/frontend-session/1"},"sessionId":{"type":"integer","minimum":1},"state":{"const":"open"},"client":{"type":"object"},"profileId":{"const":"mcp-2025-11-25-stdio-v1"},"registryDigest":{"type":"string"},"profileDigest":{"type":"string"}},"required":["schema","sessionId","state","client","profileId","registryDigest","profileDigest"],"additionalProperties":false}"#;
const CLOSE_RESULT_SCHEMA: &str = r#"{"type":"object","properties":{"schema":{"const":"wow-service/frontend-session-close/1"},"sessionId":{"type":"integer","minimum":1},"state":{"const":"closed"}},"required":["schema","sessionId","state"],"additionalProperties":false}"#;

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrontendEffectClass {
    PureRead,
    SessionLifecycle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendOperationDescriptor {
    pub operation_id: Box<str>,
    pub request_schema_id: Box<str>,
    pub request_schema_digest: Box<str>,
    pub result_schema_id: Box<str>,
    pub result_schema_digest: Box<str>,
    pub error_schema_id: Box<str>,
    pub error_schema_digest: Box<str>,
    pub effect_class: FrontendEffectClass,
    pub mcp_tool_name: Option<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendMcpToolDescriptor {
    pub name: Box<str>,
    pub service_operation: Box<str>,
    pub description: Box<str>,
    pub input_schema_id: Box<str>,
    pub input_schema_digest: Box<str>,
    pub input_schema_json: Box<str>,
    pub output_schema_id: Box<str>,
    pub output_schema_digest: Box<str>,
    pub output_schema_json: Box<str>,
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendMcpResourceDescriptor {
    pub uri: Box<str>,
    pub name: Box<str>,
    pub mime_type: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendResourceLimits {
    pub max_frame_bytes: usize,
    pub max_output_bytes: usize,
    pub max_json_depth: usize,
    pub max_client_name_bytes: usize,
    pub max_client_version_bytes: usize,
    pub max_active_sessions: usize,
    pub max_active_requests: usize,
    pub max_queued_requests: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendMcpProfile {
    pub schema: Box<str>,
    pub profile_id: Box<str>,
    pub protocol_revision: Box<str>,
    pub transport: Box<str>,
    pub server_capabilities: Vec<Box<str>>,
    pub unsupported_capabilities: Vec<Box<str>>,
    pub tool_names: Vec<Box<str>>,
    pub resource_uris: Vec<Box<str>>,
    pub limits: FrontendResourceLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendCapabilitySet {
    pub schema: Box<str>,
    pub registry_id: Box<str>,
    pub registry_digest: Box<str>,
    pub profile: FrontendMcpProfile,
    pub profile_digest: Box<str>,
    pub operations: Vec<FrontendOperationDescriptor>,
    pub tools: Vec<FrontendMcpToolDescriptor>,
    pub resources: Vec<FrontendMcpResourceDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrontendClientIdentity {
    pub name: Box<str>,
    pub version: Box<str>,
}

/// Opaque process-local identifier for one exact frontend session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct FrontendSessionId(u64);

impl FrontendSessionId {
    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl FrontendClientIdentity {
    pub fn new(name: impl Into<Box<str>>, version: impl Into<Box<str>>) -> ServiceResult<Self> {
        let identity = Self {
            name: name.into(),
            version: version.into(),
        };
        validate_client_field(&identity.name, MCP_MAX_CLIENT_NAME_BYTES, "client name")?;
        validate_client_field(
            &identity.version,
            MCP_MAX_CLIENT_VERSION_BYTES,
            "client version",
        )?;
        Ok(identity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrontendSessionOpenRequest {
    pub protocol_revision: Box<str>,
    pub profile_id: Box<str>,
    pub transport: Box<str>,
    pub client: FrontendClientIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrontendSessionSnapshot {
    pub schema: Box<str>,
    pub session_id: FrontendSessionId,
    pub state: Box<str>,
    pub client: FrontendClientIdentity,
    pub profile_id: Box<str>,
    pub registry_digest: Box<str>,
    pub profile_digest: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrontendStatus {
    pub schema: Box<str>,
    pub session_id: FrontendSessionId,
    pub state: Box<str>,
    pub client: FrontendClientIdentity,
    pub profile_id: Box<str>,
    pub registry_digest: Box<str>,
    pub profile_digest: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrontendSessionCloseResult {
    pub schema: Box<str>,
    pub session_id: FrontendSessionId,
    pub state: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct OperationRegistryIdentity<'a> {
    schema: &'a str,
    registry_id: &'a str,
    operations: &'a [FrontendOperationDescriptor],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProfileIdentity<'a> {
    schema: &'a str,
    profile: &'a FrontendMcpProfile,
    registry_digest: &'a str,
    tools: &'a [FrontendMcpToolDescriptor],
    resources: &'a [FrontendMcpResourceDescriptor],
}

/// Immutable reviewed operation and MCP capability set for the first W22 slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendOperationRegistry {
    capabilities: FrontendCapabilitySet,
}

impl FrontendOperationRegistry {
    fn new() -> ServiceResult<Self> {
        let mut operations = vec![
            operation_descriptor(OperationDefinition {
                operation_id: "frontend_status",
                request_schema_id: "wow-service/frontend-status-request/1",
                request_schema: SESSION_ID_REQUEST_SCHEMA,
                result_schema_id: STATUS_SCHEMA,
                result_schema: STATUS_OUTPUT_SCHEMA,
                effect_class: FrontendEffectClass::PureRead,
                mcp_tool_name: Some(MCP_STATUS_TOOL_NAME),
            }),
            operation_descriptor(OperationDefinition {
                operation_id: "frontend_capabilities",
                request_schema_id: "wow-service/frontend-capabilities-request/1",
                request_schema: CAPABILITIES_REQUEST_SCHEMA,
                result_schema_id: CAPABILITIES_SCHEMA,
                result_schema: CAPABILITIES_RESULT_SCHEMA,
                effect_class: FrontendEffectClass::PureRead,
                mcp_tool_name: None,
            }),
            operation_descriptor(OperationDefinition {
                operation_id: "frontend_session_open",
                request_schema_id: "wow-service/frontend-session-open-request/1",
                request_schema: SESSION_OPEN_REQUEST_SCHEMA,
                result_schema_id: SESSION_SCHEMA,
                result_schema: SESSION_RESULT_SCHEMA,
                effect_class: FrontendEffectClass::SessionLifecycle,
                mcp_tool_name: None,
            }),
            operation_descriptor(OperationDefinition {
                operation_id: "frontend_session_get",
                request_schema_id: "wow-service/frontend-session-get-request/1",
                request_schema: SESSION_ID_REQUEST_SCHEMA,
                result_schema_id: SESSION_SCHEMA,
                result_schema: SESSION_RESULT_SCHEMA,
                effect_class: FrontendEffectClass::PureRead,
                mcp_tool_name: None,
            }),
            operation_descriptor(OperationDefinition {
                operation_id: "frontend_session_close",
                request_schema_id: "wow-service/frontend-session-close-request/1",
                request_schema: SESSION_ID_REQUEST_SCHEMA,
                result_schema_id: CLOSE_SCHEMA,
                result_schema: CLOSE_RESULT_SCHEMA,
                effect_class: FrontendEffectClass::SessionLifecycle,
                mcp_tool_name: None,
            }),
        ];
        operations.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
        let registry_digest = canonical_digest(
            "frontend-operation-registry:sha256:",
            &OperationRegistryIdentity {
                schema: REGISTRY_SCHEMA,
                registry_id: FRONTEND_REGISTRY_ID,
                operations: &operations,
            },
        )?;

        let input_schema_digest = schema_digest(EMPTY_OBJECT_SCHEMA);
        let output_schema_digest = schema_digest(STATUS_OUTPUT_SCHEMA);
        let tools = vec![FrontendMcpToolDescriptor {
            name: MCP_STATUS_TOOL_NAME.into(),
            service_operation: "frontend_status".into(),
            description: "Read the active MCP frontend session and its exact implemented capability/profile digests. This does not inspect a project or run analysis.".into(),
            input_schema_id: TOOL_INPUT_SCHEMA_ID.into(),
            input_schema_digest,
            input_schema_json: EMPTY_OBJECT_SCHEMA.into(),
            output_schema_id: TOOL_OUTPUT_SCHEMA_ID.into(),
            output_schema_digest,
            output_schema_json: STATUS_OUTPUT_SCHEMA.into(),
            read_only_hint: true,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        }];

        let mut profile = FrontendMcpProfile {
            schema: "wow-service/mcp-2025-11-25-stdio-profile/1".into(),
            profile_id: MCP_STDIO_PROFILE_ID.into(),
            protocol_revision: MCP_PROTOCOL_REVISION.into(),
            transport: "stdio".into(),
            server_capabilities: vec!["tools".into(), "resources".into()],
            unsupported_capabilities: vec![
                "prompts".into(),
                "sampling".into(),
                "elicitation".into(),
                "tasks".into(),
                "server-requested-roots".into(),
                "logging".into(),
                "progress".into(),
                "cancellation".into(),
                "resource-subscriptions".into(),
                "tool-list-change-notifications".into(),
            ],
            tool_names: vec![MCP_STATUS_TOOL_NAME.into()],
            resource_uris: vec![MCP_PROFILE_RESOURCE_URI.into()],
            limits: FrontendResourceLimits {
                max_frame_bytes: MCP_MAX_FRAME_BYTES,
                max_output_bytes: MCP_MAX_OUTPUT_BYTES,
                max_json_depth: MCP_MAX_JSON_DEPTH,
                max_client_name_bytes: MCP_MAX_CLIENT_NAME_BYTES,
                max_client_version_bytes: MCP_MAX_CLIENT_VERSION_BYTES,
                max_active_sessions: MCP_MAX_ACTIVE_SESSIONS,
                max_active_requests: MCP_MAX_ACTIVE_REQUESTS,
                max_queued_requests: MCP_MAX_QUEUED_REQUESTS,
            },
        };
        // Lists are reviewed data; sorting makes the digest independent of edits
        // to construction order while preserving the one-tool mapping.
        profile.server_capabilities.sort();
        profile.unsupported_capabilities.sort();
        profile.tool_names.sort();
        profile.resource_uris.sort();
        let resource = FrontendMcpResourceDescriptor {
            uri: MCP_PROFILE_RESOURCE_URI.into(),
            name: MCP_STDIO_PROFILE_ID.into(),
            mime_type: "application/json".into(),
        };
        let profile_digest = canonical_digest(
            "frontend-profile:sha256:",
            &ProfileIdentity {
                schema: &profile.schema,
                profile: &profile,
                registry_digest: &registry_digest,
                tools: &tools,
                resources: std::slice::from_ref(&resource),
            },
        )?;
        let capabilities = FrontendCapabilitySet {
            schema: CAPABILITIES_SCHEMA.into(),
            registry_id: FRONTEND_REGISTRY_ID.into(),
            registry_digest,
            profile,
            profile_digest,
            operations,
            tools,
            resources: vec![resource],
        };
        Ok(Self { capabilities })
    }

    #[must_use]
    pub fn capabilities(&self) -> &FrontendCapabilitySet {
        &self.capabilities
    }

    pub fn manifest_json(&self) -> ServiceResult<Box<str>> {
        serde_json::to_string(&self.capabilities)
            .map(Into::into)
            .map_err(|_| {
                ServiceError::new(
                    ServiceErrorCode::CanonicalizationFailed,
                    "frontend profile manifest serialization failed",
                )
            })
    }
}

#[derive(Debug, Clone)]
struct ActiveFrontendSession {
    session_id: FrontendSessionId,
    client: FrontendClientIdentity,
}

/// Service-owned in-memory registry for the initial stdio frontend profile.
/// Session identifiers distinguish sessions; they grant no host or domain authority.
#[derive(Debug)]
pub struct FrontendService {
    registry: FrontendOperationRegistry,
    active_session: Mutex<Option<ActiveFrontendSession>>,
}

impl FrontendService {
    pub fn new() -> ServiceResult<Self> {
        Ok(Self {
            registry: FrontendOperationRegistry::new()?,
            active_session: Mutex::new(None),
        })
    }

    #[must_use]
    pub fn operation_registry(&self) -> &FrontendOperationRegistry {
        &self.registry
    }

    /// Return the exact immutable capability set for the pinned stdio profile.
    #[must_use]
    pub fn frontend_capabilities(&self) -> &FrontendCapabilitySet {
        self.registry.capabilities()
    }

    /// Open one process-local session. MCP client metadata is recorded as identity
    /// data only and never grants authorization.
    pub fn frontend_session_open(
        &self,
        request: FrontendSessionOpenRequest,
    ) -> ServiceResult<FrontendSessionSnapshot> {
        if request.protocol_revision.as_ref() != MCP_PROTOCOL_REVISION
            || request.profile_id.as_ref() != MCP_STDIO_PROFILE_ID
            || request.transport.as_ref() != "stdio"
        {
            return Err(invalid_frontend_request("unsupported frontend profile"));
        }
        validate_client_field(
            &request.client.name,
            MCP_MAX_CLIENT_NAME_BYTES,
            "client name",
        )?;
        validate_client_field(
            &request.client.version,
            MCP_MAX_CLIENT_VERSION_BYTES,
            "client version",
        )?;

        let session_id = NEXT_SESSION_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| {
                ServiceError::new(
                    ServiceErrorCode::BudgetExceeded,
                    "frontend session identity space exhausted",
                )
            })?;
        let session_id = FrontendSessionId(session_id);
        let mut active = self.lock_session()?;
        if active.is_some() {
            return Err(ServiceError::new(
                ServiceErrorCode::OperationBusy,
                "stdio frontend already has an active session",
            ));
        }
        let session = ActiveFrontendSession {
            session_id,
            client: request.client,
        };
        let snapshot = self.snapshot(&session);
        *active = Some(session);
        Ok(snapshot)
    }

    pub fn frontend_session_get(
        &self,
        session_id: FrontendSessionId,
    ) -> ServiceResult<FrontendSessionSnapshot> {
        let active = self.lock_session()?;
        let session = active
            .as_ref()
            .filter(|session| session.session_id == session_id)
            .ok_or_else(|| invalid_frontend_request("frontend session is not open"))?;
        Ok(self.snapshot(session))
    }

    /// Project actual frontend registry/session state; this does not run analysis.
    pub fn frontend_status(&self, session_id: FrontendSessionId) -> ServiceResult<FrontendStatus> {
        let active = self.lock_session()?;
        let session = active
            .as_ref()
            .filter(|session| session.session_id == session_id)
            .ok_or_else(|| invalid_frontend_request("frontend session is not open"))?;
        Ok(FrontendStatus {
            schema: STATUS_SCHEMA.into(),
            session_id: session.session_id,
            state: "open".into(),
            client: session.client.clone(),
            profile_id: MCP_STDIO_PROFILE_ID.into(),
            registry_digest: self.registry.capabilities.registry_digest.clone(),
            profile_digest: self.registry.capabilities.profile_digest.clone(),
        })
    }

    pub fn frontend_session_close(
        &self,
        session_id: FrontendSessionId,
    ) -> ServiceResult<FrontendSessionCloseResult> {
        let mut active = self.lock_session()?;
        match active.as_ref() {
            Some(session) if session.session_id == session_id => {
                *active = None;
                Ok(FrontendSessionCloseResult {
                    schema: CLOSE_SCHEMA.into(),
                    session_id,
                    state: "closed".into(),
                })
            }
            _ => Err(invalid_frontend_request("frontend session is not open")),
        }
    }

    fn snapshot(&self, session: &ActiveFrontendSession) -> FrontendSessionSnapshot {
        FrontendSessionSnapshot {
            schema: SESSION_SCHEMA.into(),
            session_id: session.session_id,
            state: "open".into(),
            client: session.client.clone(),
            profile_id: MCP_STDIO_PROFILE_ID.into(),
            registry_digest: self.registry.capabilities.registry_digest.clone(),
            profile_digest: self.registry.capabilities.profile_digest.clone(),
        }
    }

    fn lock_session(&self) -> ServiceResult<MutexGuard<'_, Option<ActiveFrontendSession>>> {
        self.active_session.lock().map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "frontend session registry lock is poisoned",
            )
        })
    }
}

struct OperationDefinition<'a> {
    operation_id: &'a str,
    request_schema_id: &'a str,
    request_schema: &'a str,
    result_schema_id: &'a str,
    result_schema: &'a str,
    effect_class: FrontendEffectClass,
    mcp_tool_name: Option<&'a str>,
}

fn operation_descriptor(definition: OperationDefinition<'_>) -> FrontendOperationDescriptor {
    FrontendOperationDescriptor {
        operation_id: definition.operation_id.into(),
        request_schema_id: definition.request_schema_id.into(),
        request_schema_digest: schema_digest(definition.request_schema),
        result_schema_id: definition.result_schema_id.into(),
        result_schema_digest: schema_digest(definition.result_schema),
        error_schema_id: ERROR_SCHEMA.into(),
        error_schema_digest: schema_digest(COMMON_ERROR_SCHEMA),
        effect_class: definition.effect_class,
        mcp_tool_name: definition.mcp_tool_name.map(Into::into),
    }
}

fn schema_digest(schema: &str) -> Box<str> {
    format!("sha256:{:x}", Sha256::digest(schema.as_bytes())).into_boxed_str()
}

fn validate_client_field(value: &str, max_bytes: usize, label: &str) -> ServiceResult<()> {
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(invalid_frontend_request(format!("invalid {label}")));
    }
    Ok(())
}

fn invalid_frontend_request(message: impl Into<Box<str>>) -> ServiceError {
    ServiceError::new(ServiceErrorCode::InvalidRequest, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(
        service: &FrontendService,
        name: &str,
    ) -> Result<FrontendSessionSnapshot, ServiceError> {
        service.frontend_session_open(FrontendSessionOpenRequest {
            protocol_revision: MCP_PROTOCOL_REVISION.into(),
            profile_id: MCP_STDIO_PROFILE_ID.into(),
            transport: "stdio".into(),
            client: FrontendClientIdentity::new(name, "1")?,
        })
    }

    #[test]
    fn registry_is_exact_and_digests_are_stable() -> Result<(), ServiceError> {
        let left = FrontendService::new()?;
        let right = FrontendService::new()?;
        let left = left.frontend_capabilities();
        let right = right.frontend_capabilities();

        assert_eq!(left.registry_digest, right.registry_digest);
        assert_eq!(left.profile_digest, right.profile_digest);
        assert_eq!(left.profile.tool_names[0].as_ref(), MCP_STATUS_TOOL_NAME);
        assert_eq!(
            left.profile.resource_uris[0].as_ref(),
            MCP_PROFILE_RESOURCE_URI
        );
        assert_eq!(left.operations.len(), 5);
        assert!(left.operations.iter().all(|operation| {
            operation.request_schema_digest.starts_with("sha256:")
                && operation.result_schema_digest.starts_with("sha256:")
                && operation.error_schema_digest.starts_with("sha256:")
        }));
        Ok(())
    }

    #[test]
    fn session_lifecycle_is_exact_and_instance_isolated() -> Result<(), ServiceError> {
        let service = FrontendService::new()?;
        let other = FrontendService::new()?;
        let opened = open(&service, "client-a")?;

        assert_eq!(
            service.frontend_session_get(opened.session_id),
            Ok(opened.clone())
        );
        let status = service.frontend_status(opened.session_id)?;
        assert_eq!(status.session_id, opened.session_id);
        assert_eq!(status.client.name.as_ref(), "client-a");
        assert!(other.frontend_session_get(opened.session_id).is_err());

        let closed = service.frontend_session_close(opened.session_id)?;
        assert_eq!(closed.state.as_ref(), "closed");
        assert!(service.frontend_session_get(opened.session_id).is_err());
        assert!(service.frontend_session_close(opened.session_id).is_err());
        Ok(())
    }

    #[test]
    fn rejects_mismatched_profile_and_second_active_session() -> Result<(), ServiceError> {
        let service = FrontendService::new()?;
        let mut request = FrontendSessionOpenRequest {
            protocol_revision: MCP_PROTOCOL_REVISION.into(),
            profile_id: MCP_STDIO_PROFILE_ID.into(),
            transport: "stdio".into(),
            client: FrontendClientIdentity::new("client-a", "1")?,
        };
        request.profile_id = "another-profile".into();
        assert!(matches!(
            service.frontend_session_open(request),
            Err(error) if error.code() == ServiceErrorCode::InvalidRequest
        ));

        let first = open(&service, "client-a")?;
        let second = service.frontend_session_open(FrontendSessionOpenRequest {
            protocol_revision: MCP_PROTOCOL_REVISION.into(),
            profile_id: MCP_STDIO_PROFILE_ID.into(),
            transport: "stdio".into(),
            client: FrontendClientIdentity::new("client-b", "2")?,
        });
        assert!(matches!(
            second,
            Err(error) if error.code() == ServiceErrorCode::OperationBusy
        ));
        service.frontend_session_close(first.session_id)?;
        Ok(())
    }
}
