//! Explicit operator-approved native metadata registration and fixed MCP query.
use super::*;
use crate::context::{
    ContextInspectLevel, ContextInspectRequest, ContextMapHostInput, ContextMapPublication,
    ContextMapReferenceGuard, ContextMapResult, ContextMapRoot, execute_context_inspect,
};
use crate::search::LocalMetadataPermission;
use serde::Deserialize;
use std::sync::{TryLockError, atomic::AtomicBool};

pub const MCP_CONTEXT_PROFILE_ID: &str = "mcp-2025-11-25-stdio-context-v1";
pub const MCP_CONTEXT_PROFILE_RESOURCE_URI: &str = "wow://profiles/mcp-2025-11-25-stdio-context-v1";
pub const MCP_CONTEXT_TOOL_NAME: &str = "wow.context_inspect";
// Native structured result plus its escaped compatibility text and framing.
pub const MCP_CONTEXT_MAX_OUTPUT_BYTES: usize = crate::context::CONTEXT_MAP_RESULT_MAX_BYTES * 4;
pub(super) const CONTEXT_STATUS_SCHEMA: &str = "wow-service/frontend-context-status/1";
pub(super) const CONTEXT_SESSION_SCHEMA: &str = "wow-service/frontend-context-session/1";
const INPUT_SCHEMA_ID: &str = "wow-service/frontend-context-inspect-input/1";
const OUTPUT_SCHEMA_ID: &str = "wow-service/context-inspect-result/1";
const INPUT_SCHEMA: &str = r#"{"type":"object","properties":{"projectStoreId":{"type":"string","minLength":1,"maxLength":256},"storeGenerationId":{"type":"string","pattern":"^project-store-generation:sha256:[0-9a-f]{64}$"},"root":{"oneOf":[{"type":"object","properties":{"kind":{"const":"project"},"id":{"type":"string","minLength":1,"maxLength":512}},"required":["kind","id"],"additionalProperties":false},{"type":"object","properties":{"kind":{"const":"graph_entity"},"id":{"type":"string","pattern":"^graph-node:sha256:[0-9a-f]{64}$"}},"required":["kind","id"],"additionalProperties":false}]},"level":{"enum":["l0","l1","both"]}},"required":["projectStoreId","storeGenerationId","root","level"],"additionalProperties":false}"#;
const OUTPUT_SCHEMA: &str = r#"{"type":"object","properties":{"schema":{"const":"wow-service/context-inspect-result/1"},"operation":{"const":"context_inspect"},"request_id":{"type":"string"},"request":{"type":"object"},"status":{"enum":["complete","partial","truncated","failed","cancelled"]},"resource_closure":{"enum":["not_acquired","closed","failed","not_evaluated"]},"failure_code":{"type":"string"},"payload":{"type":"object"}},"required":["schema","operation","request_id","request","status","resource_closure"],"additionalProperties":true}"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendContextRegistrationDescriptor {
    pub state: Box<str>,
    pub project_store_id: Box<str>,
    pub project_id: Box<str>,
    pub reference_profile_id: Box<str>,
    pub reference_generation_id: Box<str>,
    pub reference_view_digest: Box<str>,
    pub host_binding_digest: Box<str>,
    pub privacy_scope: Box<str>,
}
pub(super) struct RegisteredContextHost {
    host: ContextMapHostInput,
    descriptor: FrontendContextRegistrationDescriptor,
}
impl std::fmt::Debug for RegisteredContextHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredContextHost")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Level {
    L0,
    L1,
    Both,
}
impl Level {
    fn native(&self) -> ContextInspectLevel {
        match self {
            Self::L0 => ContextInspectLevel::L0,
            Self::L1 => ContextInspectLevel::L1,
            Self::Both => ContextInspectLevel::Both,
        }
    }
}

fn checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(ServiceError::new(
            ServiceErrorCode::Cancelled,
            "frontend operation cancelled",
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrontendContextInspectRequest {
    project_store_id: Box<str>,
    store_generation_id: Box<str>,
    root: ContextMapRoot,
    level: Level,
}
impl FrontendContextInspectRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        checkpoint(stop)?;
        if bytes.len() > 8192 {
            return Err(invalid_frontend_request(
                "context arguments exceed byte limit",
            ));
        }
        let input: Self = serde_json::from_slice(bytes)
            .map_err(|_| invalid_frontend_request("invalid context inspection arguments"))?;
        ContextMapPublication::store_generation(&input.store_generation_id)?;
        checkpoint(stop)?;
        Ok(input)
    }
}

impl FrontendService {
    /// Approved CLI host setup only. It does not acquire/claim a materialized
    /// workspace or accept filesystem registration from an MCP tool argument.
    pub fn new_with_context_registration(
        host: ContextMapHostInput,
        _permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        host.validate(stop)?;
        let (profile, generation, view) = host.reference_identity();
        let descriptor = FrontendContextRegistrationDescriptor {
            state: "configured_metadata_only".into(),
            project_store_id: host.registration_id().into(),
            project_id: host.project_identity().into(),
            reference_profile_id: profile.into(),
            reference_generation_id: generation.into(),
            reference_view_digest: view.into(),
            host_binding_digest: host.semantic_binding_digest()?,
            privacy_scope: "local_metadata_only".into(),
        };
        let registry = FrontendOperationRegistry::for_context(&descriptor)?;
        Ok(Self {
            registry,
            active_session: Mutex::new(None),
            context_registration: Some(RegisteredContextHost { host, descriptor }),
        })
    }

    pub fn frontend_context_inspect(
        &self,
        session_id: FrontendSessionId,
        request: &FrontendContextInspectRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<ContextMapResult> {
        checkpoint(stop)?;
        // One synchronous operation owns this session until native close. A
        // competing inspection is refused rather than queued or rebound.
        let session = self
            .active_session
            .try_lock()
            .map_err(|error| match error {
                TryLockError::WouldBlock => ServiceError::new(
                    ServiceErrorCode::OperationBusy,
                    "frontend operation is already active",
                ),
                TryLockError::Poisoned(_) => ServiceError::new(
                    ServiceErrorCode::InternalContractViolation,
                    "frontend session registry lock is poisoned",
                ),
            })?;
        if session
            .as_ref()
            .is_none_or(|active| active.session_id != session_id)
        {
            return Err(invalid_frontend_request("frontend session is not open"));
        }
        let registered = self.context_registration.as_ref().ok_or_else(|| {
            invalid_frontend_request("context inspection is unavailable in this profile")
        })?;
        if request.project_store_id.as_ref() != registered.host.registration_id() {
            return Err(invalid_frontend_request(
                "project registration is outside this session profile",
            ));
        }
        let (profile, generation, view) = registered.host.reference_identity();
        let native = ContextInspectRequest::primary(
            &request.project_store_id,
            ContextMapPublication::store_generation(&request.store_generation_id)?,
            request.level.native(),
            request.root.clone(),
            None,
            Some(ContextMapReferenceGuard::new(profile, generation, view)?),
        )?;
        execute_context_inspect(&registered.host, &native, stop)
    }
}

impl FrontendOperationRegistry {
    fn for_context(registration: &FrontendContextRegistrationDescriptor) -> ServiceResult<Self> {
        // The existing constructor remains the exact v1 recipe. This explicit
        // profile gets its own reviewed descriptor generation and digest.
        let mut registry = Self::new()?;
        let caps = &mut registry.capabilities;
        let status_schema = STATUS_OUTPUT_SCHEMA
            .replace(STATUS_SCHEMA, CONTEXT_STATUS_SCHEMA)
            .replace(MCP_STDIO_PROFILE_ID, MCP_CONTEXT_PROFILE_ID);
        let open_schema =
            SESSION_OPEN_REQUEST_SCHEMA.replace(MCP_STDIO_PROFILE_ID, MCP_CONTEXT_PROFILE_ID);
        let session_schema = SESSION_RESULT_SCHEMA
            .replace(SESSION_SCHEMA, CONTEXT_SESSION_SCHEMA)
            .replace(MCP_STDIO_PROFILE_ID, MCP_CONTEXT_PROFILE_ID);
        for operation in &mut caps.operations {
            if matches!(
                operation.operation_id.as_ref(),
                "frontend_session_open" | "frontend_session_get"
            ) {
                operation.result_schema_id = CONTEXT_SESSION_SCHEMA.into();
                operation.result_schema_digest = schema_digest(&session_schema);
            }
            if operation.operation_id.as_ref() == "frontend_status" {
                operation.result_schema_id = CONTEXT_STATUS_SCHEMA.into();
                operation.result_schema_digest = schema_digest(&status_schema);
            }
            if operation.operation_id.as_ref() == "frontend_session_open" {
                operation.request_schema_id =
                    "wow-service/frontend-context-session-open-request/1".into();
                operation.request_schema_digest = schema_digest(&open_schema);
            }
        }
        let mut input_schema: serde_json::Value = serde_json::from_str(INPUT_SCHEMA)
            .map_err(|_| invalid_frontend_request("invalid reviewed context schema"))?;
        input_schema["properties"]["projectStoreId"]["const"] =
            serde_json::to_value(&registration.project_store_id)
                .map_err(|_| invalid_frontend_request("invalid context registration"))?;
        let input_schema = serde_json::to_string(&input_schema)
            .map_err(|_| invalid_frontend_request("invalid reviewed context schema"))?;
        caps.operations
            .push(operation_descriptor(OperationDefinition {
                operation_id: "context_inspect",
                request_schema_id: INPUT_SCHEMA_ID,
                request_schema: &input_schema,
                result_schema_id: OUTPUT_SCHEMA_ID,
                result_schema: OUTPUT_SCHEMA,
                effect_class: FrontendEffectClass::PureRead,
                mcp_tool_name: Some(MCP_CONTEXT_TOOL_NAME),
            }));
        caps.operations
            .sort_by(|a, b| a.operation_id.cmp(&b.operation_id));
        caps.registry_digest = canonical_digest(
            "frontend-operation-registry:sha256:",
            &OperationRegistryIdentity {
                schema: REGISTRY_SCHEMA,
                registry_id: FRONTEND_REGISTRY_ID,
                operations: &caps.operations,
            },
        )?;
        let status = &mut caps.tools[0];
        status.output_schema_id = "wow-service/mcp/wow.status/context-result/1".into();
        status.output_schema_digest = schema_digest(&status_schema);
        status.output_schema_json = status_schema.into();
        caps.tools.push(FrontendMcpToolDescriptor {
            name: MCP_CONTEXT_TOOL_NAME.into(), service_operation: "context_inspect".into(),
            description: "Inspect original metadata for an exact registered project generation and explicit root. Source bodies are excluded; results retain native Partial/Truncated coverage.".into(),
            input_schema_id: INPUT_SCHEMA_ID.into(), input_schema_digest: schema_digest(&input_schema), input_schema_json: input_schema.into(),
            output_schema_id: OUTPUT_SCHEMA_ID.into(), output_schema_digest: schema_digest(OUTPUT_SCHEMA), output_schema_json: OUTPUT_SCHEMA.into(),
            read_only_hint: true, destructive_hint: false, idempotent_hint: true, open_world_hint: false,
        });
        caps.tools.sort_by(|a, b| a.name.cmp(&b.name));
        caps.profile.schema = "wow-service/mcp-2025-11-25-stdio-context-profile/1".into();
        caps.profile.profile_id = MCP_CONTEXT_PROFILE_ID.into();
        caps.profile.tool_names = caps.tools.iter().map(|tool| tool.name.clone()).collect();
        caps.profile.resource_uris = vec![MCP_CONTEXT_PROFILE_RESOURCE_URI.into()];
        caps.profile.limits.max_output_bytes = MCP_CONTEXT_MAX_OUTPUT_BYTES;
        caps.profile.context_registration = Some(registration.clone());
        caps.resources = vec![FrontendMcpResourceDescriptor {
            uri: MCP_CONTEXT_PROFILE_RESOURCE_URI.into(),
            name: MCP_CONTEXT_PROFILE_ID.into(),
            mime_type: "application/json".into(),
        }];
        caps.profile_digest = canonical_digest(
            "frontend-profile:sha256:",
            &ProfileIdentity {
                schema: &caps.profile.schema,
                profile: &caps.profile,
                registry_digest: &caps.registry_digest,
                tools: &caps.tools,
                resources: &caps.resources,
            },
        )?;
        Ok(registry)
    }
}
