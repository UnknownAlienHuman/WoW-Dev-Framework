use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use wow_core::{CanonicalResult, CapabilityId, ContentDigest, ProjectGenerationId};
use wow_emmy::{
    EmmyFactFileStatus, EmmyLocalFlowFileStatus, EmmyLocalFlowReport, EmmyMemberCallReport,
    EmmySyntaxReport, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    analyze_local_flow, analyze_member_calls, analyze_syntax,
};

use crate::identity::{canonical_id, parse_source_digest};
use crate::{
    ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectFileId,
    ProjectGenerationCandidate, ProjectInputInventory, ProjectPhase, ProjectResult,
};

/// Analyzer capability observation scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAnalyzerCapabilityScope {
    Workspace,
    File,
}

/// Project-observed capability state. It is binding state, not platform truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAnalyzerCapabilityState {
    Complete,
    Failed,
}

/// One normalized analyzer capability observation retained by the project snapshot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAnalyzerCapabilityRecord {
    scope: ProjectAnalyzerCapabilityScope,
    subject_id: Box<str>,
    capability_id: CapabilityId,
    state: ProjectAnalyzerCapabilityState,
    item_count: u64,
    parse_error_count: u64,
}

impl ProjectAnalyzerCapabilityRecord {
    #[must_use]
    pub const fn scope(&self) -> ProjectAnalyzerCapabilityScope {
        self.scope
    }

    #[must_use]
    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }

    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    #[must_use]
    pub const fn state(&self) -> ProjectAnalyzerCapabilityState {
        self.state
    }

    #[must_use]
    pub const fn item_count(&self) -> u64 {
        self.item_count
    }

    #[must_use]
    pub const fn parse_error_count(&self) -> u64 {
        self.parse_error_count
    }
}

/// Immutable analyzer read view bound to one project generation and exact file manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAnalyzerBinding {
    analyzer_snapshot_id: Box<str>,
    project_generation: ProjectGenerationId,
    analyzer_configuration_digest: ContentDigest<CanonicalResult>,
    main_workspace: LuaWorkspaceSnapshot,
    library_snapshot_ids: Vec<Box<str>>,
    syntax_report: EmmySyntaxReport,
    member_call_report: EmmyMemberCallReport,
    local_flow_report: EmmyLocalFlowReport,
    capability_records: Vec<ProjectAnalyzerCapabilityRecord>,
}

impl ProjectAnalyzerBinding {
    #[must_use]
    pub fn analyzer_snapshot_id(&self) -> &str {
        &self.analyzer_snapshot_id
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }

    #[must_use]
    pub const fn analyzer_configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.analyzer_configuration_digest
    }

    #[must_use]
    pub const fn main_workspace(&self) -> &LuaWorkspaceSnapshot {
        &self.main_workspace
    }

    pub fn library_snapshot_ids(&self) -> impl Iterator<Item = &str> {
        self.library_snapshot_ids.iter().map(|value| value.as_ref())
    }

    #[must_use]
    pub const fn syntax_report(&self) -> &EmmySyntaxReport {
        &self.syntax_report
    }

    #[must_use]
    pub const fn member_call_report(&self) -> &EmmyMemberCallReport {
        &self.member_call_report
    }

    #[must_use]
    pub const fn local_flow_report(&self) -> &EmmyLocalFlowReport {
        &self.local_flow_report
    }

    #[must_use]
    pub fn capability_records(&self) -> &[ProjectAnalyzerCapabilityRecord] {
        &self.capability_records
    }

    #[must_use]
    pub fn file_capabilities(
        &self,
        file_id: &ProjectFileId,
    ) -> Vec<&ProjectAnalyzerCapabilityRecord> {
        self.capability_records
            .iter()
            .filter(|record| {
                record.scope == ProjectAnalyzerCapabilityScope::File
                    && record.subject_id() == file_id.as_str()
            })
            .collect()
    }

    #[must_use]
    pub fn file_facts_complete(&self, file_id: &ProjectFileId) -> bool {
        self.file_capabilities(file_id).into_iter().all(|record| {
            !record.capability_id().as_str().starts_with("emmy.fact.")
                || record.state() == ProjectAnalyzerCapabilityState::Complete
        })
    }
}

pub(crate) fn build_analyzer_binding(
    configuration: &ProjectConfiguration,
    inventory: &ProjectInputInventory,
    generation: &ProjectGenerationCandidate,
    libraries: &[LuaWorkspaceSnapshot],
) -> ProjectResult<ProjectAnalyzerBinding> {
    generation.validate(configuration, inventory)?;
    if libraries.is_empty() {
        return Err(ProjectError::new(
            ProjectErrorCode::AnalyzerFailed,
            ProjectPhase::Analyzer,
            "the E0 analyzer binding requires an explicit Library snapshot",
        )
        .with_candidate_generation(generation.project_generation()));
    }
    let backend = configuration.analyzer_binding().backend();
    let mut ordered_libraries = libraries.to_vec();
    ordered_libraries.sort_by(|left, right| left.snapshot_id().cmp(right.snapshot_id()));
    let mut library_ids = BTreeSet::new();
    for library in &ordered_libraries {
        if library.universe() == LuaWorkspaceUniverse::Project
            || library.backend() != backend
            || !library_ids.insert(library.snapshot_id().to_owned())
        {
            return Err(ProjectError::new(
                ProjectErrorCode::AnalyzerSnapshotMismatch,
                ProjectPhase::Analyzer,
                "Library snapshot role, backend, or identity is invalid",
            )
            .with_candidate_generation(generation.project_generation()));
        }
    }

    let budget = configuration.budget_policy();
    let main_workspace = LuaWorkspaceSnapshot::build(
        backend.clone(),
        LuaWorkspaceUniverse::Project,
        inventory
            .files()
            .iter()
            .map(crate::ProjectInputFile::workspace_input)
            .collect(),
        LuaWorkspaceLimits::new(
            budget.max_files(),
            16_384,
            budget.max_single_file_bytes(),
            budget.max_total_source_bytes(),
        )
        .map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Analyzer,
                format!("project budget cannot configure analyzer workspace limits: {source}"),
            )
        })?,
    )
    .map_err(|source| {
        ProjectError::new(
            ProjectErrorCode::AnalyzerFailed,
            ProjectPhase::Analyzer,
            format!("Main analyzer workspace construction failed: {source}"),
        )
        .with_candidate_generation(generation.project_generation())
    })?;
    validate_workspace_manifest(&main_workspace, inventory)?;

    let syntax_report = analyze_syntax(&main_workspace).map_err(|source| {
        ProjectError::new(
            ProjectErrorCode::AnalyzerFailed,
            ProjectPhase::Analyzer,
            format!("syntax analysis failed: {source}"),
        )
        .with_candidate_generation(generation.project_generation())
    })?;
    let library_refs = ordered_libraries.iter().collect::<Vec<_>>();
    let member_call_report =
        analyze_member_calls(&main_workspace, &library_refs).map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::AnalyzerFailed,
                ProjectPhase::Analyzer,
                format!("member-call analysis failed: {source}"),
            )
            .with_candidate_generation(generation.project_generation())
        })?;
    let local_flow_report =
        analyze_local_flow(&main_workspace, &library_refs).map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::AnalyzerFailed,
                ProjectPhase::Analyzer,
                format!("local-flow analysis failed: {source}"),
            )
            .with_candidate_generation(generation.project_generation())
        })?;

    validate_report_bindings(
        &main_workspace,
        inventory,
        &ordered_libraries,
        &syntax_report,
        &member_call_report,
        &local_flow_report,
        generation.project_generation(),
    )?;
    enforce_analyzer_budgets(
        configuration,
        &syntax_report,
        &member_call_report,
        &local_flow_report,
        generation.project_generation(),
    )?;
    let capability_records = build_capability_records(
        inventory,
        &main_workspace,
        &syntax_report,
        &member_call_report,
        &local_flow_report,
    )?;
    validate_capability_policy(
        configuration,
        &capability_records,
        generation.project_generation(),
    )?;

    let library_snapshot_ids = ordered_libraries
        .iter()
        .map(|library| Box::<str>::from(library.snapshot_id()))
        .collect::<Vec<_>>();
    #[derive(Serialize)]
    struct Identity<'a> {
        schema_version: u64,
        project_generation: ProjectGenerationId,
        analyzer_configuration_digest: ContentDigest<CanonicalResult>,
        accepted_pin_id: &'a str,
        compatibility_probe_report_id: &'a str,
        main_workspace_id: &'a str,
        library_snapshot_ids: &'a [Box<str>],
        syntax_analysis_id: &'a str,
        member_call_analysis_id: &'a str,
        local_flow_analysis_id: &'a str,
        file_manifest_digest: ContentDigest<CanonicalResult>,
        capability_records: &'a [ProjectAnalyzerCapabilityRecord],
    }
    let analyzer_snapshot_id = canonical_id(
        "project-analyzer-snapshot:sha256:",
        "wow-project/analyzer-snapshot/e0-d/1",
        &Identity {
            schema_version: 1,
            project_generation: generation.project_generation(),
            analyzer_configuration_digest: configuration
                .analyzer_binding()
                .analyzer_configuration_digest(),
            accepted_pin_id: configuration.analyzer_binding().accepted_pin_id(),
            compatibility_probe_report_id: configuration
                .analyzer_binding()
                .compatibility_probe_report_id(),
            main_workspace_id: main_workspace.snapshot_id(),
            library_snapshot_ids: &library_snapshot_ids,
            syntax_analysis_id: syntax_report.analysis_id(),
            member_call_analysis_id: member_call_report.analysis_id(),
            local_flow_analysis_id: local_flow_report.analysis_id(),
            file_manifest_digest: inventory.manifest_digest(),
            capability_records: &capability_records,
        },
        ProjectPhase::Analyzer,
    )?;
    Ok(ProjectAnalyzerBinding {
        analyzer_snapshot_id,
        project_generation: generation.project_generation(),
        analyzer_configuration_digest: configuration
            .analyzer_binding()
            .analyzer_configuration_digest(),
        main_workspace,
        library_snapshot_ids,
        syntax_report,
        member_call_report,
        local_flow_report,
        capability_records,
    })
}

fn validate_workspace_manifest(
    workspace: &LuaWorkspaceSnapshot,
    inventory: &ProjectInputInventory,
) -> ProjectResult<()> {
    if workspace.files().len() != inventory.files().len() {
        return Err(ProjectError::new(
            ProjectErrorCode::AnalyzerManifestMismatch,
            ProjectPhase::Analyzer,
            "Main analyzer workspace file count differs from the project inventory",
        ));
    }
    for file in inventory.files() {
        let analyzer_file = workspace
            .file(file.relative_path().as_str())
            .ok_or_else(|| {
                ProjectError::new(
                    ProjectErrorCode::AnalyzerManifestMismatch,
                    ProjectPhase::Analyzer,
                    "project file is missing from the Main analyzer workspace",
                )
                .with_file_id(file.file_id().as_str())
                .with_relative_path(file.relative_path().as_str())
            })?;
        let analyzer_digest =
            parse_source_digest(analyzer_file.content_sha256(), ProjectPhase::Analyzer)?;
        if analyzer_digest != file.content_digest()
            || analyzer_file.byte_len() != file.byte_length()
        {
            return Err(ProjectError::new(
                ProjectErrorCode::AnalyzerManifestMismatch,
                ProjectPhase::Analyzer,
                "Main analyzer workspace content identity differs from the project inventory",
            )
            .with_file_id(file.file_id().as_str())
            .with_relative_path(file.relative_path().as_str()));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_report_bindings(
    main_workspace: &LuaWorkspaceSnapshot,
    inventory: &ProjectInputInventory,
    libraries: &[LuaWorkspaceSnapshot],
    syntax: &EmmySyntaxReport,
    member: &EmmyMemberCallReport,
    flow: &EmmyLocalFlowReport,
    generation: ProjectGenerationId,
) -> ProjectResult<()> {
    if syntax.workspace_snapshot_id() != main_workspace.snapshot_id()
        || member.main_snapshot_id() != main_workspace.snapshot_id()
        || flow.main_snapshot_id() != main_workspace.snapshot_id()
        || flow.member_call_analysis_id() != member.analysis_id()
    {
        return Err(ProjectError::new(
            ProjectErrorCode::AnalyzerSnapshotMismatch,
            ProjectPhase::Analyzer,
            "analyzer reports do not bind one exact Main workspace",
        )
        .with_candidate_generation(generation));
    }
    let expected_libraries = libraries
        .iter()
        .map(LuaWorkspaceSnapshot::snapshot_id)
        .collect::<Vec<_>>();
    let observed_libraries = member.library_snapshot_ids().collect::<Vec<_>>();
    if expected_libraries != observed_libraries {
        return Err(ProjectError::new(
            ProjectErrorCode::AnalyzerSnapshotMismatch,
            ProjectPhase::Analyzer,
            "member-call report Library identities differ from the explicit binding",
        )
        .with_candidate_generation(generation));
    }
    let expected_paths = inventory
        .files()
        .iter()
        .map(|file| file.relative_path().as_str())
        .collect::<BTreeSet<_>>();
    let syntax_paths = syntax.files().iter().map(|file| file.path()).collect();
    let member_paths = member.files().iter().map(|file| file.path()).collect();
    let flow_paths = flow.files().iter().map(|file| file.path()).collect();
    if expected_paths != syntax_paths
        || expected_paths != member_paths
        || expected_paths != flow_paths
    {
        return Err(ProjectError::new(
            ProjectErrorCode::AnalyzerManifestMismatch,
            ProjectPhase::Analyzer,
            "analyzer report file manifests differ from the project inventory",
        )
        .with_candidate_generation(generation));
    }
    Ok(())
}

fn enforce_analyzer_budgets(
    configuration: &ProjectConfiguration,
    syntax: &EmmySyntaxReport,
    member: &EmmyMemberCallReport,
    flow: &EmmyLocalFlowReport,
    generation: ProjectGenerationId,
) -> ProjectResult<()> {
    let facts = member
        .references()
        .len()
        .saturating_add(member.calls().len())
        .saturating_add(flow.bindings().len())
        .saturating_add(flow.uses().len())
        .saturating_add(flow.operations().len())
        .saturating_add(flow.guards().len())
        .saturating_add(flow.control_flow().len());
    let diagnostics = syntax.diagnostics().len();
    let budget = configuration.budget_policy();
    if u64::try_from(facts).unwrap_or(u64::MAX) > budget.max_analyzer_facts()
        || u64::try_from(diagnostics).unwrap_or(u64::MAX) > budget.max_generic_findings()
    {
        return Err(ProjectError::new(
            ProjectErrorCode::UpdateBudgetExceeded,
            ProjectPhase::Analyzer,
            "analyzer output exceeds the configured fact or finding budget",
        )
        .with_candidate_generation(generation));
    }
    let output_bytes = serde_json::to_vec(&(syntax, member, flow))
        .map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::AnalyzerFailed,
                ProjectPhase::Analyzer,
                format!("analyzer reports cannot be serialized for budget validation: {source}"),
            )
        })?
        .len();
    if u64::try_from(output_bytes).unwrap_or(u64::MAX) > budget.max_output_bytes() {
        return Err(ProjectError::new(
            ProjectErrorCode::UpdateBudgetExceeded,
            ProjectPhase::Analyzer,
            "analyzer output exceeds the configured serialized-output budget",
        )
        .with_candidate_generation(generation));
    }
    Ok(())
}

fn build_capability_records(
    inventory: &ProjectInputInventory,
    main_workspace: &LuaWorkspaceSnapshot,
    syntax: &EmmySyntaxReport,
    member: &EmmyMemberCallReport,
    flow: &EmmyLocalFlowReport,
) -> ProjectResult<Vec<ProjectAnalyzerCapabilityRecord>> {
    let mut records = vec![
        workspace_capability("emmy.session.ready", main_workspace.snapshot_id())?,
        workspace_capability("emmy.library.loaded", main_workspace.snapshot_id())?,
        workspace_capability(
            "emmy.source_coordinates.exact",
            main_workspace.snapshot_id(),
        )?,
    ];
    let syntax_by_path = syntax
        .files()
        .iter()
        .map(|file| (file.path(), file))
        .collect::<BTreeMap<_, _>>();
    let member_by_path = member
        .files()
        .iter()
        .map(|file| (file.path(), file))
        .collect::<BTreeMap<_, _>>();
    let flow_by_path = flow
        .files()
        .iter()
        .map(|file| (file.path(), file))
        .collect::<BTreeMap<_, _>>();
    for file in inventory.files() {
        let syntax_file = syntax_by_path
            .get(file.relative_path().as_str())
            .ok_or_else(|| {
                ProjectError::new(
                    ProjectErrorCode::AnalyzerManifestMismatch,
                    ProjectPhase::Analyzer,
                    "syntax file capability record is missing",
                )
                .with_file_id(file.file_id().as_str())
            })?;
        let member_file = member_by_path
            .get(file.relative_path().as_str())
            .ok_or_else(|| {
                ProjectError::new(
                    ProjectErrorCode::AnalyzerManifestMismatch,
                    ProjectPhase::Analyzer,
                    "member-call file capability record is missing",
                )
                .with_file_id(file.file_id().as_str())
            })?;
        let flow_file = flow_by_path
            .get(file.relative_path().as_str())
            .ok_or_else(|| {
                ProjectError::new(
                    ProjectErrorCode::AnalyzerManifestMismatch,
                    ProjectPhase::Analyzer,
                    "local-flow file capability record is missing",
                )
                .with_file_id(file.file_id().as_str())
            })?;
        let parsed = member_file.status() == EmmyFactFileStatus::Complete
            && flow_file.status() == EmmyLocalFlowFileStatus::Complete;
        records.push(file_capability(
            file.file_id(),
            "emmy.file.parsed",
            parsed,
            1,
            member_file
                .parse_error_count()
                .max(flow_file.parse_error_count()),
        )?);
        records.push(file_capability(
            file.file_id(),
            "emmy.file.diagnostics",
            syntax_file.status() == "complete",
            syntax_file.diagnostic_count(),
            0,
        )?);
        records.push(file_capability(
            file.file_id(),
            "emmy.fact.references",
            member_file.status() == EmmyFactFileStatus::Complete,
            member_file.reference_count(),
            member_file.parse_error_count(),
        )?);
        records.push(file_capability(
            file.file_id(),
            "emmy.fact.calls",
            member_file.status() == EmmyFactFileStatus::Complete,
            member_file.call_count(),
            member_file.parse_error_count(),
        )?);
        for (capability, count) in [
            ("emmy.fact.local_bindings", flow_file.binding_count()),
            ("emmy.fact.local_flow", flow_file.use_count()),
            ("emmy.fact.operations", flow_file.operation_count()),
            ("emmy.fact.guards", flow_file.guard_count()),
            ("emmy.fact.control_flow", flow_file.control_flow_count()),
        ] {
            records.push(file_capability(
                file.file_id(),
                capability,
                flow_file.status() == EmmyLocalFlowFileStatus::Complete,
                count,
                flow_file.parse_error_count(),
            )?);
        }
    }
    records.sort();
    Ok(records)
}

fn workspace_capability(
    capability: &str,
    subject_id: &str,
) -> ProjectResult<ProjectAnalyzerCapabilityRecord> {
    Ok(ProjectAnalyzerCapabilityRecord {
        scope: ProjectAnalyzerCapabilityScope::Workspace,
        subject_id: subject_id.into(),
        capability_id: parse_capability(capability)?,
        state: ProjectAnalyzerCapabilityState::Complete,
        item_count: 1,
        parse_error_count: 0,
    })
}

fn file_capability(
    file_id: &ProjectFileId,
    capability: &str,
    complete: bool,
    item_count: u64,
    parse_error_count: u64,
) -> ProjectResult<ProjectAnalyzerCapabilityRecord> {
    Ok(ProjectAnalyzerCapabilityRecord {
        scope: ProjectAnalyzerCapabilityScope::File,
        subject_id: file_id.as_str().into(),
        capability_id: parse_capability(capability)?,
        state: if complete {
            ProjectAnalyzerCapabilityState::Complete
        } else {
            ProjectAnalyzerCapabilityState::Failed
        },
        item_count,
        parse_error_count,
    })
}

fn validate_capability_policy(
    configuration: &ProjectConfiguration,
    records: &[ProjectAnalyzerCapabilityRecord],
    generation: ProjectGenerationId,
) -> ProjectResult<()> {
    for record in records {
        if record.state == ProjectAnalyzerCapabilityState::Failed
            && !configuration
                .capability_policy()
                .is_degradable(&record.capability_id)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::MandatoryCapabilityUnavailable,
                ProjectPhase::Analyzer,
                format!(
                    "analyzer capability {} failed outside the explicit degraded policy",
                    record.capability_id
                ),
            )
            .with_candidate_generation(generation)
            .with_file_id(record.subject_id.as_ref()));
        }
    }
    Ok(())
}

fn parse_capability(value: &str) -> ProjectResult<CapabilityId> {
    value.parse().map_err(|source| {
        ProjectError::new(
            ProjectErrorCode::InvalidCapabilityPolicy,
            ProjectPhase::Analyzer,
            format!("invalid analyzer capability ID {value:?}: {source}"),
        )
    })
}
