use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wow_core::{
    CanonicalResult, CapabilityId, ContentDigest, NormalizedSourcePath, ProfileIdentity,
    ProfileKind, ReferenceGenerationId,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    function_calls::{PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe},
};

use crate::identity::canonical_digest;
use crate::{
    ProjectError, ProjectErrorCode, ProjectId, ProjectPhase, ProjectResult, ProjectSourceOriginId,
    ProjectWorkspaceId,
};

mod graph_capabilities;
mod platform;
mod xml_classification;
use crate::platform_source::{PlatformPackageBinding, PlatformPackageSpecialization};
pub use graph_capabilities::{PlatformGraphCapabilities, PlatformGraphCapability};
use platform::RetainedPlatformPackages;
use xml_classification::RetainedXmlClassification;

pub const PROJECT_CONFIGURATION_SCHEMA_VERSION: u64 = 1;
/// Frozen legacy receipt recipe. New publication selects generation v2 explicitly.
pub const PROJECT_GENERATION_SCHEMA_VERSION: u64 = 1;
pub const PROJECT_SNAPSHOT_SCHEMA_VERSION: u64 = 1;
pub const PROJECT_CONTRACT_ID: &str = "wow-project/e0-d/1";

/// Project source class supported by the configuration contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Fixture,
    Repository,
    BlizzardUiPlatformSource,
}

/// Explicit native Reference callable/use composition. Missing selection keeps
/// the ordinary source registry and previously admitted replay bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeApiUseRecipe {
    ReferenceV1,
}

/// Exact source projection selection. Absence retains the original rejection of
/// incomplete state paths, including its source/evidence catalog bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateProjectionRecipe {
    DynamicPrefixesV1,
}

/// Explicit additive analyzer recipe for genuine platform package inputs.
/// An absent selection retains the historical package analysis unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageXmlBindingProfile {
    SameSessionV1,
}

/// Opt-in graph recipe over the genuine retained platform package owners.
/// An absent selection preserves the published source graph recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformGraphProfile {
    PackageProjectionV1,
    PackageProjectionWithRawInventoryV1,
    DirectPlatformProducersWithRawInventoryV1,
    DirectCapabilitiesV1(PlatformGraphCapabilities),
}

impl PlatformGraphProfile {
    #[must_use]
    pub const fn is_direct(self) -> bool {
        matches!(
            self,
            Self::DirectPlatformProducersWithRawInventoryV1 | Self::DirectCapabilitiesV1(_)
        )
    }
}

/// Explicit publication capability policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCapabilityPolicy {
    mandatory_for_publication: Vec<CapabilityId>,
    degradable_for_publication: Vec<CapabilityId>,
    explicitly_deferred: Vec<CapabilityId>,
}

impl ProjectCapabilityPolicy {
    pub fn new(
        mut mandatory_for_publication: Vec<CapabilityId>,
        mut degradable_for_publication: Vec<CapabilityId>,
        mut explicitly_deferred: Vec<CapabilityId>,
    ) -> ProjectResult<Self> {
        mandatory_for_publication.sort();
        degradable_for_publication.sort();
        explicitly_deferred.sort();
        reject_duplicates(&mandatory_for_publication)?;
        reject_duplicates(&degradable_for_publication)?;
        reject_duplicates(&explicitly_deferred)?;
        let mandatory = mandatory_for_publication.iter().collect::<BTreeSet<_>>();
        let degradable = degradable_for_publication.iter().collect::<BTreeSet<_>>();
        let deferred = explicitly_deferred.iter().collect::<BTreeSet<_>>();
        if mandatory.is_empty()
            || !mandatory.is_disjoint(&degradable)
            || !mandatory.is_disjoint(&deferred)
            || !degradable.is_disjoint(&deferred)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidCapabilityPolicy,
                ProjectPhase::Configuration,
                "project capability policy overlaps or has no mandatory capabilities",
            ));
        }
        Ok(Self {
            mandatory_for_publication,
            degradable_for_publication,
            explicitly_deferred,
        })
    }

    /// Strict E0 policy: all implemented analyzer fact surfaces are mandatory.
    pub fn strict_e0() -> ProjectResult<Self> {
        Self::new(
            parse_capabilities(&[
                "project.fixture.configuration.valid",
                "project.fixture.files.complete",
                "project.source.registry.complete",
                "project.source.handle.resolve",
                "project.generation.coherent",
                "project.analyzer.snapshot.available",
                "project.analyzer.facts.available",
                "project.analyzer.generic_diagnostics.available",
                "emmy.session.ready",
                "emmy.library.loaded",
                "emmy.file.parsed",
                "emmy.file.diagnostics",
                "emmy.fact.references",
                "emmy.fact.calls",
                "emmy.fact.local_bindings",
                "emmy.fact.local_flow",
                "emmy.fact.operations",
                "emmy.fact.guards",
                "emmy.fact.control_flow",
                "emmy.source_coordinates.exact",
            ])?,
            Vec::new(),
            deferred_e0()?,
        )
    }

    /// Explicit degraded E0 policy. Structural/session/source-coordinate seams
    /// remain mandatory while per-file diagnostics and semantic facts may fail.
    pub fn degraded_e0() -> ProjectResult<Self> {
        Self::new(
            parse_capabilities(&[
                "project.fixture.configuration.valid",
                "project.fixture.files.complete",
                "project.source.registry.complete",
                "project.source.handle.resolve",
                "project.generation.coherent",
                "project.analyzer.snapshot.available",
                "emmy.session.ready",
                "emmy.library.loaded",
                "emmy.source_coordinates.exact",
            ])?,
            parse_capabilities(&[
                "project.analyzer.facts.available",
                "project.analyzer.generic_diagnostics.available",
                "emmy.file.parsed",
                "emmy.file.diagnostics",
                "emmy.fact.references",
                "emmy.fact.calls",
                "emmy.fact.local_bindings",
                "emmy.fact.local_flow",
                "emmy.fact.operations",
                "emmy.fact.guards",
                "emmy.fact.control_flow",
            ])?,
            deferred_e0()?,
        )
    }

    #[must_use]
    pub fn mandatory_for_publication(&self) -> &[CapabilityId] {
        &self.mandatory_for_publication
    }

    #[must_use]
    pub fn degradable_for_publication(&self) -> &[CapabilityId] {
        &self.degradable_for_publication
    }

    #[must_use]
    pub fn explicitly_deferred(&self) -> &[CapabilityId] {
        &self.explicitly_deferred
    }

    #[must_use]
    pub fn is_mandatory(&self, capability: &CapabilityId) -> bool {
        self.mandatory_for_publication
            .binary_search(capability)
            .is_ok()
    }

    #[must_use]
    pub fn is_degradable(&self, capability: &CapabilityId) -> bool {
        self.degradable_for_publication
            .binary_search(capability)
            .is_ok()
    }

    #[must_use]
    pub fn is_deferred(&self, capability: &CapabilityId) -> bool {
        self.explicitly_deferred.binary_search(capability).is_ok()
    }
}

/// Explicit source/analysis/output limits, validated on construction and decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "ProjectBudgetPolicyInput")]
pub struct ProjectBudgetPolicy {
    max_files: u64,
    max_total_source_bytes: u64,
    max_single_file_bytes: u64,
    max_update_operations: u64,
    max_analyzer_facts: u64,
    max_generic_findings: u64,
    max_output_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_calls: Option<wow_emmy::function_calls::FunctionCallBudget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call_evidence_profile: Option<wow_emmy::function_calls::FunctionCallEvidenceProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hook_target_recipe: Option<wow_emmy::function_calls::HookTargetRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_graph: Option<crate::graph::SourceGraphBudget>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectBudgetPolicyInput {
    max_files: u64,
    max_total_source_bytes: u64,
    max_single_file_bytes: u64,
    max_update_operations: u64,
    max_analyzer_facts: u64,
    max_generic_findings: u64,
    max_output_bytes: u64,
    #[serde(default, deserialize_with = "present_function_call_budget")]
    function_calls: Option<wow_emmy::function_calls::FunctionCallBudget>,
    #[serde(default, deserialize_with = "present_function_call_evidence_profile")]
    function_call_evidence_profile: Option<wow_emmy::function_calls::FunctionCallEvidenceProfile>,
    #[serde(default, deserialize_with = "present_hook_target_recipe")]
    hook_target_recipe: Option<wow_emmy::function_calls::HookTargetRecipe>,
    #[serde(default, deserialize_with = "present_source_graph_budget")]
    source_graph: Option<crate::graph::SourceGraphBudget>,
}

pub(crate) fn present_source_graph_budget<'de, D>(
    deserializer: D,
) -> Result<Option<crate::graph::SourceGraphBudget>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    crate::graph::SourceGraphBudget::deserialize(deserializer).map(Some)
}

pub(crate) fn present_function_call_budget<'de, D>(
    deserializer: D,
) -> Result<Option<wow_emmy::function_calls::FunctionCallBudget>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    wow_emmy::function_calls::FunctionCallBudget::deserialize(deserializer).map(Some)
}

pub(crate) fn present_hook_target_recipe<'de, D>(
    deserializer: D,
) -> Result<Option<wow_emmy::function_calls::HookTargetRecipe>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    wow_emmy::function_calls::HookTargetRecipe::deserialize(deserializer).map(Some)
}

pub(crate) fn present_function_call_evidence_profile<'de, D>(
    deserializer: D,
) -> Result<Option<wow_emmy::function_calls::FunctionCallEvidenceProfile>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    wow_emmy::function_calls::FunctionCallEvidenceProfile::deserialize(deserializer).map(Some)
}

impl TryFrom<ProjectBudgetPolicyInput> for ProjectBudgetPolicy {
    type Error = ProjectError;

    fn try_from(input: ProjectBudgetPolicyInput) -> ProjectResult<Self> {
        let policy = Self::new(
            input.max_files,
            input.max_total_source_bytes,
            input.max_single_file_bytes,
            input.max_update_operations,
            input.max_analyzer_facts,
            input.max_generic_findings,
            input.max_output_bytes,
        )?;
        let policy = match input.function_calls {
            Some(budget) => policy.with_function_call_budget(budget),
            None => policy,
        };
        let policy = match input.function_call_evidence_profile {
            Some(profile) => policy.with_function_call_evidence_profile(profile),
            None => policy,
        };
        let policy = match input.hook_target_recipe {
            Some(recipe) => policy.with_hook_target_recipe(recipe),
            None => policy,
        };
        Ok(match input.source_graph {
            Some(budget) => policy.with_source_graph_budget(budget),
            None => policy,
        })
    }
}

impl ProjectBudgetPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_files: u64,
        max_total_source_bytes: u64,
        max_single_file_bytes: u64,
        max_update_operations: u64,
        max_analyzer_facts: u64,
        max_generic_findings: u64,
        max_output_bytes: u64,
    ) -> ProjectResult<Self> {
        let valid = max_files > 0
            && max_total_source_bytes > 0
            && max_single_file_bytes > 0
            && max_single_file_bytes <= max_total_source_bytes
            && max_update_operations > 0
            && max_analyzer_facts > 0
            && max_generic_findings > 0
            && max_output_bytes > 0;
        if !valid {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Configuration,
                "project budget contains zero or internally inconsistent limits",
            ));
        }
        Ok(Self {
            max_files,
            max_total_source_bytes,
            max_single_file_bytes,
            max_update_operations,
            max_analyzer_facts,
            max_generic_findings,
            max_output_bytes,
            function_calls: None,
            function_call_evidence_profile: None,
            hook_target_recipe: None,
            source_graph: None,
        })
    }

    pub fn fixture_e0() -> ProjectResult<Self> {
        Self::new(
            64,
            2 * 1024 * 1024,
            256 * 1024,
            64,
            262_144,
            65_536,
            16 * 1024 * 1024,
        )
    }

    #[must_use]
    pub const fn max_files(self) -> u64 {
        self.max_files
    }

    #[must_use]
    pub const fn max_total_source_bytes(self) -> u64 {
        self.max_total_source_bytes
    }

    #[must_use]
    pub const fn max_single_file_bytes(self) -> u64 {
        self.max_single_file_bytes
    }

    #[must_use]
    pub const fn max_update_operations(self) -> u64 {
        self.max_update_operations
    }

    #[must_use]
    pub const fn max_analyzer_facts(self) -> u64 {
        self.max_analyzer_facts
    }

    #[must_use]
    pub const fn max_generic_findings(self) -> u64 {
        self.max_generic_findings
    }

    #[must_use]
    pub const fn max_output_bytes(self) -> u64 {
        self.max_output_bytes
    }

    /// Selected sidecar capacity is independent of aggregate project facts/output.
    #[must_use]
    pub const fn with_function_call_budget(
        mut self,
        budget: wow_emmy::function_calls::FunctionCallBudget,
    ) -> Self {
        self.function_calls = Some(budget);
        self
    }

    #[must_use]
    pub const fn function_call_budget(
        self,
    ) -> Option<wow_emmy::function_calls::FunctionCallBudget> {
        self.function_calls
    }

    /// Closed same-call literal lookup recipe consumed by native W5.
    #[must_use]
    pub const fn with_hook_target_recipe(
        mut self,
        recipe: wow_emmy::function_calls::HookTargetRecipe,
    ) -> Self {
        self.hook_target_recipe = Some(recipe);
        self
    }
    #[must_use]
    pub const fn hook_target_recipe(self) -> Option<wow_emmy::function_calls::HookTargetRecipe> {
        self.hook_target_recipe
    }

    /// Select an opt-in typed evidence recipe for a named function-call consumer.
    #[must_use]
    pub const fn with_function_call_evidence_profile(
        mut self,
        profile: wow_emmy::function_calls::FunctionCallEvidenceProfile,
    ) -> Self {
        self.function_call_evidence_profile = Some(profile);
        self
    }

    #[must_use]
    pub const fn function_call_evidence_profile(
        self,
    ) -> Option<wow_emmy::function_calls::FunctionCallEvidenceProfile> {
        self.function_call_evidence_profile
    }

    #[must_use]
    pub const fn with_source_graph_budget(
        mut self,
        budget: crate::graph::SourceGraphBudget,
    ) -> Self {
        self.source_graph = Some(budget);
        self
    }

    #[must_use]
    pub const fn source_graph_budget(self) -> Option<crate::graph::SourceGraphBudget> {
        self.source_graph
    }
}

/// Exact accepted analyzer contract and configuration identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerBindingDeclaration {
    analyzer_contract_id: Box<str>,
    accepted_pin_id: Box<str>,
    compatibility_probe_report_id: Box<str>,
    analyzer_configuration_digest: ContentDigest<CanonicalResult>,
    analyzer_fixture_contract_id: Box<str>,
    expected_library_contract_id: Box<str>,
    backend: EmmyBackendIdentity,
}

impl AnalyzerBindingDeclaration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        analyzer_contract_id: impl Into<String>,
        accepted_pin_id: impl Into<String>,
        compatibility_probe_report_id: impl Into<String>,
        analyzer_configuration_digest: ContentDigest<CanonicalResult>,
        analyzer_fixture_contract_id: impl Into<String>,
        expected_library_contract_id: impl Into<String>,
        backend: EmmyBackendIdentity,
    ) -> ProjectResult<Self> {
        let value = Self {
            analyzer_contract_id: checked_text(analyzer_contract_id.into(), "analyzer contract")?,
            accepted_pin_id: checked_text(accepted_pin_id.into(), "analyzer pin")?,
            compatibility_probe_report_id: checked_text(
                compatibility_probe_report_id.into(),
                "analyzer compatibility report",
            )?,
            analyzer_configuration_digest,
            analyzer_fixture_contract_id: checked_text(
                analyzer_fixture_contract_id.into(),
                "analyzer fixture contract",
            )?,
            expected_library_contract_id: checked_text(
                expected_library_contract_id.into(),
                "analyzer Library contract",
            )?,
            backend,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ProjectResult<()> {
        let exact = self.backend.crate_name() == "emmylua_code_analysis"
            && self.backend.crate_version() == Some(EMMYLUA_CODE_ANALYSIS_VERSION)
            && self.backend.revision() == EMMYLUA_REVISION
            && self.backend.tree() == EMMYLUA_TREE
            && self.compatibility_probe_report_id.as_ref()
                == self.backend.compatibility_report_sha256();
        if exact && !is_floating(self.accepted_pin_id.as_ref()) {
            Ok(())
        } else {
            Err(ProjectError::new(
                ProjectErrorCode::InvalidAnalyzerBinding,
                ProjectPhase::Configuration,
                "analyzer binding does not match the compiled accepted pin/probe identity",
            ))
        }
    }

    #[must_use]
    pub fn analyzer_contract_id(&self) -> &str {
        &self.analyzer_contract_id
    }

    #[must_use]
    pub fn accepted_pin_id(&self) -> &str {
        &self.accepted_pin_id
    }

    #[must_use]
    pub fn compatibility_probe_report_id(&self) -> &str {
        &self.compatibility_probe_report_id
    }

    #[must_use]
    pub const fn analyzer_configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.analyzer_configuration_digest
    }

    #[must_use]
    pub fn analyzer_fixture_contract_id(&self) -> &str {
        &self.analyzer_fixture_contract_id
    }

    #[must_use]
    pub fn expected_library_contract_id(&self) -> &str {
        &self.expected_library_contract_id
    }

    #[must_use]
    pub const fn backend(&self) -> &EmmyBackendIdentity {
        &self.backend
    }
}

/// Validated, content-addressed project configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfiguration {
    project_id: ProjectId,
    project_kind: ProjectKind,
    configuration_schema_version: u64,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: NormalizedSourcePath,
    selected_profile: ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    analyzer_binding: AnalyzerBindingDeclaration,
    capability_policy: ProjectCapabilityPolicy,
    budget_policy: ProjectBudgetPolicy,
    configuration_digest: ContentDigest<CanonicalResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_plan_digest: Option<ContentDigest<CanonicalResult>>,
    #[serde(skip)]
    retained_load_plan: Option<Arc<crate::load::ProjectLoadPlan>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_load_plan_digest: Option<ContentDigest<CanonicalResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_main_plan_digest: Option<ContentDigest<CanonicalResult>>,
    #[serde(skip)]
    retained_package_load_plan: Option<Arc<crate::load::ProjectPackageLoadPlan>>,
    #[serde(skip)]
    retained_package_main_plan: Option<Arc<crate::load::ProjectPackageMainPlan>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_package_binding_digest: Option<ContentDigest<CanonicalResult>>,
    #[serde(skip)]
    retained_platform_packages: Option<RetainedPlatformPackages>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_xml_binding_profile: Option<PackageXmlBindingProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_graph_profile: Option<PlatformGraphProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml_schema_classification: Option<RetainedXmlClassification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    physical_function_declarations: Option<PhysicalFunctionDeclarationRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    physical_function_signatures: Option<PhysicalFunctionSignatureRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_library_input_digest: Option<ContentDigest<CanonicalResult>>,
    #[serde(skip)]
    native_library_input: Option<crate::NativeLibraryInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_api_uses: Option<NativeApiUseRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_projection: Option<StateProjectionRecipe>,
}

impl ProjectConfiguration {
    pub fn validate(&self) -> ProjectResult<()> {
        self.selected_profile.validate().map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                format!("selected profile is invalid: {source}"),
            )
        })?;
        self.analyzer_binding.validate()?;
        if self.state_projection.is_some()
            && (self.platform_graph_profile.is_some()
                || self.project_kind == ProjectKind::BlizzardUiPlatformSource)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "state prefix projection requires the ordinary source graph",
            ));
        }
        if self.native_api_uses.is_some()
            && (self.native_library_input.is_none()
                || self.platform_graph_profile.is_some()
                || self.project_kind == ProjectKind::BlizzardUiPlatformSource)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "native API uses require the ordinary retained native Library input",
            ));
        }
        if self.native_library_input_digest
            != self
                .native_library_input
                .as_ref()
                .map(crate::NativeLibraryInput::digest)
            || (self.native_library_input.is_some()
                && self.project_kind == ProjectKind::BlizzardUiPlatformSource)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "native Library producer input is not retained by its configuration",
            ));
        }
        if self.load_plan_digest != self.retained_load_plan.as_ref().map(|plan| plan.digest()) {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "load provenance is not retained by its configuration",
            ));
        }
        if let Some(plan) = &self.retained_load_plan {
            plan.validate_profile(&self.selected_profile)?;
        }
        if self.package_load_plan_digest
            != self
                .retained_package_load_plan
                .as_ref()
                .map(|plan| plan.digest())
            || self.package_main_plan_digest
                != self
                    .retained_package_main_plan
                    .as_ref()
                    .map(|plan| plan.digest())
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "package load provenance is not retained by its configuration",
            ));
        }
        if self.load_plan_digest.is_some() && self.package_load_plan_digest.is_some() {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "single-package and package-universe load plans are mutually exclusive",
            ));
        }
        match (
            &self.retained_package_load_plan,
            &self.retained_package_main_plan,
        ) {
            (Some(load), Some(main)) => {
                load.validate_profile(&self.selected_profile)?;
                main.validate_load_plan(load)?;
            }
            (None, None) => {}
            _ => {
                return Err(ProjectError::new(
                    ProjectErrorCode::InvalidConfiguration,
                    ProjectPhase::Configuration,
                    "package load and Main namespace plans must be retained together",
                ));
            }
        }
        if self.platform_package_binding_digest
            != self
                .retained_platform_packages
                .as_ref()
                .map(|packages| packages.binding().binding_digest())
            || (self.project_kind == ProjectKind::BlizzardUiPlatformSource)
                != self.retained_platform_packages.is_some()
        {
            return Err(platform::invalid());
        }
        if let Some(packages) = &self.retained_platform_packages {
            packages.validate(
                &self.selected_profile,
                self.reference_generation,
                self.package_load_plan(),
                self.package_main_plan(),
            )?;
        }
        // The closed hook recipe also runs in the explicit Main/Library session.
        // Other selected call budgets still require retained platform packages.
        let plain_function_call_budget = self.budget_policy.function_call_budget().is_some()
            && !(self.budget_policy.hook_target_recipe().is_some()
                && matches!(
                    self.project_kind,
                    ProjectKind::Fixture | ProjectKind::Repository
                ));
        if (plain_function_call_budget
            || self
                .budget_policy
                .function_call_evidence_profile()
                .is_some())
            && self
                .platform_packages()
                .and_then(|packages| packages.retention_limits())
                .is_none()
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Configuration,
                "selected function-call budgets require native retained packages",
            ));
        }
        if (self.budget_policy.hook_target_recipe().is_some()
            || self
                .budget_policy
                .function_call_evidence_profile()
                .is_some())
            && self.budget_policy.function_call_budget().is_none()
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Configuration,
                "selected function-call evidence profiles require an explicit function-call budget",
            ));
        }
        if self.budget_policy.hook_target_recipe().is_some() {
            let ordinary_main = matches!(
                self.project_kind,
                ProjectKind::Fixture | ProjectKind::Repository
            ) && self.load_plan_digest.is_none()
                && self.package_load_plan_digest.is_none()
                && self.package_main_plan_digest.is_none()
                && self.package_xml_binding_profile.is_none()
                && self.platform_graph_profile.is_none()
                && self.xml_schema_classification.is_none();
            if !ordinary_main
                && !(matches!(self.project_kind, ProjectKind::BlizzardUiPlatformSource)
                    && self
                        .platform_packages()
                        .and_then(|packages| packages.retention_limits())
                        .is_some())
            {
                return Err(ProjectError::new(
                    ProjectErrorCode::InvalidBudgetPolicy,
                    ProjectPhase::Configuration,
                    "selected hook-target literal recipe requires retained platform packages or explicit Main inputs",
                ));
            }
        }
        if self.budget_policy.source_graph_budget().is_some()
            && (self.platform_graph_profile.is_none()
                || self
                    .platform_packages()
                    .and_then(|packages| packages.retention_limits())
                    .is_none())
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Configuration,
                "selected graph metadata budgets require native retained platform graph input",
            ));
        }
        if self.package_xml_binding_profile.is_some()
            && (self.project_kind != ProjectKind::BlizzardUiPlatformSource
                || self.retained_platform_packages.is_none())
        {
            return Err(platform::invalid());
        }
        if self.platform_graph_profile.is_some()
            && (self.project_kind != ProjectKind::BlizzardUiPlatformSource
                || self.retained_platform_packages.is_none()
                || self.package_xml_binding_profile
                    != Some(PackageXmlBindingProfile::SameSessionV1))
        {
            return Err(platform::invalid());
        }
        let primary_callables = matches!(
            self.project_kind,
            ProjectKind::Fixture | ProjectKind::Repository
        ) && self.load_plan_digest.is_none()
            && self.package_load_plan_digest.is_none()
            && self.package_main_plan_digest.is_none()
            && self.package_xml_binding_profile.is_none()
            && self.platform_graph_profile.is_none()
            && self.xml_schema_classification.is_none()
            && matches!(
                self.physical_function_declarations,
                None | Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)
            )
            && matches!(
                self.physical_function_signatures,
                None | Some(PhysicalFunctionSignatureRecipe::IndexedTypesV1)
            );
        let platform_callables = self.platform_packages().is_some()
            && matches!(
                self.platform_graph_profile,
                Some(PlatformGraphProfile::DirectCapabilitiesV1(_))
            );
        if (self.physical_function_declarations.is_some()
            || self.physical_function_signatures.is_some())
            && !(primary_callables || platform_callables)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "physical callable recipes require an admitted Primary or direct platform configuration",
            ));
        }
        let classification_selected = matches!(
            self.platform_graph_profile,
            Some(PlatformGraphProfile::DirectCapabilitiesV1(capabilities))
                if capabilities.contains(PlatformGraphCapability::XmlSchemaClassificationV1)
        );
        if classification_selected != self.xml_schema_classification.is_some() {
            return Err(platform::invalid());
        }
        if let Some(classification) = &self.xml_schema_classification {
            classification.validate(self.platform_packages().ok_or_else(platform::invalid)?)?;
        }
        if self.configuration_schema_version != PROJECT_CONFIGURATION_SCHEMA_VERSION {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "unsupported project configuration schema version",
            ));
        }
        let kind_matches = matches!(
            (self.project_kind, self.selected_profile.profile_kind()),
            (ProjectKind::Fixture, ProfileKind::Fixture)
                | (ProjectKind::Repository, ProfileKind::Release)
                | (
                    ProjectKind::BlizzardUiPlatformSource,
                    ProfileKind::Fixture | ProfileKind::Release
                )
        );
        if !kind_matches {
            return Err(ProjectError::new(
                ProjectErrorCode::FixtureReleaseMasquerade,
                ProjectPhase::Configuration,
                "project kind and selected profile class do not match",
            ));
        }
        let expected = configuration_digest(
            &self.project_id,
            self.project_kind,
            &self.workspace_id,
            &self.source_origin_id,
            &self.logical_root,
            &self.selected_profile,
            self.reference_generation,
            &self.analyzer_binding,
            &self.capability_policy,
            self.budget_policy,
            self.load_plan_digest,
            self.package_load_plan_digest,
            self.package_main_plan_digest,
            self.platform_package_binding_digest,
            self.package_xml_binding_profile,
            self.platform_graph_profile,
            self.xml_schema_classification.as_ref(),
            self.physical_function_declarations,
            self.physical_function_signatures,
            self.native_library_input_digest,
            self.native_api_uses,
            self.state_projection,
        )?;
        if expected != self.configuration_digest {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "project configuration digest does not match its semantic fields",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    #[must_use]
    pub const fn project_kind(&self) -> ProjectKind {
        self.project_kind
    }

    #[must_use]
    pub const fn workspace_id(&self) -> &ProjectWorkspaceId {
        &self.workspace_id
    }

    #[must_use]
    pub const fn source_origin_id(&self) -> &ProjectSourceOriginId {
        &self.source_origin_id
    }

    #[must_use]
    pub const fn logical_root(&self) -> &NormalizedSourcePath {
        &self.logical_root
    }

    #[must_use]
    pub const fn selected_profile(&self) -> &ProfileIdentity {
        &self.selected_profile
    }

    #[must_use]
    pub const fn reference_generation(&self) -> ReferenceGenerationId {
        self.reference_generation
    }

    #[must_use]
    pub const fn analyzer_binding(&self) -> &AnalyzerBindingDeclaration {
        &self.analyzer_binding
    }

    #[must_use]
    pub const fn capability_policy(&self) -> &ProjectCapabilityPolicy {
        &self.capability_policy
    }

    #[must_use]
    pub const fn budget_policy(&self) -> ProjectBudgetPolicy {
        self.budget_policy
    }

    #[must_use]
    pub fn load_plan(&self) -> Option<&crate::load::ProjectLoadPlan> {
        self.retained_load_plan.as_deref()
    }

    #[must_use]
    pub const fn load_plan_digest(&self) -> Option<ContentDigest<CanonicalResult>> {
        self.load_plan_digest
    }

    #[must_use]
    pub fn package_load_plan(&self) -> Option<&crate::load::ProjectPackageLoadPlan> {
        self.retained_package_load_plan.as_deref()
    }

    #[must_use]
    pub fn package_main_plan(&self) -> Option<&crate::load::ProjectPackageMainPlan> {
        self.retained_package_main_plan.as_deref()
    }

    #[must_use]
    pub const fn package_load_plan_digest(&self) -> Option<ContentDigest<CanonicalResult>> {
        self.package_load_plan_digest
    }

    #[must_use]
    pub const fn package_main_plan_digest(&self) -> Option<ContentDigest<CanonicalResult>> {
        self.package_main_plan_digest
    }

    /// Genuine retained source admission and native package/Main owners.
    #[must_use]
    pub fn platform_packages(&self) -> Option<&PlatformPackageSpecialization> {
        self.retained_platform_packages
            .as_ref()
            .map(RetainedPlatformPackages::owner)
    }

    #[must_use]
    pub fn platform_package_binding(&self) -> Option<&PlatformPackageBinding> {
        self.retained_platform_packages
            .as_ref()
            .map(RetainedPlatformPackages::binding)
    }

    #[must_use]
    pub const fn platform_package_binding_digest(&self) -> Option<ContentDigest<CanonicalResult>> {
        self.platform_package_binding_digest
    }

    #[must_use]
    pub const fn package_xml_binding_profile(&self) -> Option<PackageXmlBindingProfile> {
        self.package_xml_binding_profile
    }

    #[must_use]
    pub const fn platform_graph_profile(&self) -> Option<PlatformGraphProfile> {
        self.platform_graph_profile
    }

    /// Exact selected native Main declaration recipe. Absence preserves old reports.
    #[must_use]
    pub const fn physical_function_declarations(
        &self,
    ) -> Option<PhysicalFunctionDeclarationRecipe> {
        self.physical_function_declarations
    }

    #[must_use]
    pub const fn physical_function_signatures(&self) -> Option<PhysicalFunctionSignatureRecipe> {
        self.physical_function_signatures
    }

    /// Sealed same-source classification owner, never reconstructed from receipt JSON.
    #[must_use]
    pub fn xml_classification_profile(
        &self,
    ) -> Option<&Arc<crate::load::schema::classification::ReviewedXmlClassificationProfile>> {
        self.xml_schema_classification
            .as_ref()
            .map(RetainedXmlClassification::owner)
    }

    #[must_use]
    pub const fn configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.configuration_digest
    }

    /// Borrow original producer inputs. Qualification belongs to service, not Project.
    #[must_use]
    pub const fn native_library_input(&self) -> Option<&crate::NativeLibraryInput> {
        self.native_library_input.as_ref()
    }

    #[must_use]
    pub const fn native_api_uses(&self) -> Option<NativeApiUseRecipe> {
        self.native_api_uses
    }

    /// Exact selected state source interpretation; no runtime existence follows.
    #[must_use]
    pub const fn state_projection(&self) -> Option<StateProjectionRecipe> {
        self.state_projection
    }

    pub(crate) fn with_state_projection(
        mut self,
        recipe: StateProjectionRecipe,
    ) -> ProjectResult<Self> {
        if self.state_projection.is_some() {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "state projection is already selected",
            ));
        }
        self.state_projection = Some(recipe);
        self.configuration_digest = configuration_digest(
            &self.project_id,
            self.project_kind,
            &self.workspace_id,
            &self.source_origin_id,
            &self.logical_root,
            &self.selected_profile,
            self.reference_generation,
            &self.analyzer_binding,
            &self.capability_policy,
            self.budget_policy,
            self.load_plan_digest,
            self.package_load_plan_digest,
            self.package_main_plan_digest,
            self.platform_package_binding_digest,
            self.package_xml_binding_profile,
            self.platform_graph_profile,
            self.xml_schema_classification.as_ref(),
            self.physical_function_declarations,
            self.physical_function_signatures,
            self.native_library_input_digest,
            self.native_api_uses,
            self.state_projection,
        )?;
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn with_native_library_input(
        mut self,
        input: crate::NativeLibraryInput,
    ) -> ProjectResult<Self> {
        if self.native_library_input.is_some()
            || self.project_kind == ProjectKind::BlizzardUiPlatformSource
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "native Library producer input is already selected",
            ));
        }
        self.native_library_input_digest = Some(input.digest());
        self.native_library_input = Some(input);
        self.configuration_digest = configuration_digest(
            &self.project_id,
            self.project_kind,
            &self.workspace_id,
            &self.source_origin_id,
            &self.logical_root,
            &self.selected_profile,
            self.reference_generation,
            &self.analyzer_binding,
            &self.capability_policy,
            self.budget_policy,
            self.load_plan_digest,
            self.package_load_plan_digest,
            self.package_main_plan_digest,
            self.platform_package_binding_digest,
            self.package_xml_binding_profile,
            self.platform_graph_profile,
            self.xml_schema_classification.as_ref(),
            self.physical_function_declarations,
            self.physical_function_signatures,
            self.native_library_input_digest,
            self.native_api_uses,
            self.state_projection,
        )?;
        self.validate()?;
        Ok(self)
    }
    pub(crate) fn with_native_api_uses(
        mut self,
        recipe: NativeApiUseRecipe,
    ) -> ProjectResult<Self> {
        if self.native_api_uses.is_some()
            || self.native_library_input.is_none()
            || self.platform_graph_profile.is_some()
            || self.project_kind == ProjectKind::BlizzardUiPlatformSource
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "native API use selection requires an unselected ordinary native project",
            ));
        }
        self.native_api_uses = Some(recipe);
        self.configuration_digest = configuration_digest(
            &self.project_id,
            self.project_kind,
            &self.workspace_id,
            &self.source_origin_id,
            &self.logical_root,
            &self.selected_profile,
            self.reference_generation,
            &self.analyzer_binding,
            &self.capability_policy,
            self.budget_policy,
            self.load_plan_digest,
            self.package_load_plan_digest,
            self.package_main_plan_digest,
            self.platform_package_binding_digest,
            self.package_xml_binding_profile,
            self.platform_graph_profile,
            self.xml_schema_classification.as_ref(),
            self.physical_function_declarations,
            self.physical_function_signatures,
            self.native_library_input_digest,
            self.native_api_uses,
            self.state_projection,
        )?;
        self.validate()?;
        Ok(self)
    }
}

/// Builder requiring every E0 project input explicitly before construction.
#[derive(Debug, Clone)]
pub struct ProjectConfigurationBuilder {
    project_id: ProjectId,
    project_kind: ProjectKind,
    selected_profile: ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    analyzer_binding: AnalyzerBindingDeclaration,
    workspace_id: Option<ProjectWorkspaceId>,
    source_origin_id: Option<ProjectSourceOriginId>,
    logical_root: Option<String>,
    capability_policy: Option<ProjectCapabilityPolicy>,
    budget_policy: Option<ProjectBudgetPolicy>,
    load_plan_digest: Option<ContentDigest<CanonicalResult>>,
    retained_load_plan: Option<Arc<crate::load::ProjectLoadPlan>>,
    package_load_plan_digest: Option<ContentDigest<CanonicalResult>>,
    package_main_plan_digest: Option<ContentDigest<CanonicalResult>>,
    retained_package_load_plan: Option<Arc<crate::load::ProjectPackageLoadPlan>>,
    retained_package_main_plan: Option<Arc<crate::load::ProjectPackageMainPlan>>,
    retained_platform_packages: Option<RetainedPlatformPackages>,
    package_xml_binding_profile: Option<PackageXmlBindingProfile>,
    platform_graph_profile: Option<PlatformGraphProfile>,
    xml_schema_classification: Option<RetainedXmlClassification>,
    physical_function_declarations: Option<PhysicalFunctionDeclarationRecipe>,
    physical_function_signatures: Option<PhysicalFunctionSignatureRecipe>,
    state_projection: Option<StateProjectionRecipe>,
}

impl ProjectConfigurationBuilder {
    #[must_use]
    pub fn new(
        project_id: ProjectId,
        project_kind: ProjectKind,
        selected_profile: ProfileIdentity,
        reference_generation: ReferenceGenerationId,
        analyzer_binding: AnalyzerBindingDeclaration,
    ) -> Self {
        Self {
            project_id,
            project_kind,
            selected_profile,
            reference_generation,
            analyzer_binding,
            workspace_id: None,
            source_origin_id: None,
            logical_root: None,
            capability_policy: None,
            budget_policy: None,
            load_plan_digest: None,
            retained_load_plan: None,
            package_load_plan_digest: None,
            package_main_plan_digest: None,
            retained_package_load_plan: None,
            retained_package_main_plan: None,
            retained_platform_packages: None,
            package_xml_binding_profile: None,
            platform_graph_profile: None,
            xml_schema_classification: None,
            physical_function_declarations: None,
            physical_function_signatures: None,
            state_projection: None,
        }
    }

    #[must_use]
    pub fn workspace_id(mut self, workspace_id: ProjectWorkspaceId) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    #[must_use]
    pub fn source_origin_id(mut self, source_origin_id: ProjectSourceOriginId) -> Self {
        self.source_origin_id = Some(source_origin_id);
        self
    }

    #[must_use]
    pub fn logical_root(mut self, logical_root: impl Into<String>) -> Self {
        self.logical_root = Some(logical_root.into());
        self
    }

    #[must_use]
    pub fn capability_policy(mut self, capability_policy: ProjectCapabilityPolicy) -> Self {
        self.capability_policy = Some(capability_policy);
        self
    }

    #[must_use]
    pub const fn budget_policy(mut self, budget_policy: ProjectBudgetPolicy) -> Self {
        self.budget_policy = Some(budget_policy);
        self
    }

    /// Select bounded Possible roots/literal prefixes for dynamic state suffixes.
    #[must_use]
    pub const fn with_state_projection(mut self, recipe: StateProjectionRecipe) -> Self {
        self.state_projection = Some(recipe);
        self
    }

    /// Select package named XML analysis explicitly. Construction requires the
    /// genuine platform source and its retained package/Main owners.
    #[must_use]
    pub const fn with_package_xml_bindings(mut self, profile: PackageXmlBindingProfile) -> Self {
        self.package_xml_binding_profile = Some(profile);
        self
    }

    /// Select the package graph recipe. Requires genuine platform admission and
    /// the explicit same-session package XML binding recipe at construction.
    #[must_use]
    pub const fn with_platform_graph_profile(mut self, profile: PlatformGraphProfile) -> Self {
        self.platform_graph_profile = Some(profile);
        self
    }

    /// Select native physical declaration syntax in the same function-call sidecar.
    /// Build validates the genuine held platform owners and composed graph path.
    #[must_use]
    pub const fn with_physical_function_declarations(
        mut self,
        recipe: PhysicalFunctionDeclarationRecipe,
    ) -> Self {
        self.physical_function_declarations = Some(recipe);
        self
    }

    #[must_use]
    pub const fn with_physical_function_signatures(
        mut self,
        recipe: PhysicalFunctionSignatureRecipe,
    ) -> Self {
        self.physical_function_signatures = Some(recipe);
        self
    }

    /// Retain an already reviewed classifier on the genuine held package source.
    /// The matching capability must be explicitly selected when building.
    pub fn with_xml_classification(
        mut self,
        profile: Arc<crate::load::schema::classification::ReviewedXmlClassificationProfile>,
        stop: &std::sync::atomic::AtomicBool,
    ) -> ProjectResult<Self> {
        let packages = self
            .retained_platform_packages
            .as_ref()
            .ok_or_else(platform::invalid)?
            .owner();
        let retained = RetainedXmlClassification::new(profile, packages, stop)?;
        if self
            .xml_schema_classification
            .as_ref()
            .is_some_and(|held| held != &retained)
        {
            return Err(platform::invalid());
        }
        self.xml_schema_classification = Some(retained);
        Ok(self)
    }

    /// Bind the actual selected TOC/XML receipt without changing E0 identities.
    pub fn load_plan(mut self, plan: &crate::load::ProjectLoadPlan) -> ProjectResult<Self> {
        if self.package_load_plan_digest.is_some() || self.package_main_plan_digest.is_some() {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "single-package and package-universe load plans are mutually exclusive",
            ));
        }
        plan.validate_profile(&self.selected_profile)?;
        self.load_plan_digest = Some(plan.digest());
        self.retained_load_plan = Some(Arc::new(plan.clone()));
        Ok(self)
    }

    /// Bind an explicit multi-package dependency/load closure and its exact
    /// collision-free analyzer Main namespace.
    pub fn package_load_plan(
        mut self,
        load_plan: &crate::load::ProjectPackageLoadPlan,
        main_plan: &crate::load::ProjectPackageMainPlan,
    ) -> ProjectResult<Self> {
        if self.load_plan_digest.is_some() {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "single-package and package-universe load plans are mutually exclusive",
            ));
        }
        if let Some(packages) = &self.retained_platform_packages
            && (packages.owner().load_plan() != load_plan
                || packages.owner().main_plan() != main_plan)
        {
            return Err(platform::invalid());
        }
        load_plan.validate_profile(&self.selected_profile)?;
        main_plan.validate_load_plan(load_plan)?;
        self.package_load_plan_digest = Some(load_plan.digest());
        self.package_main_plan_digest = Some(main_plan.digest());
        self.retained_package_load_plan = Some(Arc::new(load_plan.clone()));
        self.retained_package_main_plan = Some(Arc::new(main_plan.clone()));
        Ok(self)
    }

    /// Bind a platform project to its actual admitted source and native package
    /// owners. A digest or decoded receipt cannot construct this capability.
    pub fn platform_packages(
        mut self,
        packages: Arc<PlatformPackageSpecialization>,
    ) -> ProjectResult<Self> {
        if self.project_kind != ProjectKind::BlizzardUiPlatformSource
            || self.load_plan_digest.is_some()
            || self
                .retained_platform_packages
                .as_ref()
                .is_some_and(|held| held.binding() != packages.binding())
            || self
                .retained_package_load_plan
                .as_ref()
                .is_some_and(|held| held.as_ref() != packages.load_plan())
            || self
                .retained_package_main_plan
                .as_ref()
                .is_some_and(|held| held.as_ref() != packages.main_plan())
        {
            return Err(platform::invalid());
        }
        let retained = RetainedPlatformPackages::new(packages);
        retained.validate(
            &self.selected_profile,
            self.reference_generation,
            Some(retained.owner().load_plan()),
            Some(retained.owner().main_plan()),
        )?;
        self =
            self.package_load_plan(retained.owner().load_plan(), retained.owner().main_plan())?;
        self.retained_platform_packages = Some(retained);
        Ok(self)
    }

    pub fn build(self) -> ProjectResult<ProjectConfiguration> {
        let workspace_id = required(self.workspace_id, "workspace ID")?;
        let source_origin_id = required(self.source_origin_id, "source-origin ID")?;
        let logical_root = required(self.logical_root, "logical root")?;
        let parsed_logical_root = NormalizedSourcePath::parse(&logical_root).map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "logical root is not a canonical relative identity",
            )
        })?;
        if !parsed_logical_root.was_canonical()
            || parsed_logical_root.value().as_str() != logical_root
        {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "logical root must already use its canonical relative form",
            ));
        }
        let logical_root = parsed_logical_root.into_value();
        let capability_policy = required(self.capability_policy, "capability policy")?;
        let budget_policy = required(self.budget_policy, "budget policy")?;
        let platform_package_binding_digest = self
            .retained_platform_packages
            .as_ref()
            .map(|packages| packages.binding().binding_digest());
        let configuration_digest = configuration_digest(
            &self.project_id,
            self.project_kind,
            &workspace_id,
            &source_origin_id,
            &logical_root,
            &self.selected_profile,
            self.reference_generation,
            &self.analyzer_binding,
            &capability_policy,
            budget_policy,
            self.load_plan_digest,
            self.package_load_plan_digest,
            self.package_main_plan_digest,
            platform_package_binding_digest,
            self.package_xml_binding_profile,
            self.platform_graph_profile,
            self.xml_schema_classification.as_ref(),
            self.physical_function_declarations,
            self.physical_function_signatures,
            None,
            None,
            self.state_projection,
        )?;
        let configuration = ProjectConfiguration {
            project_id: self.project_id,
            project_kind: self.project_kind,
            configuration_schema_version: PROJECT_CONFIGURATION_SCHEMA_VERSION,
            workspace_id,
            source_origin_id,
            logical_root,
            selected_profile: self.selected_profile,
            reference_generation: self.reference_generation,
            analyzer_binding: self.analyzer_binding,
            capability_policy,
            budget_policy,
            configuration_digest,
            load_plan_digest: self.load_plan_digest,
            retained_load_plan: self.retained_load_plan,
            package_load_plan_digest: self.package_load_plan_digest,
            package_main_plan_digest: self.package_main_plan_digest,
            retained_package_load_plan: self.retained_package_load_plan,
            retained_package_main_plan: self.retained_package_main_plan,
            platform_package_binding_digest,
            retained_platform_packages: self.retained_platform_packages,
            package_xml_binding_profile: self.package_xml_binding_profile,
            platform_graph_profile: self.platform_graph_profile,
            xml_schema_classification: self.xml_schema_classification,
            physical_function_declarations: self.physical_function_declarations,
            physical_function_signatures: self.physical_function_signatures,
            native_library_input_digest: None,
            native_library_input: None,
            native_api_uses: None,
            state_projection: self.state_projection,
        };
        configuration.validate()?;
        Ok(configuration)
    }
}

#[allow(clippy::too_many_arguments)]
fn configuration_digest(
    project_id: &ProjectId,
    project_kind: ProjectKind,
    workspace_id: &ProjectWorkspaceId,
    source_origin_id: &ProjectSourceOriginId,
    logical_root: &NormalizedSourcePath,
    selected_profile: &ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    analyzer_binding: &AnalyzerBindingDeclaration,
    capability_policy: &ProjectCapabilityPolicy,
    budget_policy: ProjectBudgetPolicy,
    load_plan_digest: Option<ContentDigest<CanonicalResult>>,
    package_load_plan_digest: Option<ContentDigest<CanonicalResult>>,
    package_main_plan_digest: Option<ContentDigest<CanonicalResult>>,
    platform_package_binding_digest: Option<ContentDigest<CanonicalResult>>,
    package_xml_binding_profile: Option<PackageXmlBindingProfile>,
    platform_graph_profile: Option<PlatformGraphProfile>,
    xml_schema_classification: Option<&RetainedXmlClassification>,
    physical_function_declarations: Option<PhysicalFunctionDeclarationRecipe>,
    physical_function_signatures: Option<PhysicalFunctionSignatureRecipe>,
    native_library_input_digest: Option<ContentDigest<CanonicalResult>>,
    native_api_uses: Option<NativeApiUseRecipe>,
    state_projection: Option<StateProjectionRecipe>,
) -> ProjectResult<ContentDigest<CanonicalResult>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        configuration_schema_version: u64,
        project_id: &'a ProjectId,
        project_kind: ProjectKind,
        workspace_id: &'a ProjectWorkspaceId,
        source_origin_id: &'a ProjectSourceOriginId,
        logical_root: &'a NormalizedSourcePath,
        selected_profile: &'a ProfileIdentity,
        reference_generation: ReferenceGenerationId,
        analyzer_binding: &'a AnalyzerBindingDeclaration,
        capability_policy: &'a ProjectCapabilityPolicy,
        budget_policy: ProjectBudgetPolicy,
        #[serde(skip_serializing_if = "Option::is_none")]
        load_plan_digest: Option<ContentDigest<CanonicalResult>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        package_load_plan_digest: Option<ContentDigest<CanonicalResult>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        package_main_plan_digest: Option<ContentDigest<CanonicalResult>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        xml_lua_adapter: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        xml_binding_adapter: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        platform_package_binding_digest: Option<ContentDigest<CanonicalResult>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        package_xml_binding_profile: Option<PackageXmlBindingProfile>,
        #[serde(skip_serializing_if = "Option::is_none")]
        platform_graph_profile: Option<PlatformGraphProfile>,
        #[serde(skip_serializing_if = "Option::is_none")]
        xml_schema_classification: Option<&'a RetainedXmlClassification>,
        #[serde(skip_serializing_if = "Option::is_none")]
        physical_function_declarations: Option<PhysicalFunctionDeclarationRecipe>,
        #[serde(skip_serializing_if = "Option::is_none")]
        physical_function_signatures: Option<PhysicalFunctionSignatureRecipe>,
        #[serde(skip_serializing_if = "Option::is_none")]
        native_library_input_digest: Option<ContentDigest<CanonicalResult>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        native_api_uses: Option<NativeApiUseRecipe>,
        #[serde(skip_serializing_if = "Option::is_none")]
        state_projection: Option<StateProjectionRecipe>,
    }
    canonical_digest(
        "wow-project/configuration/e0-d/1",
        &Identity {
            configuration_schema_version: PROJECT_CONFIGURATION_SCHEMA_VERSION,
            project_id,
            project_kind,
            workspace_id,
            source_origin_id,
            logical_root,
            selected_profile,
            reference_generation,
            analyzer_binding,
            capability_policy,
            budget_policy,
            load_plan_digest,
            package_load_plan_digest,
            package_main_plan_digest,
            xml_lua_adapter: load_plan_digest.map(|_| crate::xml_lua::XML_LUA_ANALYSIS_PROFILE),
            xml_binding_adapter: load_plan_digest
                .map(|_| crate::xml_bindings::XML_LUA_BINDING_PROFILE),
            platform_package_binding_digest,
            package_xml_binding_profile,
            platform_graph_profile,
            xml_schema_classification,
            physical_function_declarations,
            physical_function_signatures,
            native_library_input_digest,
            native_api_uses,
            state_projection,
        },
        ProjectPhase::Configuration,
    )
}

fn required<T>(value: Option<T>, label: &str) -> ProjectResult<T> {
    value.ok_or_else(|| {
        ProjectError::new(
            ProjectErrorCode::InvalidConfiguration,
            ProjectPhase::Configuration,
            format!("project configuration is missing explicit {label}"),
        )
    })
}

fn reject_duplicates(values: &[CapabilityId]) -> ProjectResult<()> {
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(ProjectError::new(
            ProjectErrorCode::InvalidCapabilityPolicy,
            ProjectPhase::Configuration,
            "project capability policy contains duplicate entries",
        ))
    } else {
        Ok(())
    }
}

fn parse_capabilities(values: &[&str]) -> ProjectResult<Vec<CapabilityId>> {
    values
        .iter()
        .map(|value| {
            value.parse().map_err(|source| {
                ProjectError::new(
                    ProjectErrorCode::InvalidCapabilityPolicy,
                    ProjectPhase::Configuration,
                    format!("invalid capability ID {value:?}: {source}"),
                )
            })
        })
        .collect()
}

fn deferred_e0() -> ProjectResult<Vec<CapabilityId>> {
    parse_capabilities(&[
        "project.toc.complete",
        "project.xml.complete",
        "project.load_graph.complete",
        "project.state_index.complete",
        "project.event_hook_index.complete",
        "project.graph.complete",
        "project.persistence.available",
        "project.installed_runtime_universe.available",
    ])
}

fn checked_text(value: String, label: &str) -> ProjectResult<Box<str>> {
    let valid = !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && !value.contains("://")
        && !is_floating(&value);
    if valid {
        Ok(value.into_boxed_str())
    } else {
        Err(ProjectError::new(
            ProjectErrorCode::FloatingIdentity,
            ProjectPhase::Configuration,
            format!("{label} is empty, unsafe, or floating"),
        ))
    }
}

fn is_floating(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "main" | "master" | "current" | "latest" | "live" | "head" | "default" | "auto"
    )
}
