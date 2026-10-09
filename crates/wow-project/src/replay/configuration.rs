//! Input DTOs rebuild through existing owner validators. Load receipts are
//! reconstructed from source bytes before being attached to the configuration.
use super::invalid;
use crate::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy, ProjectConfiguration,
    ProjectConfigurationBuilder, ProjectId, ProjectKind, ProjectResult, ProjectSourceOriginId,
    ProjectWorkspaceId,
};
use serde::{Deserialize, Serialize};
use wow_core::{
    CanonicalResult, CapabilityId, ContentDigest, ProfileIdentity, ReferenceGenerationId,
};
use wow_emmy::EmmyBackendIdentity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayBackend {
    crate_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    crate_version: Option<String>,
    revision: String,
    tree: String,
    surface_sha256: String,
    compatibility_report_sha256: String,
}
impl ReplayBackend {
    fn capture(backend: &EmmyBackendIdentity) -> Self {
        Self {
            crate_name: backend.crate_name().into(),
            crate_version: backend.crate_version().map(str::to_owned),
            revision: backend.revision().into(),
            tree: backend.tree().into(),
            surface_sha256: backend.surface_sha256().into(),
            compatibility_report_sha256: backend.compatibility_report_sha256().into(),
        }
    }
    fn rebuild(&self) -> ProjectResult<EmmyBackendIdentity> {
        EmmyBackendIdentity::new(
            self.crate_name.clone(),
            self.crate_version.clone(),
            self.revision.clone(),
            self.tree.clone(),
            self.surface_sha256.clone(),
            self.compatibility_report_sha256.clone(),
        )
        .map_err(|_| invalid())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayBinding {
    analyzer_contract_id: String,
    accepted_pin_id: String,
    compatibility_probe_report_id: String,
    analyzer_configuration_digest: ContentDigest<CanonicalResult>,
    analyzer_fixture_contract_id: String,
    expected_library_contract_id: String,
    backend: ReplayBackend,
}
impl ReplayBinding {
    fn capture(binding: &AnalyzerBindingDeclaration) -> Self {
        Self {
            analyzer_contract_id: binding.analyzer_contract_id().into(),
            accepted_pin_id: binding.accepted_pin_id().into(),
            compatibility_probe_report_id: binding.compatibility_probe_report_id().into(),
            analyzer_configuration_digest: binding.analyzer_configuration_digest(),
            analyzer_fixture_contract_id: binding.analyzer_fixture_contract_id().into(),
            expected_library_contract_id: binding.expected_library_contract_id().into(),
            backend: ReplayBackend::capture(binding.backend()),
        }
    }
    fn rebuild(&self) -> ProjectResult<AnalyzerBindingDeclaration> {
        AnalyzerBindingDeclaration::new(
            self.analyzer_contract_id.clone(),
            self.accepted_pin_id.clone(),
            self.compatibility_probe_report_id.clone(),
            self.analyzer_configuration_digest,
            self.analyzer_fixture_contract_id.clone(),
            self.expected_library_contract_id.clone(),
            self.backend.rebuild()?,
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayCapabilities {
    mandatory: Vec<CapabilityId>,
    degradable: Vec<CapabilityId>,
    deferred: Vec<CapabilityId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayBudgets {
    max_files: u64,
    max_total_source_bytes: u64,
    max_single_file_bytes: u64,
    max_update_operations: u64,
    max_analyzer_facts: u64,
    max_generic_findings: u64,
    max_output_bytes: u64,
}
impl ReplayBudgets {
    fn capture(b: ProjectBudgetPolicy) -> Self {
        Self {
            max_files: b.max_files(),
            max_total_source_bytes: b.max_total_source_bytes(),
            max_single_file_bytes: b.max_single_file_bytes(),
            max_update_operations: b.max_update_operations(),
            max_analyzer_facts: b.max_analyzer_facts(),
            max_generic_findings: b.max_generic_findings(),
            max_output_bytes: b.max_output_bytes(),
        }
    }
    fn rebuild(&self) -> ProjectResult<ProjectBudgetPolicy> {
        ProjectBudgetPolicy::new(
            self.max_files,
            self.max_total_source_bytes,
            self.max_single_file_bytes,
            self.max_update_operations,
            self.max_analyzer_facts,
            self.max_generic_findings,
            self.max_output_bytes,
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReplayKind {
    Fixture,
    Repository,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplayConfiguration {
    project_id: ProjectId,
    kind: ReplayKind,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: String,
    profile: ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    analyzer: ReplayBinding,
    capabilities: ReplayCapabilities,
    budgets: ReplayBudgets,
    expected_configuration_digest: ContentDigest<CanonicalResult>,
}
impl ReplayConfiguration {
    pub(super) fn from_configuration(config: &ProjectConfiguration) -> ProjectResult<Self> {
        config.validate()?;
        let capabilities = config.capability_policy();
        Ok(Self {
            project_id: config.project_id().clone(),
            kind: match config.project_kind() {
                ProjectKind::Fixture => ReplayKind::Fixture,
                ProjectKind::Repository => ReplayKind::Repository,
            },
            workspace_id: config.workspace_id().clone(),
            source_origin_id: config.source_origin_id().clone(),
            logical_root: config.logical_root().as_str().into(),
            profile: config.selected_profile().clone(),
            reference_generation: config.reference_generation(),
            analyzer: ReplayBinding::capture(config.analyzer_binding()),
            capabilities: ReplayCapabilities {
                mandatory: capabilities.mandatory_for_publication().to_vec(),
                degradable: capabilities.degradable_for_publication().to_vec(),
                deferred: capabilities.explicitly_deferred().to_vec(),
            },
            budgets: ReplayBudgets::capture(config.budget_policy()),
            expected_configuration_digest: config.configuration_digest(),
        })
    }
    pub(super) fn profile(&self) -> &ProfileIdentity {
        &self.profile
    }
    pub(super) fn rebuild(
        &self,
        load_plan: Option<&crate::load::ProjectLoadPlan>,
        package_plans: Option<(
            &crate::load::ProjectPackageLoadPlan,
            &crate::load::ProjectPackageMainPlan,
        )>,
    ) -> ProjectResult<ProjectConfiguration> {
        let builder = ProjectConfigurationBuilder::new(
            self.project_id.clone(),
            match self.kind {
                ReplayKind::Fixture => ProjectKind::Fixture,
                ReplayKind::Repository => ProjectKind::Repository,
            },
            self.profile.clone(),
            self.reference_generation,
            self.analyzer.rebuild()?,
        )
        .workspace_id(self.workspace_id.clone())
        .source_origin_id(self.source_origin_id.clone())
        .logical_root(self.logical_root.clone())
        .capability_policy(ProjectCapabilityPolicy::new(
            self.capabilities.mandatory.clone(),
            self.capabilities.degradable.clone(),
            self.capabilities.deferred.clone(),
        )?)
        .budget_policy(self.budgets.rebuild()?);
        let builder = match load_plan {
            Some(plan) => builder.load_plan(plan)?,
            None => builder,
        };
        let builder = match package_plans {
            Some((load, main)) => builder.package_load_plan(load, main)?,
            None => builder,
        };
        let config = builder.build()?;
        if config.configuration_digest() != self.expected_configuration_digest {
            return Err(invalid());
        }
        Ok(config)
    }
}
