use super::{cancelled, owner_error};
use crate::{
    BlockerKind, CapabilityState, CheckContext, CheckScope, CleanEvaluation, ComponentHealth,
    ComponentSnapshot, ContextIdentity, ExactSourceLocation, GenericFinding, RuleBlocker,
    RuleEvaluation, RuleFinding, ServiceConfiguration, ServiceError, ServiceErrorCode,
    ServiceResult,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use wow_emmy::{EmmyDiagnosticSeverity, EmmySyntaxReport};
use wow_project::{
    ProjectAnalyzerCapabilityRecord, ProjectAnalyzerCapabilityState, ProjectFileId,
    ProjectFileRecord, ProjectView,
};
use wow_reference::{CoverageStatus, ReferenceView};
use wow_rules::{
    RuleEvaluationOutcome, RuleExecutionBudget, RuleExecutionContext, RuleExecutionReport,
    RuleFixturePolicy, RuleRegistry, RuleScope,
};

/// Exact owner receipts retained alongside service presentation. These contain
/// normalized metadata and findings, never Main/Library source bodies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnerAnalysis {
    schema: &'static str,
    input_mode: &'static str,
    profile_kind: wow_core::ProfileKind,
    reference_view_digest: Box<str>,
    project_files: Vec<ProjectFileRecord>,
    project_coverage: Vec<wow_core::CoverageRecord>,
    analyzer_capabilities: Vec<ProjectAnalyzerCapabilityRecord>,
    generic_report_scope: &'static str,
    generic_report: EmmySyntaxReport,
    selected_rules: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_report: Option<RuleExecutionReport>,
    runtime_status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_plan: Option<wow_project::load::ProjectLoadPlan>,
}

impl OwnerAnalysis {
    #[must_use]
    pub fn load_plan(&self) -> Option<&wow_project::load::ProjectLoadPlan> {
        self.load_plan.as_ref()
    }
}

pub(super) fn components(
    config: &ServiceConfiguration,
    reference: &ReferenceView,
    project_identity: &str,
    project_health: ComponentHealth,
    project: Option<&ProjectView>,
    load_plan: Option<&wow_project::load::ProjectLoadPlan>,
) -> ServiceResult<Vec<ComponentSnapshot>> {
    let reference_partial = reference.partitions().is_empty()
        || reference
            .partitions()
            .iter()
            .any(|part| part.coverage() != CoverageStatus::Complete)
        || !reference.conflicts().is_empty();
    let analyzer_partial = project.is_some_and(|view| {
        view.snapshot()
            .analyzer_binding()
            .capability_records()
            .iter()
            .any(|record| record.state() != ProjectAnalyzerCapabilityState::Complete)
    });
    let fixture_rules = config.profile_id() == wow_rules::FIXTURE_PROFILE_ID;
    let health = |partial| {
        if partial {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Ready
        }
    };
    let mut components = vec![
        ComponentSnapshot::new(
            "wow-core",
            env!("CARGO_PKG_VERSION"),
            "wow-core/e0-a/1",
            ComponentHealth::Ready,
        )?,
        ComponentSnapshot::new(
            "wow-reference",
            env!("CARGO_PKG_VERSION"),
            reference.self_digest(),
            health(reference_partial),
        )?
        .with_capability(
            "reference.symbol.exact_lookup",
            if reference_partial {
                CapabilityState::Partial
            } else {
                CapabilityState::Available
            },
        )?,
        ComponentSnapshot::new(
            "wow-emmy",
            wow_emmy::EMMYLUA_CODE_ANALYSIS_VERSION,
            config.analyzer_pin_id(),
            health(analyzer_partial),
        )?
        .with_capability(
            "emmy.file.diagnostics",
            if analyzer_partial {
                CapabilityState::Partial
            } else {
                CapabilityState::Available
            },
        )?,
        ComponentSnapshot::new(
            "wow-project",
            env!("CARGO_PKG_VERSION"),
            project_identity,
            project_health,
        )?
        .with_capability(
            "project.generation.coherent",
            if project.is_some() {
                CapabilityState::Available
            } else {
                CapabilityState::Partial
            },
        )?,
        ComponentSnapshot::new(
            "wow-rules",
            env!("CARGO_PKG_VERSION"),
            config.rule_registry_id(),
            health(!fixture_rules),
        )?
        .with_capability(
            "rules.profile.supported",
            if fixture_rules {
                CapabilityState::Available
            } else {
                CapabilityState::Partial
            },
        )?,
    ];
    if let Some(plan) = load_plan {
        components.push(
            ComponentSnapshot::new(
                "wow-project-load",
                "4",
                plan.digest().to_string(),
                health(!plan.external_files_complete()),
            )?
            .with_capability(
                "project.load_files.resolved",
                if plan.external_files_complete() {
                    CapabilityState::Available
                } else {
                    CapabilityState::Partial
                },
            )?,
        );
        if !plan.xml_documents().is_empty() {
            let pending_inline = plan.xml_documents().values().any(|index| {
                index.scripts().any(|element| {
                    element.script.as_ref().is_some_and(|script| {
                        matches!(
                            script.source_kind,
                            wow_project::load::XmlScriptSource::InlineBody
                                | wow_project::load::XmlScriptSource::Unresolved
                        )
                    })
                })
            });
            let mut xml = ComponentSnapshot::new(
                "wow-project-xml",
                "1",
                plan.digest().to_string(),
                health(pending_inline),
            )?
            .with_capability("project.xml.syntax.indexed", CapabilityState::Available)?;
            if pending_inline {
                xml = xml
                    .with_capability("project.xml.inline_lua.analyzed", CapabilityState::Partial)?;
            }
            components.push(xml);
        }
    }
    Ok(components)
}

#[allow(clippy::too_many_arguments)] // Explicit owner inputs plus request scope and cancellation.
pub(super) fn check_context(
    project: &ProjectView,
    reference: &ReferenceView,
    registry: &RuleRegistry,
    config: &ServiceConfiguration,
    identity: ContextIdentity,
    scope: &CheckScope,
    selected: &[Box<str>],
    load_plan: Option<&wow_project::load::ProjectLoadPlan>,
    stop: &AtomicBool,
) -> ServiceResult<CheckContext> {
    let files = resolve_scope(project, scope)?;
    let paths: BTreeSet<&str> = files
        .iter()
        .map(|file| file.relative_path().as_str())
        .collect();
    let mut generic = Vec::new();
    for diagnostic in project.syntax_report().diagnostics() {
        cancelled(stop)?;
        if !paths.contains(diagnostic.path()) {
            continue;
        }
        let location = exact_location(
            project,
            diagnostic.path(),
            diagnostic.content_sha256(),
            diagnostic.span(),
        )?;
        let id = crate::identity::canonical_digest(
            "service-generic:sha256:",
            &(project.analyzer_snapshot_id(), diagnostic),
        )?;
        let severity = match diagnostic.normalized_severity() {
            EmmyDiagnosticSeverity::Error => "error",
            EmmyDiagnosticSeverity::Warning => "warning",
            EmmyDiagnosticSeverity::Information => "information",
            EmmyDiagnosticSeverity::Hint => "hint",
            EmmyDiagnosticSeverity::Unknown => "unknown",
        };
        generic.push(GenericFinding::new(
            id,
            diagnostic.category(),
            diagnostic.upstream_code(),
            severity,
            location,
        )?);
    }
    let rules: Vec<Box<str>> = if selected.is_empty() {
        registry
            .descriptors()
            .iter()
            .map(|rule| format!("{}@1", rule.rule_id().as_str()).into())
            .collect()
    } else {
        selected.to_vec()
    };
    let mut evaluations = Vec::new();
    let report = if identity.profile_id() == wow_rules::FIXTURE_PROFILE_ID {
        let policy =
            RuleFixturePolicy::e0().map_err(|_| owner_error("fixture rule policy failed"))?;
        let budget = RuleExecutionBudget::new(65_536, 65_536, 262_144, 262_144, 16 * 1024 * 1024)
            .map_err(|_| owner_error("rule budget failed"))?;
        let context =
            RuleExecutionContext::new(registry, &policy, project, reference, budget, stop);
        let scope = RuleScope::files(files.iter().map(|file| file.file_id().clone()).collect());
        let report = wow_rules::execute_e0(&context, &scope)
            .map_err(|_| owner_error("rule execution failed"))?;
        cancelled(stop)?;
        for evaluation in report.evaluations() {
            if rules
                .iter()
                .any(|rule| rule.as_ref() == format!("{}@1", evaluation.rule_id()))
            {
                evaluations.push(project_evaluation(project, evaluation)?);
            }
        }
        Some(report)
    } else {
        // A real repository profile must never acquire E0 fixture-only authority.
        for rule in &rules {
            let id = crate::identity::canonical_digest(
                "service-not-evaluated:sha256:",
                &(
                    project.snapshot_id(),
                    rule,
                    scope,
                    "unsupported_rule_profile",
                ),
            )?;
            evaluations.push(RuleEvaluation::not_evaluated(
                id.clone(),
                rule.clone(),
                "selected_project_scope",
                RuleBlocker::new(
                    id,
                    BlockerKind::MissingCapability,
                    Some("rules.profile.supported".into()),
                )?,
            )?);
        }
        None
    };
    let components = components(
        config,
        reference,
        project.snapshot_id(),
        ComponentHealth::Ready,
        Some(project),
        load_plan,
    )?;
    let analysis = OwnerAnalysis {
        schema: "wow-service/owner-analysis/1",
        input_mode: if load_plan.is_some() {
            "selected_toc_project"
        } else {
            "explicit_materialized_project"
        },
        profile_kind: project.configuration().selected_profile().profile_kind(),
        reference_view_digest: reference.self_digest().into(),
        project_files: project.file_manifest().to_vec(),
        project_coverage: project.project_coverage_records().to_vec(),
        analyzer_capabilities: project
            .snapshot()
            .analyzer_binding()
            .capability_records()
            .to_vec(),
        generic_report_scope: "whole_project",
        generic_report: project.syntax_report().clone(),
        selected_rules: rules,
        rule_report: report,
        runtime_status: "not_evaluated",
        load_plan: load_plan.cloned(),
    };
    Ok(CheckContext::new(
        identity,
        scope.clone(),
        components,
        generic,
        evaluations,
        Vec::new(),
    )
    .with_owner_analysis(analysis))
}

fn resolve_scope<'a>(
    project: &'a ProjectView,
    scope: &CheckScope,
) -> ServiceResult<Vec<&'a ProjectFileRecord>> {
    let files = match scope {
        CheckScope::WholeProject => project.file_manifest().iter().collect(),
        CheckScope::ProjectFiles(ids) => ids
            .iter()
            .map(|id| {
                let id = ProjectFileId::parse(id).map_err(|_| invalid_scope())?;
                project.file_by_id(&id).ok_or_else(invalid_scope)
            })
            .collect::<ServiceResult<Vec<_>>>()?,
        CheckScope::Files(paths) => paths
            .iter()
            .map(|path| {
                project
                    .file_by_path(path)
                    .map_err(|_| invalid_scope())?
                    .ok_or_else(invalid_scope)
            })
            .collect::<ServiceResult<Vec<_>>>()?,
    };
    if files.is_empty() {
        return Err(invalid_scope());
    }
    Ok(files)
}

fn invalid_scope() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::InvalidRequest,
        "selected files are not in the exact project snapshot",
    )
}

fn exact_location(
    project: &ProjectView,
    path: &str,
    digest: &str,
    span: wow_core::SourceSpan,
) -> ServiceResult<ExactSourceLocation> {
    let file = project
        .file_by_path(path)
        .map_err(|_| invalid_scope())?
        .ok_or_else(invalid_scope)?;
    let start = span
        .byte_start()
        .ok_or_else(|| owner_error("owner finding has no exact byte range"))?;
    let end = span
        .byte_end()
        .ok_or_else(|| owner_error("owner finding has no exact byte range"))?;
    if file.content_digest().to_string() != digest || end > file.byte_length() {
        return Err(owner_error(
            "finding coordinates do not match the exact source artifact",
        ));
    }
    ExactSourceLocation::new(path, digest, start, end)
}

fn project_evaluation(
    project: &ProjectView,
    evaluation: &wow_rules::RuleEvaluationRecord,
) -> ServiceResult<RuleEvaluation> {
    let rule = format!("{}@1", evaluation.rule_id());
    let scope = evaluation.scope_id();
    match evaluation.outcome() {
        RuleEvaluationOutcome::Findings { result } => {
            let mut findings = Vec::new();
            for finding in result.findings() {
                let handle = result
                    .source_handles()
                    .iter()
                    .find(|handle| handle.handle_id() == finding.primary_source_handle_id())
                    .ok_or_else(|| owner_error("rule finding source handle is missing"))?;
                let location = exact_location(
                    project,
                    handle.path().as_str(),
                    &handle.content_digest().to_string(),
                    handle.span(),
                )?;
                let severity = match finding.severity() {
                    wow_core::Severity::Error => "error",
                    wow_core::Severity::Warning => "warning",
                    wow_core::Severity::Information => "information",
                    wow_core::Severity::Hint => "hint",
                };
                findings.push(RuleFinding::new(
                    finding.finding_id().to_string(),
                    rule.clone(),
                    finding.finding_code().as_str(),
                    severity,
                    location,
                    handle.handle_id().to_string(),
                    finding
                        .evidence_ids()
                        .iter()
                        .map(|id| id.to_string().into())
                        .collect(),
                )?);
            }
            RuleEvaluation::findings(result.evaluation_id(), rule, scope, findings)
        }
        RuleEvaluationOutcome::EvaluatedClean { record } => {
            let claim = match record.clean_claim_kind() {
                wow_rules::RuleCleanClaimKind::ApiExistsForExactUse => "api_exists_for_exact_use",
                wow_rules::RuleCleanClaimKind::SecretFixtureOperationGuardedForExactValueAndScope => "secret_fixture_operation_guarded_for_exact_value_and_scope",
                wow_rules::RuleCleanClaimKind::SecretProducerHasNoMatchingFacet => "secret_producer_has_no_matching_facet",
            };
            RuleEvaluation::clean(
                record.evaluation_id(),
                rule.clone(),
                scope,
                vec![CleanEvaluation::new(
                    record.evaluation_id(),
                    rule,
                    scope,
                    claim,
                    record
                        .coverage_ids()
                        .iter()
                        .map(|id| id.to_string().into())
                        .collect(),
                )?],
            )
        }
        RuleEvaluationOutcome::NotEvaluated { detail } => {
            let kind = if detail
                .blockers()
                .contains(&wow_rules::RuleBlockerKind::ReferenceConflict)
            {
                BlockerKind::ConflictingEvidence
            } else if detail
                .blockers()
                .contains(&wow_rules::RuleBlockerKind::BudgetIncomplete)
            {
                BlockerKind::BudgetTruncated
            } else {
                BlockerKind::IncompleteCoverage
            };
            let capability = detail
                .core_record()
                .blocking_capability_ids()
                .first()
                .map(|id| id.to_string().into());
            RuleEvaluation::not_evaluated(
                detail.evaluation_id(),
                rule,
                scope,
                RuleBlocker::new(
                    detail.core_record().not_evaluated_id().to_string(),
                    kind,
                    capability,
                )?,
            )
        }
        RuleEvaluationOutcome::Failed { failure } => RuleEvaluation::failed(
            failure.evaluation_id(),
            rule,
            scope,
            format!("{:?}", failure.error_code()),
            false,
        ),
        RuleEvaluationOutcome::Cancelled => Err(ServiceError::new(
            ServiceErrorCode::Cancelled,
            "rule execution cancelled",
        )),
    }
}
