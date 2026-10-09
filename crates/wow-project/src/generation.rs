use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId, ReferenceGenerationId};
use wow_emmy::LuaWorkspaceSnapshot;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    library_snapshot_ids: Option<Vec<Box<str>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call_facts: Option<bool>,
    derived_project_generation: ProjectGenerationId,
}

impl ProjectGenerationCandidate {
    /// Original v1 derivation for retained receipts. New publications use
    /// `derive_with_analysis` so Library content and fact profile participate in identity.
    pub fn derive(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
    ) -> ProjectResult<Self> {
        Self::derive_from_ids(configuration, inventory, None, None)
    }

    pub fn derive_with_libraries(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
        libraries: &[LuaWorkspaceSnapshot],
    ) -> ProjectResult<Self> {
        Self::derive_with_analysis(configuration, inventory, libraries, false)
    }

    /// Derive v2 from the exact Library corpus and owned analyzer fact profile.
    pub fn derive_with_analysis(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
        libraries: &[LuaWorkspaceSnapshot],
        function_call_facts: bool,
    ) -> ProjectResult<Self> {
        let mut ids = libraries
            .iter()
            .map(|library| Box::<str>::from(library.snapshot_id()))
            .collect::<Vec<_>>();
        ids.sort();
        Self::derive_from_ids(
            configuration,
            inventory,
            Some(ids),
            Some(function_call_facts),
        )
    }

    fn derive_from_ids(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
        library_snapshot_ids: Option<Vec<Box<str>>>,
        function_call_facts: Option<bool>,
    ) -> ProjectResult<Self> {
        configuration.validate()?;
        if let Some(plan) = configuration.load_plan() {
            plan.validate_main_files(inventory.files())?;
        }
        if let Some(plan) = configuration.package_main_plan() {
            plan.validate_main_files(inventory.files())?;
        }
        let manifest = inventory.manifest_entries();
        let version = match (
            library_snapshot_ids.is_some(),
            function_call_facts.is_some(),
        ) {
            (true, true) => 2,
            (false, false) => PROJECT_GENERATION_SCHEMA_VERSION,
            _ => {
                return Err(ProjectError::new(
                    ProjectErrorCode::GenerationDerivationFailed,
                    ProjectPhase::Generation,
                    "project generation recipe has incomplete analyzer inputs",
                ));
            }
        };
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
            #[serde(skip_serializing_if = "Option::is_none")]
            library_snapshot_ids: Option<&'a [Box<str>]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            function_call_facts: Option<bool>,
        }
        let input = DerivationInput {
            project_generation_schema_version: version,
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
            library_snapshot_ids: library_snapshot_ids.as_deref(),
            function_call_facts,
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
            if version == 2 {
                "wow-project/generation-candidate/e0-d/2"
            } else {
                "wow-project/generation-candidate/e0-d/1"
            },
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
            project_generation_schema_version: version,
            library_snapshot_ids,
            function_call_facts,
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

    #[must_use]
    pub const fn project_generation_schema_version(&self) -> u64 {
        self.project_generation_schema_version
    }

    /// Exact sorted Library snapshot identities in v2; absent only for legacy v1.
    #[must_use]
    pub fn library_snapshot_ids(&self) -> Option<&[Box<str>]> {
        self.library_snapshot_ids.as_deref()
    }

    #[must_use]
    pub const fn function_call_facts(&self) -> Option<bool> {
        self.function_call_facts
    }

    pub(crate) fn validate_function_call_facts(&self, enabled: bool) -> ProjectResult<()> {
        if self
            .function_call_facts
            .is_some_and(|expected| expected != enabled)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::AnalyzerSnapshotMismatch,
                ProjectPhase::Generation,
                "project generation fact profile differs from analyzer inputs",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_library_ids<'a>(
        &self,
        ids: impl IntoIterator<Item = &'a str>,
    ) -> ProjectResult<()> {
        if let Some(expected) = self.library_snapshot_ids() {
            let mut actual = ids.into_iter().collect::<Vec<_>>();
            actual.sort_unstable();
            if !expected.iter().map(AsRef::as_ref).eq(actual) {
                return Err(ProjectError::new(
                    ProjectErrorCode::AnalyzerSnapshotMismatch,
                    ProjectPhase::Generation,
                    "project generation Library identities differ from analyzer inputs",
                ));
            }
        }
        Ok(())
    }

    pub fn validate(
        &self,
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
    ) -> ProjectResult<()> {
        let expected = Self::derive_from_ids(
            configuration,
            inventory,
            self.library_snapshot_ids.clone(),
            self.function_call_facts,
        )?;
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
