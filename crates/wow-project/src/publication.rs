use std::sync::Arc;

use wow_core::ProjectGenerationId;
use wow_emmy::LuaWorkspaceSnapshot;

use crate::analyzer::build_analyzer_binding;
use crate::update::apply_file_operations;
use crate::{
    ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectGenerationCandidate,
    ProjectInputBundle, ProjectInputFile, ProjectInputInventory, ProjectPhase, ProjectResult,
    ProjectSnapshot, ProjectSourceRegistry, ProjectUpdateOutcome, ProjectUpdateRequest,
    ProjectView,
};

/// Synchronous atomic publisher for one explicit project identity.
///
/// Candidate state is private. The current pointer changes only after the full
/// configuration/inventory/analyzer/registry/snapshot transaction validates.
#[derive(Debug, Default)]
pub struct ProjectPublisher {
    current: Option<Arc<ProjectSnapshot>>,
    current_inputs: Vec<ProjectInputFile>,
    libraries: Vec<LuaWorkspaceSnapshot>,
    last_failure: Option<ProjectError>,
}

impl ProjectPublisher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current: None,
            current_inputs: Vec::new(),
            libraries: Vec::new(),
            last_failure: None,
        }
    }

    pub fn publish_initial(
        &mut self,
        bundle: ProjectInputBundle,
    ) -> ProjectResult<Arc<ProjectSnapshot>> {
        if self.current.is_some() {
            return Err(ProjectError::new(
                ProjectErrorCode::AlreadyPublished,
                ProjectPhase::Publication,
                "initial project publication cannot replace an existing current snapshot",
            ));
        }
        let (configuration, inventory, libraries) = bundle.into_parts();
        match build_snapshot(configuration, inventory.clone(), libraries.clone()) {
            Ok(snapshot) => {
                let snapshot = Arc::new(snapshot);
                self.current_inputs = inventory.files().to_vec();
                self.libraries = libraries;
                self.current = Some(Arc::clone(&snapshot));
                self.last_failure = None;
                Ok(snapshot)
            }
            Err(error) => {
                self.last_failure = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn apply_update(
        &mut self,
        request: ProjectUpdateRequest,
    ) -> ProjectResult<ProjectUpdateOutcome> {
        let current = self.current.clone().ok_or_else(|| {
            ProjectError::new(
                ProjectErrorCode::NoPublishedSnapshot,
                ProjectPhase::Update,
                "project update requires an existing published snapshot",
            )
        })?;
        let (
            expected_generation,
            expected_snapshot_digest,
            target_configuration,
            operations,
            requested_libraries,
        ) = request.into_parts();
        if target_configuration.project_id() != current.configuration().project_id() {
            return self.reject_with_current(
                ProjectError::new(
                    ProjectErrorCode::UpdateRequestInvalid,
                    ProjectPhase::Update,
                    "project update target identity differs from the current project",
                ),
                &current,
            );
        }
        if expected_generation.is_some_and(|expected| expected != current.project_generation()) {
            return self.reject_with_current(
                ProjectError::new(
                    ProjectErrorCode::ExpectedGenerationMismatch,
                    ProjectPhase::Update,
                    "expected current project generation does not match",
                ),
                &current,
            );
        }
        if expected_snapshot_digest
            .is_some_and(|expected| expected != current.canonical_snapshot_digest())
        {
            return self.reject_with_current(
                ProjectError::new(
                    ProjectErrorCode::ExpectedSnapshotDigestMismatch,
                    ProjectPhase::Update,
                    "expected current project snapshot digest does not match",
                ),
                &current,
            );
        }
        let final_files =
            match apply_file_operations(&self.current_inputs, &target_configuration, operations) {
                Ok(files) => files,
                Err(error) => return self.reject_with_current(error, &current),
            };
        let declared_paths = final_files
            .iter()
            .map(|file| file.relative_path().as_str().to_owned())
            .collect::<Vec<_>>();
        let inventory = match ProjectInputInventory::build(
            &target_configuration,
            declared_paths,
            final_files,
        ) {
            Ok(inventory) => inventory,
            Err(error) => return self.reject_with_current(error, &current),
        };
        let libraries = if requested_libraries.is_empty() {
            self.libraries.clone()
        } else {
            requested_libraries
        };
        let current_library_ids = current
            .analyzer_binding()
            .library_snapshot_ids()
            .collect::<Vec<_>>();
        let mut target_library_ids = libraries
            .iter()
            .map(LuaWorkspaceSnapshot::snapshot_id)
            .collect::<Vec<_>>();
        target_library_ids.sort_unstable();
        if target_configuration.configuration_digest()
            == current.configuration().configuration_digest()
            && inventory.manifest_digest()
                == current.generation_candidate().final_file_manifest_digest()
            && target_library_ids == current_library_ids
        {
            self.last_failure = None;
            return Ok(ProjectUpdateOutcome::NoChange(current));
        }
        match build_snapshot(target_configuration, inventory.clone(), libraries.clone()) {
            Ok(snapshot) => {
                let snapshot = Arc::new(snapshot);
                self.current_inputs = inventory.files().to_vec();
                self.libraries = libraries;
                self.current = Some(Arc::clone(&snapshot));
                self.last_failure = None;
                Ok(ProjectUpdateOutcome::Published(snapshot))
            }
            Err(error) => self.reject_with_current(error, &current),
        }
    }

    #[must_use]
    pub fn current_snapshot(&self) -> Option<&Arc<ProjectSnapshot>> {
        self.current.as_ref()
    }

    #[must_use]
    pub fn last_known_good(&self) -> Option<&Arc<ProjectSnapshot>> {
        self.current.as_ref()
    }

    #[must_use]
    pub const fn last_failure(&self) -> Option<&ProjectError> {
        self.last_failure.as_ref()
    }

    pub fn require_current_generation(
        &self,
        expected: ProjectGenerationId,
    ) -> ProjectResult<&Arc<ProjectSnapshot>> {
        let current = self.current.as_ref().ok_or_else(|| {
            ProjectError::new(
                ProjectErrorCode::NoPublishedSnapshot,
                ProjectPhase::View,
                "no current project snapshot is published",
            )
        })?;
        if current.project_generation() == expected {
            Ok(current)
        } else {
            Err(ProjectError::new(
                ProjectErrorCode::ExpectedGenerationMismatch,
                ProjectPhase::View,
                "requested project generation is not current",
            )
            .with_current(
                current.project_generation(),
                current.canonical_snapshot_digest(),
            ))
        }
    }

    pub fn open_current(&self) -> ProjectResult<ProjectView> {
        self.current
            .as_ref()
            .map(|snapshot| snapshot.open_view())
            .ok_or_else(|| {
                ProjectError::new(
                    ProjectErrorCode::NoPublishedSnapshot,
                    ProjectPhase::View,
                    "no current project snapshot is published",
                )
            })
    }

    fn reject_with_current<T>(
        &mut self,
        error: ProjectError,
        current: &ProjectSnapshot,
    ) -> ProjectResult<T> {
        let error = error.with_current(
            current.project_generation(),
            current.canonical_snapshot_digest(),
        );
        self.last_failure = Some(error.clone());
        Err(error)
    }
}

fn build_snapshot(
    configuration: ProjectConfiguration,
    inventory: ProjectInputInventory,
    libraries: Vec<LuaWorkspaceSnapshot>,
) -> ProjectResult<ProjectSnapshot> {
    let generation = ProjectGenerationCandidate::derive(&configuration, &inventory)?;
    let analyzer = build_analyzer_binding(&configuration, &inventory, &generation, &libraries)?;
    let registry = ProjectSourceRegistry::build(
        &configuration,
        &inventory,
        &analyzer,
        generation.project_generation(),
    )?;
    ProjectSnapshot::build(configuration, generation, inventory, registry, analyzer)
}
