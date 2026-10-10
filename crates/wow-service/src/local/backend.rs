use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use wow_project::replay::publication::ProjectPublicationBundle;
use wow_project::{ProjectGenerationCandidate, ProjectInputBundle, ProjectPublisher, ProjectView};
use wow_reference::ReferenceView;
use wow_rules::RuleRegistry;
use wow_store::project::ProjectStoreNamespace;

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
    load: super::input::LocalProjectLoad,
    native_input: Option<std::sync::Arc<super::native_input::NativeInputEvidence>>,
    native_artifact: Option<std::sync::Arc<super::native_artifact::NativeArtifactEvidence>>,
    registry: RuleRegistry,
    configuration: ServiceConfiguration,
    target_generation: String,
    published: Mutex<Option<PublishedLocalProject>>,
    function_calls: bool,
}

/// One published native project retained with its original publisher, so a
/// same-session capture reuses the admitted analysis instead of re-deriving it.
struct PublishedLocalProject {
    view: ProjectView,
    publisher: ProjectPublisher,
}

impl LocalProjectBackend {
    pub fn new(input: LocalProjectInput) -> ServiceResult<Self> {
        Self::configured(input, false)
    }

    pub(crate) fn for_graph(input: LocalProjectInput) -> ServiceResult<Self> {
        Self::configured(input, true)
    }

    /// Retain an already published native publisher for this exact input. The
    /// publisher's current snapshot must describe the same configuration,
    /// inventory manifest, Library set, analysis profile and target generation as
    /// the supplied input, so a durable full-graph update can reuse one native
    /// session instead of opening a second one.
    pub(crate) fn for_graph_with_publisher(
        input: LocalProjectInput,
        publisher: ProjectPublisher,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        crate::local::cancelled(stop)?;
        let backend = Self::configured(input, true)?;
        let snapshot = publisher
            .open_current()
            .map_err(|_| owner_error("retained publisher has no current project snapshot"))?;
        let view = snapshot;
        let configuration = view.configuration();
        let analyzer = view.snapshot().analyzer_binding();
        let candidate = view.snapshot().generation_candidate();
        if view.project_generation().to_string() != backend.target_generation {
            return Err(owner_error(
                "retained publisher generation differs from the supplied input",
            ));
        }
        if configuration.configuration_digest()
            != backend.input.configuration().configuration_digest()
        {
            return Err(owner_error(
                "retained publisher configuration differs from the supplied input",
            ));
        }
        if candidate.final_file_manifest_digest() != backend.input.inventory().manifest_digest() {
            return Err(owner_error(
                "retained publisher inventory differs from the supplied input",
            ));
        }
        let current_library_ids = analyzer.library_snapshot_ids().collect::<Vec<_>>();
        let mut target_library_ids = backend
            .input
            .libraries()
            .iter()
            .map(|library| library.snapshot_id())
            .collect::<Vec<_>>();
        target_library_ids.sort_unstable();
        if current_library_ids != target_library_ids {
            return Err(owner_error(
                "retained publisher Library set differs from the supplied input",
            ));
        }
        if analyzer.function_call_report().is_none() {
            return Err(owner_error(
                "retained publisher has no function-call report for the graph backend",
            ));
        }
        crate::local::cancelled(stop)?;
        backend
            .published
            .lock()
            .map_err(|_| owner_error("project publication lock poisoned"))?
            .replace(PublishedLocalProject { view, publisher });
        Ok(backend)
    }

    fn configured(input: LocalProjectInput, function_calls: bool) -> ServiceResult<Self> {
        let project = input.bundle.configuration();
        let registry = RuleRegistry::for_profile(project.selected_profile().profile_id().as_str())
            .map_err(|_| owner_error("rule registry construction failed"))?;
        let configuration = ServiceConfiguration::builder()
            .project_id(project.project_id().as_str())
            .profile_id(project.selected_profile().profile_id().as_str())
            .reference_generation_id(project.reference_generation().to_string())
            .analyzer_pin_id(project.analyzer_binding().accepted_pin_id())
            .rule_registry_id(registry.registry_id())
            .build()?;
        let target = ProjectGenerationCandidate::derive_with_analysis(
            project,
            input.bundle.inventory(),
            input.bundle.libraries(),
            function_calls,
        )
        .map_err(|_| owner_error("project generation derivation failed"))?;
        Ok(Self {
            input: input.bundle,
            reference: input.reference,
            load: input.load,
            native_input: input.native_input,
            native_artifact: input.native_artifact,
            registry,
            configuration,
            target_generation: target.project_generation().to_string(),
            published: Mutex::new(None),
            function_calls,
        })
    }

    fn native_receipt(&self) -> Option<super::NativeEvidenceReceipt<'_>> {
        self.native_input
            .as_ref()
            .map(|evidence| super::NativeEvidenceReceipt::Source(&evidence.receipt))
            .or_else(|| {
                self.native_artifact
                    .as_ref()
                    .map(|evidence| super::NativeEvidenceReceipt::Artifact(&evidence.receipt))
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

    pub(crate) fn acquire_project(
        &self,
        selector: &GenerationSelector,
        stop: &AtomicBool,
    ) -> ServiceResult<ProjectView> {
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
            Some(retained) => retained.view.clone(),
            None => {
                // Publication is in-memory only. No source or persistent current pointer is written.
                let mut publisher = if self.function_calls {
                    ProjectPublisher::with_function_call_facts()
                } else {
                    ProjectPublisher::new()
                };
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
                *retained = Some(PublishedLocalProject {
                    view: view.clone(),
                    publisher,
                });
                view
            }
        };
        drop(retained);
        cancelled(stop)?;
        Ok(project)
    }

    pub(crate) fn capture_project_bundle(
        &self,
        graph: &wow_graph::GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ServiceResult<ProjectPublicationBundle> {
        cancelled(stop)?;
        let retained = self
            .published
            .lock()
            .map_err(|_| owner_error("project publication lock poisoned"))?;
        let project = retained
            .as_ref()
            .ok_or_else(|| owner_error("project must be materialized before native publication"))?;
        ProjectPublicationBundle::build(&project.publisher, graph, stop)
            .map_err(crate::live_project::project_error)
    }

    pub(crate) fn capture_project_bundle_in_namespace(
        &self,
        graph: &wow_graph::GraphPartitionSnapshot,
        namespace: &ProjectStoreNamespace,
        stop: &AtomicBool,
    ) -> ServiceResult<ProjectPublicationBundle> {
        cancelled(stop)?;
        let retained = self
            .published
            .lock()
            .map_err(|_| owner_error("project publication lock poisoned"))?;
        let project = retained
            .as_ref()
            .ok_or_else(|| owner_error("project must be materialized before native publication"))?;
        ProjectPublicationBundle::build_in_namespace(&project.publisher, graph, namespace, stop)
            .map_err(crate::live_project::project_error)
    }

    fn acquire(
        &self,
        selector: &GenerationSelector,
        scope: &CheckScope,
        rules: &[Box<str>],
        stop: &AtomicBool,
    ) -> ServiceResult<CheckContext> {
        let project = self.acquire_project(selector, stop)?;
        projection::check_context(
            &project,
            &self.reference,
            &self.registry,
            &self.configuration,
            self.identity(&project)?,
            scope,
            rules,
            self.load.evidence(),
            self.native_receipt(),
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
            .map(|project| self.identity(&project.view))
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
                    .map(|project| project.view.snapshot_id())
                    .unwrap_or(&self.target_generation),
                project_health,
                retained.as_ref().map(|project| &project.view),
                self.load.evidence(),
                self.native_receipt(),
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
