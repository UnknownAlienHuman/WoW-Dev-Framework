use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId, ReferenceGenerationId};

use crate::configuration::PROJECT_GENERATION_SCHEMA_VERSION;
use crate::identity::canonical_id;
use crate::{
    ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectFileManifestEntry,
    ProjectInputInventory, ProjectPhase, ProjectResult,
};

/// Immutable derivation receipt for one intended final project state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGenerationCandidate {
    candidate_id: Box<str>,
    project_configuration_digest: ContentDigest<CanonicalResult>,
    selected_profile_id: Box<str>,
    reference_generation: ReferenceGenerationId,
    analyzer_pin_id: Box<str>,
    analyzer_probe_report_id: Box<str>,
    analyzer_configuration_digest: ContentDigest<CanonicalResult>,
    final_file_manifest_digest: ContentDigest<CanonicalResult>,
    project_generation_schema_version: u64,
    derived_project_generation: ProjectGenerationId,
}

impl ProjectGenerationCandidate {
    pub fn derive(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
    ) -> ProjectResult<Self> {
        configuration.validate()?;
        let manifest = inventory.manifest_entries();
        #[derive(Serialize)]
        struct DerivationInput<'a> {
            project_generation_schema_version: u64,
            project_configuration_digest: ContentDigest<CanonicalResult>,
            project_id: &'a crate::ProjectId,
            project_kind: crate::ProjectKind,
            workspace_id: &'a crate::ProjectWorkspaceId,
            source_origin_id: &'a crate::ProjectSourceOriginId,
            logical_root: &'a wow_core::NormalizedSourcePath,
            selected_profile: &'a wow_core::ProfileIdentity,
            reference_generation: ReferenceGenerationId,
            analyzer_pin_id: &'a str,
            analyzer_probe_report_id: &'a str,
            analyzer_configuration_digest: ContentDigest<CanonicalResult>,
            capability_policy: &'a crate::ProjectCapabilityPolicy,
            budget_policy: crate::ProjectBudgetPolicy,
            final_file_manifest_digest: ContentDigest<CanonicalResult>,
            files: &'a [ProjectFileManifestEntry],
        }
        let input = DerivationInput {
            project_generation_schema_version: PROJECT_GENERATION_SCHEMA_VERSION,
            project_configuration_digest: configuration.configuration_digest(),
            project_id: configuration.project_id(),
            project_kind: configuration.project_kind(),
            workspace_id: configuration.workspace_id(),
            source_origin_id: configuration.source_origin_id(),
            logical_root: configuration.logical_root(),
            selected_profile: configuration.selected_profile(),
            reference_generation: configuration.reference_generation(),
            analyzer_pin_id: configuration.analyzer_binding().accepted_pin_id(),
            analyzer_probe_report_id: configuration
                .analyzer_binding()
                .compatibility_probe_report_id(),
            analyzer_configuration_digest: configuration
                .analyzer_binding()
                .analyzer_configuration_digest(),
            capability_policy: configuration.capability_policy(),
            budget_policy: configuration.budget_policy(),
            final_file_manifest_digest: inventory.manifest_digest(),
            files: &manifest,
        };
        let derived_project_generation = ProjectGenerationId::derive(&input).map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::GenerationDerivationFailed,
                ProjectPhase::Generation,
                format!("project generation cannot be derived: {source}"),
            )
        })?;
        #[derive(Serialize)]
        struct CandidateIdentity<'a> {
            project_generation: ProjectGenerationId,
            derivation_input: &'a DerivationInput<'a>,
        }
        let candidate_id = canonical_id(
            "project-candidate:sha256:",
            "wow-project/generation-candidate/e0-d/1",
            &CandidateIdentity {
                project_generation: derived_project_generation,
                derivation_input: &input,
            },
            ProjectPhase::Generation,
        )?;
        Ok(Self {
            candidate_id,
            project_configuration_digest: configuration.configuration_digest(),
            selected_profile_id: configuration
                .selected_profile()
                .profile_id()
                .as_str()
                .into(),
            reference_generation: configuration.reference_generation(),
            analyzer_pin_id: configuration.analyzer_binding().accepted_pin_id().into(),
            analyzer_probe_report_id: configuration
                .analyzer_binding()
                .compatibility_probe_report_id()
                .into(),
            analyzer_configuration_digest: configuration
                .analyzer_binding()
                .analyzer_configuration_digest(),
            final_file_manifest_digest: inventory.manifest_digest(),
            project_generation_schema_version: PROJECT_GENERATION_SCHEMA_VERSION,
            derived_project_generation,
        })
    }

    #[must_use]
    pub fn candidate_id(&self) -> &str {
        &self.candidate_id
    }

    #[must_use]
    pub const fn project_configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.project_configuration_digest
    }

    #[must_use]
    pub fn selected_profile_id(&self) -> &str {
        &self.selected_profile_id
    }

    #[must_use]
    pub const fn reference_generation(&self) -> ReferenceGenerationId {
        self.reference_generation
    }

    #[must_use]
    pub fn analyzer_pin_id(&self) -> &str {
        &self.analyzer_pin_id
    }

    #[must_use]
    pub const fn analyzer_configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.analyzer_configuration_digest
    }

    #[must_use]
    pub const fn final_file_manifest_digest(&self) -> ContentDigest<CanonicalResult> {
        self.final_file_manifest_digest
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.derived_project_generation
    }

    pub fn validate(
        &self,
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
    ) -> ProjectResult<()> {
        let expected = Self::derive(configuration, inventory)?;
        if &expected == self {
            Ok(())
        } else {
            Err(ProjectError::new(
                ProjectErrorCode::GenerationCollision,
                ProjectPhase::Generation,
                "project generation candidate does not match its canonical derivation inputs",
            ))
        }
    }
}
