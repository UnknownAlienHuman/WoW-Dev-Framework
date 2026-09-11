use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::backend::ServiceBackendStatus;
use crate::configuration::{RESULT_SCHEMA, SERVICE_SCHEMA, SERVICE_VERSION};
use crate::identity::canonical_digest;
use crate::operation::{CompletedResult, OperationKind, OperationRegistry, RegistryDecision};
use crate::presentation;
use crate::{
    CapabilityState, CheckContext, CheckRequest, CheckScope, ComponentHealth, ComponentSnapshot,
    ContextIdentity, DeferredOperation, GenerationSelector, OperationId, OperationRegistrySnapshot,
    PresentationGraph, RawFinding, RuleEvaluation, RuleEvaluationState, ServiceBackend,
    ServiceConfiguration, ServiceError, ServiceErrorCode, ServiceResult, ServiceSemanticStatus,
    StatusRequest,
};

const REQUIRED_COMPONENTS: [&str; 5] = [
    "wow-core",
    "wow-reference",
    "wow-emmy",
    "wow-project",
    "wow-rules",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatusResult {
    schema: &'static str,
    result_id: Box<str>,
    operation_id: OperationId,
    request_digest: Box<str>,
    service_schema: &'static str,
    service_version: &'static str,
    configuration_id: Box<str>,
    health: ComponentHealth,
    current_context: Option<ContextIdentity>,
    components: Vec<ComponentSnapshot>,
    deferred_operations: Vec<DeferredOperation>,
}

impl StatusResult {
    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
    }
    #[must_use]
    pub const fn health(&self) -> ComponentHealth {
        self.health
    }
    #[must_use]
    pub const fn current_context(&self) -> Option<&ContextIdentity> {
        self.current_context.as_ref()
    }
    #[must_use]
    pub fn components(&self) -> &[ComponentSnapshot] {
        &self.components
    }
    #[must_use]
    pub fn deferred_operations(&self) -> &[DeferredOperation] {
        &self.deferred_operations
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckResult {
    schema: &'static str,
    result_id: Box<str>,
    operation_id: OperationId,
    request_digest: Box<str>,
    service_schema: &'static str,
    service_version: &'static str,
    configuration_id: Box<str>,
    semantic_status: ServiceSemanticStatus,
    context: ContextIdentity,
    selected_scope: CheckScope,
    components: Vec<ComponentSnapshot>,
    raw_findings: Vec<RawFinding>,
    presentation_graph: PresentationGraph,
    rule_evaluations: Vec<RuleEvaluation>,
    deferred_operations: Vec<DeferredOperation>,
}

impl CheckResult {
    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
    }
    #[must_use]
    pub const fn semantic_status(&self) -> ServiceSemanticStatus {
        self.semantic_status
    }
    #[must_use]
    pub const fn context(&self) -> &ContextIdentity {
        &self.context
    }
    #[must_use]
    pub fn raw_findings(&self) -> &[RawFinding] {
        &self.raw_findings
    }
    #[must_use]
    pub const fn presentation_graph(&self) -> &PresentationGraph {
        &self.presentation_graph
    }
    #[must_use]
    pub fn rule_evaluations(&self) -> &[RuleEvaluation] {
        &self.rule_evaluations
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", content = "result", rename_all = "snake_case")]
pub enum ServiceResultEnvelope {
    Status(StatusResult),
    Check(CheckResult),
}

/// Synchronous E0 service. Every operation binds one canonical request to one
/// retained result or fails without publishing a replacement generation.
#[derive(Debug)]
pub struct Service<B: ServiceBackend> {
    configuration: Arc<ServiceConfiguration>,
    backend: Arc<B>,
    operations: OperationRegistry,
}

impl<B: ServiceBackend> Service<B> {
    pub fn new(configuration: ServiceConfiguration, backend: B) -> ServiceResult<Self> {
        let service = Self {
            configuration: Arc::new(configuration),
            backend: Arc::new(backend),
            operations: OperationRegistry::default(),
        };
        let status = service.backend.status()?;
        service.validate_backend_status(&status)?;
        Ok(service)
    }

    #[must_use]
    pub fn configuration(&self) -> &ServiceConfiguration {
        &self.configuration
    }

    pub fn status(
        &self,
        request: &StatusRequest,
        cancelled: &AtomicBool,
    ) -> ServiceResult<Arc<StatusResult>> {
        let request_digest = canonical_digest(
            "service-request:sha256:",
            &(self.configuration.configuration_id(), request),
        )?;
        match self.operations.begin(
            request.operation_id(),
            &request_digest,
            OperationKind::Status,
        )? {
            RegistryDecision::Replay(CompletedResult::Status(result)) => return Ok(result),
            RegistryDecision::Replay(CompletedResult::Check(_)) => {
                return Err(ServiceError::for_operation(
                    ServiceErrorCode::InternalContractViolation,
                    "status operation replay resolved to a check result",
                    request.operation_id().as_str(),
                ));
            }
            RegistryDecision::Started => {}
        }
        let result = self.build_status(request, &request_digest, cancelled);
        match result {
            Ok(result) => {
                let result = Arc::new(result);
                self.operations.complete(
                    request.operation_id(),
                    &request_digest,
                    OperationKind::Status,
                    CompletedResult::Status(result.clone()),
                )?;
                Ok(result)
            }
            Err(error) => {
                self.operations.abandon(
                    request.operation_id(),
                    &request_digest,
                    OperationKind::Status,
                );
                Err(error)
            }
        }
    }

    pub fn check(
        &self,
        request: &CheckRequest,
        cancelled: &AtomicBool,
    ) -> ServiceResult<Arc<CheckResult>> {
        self.validate_scope(request.scope())?;
        let request_digest = canonical_digest(
            "service-request:sha256:",
            &(self.configuration.configuration_id(), request),
        )?;
        match self.operations.begin(
            request.operation_id(),
            &request_digest,
            OperationKind::Check,
        )? {
            RegistryDecision::Replay(CompletedResult::Check(result)) => return Ok(result),
            RegistryDecision::Replay(CompletedResult::Status(_)) => {
                return Err(ServiceError::for_operation(
                    ServiceErrorCode::InternalContractViolation,
                    "check operation replay resolved to a status result",
                    request.operation_id().as_str(),
                ));
            }
            RegistryDecision::Started => {}
        }
        let result = self.build_check(request, &request_digest, cancelled);
        match result {
            Ok(result) => {
                let result = Arc::new(result);
                self.operations.complete(
                    request.operation_id(),
                    &request_digest,
                    OperationKind::Check,
                    CompletedResult::Check(result.clone()),
                )?;
                Ok(result)
            }
            Err(error) => {
                self.operations.abandon(
                    request.operation_id(),
                    &request_digest,
                    OperationKind::Check,
                );
                Err(error)
            }
        }
    }

    pub fn deferred(&self, operation: DeferredOperation) -> ServiceResult<()> {
        if !self
            .configuration
            .deferred_operations()
            .contains(&operation)
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "operation is not registered in the E0 deferred set",
            ));
        }
        Err(ServiceError::new(
            ServiceErrorCode::OperationNotImplementedForMilestone,
            format!("{operation:?} is deferred for the E0 milestone"),
        ))
    }

    pub fn operation_registry(&self) -> ServiceResult<OperationRegistrySnapshot> {
        self.operations.snapshot()
    }

    fn build_status(
        &self,
        request: &StatusRequest,
        request_digest: &str,
        cancelled: &AtomicBool,
    ) -> ServiceResult<StatusResult> {
        check_cancelled(cancelled, request.operation_id())?;
        let status = self.backend.status()?;
        self.validate_backend_status(&status)?;
        check_cancelled(cancelled, request.operation_id())?;
        let health = aggregate_health(status.components());

        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            operation_id: &'a OperationId,
            request_digest: &'a str,
            service_schema: &'static str,
            service_version: &'static str,
            configuration_id: &'a str,
            health: ComponentHealth,
            current_context: Option<&'a ContextIdentity>,
            components: &'a [ComponentSnapshot],
            deferred_operations: &'a [DeferredOperation],
        }
        let identity = Identity {
            schema: RESULT_SCHEMA,
            operation_id: request.operation_id(),
            request_digest,
            service_schema: SERVICE_SCHEMA,
            service_version: SERVICE_VERSION,
            configuration_id: self.configuration.configuration_id(),
            health,
            current_context: status.current_context(),
            components: status.components(),
            deferred_operations: self.configuration.deferred_operations(),
        };
        let result_id = canonical_digest("service-result:sha256:", &identity)?;
        Ok(StatusResult {
            schema: RESULT_SCHEMA,
            result_id,
            operation_id: request.operation_id().clone(),
            request_digest: request_digest.into(),
            service_schema: SERVICE_SCHEMA,
            service_version: SERVICE_VERSION,
            configuration_id: self.configuration.configuration_id().into(),
            health,
            current_context: status.current_context().cloned(),
            components: status.components().to_vec(),
            deferred_operations: self.configuration.deferred_operations().to_vec(),
        })
    }

    fn build_check(
        &self,
        request: &CheckRequest,
        request_digest: &str,
        cancelled: &AtomicBool,
    ) -> ServiceResult<CheckResult> {
        check_cancelled(cancelled, request.operation_id())?;
        let context = self
            .backend
            .acquire_context(request.selector(), request.scope())?;
        self.validate_context(&context, request)?;
        check_cancelled(cancelled, request.operation_id())?;

        let mut raw_findings = context
            .generic_findings()
            .iter()
            .cloned()
            .map(RawFinding::Generic)
            .collect::<Vec<_>>();
        for evaluation in context.rule_evaluations() {
            raw_findings.extend(
                evaluation
                    .findings_slice()
                    .iter()
                    .cloned()
                    .map(RawFinding::Rule),
            );
        }
        raw_findings.sort_by(|left, right| left.finding_id().cmp(right.finding_id()));
        ensure_unique(
            raw_findings.iter().map(RawFinding::finding_id),
            "raw finding identity",
        )?;
        if raw_findings.len() > limit(self.configuration.budgets().max_raw_findings)? {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "raw finding budget exceeded",
            ));
        }

        let semantic_status = derive_status(
            context.components(),
            context.rule_evaluations(),
            &raw_findings,
        );
        let presentation_graph = presentation::build(
            &raw_findings,
            context.rule_evaluations(),
            context.causal_relations(),
            self.configuration.budgets().max_presentation_relations,
        )?;
        check_cancelled(cancelled, request.operation_id())?;

        let mut components = context.components().to_vec();
        components.sort_by(|left, right| left.component_id().cmp(right.component_id()));
        let mut rule_evaluations = context.rule_evaluations().to_vec();
        rule_evaluations.sort_by(|left, right| left.evaluation_id().cmp(right.evaluation_id()));

        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            operation_id: &'a OperationId,
            request_digest: &'a str,
            service_schema: &'static str,
            service_version: &'static str,
            configuration_id: &'a str,
            semantic_status: ServiceSemanticStatus,
            context: &'a ContextIdentity,
            selected_scope: &'a CheckScope,
            components: &'a [ComponentSnapshot],
            raw_findings: &'a [RawFinding],
            presentation_graph: &'a PresentationGraph,
            rule_evaluations: &'a [RuleEvaluation],
            deferred_operations: &'a [DeferredOperation],
        }
        let identity = Identity {
            schema: RESULT_SCHEMA,
            operation_id: request.operation_id(),
            request_digest,
            service_schema: SERVICE_SCHEMA,
            service_version: SERVICE_VERSION,
            configuration_id: self.configuration.configuration_id(),
            semantic_status,
            context: context.identity(),
            selected_scope: context.selected_scope(),
            components: &components,
            raw_findings: &raw_findings,
            presentation_graph: &presentation_graph,
            rule_evaluations: &rule_evaluations,
            deferred_operations: self.configuration.deferred_operations(),
        };
        let result_id = canonical_digest("service-result:sha256:", &identity)?;
        Ok(CheckResult {
            schema: RESULT_SCHEMA,
            result_id,
            operation_id: request.operation_id().clone(),
            request_digest: request_digest.into(),
            service_schema: SERVICE_SCHEMA,
            service_version: SERVICE_VERSION,
            configuration_id: self.configuration.configuration_id().into(),
            semantic_status,
            context: context.identity().clone(),
            selected_scope: context.selected_scope().clone(),
            components,
            raw_findings,
            presentation_graph,
            rule_evaluations,
            deferred_operations: self.configuration.deferred_operations().to_vec(),
        })
    }

    fn validate_scope(&self, scope: &CheckScope) -> ServiceResult<()> {
        if scope.file_count().is_some_and(|count| {
            count > usize::try_from(self.configuration.budgets().max_scope_files).unwrap_or(0)
        }) {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "requested file scope exceeds the configured budget",
            ));
        }
        Ok(())
    }

    fn validate_backend_status(&self, status: &ServiceBackendStatus) -> ServiceResult<()> {
        if status.components().len() > limit(self.configuration.budgets().max_components)? {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "component inventory exceeds the configured budget",
            ));
        }
        ensure_required_components(status.components())?;
        if let Some(context) = status.current_context() {
            self.validate_identity(context)?;
        }
        Ok(())
    }

    fn validate_context(
        &self,
        context: &CheckContext,
        request: &CheckRequest,
    ) -> ServiceResult<()> {
        self.validate_identity(context.identity())?;
        if context.selected_scope() != request.scope() {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "selected context scope differs from the request",
            ));
        }
        match request.selector() {
            GenerationSelector::Exact(generation)
                if context.identity().project_generation_id() != generation.as_ref() =>
            {
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "backend returned another project generation",
                ));
            }
            GenerationSelector::CurrentPublished { project_id }
                if context.identity().project_id() != project_id.as_ref() =>
            {
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "backend returned another project",
                ));
            }
            GenerationSelector::Exact(_) | GenerationSelector::CurrentPublished { .. } => {}
        }
        if context.components().len() > limit(self.configuration.budgets().max_components)?
            || context.generic_findings().len()
                > limit(self.configuration.budgets().max_generic_findings)?
            || context.rule_evaluations().len()
                > limit(self.configuration.budgets().max_rule_evaluations)?
        {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "context exceeds configured service budgets",
            ));
        }
        ensure_required_components(context.components())?;
        ensure_unique(
            context
                .rule_evaluations()
                .iter()
                .map(RuleEvaluation::evaluation_id),
            "rule evaluation identity",
        )?;
        if context.rule_evaluations().iter().any(|evaluation| {
            evaluation.state() == RuleEvaluationState::Failed && !evaluation.degradable()
        }) {
            return Err(ServiceError::new(
                ServiceErrorCode::ComponentUnavailable,
                "mandatory rule evaluation failed",
            ));
        }
        Ok(())
    }

    fn validate_identity(&self, identity: &ContextIdentity) -> ServiceResult<()> {
        let expected = &self.configuration;
        if identity.configuration_id() != expected.configuration_id()
            || identity.project_id() != expected.project_id()
            || identity.profile_id() != expected.profile_id()
            || identity.reference_generation_id() != expected.reference_generation_id()
            || identity.analyzer_pin_id() != expected.analyzer_pin_id()
            || identity.rule_registry_id() != expected.rule_registry_id()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "service context does not match the configured exact identities",
            ));
        }
        Ok(())
    }
}

fn ensure_required_components(components: &[ComponentSnapshot]) -> ServiceResult<()> {
    let names = components
        .iter()
        .map(ComponentSnapshot::component_id)
        .collect::<BTreeSet<_>>();
    if names.len() != components.len()
        || REQUIRED_COMPONENTS
            .iter()
            .any(|component| !names.contains(component))
    {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidContext,
            "component inventory is duplicate or missing a mandatory E0 owner",
        ));
    }
    Ok(())
}

fn ensure_unique<'a>(values: impl IntoIterator<Item = &'a str>, label: &str) -> ServiceResult<()> {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.iter().copied().collect::<BTreeSet<_>>().len() != values.len() {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidContext,
            format!("duplicate {label}"),
        ));
    }
    Ok(())
}

fn aggregate_health(components: &[ComponentSnapshot]) -> ComponentHealth {
    if components
        .iter()
        .any(|component| component.health() == ComponentHealth::Failed)
    {
        ComponentHealth::Failed
    } else if components
        .iter()
        .any(|component| component.health() == ComponentHealth::Unavailable)
    {
        ComponentHealth::Unavailable
    } else if components
        .iter()
        .any(|component| component.health() == ComponentHealth::Degraded)
    {
        ComponentHealth::Degraded
    } else {
        ComponentHealth::Ready
    }
}

fn derive_status(
    components: &[ComponentSnapshot],
    evaluations: &[RuleEvaluation],
    findings: &[RawFinding],
) -> ServiceSemanticStatus {
    if evaluations
        .iter()
        .any(|evaluation| evaluation.state() == RuleEvaluationState::Cancelled)
    {
        ServiceSemanticStatus::Cancelled
    } else if evaluations.iter().any(|evaluation| {
        matches!(
            evaluation.state(),
            RuleEvaluationState::NotEvaluated | RuleEvaluationState::Failed
        )
    }) || components.iter().any(|component| {
        component.health() != ComponentHealth::Ready
            || component
                .capabilities()
                .values()
                .any(|state| *state == CapabilityState::Partial)
    }) {
        ServiceSemanticStatus::Partial
    } else if findings.is_empty() {
        ServiceSemanticStatus::Clean
    } else {
        ServiceSemanticStatus::Findings
    }
}

fn check_cancelled(cancelled: &AtomicBool, operation_id: &OperationId) -> ServiceResult<()> {
    if cancelled.load(Ordering::Relaxed) {
        Err(ServiceError::for_operation(
            ServiceErrorCode::Cancelled,
            "service operation was cancelled before publication",
            operation_id.as_str(),
        ))
    } else {
        Ok(())
    }
}

fn limit(value: u32) -> ServiceResult<usize> {
    usize::try_from(value).map_err(|_| {
        ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "configured budget cannot fit this platform",
        )
    })
}
