//! Input DTOs rebuild through existing owner validators. Load receipts are
//! reconstructed from source bytes before being attached to the configuration.
use super::{invalid, xml_classification::ReplayXmlClassification};
use crate::{
    AnalyzerBindingDeclaration, PackageXmlBindingProfile, ProjectBudgetPolicy,
    ProjectCapabilityPolicy, ProjectConfiguration, ProjectConfigurationBuilder, ProjectId,
    ProjectKind, ProjectResult, ProjectSourceOriginId, ProjectWorkspaceId,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::configuration::present_function_call_budget"
    )]
    function_calls: Option<wow_emmy::function_calls::FunctionCallBudget>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::configuration::present_function_call_evidence_profile"
    )]
    function_call_evidence_profile: Option<wow_emmy::function_calls::FunctionCallEvidenceProfile>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::configuration::present_hook_target_recipe"
    )]
    hook_target_recipe: Option<wow_emmy::function_calls::HookTargetRecipe>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::configuration::present_source_graph_budget"
    )]
    source_graph: Option<crate::graph::SourceGraphBudget>,
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
            function_calls: b.function_call_budget(),
            function_call_evidence_profile: b.function_call_evidence_profile(),
            hook_target_recipe: b.hook_target_recipe(),
            source_graph: b.source_graph_budget(),
        }
    }
    fn rebuild(&self) -> ProjectResult<ProjectBudgetPolicy> {
        let policy = ProjectBudgetPolicy::new(
            self.max_files,
            self.max_total_source_bytes,
            self.max_single_file_bytes,
            self.max_update_operations,
            self.max_analyzer_facts,
            self.max_generic_findings,
            self.max_output_bytes,
        )?;
        let policy = match self.function_calls {
            Some(budget) => policy.with_function_call_budget(budget),
            None => policy,
        };
        let policy = match self.function_call_evidence_profile {
            Some(profile) => policy.with_function_call_evidence_profile(profile),
            None => policy,
        };
        let policy = match self.hook_target_recipe {
            Some(recipe) => policy.with_hook_target_recipe(recipe),
            None => policy,
        };
        Ok(match self.source_graph {
            Some(budget) => policy.with_source_graph_budget(budget),
            None => policy,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReplayKind {
    Fixture,
    Repository,
    BlizzardUiPlatformSource,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    package_xml_binding_profile: Option<PackageXmlBindingProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    platform_graph_profile: Option<crate::PlatformGraphProfile>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null_xml_classification"
    )]
    xml_schema_classification: Option<ReplayXmlClassification>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null_physical_function_declarations"
    )]
    physical_function_declarations:
        Option<wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null_physical_function_signatures"
    )]
    physical_function_signatures: Option<wow_emmy::function_calls::PhysicalFunctionSignatureRecipe>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null_native_library_input"
    )]
    native_library_input: Option<crate::NativeLibraryInput>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null_native_api_uses"
    )]
    native_api_uses: Option<crate::NativeApiUseRecipe>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null_state_projection"
    )]
    state_projection: Option<crate::StateProjectionRecipe>,
}

fn non_null_state_projection<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::StateProjectionRecipe>, D::Error> {
    crate::StateProjectionRecipe::deserialize(deserializer).map(Some)
}

fn non_null_native_api_uses<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::NativeApiUseRecipe>, D::Error> {
    crate::NativeApiUseRecipe::deserialize(deserializer).map(Some)
}

fn non_null_native_library_input<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::NativeLibraryInput>, D::Error> {
    crate::NativeLibraryInput::deserialize(deserializer).map(Some)
}

fn non_null_xml_classification<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ReplayXmlClassification>, D::Error> {
    ReplayXmlClassification::deserialize(deserializer).map(Some)
}

fn non_null_physical_function_declarations<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe>, D::Error> {
    wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe::deserialize(deserializer).map(Some)
}

fn non_null_physical_function_signatures<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<wow_emmy::function_calls::PhysicalFunctionSignatureRecipe>, D::Error> {
    wow_emmy::function_calls::PhysicalFunctionSignatureRecipe::deserialize(deserializer).map(Some)
}

impl ReplayConfiguration {
    pub(super) const fn state_projection(&self) -> Option<crate::StateProjectionRecipe> {
        self.state_projection
    }
    pub(super) const fn native_library_input(&self) -> Option<&crate::NativeLibraryInput> {
        self.native_library_input.as_ref()
    }
    pub(super) const fn native_api_uses(&self) -> Option<crate::NativeApiUseRecipe> {
        self.native_api_uses
    }
    pub(super) const fn physical_function_signatures(
        &self,
    ) -> Option<wow_emmy::function_calls::PhysicalFunctionSignatureRecipe> {
        self.physical_function_signatures
    }
    pub(super) const fn physical_function_declarations(
        &self,
    ) -> Option<wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe> {
        self.physical_function_declarations
    }
    pub(super) const fn source_graph_budget(&self) -> Option<crate::graph::SourceGraphBudget> {
        self.budgets.source_graph
    }
    pub(super) const fn function_call_budget(
        &self,
    ) -> Option<wow_emmy::function_calls::FunctionCallBudget> {
        self.budgets.function_calls
    }

    pub(super) const fn function_call_evidence_profile(
        &self,
    ) -> Option<wow_emmy::function_calls::FunctionCallEvidenceProfile> {
        self.budgets.function_call_evidence_profile
    }
    pub(super) const fn hook_target_recipe(
        &self,
    ) -> Option<wow_emmy::function_calls::HookTargetRecipe> {
        self.budgets.hook_target_recipe
    }
    pub(super) fn from_configuration(config: &ProjectConfiguration) -> ProjectResult<Self> {
        config.validate()?;
        let kind = match config.project_kind() {
            ProjectKind::Fixture => ReplayKind::Fixture,
            ProjectKind::Repository => ReplayKind::Repository,
            ProjectKind::BlizzardUiPlatformSource => ReplayKind::BlizzardUiPlatformSource,
        };
        let capabilities = config.capability_policy();
        Ok(Self {
            project_id: config.project_id().clone(),
            kind,
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
            package_xml_binding_profile: config.package_xml_binding_profile(),
            platform_graph_profile: config.platform_graph_profile(),
            xml_schema_classification: config
                .xml_classification_profile()
                .map(|profile| ReplayXmlClassification::capture(profile)),
            physical_function_declarations: config.physical_function_declarations(),
            physical_function_signatures: config.physical_function_signatures(),
            native_library_input: config.native_library_input().cloned(),
            native_api_uses: config.native_api_uses(),
            state_projection: config.state_projection(),
        })
    }
    pub(super) fn profile(&self) -> &ProfileIdentity {
        &self.profile
    }
    pub(super) fn is_platform(&self) -> bool {
        self.kind == ReplayKind::BlizzardUiPlatformSource
    }
    pub(super) fn package_xml_binding_profile(&self) -> Option<PackageXmlBindingProfile> {
        self.package_xml_binding_profile
    }
    pub(super) fn platform_graph_profile(&self) -> Option<crate::PlatformGraphProfile> {
        self.platform_graph_profile
    }

    /// Refuse missing intent, a stray binding, or downgrade to an unselected recipe.
    pub(super) fn has_xml_classification(&self) -> ProjectResult<bool> {
        let selected = matches!(
            self.platform_graph_profile,
            Some(crate::PlatformGraphProfile::DirectCapabilitiesV1(capabilities))
                if capabilities.contains(crate::PlatformGraphCapability::XmlSchemaClassificationV1)
        );
        if selected != self.xml_schema_classification.is_some() {
            return Err(invalid());
        }
        Ok(selected)
    }
    pub(super) fn rebuild(
        &self,
        load_plan: Option<&crate::load::ProjectLoadPlan>,
        package_plans: Option<(
            &crate::load::ProjectPackageLoadPlan,
            &crate::load::ProjectPackageMainPlan,
        )>,
        platform: Option<std::sync::Arc<crate::platform_source::PlatformPackageSpecialization>>,
        stop: &std::sync::atomic::AtomicBool,
    ) -> ProjectResult<ProjectConfiguration> {
        if self.is_platform() != platform.is_some()
            || (platform.is_some() && (load_plan.is_some() || package_plans.is_some()))
        {
            return Err(invalid());
        }
        self.has_xml_classification()?;
        let classification = self
            .xml_schema_classification
            .as_ref()
            .map(|selection| {
                selection.rebuild(platform.as_ref().ok_or_else(invalid)?.source(), stop)
            })
            .transpose()?;
        let builder = ProjectConfigurationBuilder::new(
            self.project_id.clone(),
            match self.kind {
                ReplayKind::Fixture => ProjectKind::Fixture,
                ReplayKind::Repository => ProjectKind::Repository,
                ReplayKind::BlizzardUiPlatformSource => ProjectKind::BlizzardUiPlatformSource,
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
        let builder = match platform {
            Some(owner) => builder.platform_packages(owner)?,
            None => builder,
        };
        let builder = match self.package_xml_binding_profile {
            Some(profile) => builder.with_package_xml_bindings(profile),
            None => builder,
        };
        let builder = match self.platform_graph_profile {
            Some(profile) => builder.with_platform_graph_profile(profile),
            None => builder,
        };
        let builder = match self.physical_function_declarations {
            Some(recipe) => builder.with_physical_function_declarations(recipe),
            None => builder,
        };
        let builder = match self.physical_function_signatures {
            Some(recipe) => builder.with_physical_function_signatures(recipe),
            None => builder,
        };
        let builder = match classification {
            Some(profile) => builder.with_xml_classification(profile, stop)?,
            None => builder,
        };
        let builder = match self.state_projection {
            Some(recipe) => builder.with_state_projection(recipe),
            None => builder,
        };
        let config = builder.build()?;
        let config = match &self.native_library_input {
            Some(input) => config.with_native_library_input(input.clone())?,
            None => config,
        };
        let config = match self.native_api_uses {
            Some(recipe) => config.with_native_api_uses(recipe)?,
            None => config,
        };
        if config.configuration_digest() != self.expected_configuration_digest {
            return Err(invalid());
        }
        Ok(config)
    }
}
