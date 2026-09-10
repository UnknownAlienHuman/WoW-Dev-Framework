use std::collections::BTreeSet;

use serde::Serialize;
use wow_core::{
    CanonicalResult, CapabilityId, ContentDigest, NormalizedSourcePath, ProfileIdentity,
    ProfileKind, ReferenceGenerationId,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
};

use crate::identity::canonical_digest;
use crate::{
    ProjectError, ProjectErrorCode, ProjectId, ProjectPhase, ProjectResult, ProjectSourceOriginId,
    ProjectWorkspaceId,
};

pub const PROJECT_CONFIGURATION_SCHEMA_VERSION: u64 = 1;
pub const PROJECT_GENERATION_SCHEMA_VERSION: u64 = 1;
pub const PROJECT_SNAPSHOT_SCHEMA_VERSION: u64 = 1;
pub const PROJECT_CONTRACT_ID: &str = "wow-project/e0-d/1";

/// Project source class supported by the configuration contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Fixture,
    Repository,
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

/// Explicit E0 source/analysis/output limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBudgetPolicy {
    max_files: u64,
    max_total_source_bytes: u64,
    max_single_file_bytes: u64,
    max_update_operations: u64,
    max_analyzer_facts: u64,
    max_generic_findings: u64,
    max_output_bytes: u64,
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
    pub const fn configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.configuration_digest
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
