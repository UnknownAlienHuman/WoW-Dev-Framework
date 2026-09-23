use super::{LocalProjectBackend, LocalProjectInput};
use crate::{
    CheckRequest, CheckResult, CheckScope, ComponentHealth, GenerationSelector, OperationId,
    Service, ServiceError, ServiceErrorCode, ServiceResult, ServiceSemanticStatus, StatusRequest,
    StatusResult,
};
use serde::Serialize;
use std::sync::atomic::AtomicBool;

/// Closed public command surface. Constructors validate exact owner IDs, while
/// the service resolves `current` only against the explicitly supplied input.
#[derive(Debug, Clone)]
pub enum LocalCommand {
    Status {
        request: StatusRequest,
        project_id: Option<Box<str>>,
    },
    Check {
        request: CheckRequest,
        project_id: Box<str>,
    },
}

impl LocalCommand {
    pub fn status(project: Option<String>) -> ServiceResult<Self> {
        if let Some(project) = &project {
            validate_project(project)?;
        }
        let id =
            crate::identity::canonical_digest("local-operation:sha256:", &("status", &project))?;
        Ok(Self::Status {
            request: StatusRequest::new(OperationId::new(id)?),
            project_id: project.map(Into::into),
        })
    }

    pub fn check(
        project: String,
        generation: String,
        files: Vec<Box<str>>,
        rules: Vec<Box<str>>,
    ) -> ServiceResult<Self> {
        validate_project(&project)?;
        let selector = if generation == "current" {
            GenerationSelector::current_published(project.clone())?
        } else {
            generation
                .parse::<wow_core::ProjectGenerationId>()
                .map_err(|_| {
                    ServiceError::new(
                        ServiceErrorCode::InvalidRequest,
                        "invalid exact ProjectGenerationId",
                    )
                })?;
            GenerationSelector::exact(generation)?
        };
        let scope = if files.is_empty() {
            CheckScope::WholeProject
        } else {
            CheckScope::project_files(files)?
        };
        let temporary = CheckRequest::new(OperationId::new("local.check")?, selector, scope)
            .with_rules(rules)?;
        let id = crate::identity::canonical_digest(
            "local-operation:sha256:",
            &(
                "check",
                &project,
                temporary.selector(),
                temporary.scope(),
                temporary.rules(),
            ),
        )?;
        let request = CheckRequest::new(
            OperationId::new(id)?,
            temporary.selector().clone(),
            temporary.scope().clone(),
        )
        .with_rules(temporary.rules().to_vec())?;
        Ok(Self::Check {
            request,
            project_id: project.into(),
        })
    }

    fn operation_id(&self) -> &OperationId {
        match self {
            Self::Status { request, .. } => request.operation_id(),
            Self::Check { request, .. } => request.operation_id(),
        }
    }
    fn project(&self) -> Option<&str> {
        match self {
            Self::Status { project_id, .. } => project_id.as_deref(),
            Self::Check { project_id, .. } => Some(project_id),
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Status { .. } => "status",
            Self::Check { .. } => "check",
        }
    }
}

fn validate_project(project: &str) -> ServiceResult<()> {
    wow_project::ProjectId::new(project)
        .map(|_| ())
        .map_err(|_| ServiceError::new(ServiceErrorCode::InvalidRequest, "invalid ProjectId"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OperationFailure {
    schema: &'static str,
    operation: &'static str,
    operation_id: OperationId,
    code: ServiceErrorCode,
}

/// Service-owned output; the CLI adds no semantic members to these records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", content = "result", rename_all = "snake_case")]
pub enum LocalOperationResult {
    Status(Box<StatusResult>),
    Check(Box<CheckResult>),
    Failure(OperationFailure),
    Cancelled(OperationFailure),
}

impl LocalOperationResult {
    pub fn canonical_bytes(&self) -> ServiceResult<Vec<u8>> {
        struct Buffer(Vec<u8>);
        impl std::io::Write for Buffer {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.0.len().saturating_add(bytes.len()) > 32 * 1024 * 1024 {
                    return Err(std::io::Error::other("service output byte limit"));
                }
                self.0.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut output = Buffer(Vec::new());
        serde_json::to_writer(&mut output, self).map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::CanonicalizationFailed,
                "service result encoding failed or exceeded its byte limit",
            )
        })?;
        Ok(output.0)
    }

    /// Build a cancellation result before output starts, without another owner call.
    #[must_use]
    pub fn cancelled(command: &LocalCommand) -> Self {
        Self::failure(command, ServiceErrorCode::Cancelled)
    }

    fn failure(command: &LocalCommand, code: ServiceErrorCode) -> Self {
        let result = OperationFailure {
            schema: "wow-service/local-operation-failure/1",
            operation: command.name(),
            operation_id: command.operation_id().clone(),
            code,
        };
        if code == ServiceErrorCode::Cancelled {
            Self::Cancelled(result)
        } else {
            Self::Failure(result)
        }
    }

    /// Stable outcome classification, not a release or runtime-safety policy.
    #[must_use]
    pub fn outcome_code(&self) -> LocalOutcome {
        match self {
            Self::Status(result) => match result.health() {
                ComponentHealth::Ready => LocalOutcome::Available,
                ComponentHealth::Degraded => LocalOutcome::Partial,
                ComponentHealth::Failed => LocalOutcome::InternalFailure,
                ComponentHealth::Unavailable => LocalOutcome::Unavailable,
            },
            Self::Check(result) => match result.semantic_status() {
                ServiceSemanticStatus::Clean => LocalOutcome::Available,
                ServiceSemanticStatus::Findings => LocalOutcome::Findings,
                ServiceSemanticStatus::Partial => LocalOutcome::Partial,
                ServiceSemanticStatus::Failed => LocalOutcome::InternalFailure,
                ServiceSemanticStatus::Cancelled => LocalOutcome::Cancelled,
            },
            Self::Cancelled(_) => LocalOutcome::Cancelled,
            Self::Failure(result) => match result.code {
                ServiceErrorCode::CanonicalizationFailed
                | ServiceErrorCode::InternalContractViolation => LocalOutcome::InternalFailure,
                ServiceErrorCode::Cancelled => LocalOutcome::Cancelled,
                _ => LocalOutcome::Unavailable,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalOutcome {
    Available,
    Findings,
    Partial,
    Unavailable,
    InternalFailure,
    Cancelled,
}

/// Executes one service operation. All semantic failures become typed records;
/// source/parser diagnostic prose is not copied into public failures.
pub fn execute_local(
    input: LocalProjectInput,
    command: &LocalCommand,
    stop: &AtomicBool,
) -> LocalOperationResult {
    fn run(
        input: LocalProjectInput,
        command: &LocalCommand,
        stop: &AtomicBool,
    ) -> ServiceResult<LocalOperationResult> {
        super::cancelled(stop)?;
        let backend = LocalProjectBackend::new(input)?;
        if command
            .project()
            .is_some_and(|id| id != backend.configuration().project_id())
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "requested project is not configured",
            ));
        }
        let service = Service::new(backend.configuration().clone(), backend)?;
        match command {
            LocalCommand::Status { request, .. } => service
                .status(request, stop)
                .map(|result| LocalOperationResult::Status(Box::new((*result).clone()))),
            LocalCommand::Check { request, .. } => service
                .check(request, stop)
                .map(|result| LocalOperationResult::Check(Box::new((*result).clone()))),
        }
    }
    match run(input, command, stop) {
        Ok(result) => result,
        Err(error) => LocalOperationResult::failure(command, error.code()),
    }
}
