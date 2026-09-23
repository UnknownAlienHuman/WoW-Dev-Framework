use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use wow_project::{ProjectGenerationCandidate, ProjectInputBundle, ProjectPublisher, ProjectView};
use wow_reference::ReferenceView;
use wow_rules::RuleRegistry;

use super::{LocalProjectInput, cancelled, owner_error, projection};
use crate::{
    CheckContext, CheckRequest, CheckScope, ComponentHealth, ContextIdentity, GenerationSelector,
    ServiceBackend, ServiceBackendStatus, ServiceConfiguration, ServiceError, ServiceErrorCode,
    ServiceResult,
};

/// Owns one explicitly supplied immutable input generation for this process.
/// `status` does not run diagnostics. `check` materializes the project once;
/// subsequent requests reuse the same snapshot, never a newly discovered input.
pub struct LocalProjectBackend {
    input: ProjectInputBundle,
    reference: ReferenceView,
    load_plan: Option<wow_project::load::ProjectLoadPlan>,
    registry: RuleRegistry,
    configuration: ServiceConfiguration,
    target_generation: String,
    published: Mutex<Option<ProjectView>>,
}

impl LocalProjectBackend {
    pub fn new(input: LocalProjectInput) -> ServiceResult<Self> {
        let project = input.bundle.configuration();
        let registry =
            RuleRegistry::e0().map_err(|_| owner_error("rule registry construction failed"))?;
        let configuration = ServiceConfiguration::builder()
            .project_id(project.project_id().as_str())
            .profile_id(project.selected_profile().profile_id().as_str())
            .reference_generation_id(project.reference_generation().to_string())
            .analyzer_pin_id(project.analyzer_binding().accepted_pin_id())
            .rule_registry_id(registry.registry_id())
            .build()?;
        let target = ProjectGenerationCandidate::derive(project, input.bundle.inventory())
            .map_err(|_| owner_error("project generation derivation failed"))?;
        Ok(Self {
            input: input.bundle,
            reference: input.reference,
            load_plan: input.load_plan,
            registry,
            configuration,
            target_generation: target.project_generation().to_string(),
            published: Mutex::new(None),
        })
    }

    #[must_use]
    pub fn configuration(&self) -> &ServiceConfiguration {
        &self.configuration
    }

    #[must_use]
    pub fn target_generation(&self) -> &str {
        &self.target_generation
    }

    fn identity(&self, project: &ProjectView) -> ServiceResult<ContextIdentity> {
        ContextIdentity::builder()
            .configuration_id(self.configuration.configuration_id())
            .project_id(self.configuration.project_id())
            .profile_id(self.configuration.profile_id())
            .reference_generation_id(self.configuration.reference_generation_id())
            .project_generation_id(project.project_generation().to_string())
            .project_snapshot_id(project.snapshot_id())
            .analyzer_snapshot_id(project.analyzer_snapshot_id())
            .analyzer_pin_id(self.configuration.analyzer_pin_id())
            .rule_registry_id(self.registry.registry_id())
            .build()
    }

    fn acquire(
        &self,
        selector: &GenerationSelector,
        scope: &CheckScope,
        rules: &[Box<str>],
        stop: &AtomicBool,
    ) -> ServiceResult<CheckContext> {
        cancelled(stop)?;
        match selector {
            GenerationSelector::Exact(generation)
                if generation.as_ref() != self.target_generation =>
            {
                return Err(ServiceError::new(
                    ServiceErrorCode::ExactGenerationUnavailable,
                    "exact generation differs from the supplied project input",
                ));
            }
            GenerationSelector::CurrentPublished { project_id }
                if project_id.as_ref() != self.configuration.project_id() =>
            {
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "requested project differs from the supplied input",
                ));
            }
            _ => {}
        }
        let mut retained = self
            .published
            .lock()
            .map_err(|_| owner_error("project publication lock poisoned"))?;
        let project = match retained.as_ref() {
            Some(project) => project.clone(),
            None => {
                // Publication is in-memory only. No source or persistent current pointer is written.
                let mut publisher = ProjectPublisher::new();
                let snapshot = publisher
                    .publish_initial_cancellable(self.input.clone(), stop)
                    .map_err(|error| {
                        if error.code() == wow_project::ProjectErrorCode::AnalysisCancelled {
                            ServiceError::new(
                                ServiceErrorCode::Cancelled,
                                "project analysis cancelled",
                            )
                        } else if error.code()
                            == wow_project::ProjectErrorCode::SourceBudgetExceeded
                        {
                            ServiceError::new(
                                ServiceErrorCode::BudgetExceeded,
                                "project analysis budget exceeded",
                            )
                        } else {
                            owner_error("project/analyzer could not publish a coherent snapshot")
                        }
                    })?;
                cancelled(stop)?;
                let view = snapshot.open_view();
                if view.project_generation().to_string() != self.target_generation {
                    return Err(owner_error(
                        "published project differs from its admitted generation",
                    ));
                }
                *retained = Some(view.clone());
                view
            }
        };
        drop(retained);
        cancelled(stop)?;
        projection::check_context(
            &project,
            &self.reference,
            &self.registry,
            &self.configuration,
            self.identity(&project)?,
            scope,
            rules,
            self.load_plan.as_ref(),
            stop,
        )
    }
}

impl ServiceBackend for LocalProjectBackend {
    fn status(&self) -> ServiceResult<ServiceBackendStatus> {
        let retained = self
            .published
            .lock()
            .map_err(|_| owner_error("project publication lock poisoned"))?;
        let identity = retained
            .as_ref()
            .map(|view| self.identity(view))
            .transpose()?;
        let project_health = if retained.is_some() {
            ComponentHealth::Ready
        } else {
            ComponentHealth::Degraded
        };
        ServiceBackendStatus::new(
            identity,
            projection::components(
                &self.configuration,
                &self.reference,
                retained
                    .as_ref()
                    .map(ProjectView::snapshot_id)
                    .unwrap_or(&self.target_generation),
                project_health,
                retained.as_ref(),
                self.load_plan.as_ref(),
            )?,
        )
    }

    fn acquire_context(
        &self,
        selector: &GenerationSelector,
        scope: &CheckScope,
    ) -> ServiceResult<CheckContext> {
        self.acquire(selector, scope, &[], &AtomicBool::new(false))
    }

    fn acquire_for_check(
        &self,
        request: &CheckRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<CheckContext> {
        self.acquire(request.selector(), request.scope(), request.rules(), stop)
    }
}
