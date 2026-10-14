use std::collections::BTreeMap;
use std::str::FromStr;

use wow_core::{
    BlockingPartitionRef, CapabilityId, ClaimScope, ContentDigest, CoverageId, CoverageStatus,
    EntityKey, EvidenceConfidence, EvidenceRecord, FindingDraft, MessageArgument,
    MessageArgumentKind, MessageCode, NotEvaluatedRecord, ProducerId, ProvenanceClass, Remediation,
    RemediationClass, RolloutPolicy, Severity, SourceContent, SourceHandle, SourceHandleBuilder,
    SourceOriginKind, SourceSpan, ToolVersion, derive_evidence, derive_root_cause_key,
    validate_evidence_derivation_graph,
};
use wow_emmy::{
    EmmyControlFlowRelationKind, EmmyGuardFact, EmmyGuardKind, EmmyLocalBindingFact,
    EmmyMemberCallFact, EmmyMemberReferenceFact, EmmyOperationFact, EmmyOperationKind,
    EmmyReferenceResolution, EmmySyntaxDiagnosticKind,
};
use wow_project::{
    ProjectAnalyzerCapabilityScope, ProjectAnalyzerCapabilityState, ProjectFileId,
    ProjectFileRecord,
    xml_lua::{
        XmlLuaDiagnosticMapping, XmlLuaMappedSpan, XmlLuaMemberCall, XmlLuaMemberReference,
        XmlLuaSemanticState, XmlLuaUnitAnalysis,
    },
};
use wow_reference::{
    LookupResult, LookupUnknownReason, ReferenceRecord, ReferenceRecordKind, RestrictionState,
};

use crate::budget::{
    Admission, ArgumentPlan, FindingMaterial, ReportMaterial, SourcePlan, admit_source_plan,
    preflight_cancelled, preflight_clean, preflight_finding, preflight_lookup,
    preflight_not_evaluated, preflight_report, verify_canonical_output,
};
use crate::descriptor::{API_EXISTS_RULE, SECRET_LOCAL_RULE};
use crate::identity::canonical_id;
use crate::output::RuleFindingSetInput;
use crate::{
    CleanEvaluationRecord, RuleBlockerKind, RuleBudgetUsage, RuleCleanClaimKind, RuleError,
    RuleErrorCode, RuleEvaluationOutcome, RuleEvaluationRecord, RuleExecutionContext,
    RuleExecutionReport, RuleFindingSet, RuleGuardClassification, RuleNotEvaluatedDetail,
    RuleReferenceLookupRecord, RuleReferenceOutcome, RuleResult, RuleScope,
};

const FIXTURE_SECRET_ENTITY: &str = "function:C_E0Fixture.SecretText";
const PRODUCER_ID: &str = "wow.rules";
const PRODUCER_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Construction {
    Evaluation,
    Lookup,
    Handle,
    Evidence,
    Root,
    Finding,
    Clean,
    Report,
    ApiProviderComplete,
    SecretOperationComplete,
}

#[cfg(test)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct ConstructionCounts {
    evaluations: usize,
    lookups: usize,
    handles: usize,
    evidence: usize,
    roots: usize,
    findings: usize,
    cleans: usize,
    reports: usize,
    api_provider_completions: usize,
    secret_operations: usize,
}
#[cfg(test)]
thread_local! { static CONSTRUCTIONS: std::cell::Cell<ConstructionCounts> = std::cell::Cell::new(ConstructionCounts::default()); }
#[cfg(test)]
type CancellationHook = (
    Construction,
    usize,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
);
#[cfg(test)]
thread_local! {
    static CANCELLATION_HOOK: std::cell::RefCell<Option<CancellationHook>> = const { std::cell::RefCell::new(None) };
    static CANCELLATION_COUNTS: std::cell::Cell<Option<ConstructionCounts>> = const { std::cell::Cell::new(None) };
}
#[cfg(test)]
fn observe(construction: Construction) {
    let counts = CONSTRUCTIONS.with(|cell| {
        let mut counts = cell.get();
        let count = match construction {
            Construction::Evaluation => {
                counts.evaluations += 1;
                counts.evaluations
            }
            Construction::Lookup => {
                counts.lookups += 1;
                counts.lookups
            }
            Construction::Handle => {
                counts.handles += 1;
                counts.handles
            }
            Construction::Evidence => {
                counts.evidence += 1;
                counts.evidence
            }
            Construction::Root => {
                counts.roots += 1;
                counts.roots
            }
            Construction::Finding => {
                counts.findings += 1;
                counts.findings
            }
            Construction::Clean => {
                counts.cleans += 1;
                counts.cleans
            }
            Construction::Report => {
                counts.reports += 1;
                counts.reports
            }
            Construction::ApiProviderComplete => {
                counts.api_provider_completions += 1;
                counts.api_provider_completions
            }
            Construction::SecretOperationComplete => {
                counts.secret_operations += 1;
                counts.secret_operations
            }
        };
        cell.set(counts);
        (counts, count)
    });
    CANCELLATION_HOOK.with(|cell| {
        let hook = cell.borrow();
        match hook.as_ref() {
            Some((target, occurrence, stop))
                if *target == construction && counts.1 == *occurrence =>
            {
                CANCELLATION_COUNTS.with(|snapshot| snapshot.set(Some(counts.0)));
                stop.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            _ => {}
        }
    });
}
#[cfg(not(test))]
fn observe(_: Construction) {}

#[cfg(test)]
struct CancellationHookGuard;

#[cfg(test)]
impl Drop for CancellationHookGuard {
    fn drop(&mut self) {
        CANCELLATION_HOOK.with(|cell| *cell.borrow_mut() = None);
        CANCELLATION_COUNTS.with(|cell| cell.set(None));
    }
}

#[cfg(test)]
fn cancel_on_construction(
    construction: Construction,
    occurrence: usize,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> CancellationHookGuard {
    CANCELLATION_COUNTS.with(|cell| cell.set(None));
    CANCELLATION_HOOK.with(|cell| *cell.borrow_mut() = Some((construction, occurrence, stop)));
    CancellationHookGuard
}

#[cfg(test)]
fn cancellation_snapshot() -> Option<ConstructionCounts> {
    CANCELLATION_COUNTS.with(std::cell::Cell::take)
}

struct ApiFindingInput<'a> {
    file: &'a ProjectFileRecord,
    fact: &'a EmmyMemberReferenceFact,
    call: Option<&'a EmmyMemberCallFact>,
    entity_lookup_key: &'a str,
    lookup: RuleReferenceLookupRecord,
    coverage_ids: Vec<CoverageId>,
}

struct XmlApiFindingInput<'a> {
    unit: &'a XmlLuaUnitAnalysis,
    fact: &'a XmlLuaMemberReference,
    call: Option<&'a XmlLuaMemberCall>,
    entity_lookup_key: &'a str,
    lookup: RuleReferenceLookupRecord,
    coverage_ids: Vec<CoverageId>,
}

struct CleanInput<'a> {
    scope_id: &'a str,
    claim: RuleCleanClaimKind,
    fact_ids: Vec<Box<str>>,
    lookup: &'a RuleReferenceLookupRecord,
    coverage_ids: Vec<CoverageId>,
    guard: Option<RuleGuardClassification>,
}

/// Executes the fixture registry and rejects a production policy.
pub fn execute_e0(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
) -> RuleResult<RuleExecutionReport> {
    if !context.is_fixture_policy() {
        return Err(RuleError::new(
            RuleErrorCode::RuleProfileMismatch,
            "execute_e0 requires the closed fixture policy",
        ));
    }
    execute(context, scope)
}

/// Executes the two active providers under the selected fixture or production policy.
pub fn execute(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
) -> RuleResult<RuleExecutionReport> {
    run_providers(context, scope)
}

fn run_providers(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
) -> RuleResult<RuleExecutionReport> {
    context.validate(scope)?;
    if context.is_cancelled() {
        return cancelled_report(context);
    }
    let api = context
        .registry()
        .descriptor(API_EXISTS_RULE)
        .ok_or_else(|| registry_error("API rule descriptor is missing"))?;
    let secret = context
        .registry()
        .descriptor(SECRET_LOCAL_RULE)
        .ok_or_else(|| registry_error("Secret rule descriptor is missing"))?;

    // One continuing request-local admission. Providers never reset it, so the
    // selected limits bound construction as well as the final result.
    let mut admission = Admission::new(context);
    let mut evaluations = Vec::new();
    let mut api_evaluations = evaluate_api(context, scope, api, &mut admission)?;
    evaluations.append(&mut api_evaluations);
    observe(Construction::ApiProviderComplete);
    ensure_not_cancelled(context, API_EXISTS_RULE, "api:secret-provider-handoff")?;
    let mut secret_evaluations = evaluate_secret(context, scope, secret, &mut admission)?;
    evaluations.append(&mut secret_evaluations);
    ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:report-handoff")?;
    build_report(context, evaluations, &admission)
}

fn cancelled_report(context: &RuleExecutionContext<'_>) -> RuleResult<RuleExecutionReport> {
    let version = rule_version()?;
    let mut admission = Admission::new(context);
    let mut evaluations = Vec::with_capacity(2);
    for rule in [API_EXISTS_RULE, SECRET_LOCAL_RULE] {
        admission.admit_evaluation()?;
        let rule_id = rule.parse().map_err(core_construction)?;
        preflight_cancelled(
            &mut admission,
            context,
            &rule_id,
            &version,
            "cancelled:preflight",
        )?;
        observe(Construction::Evaluation);
        evaluations.push(RuleEvaluationRecord::new(
            rule_id,
            version.clone(),
            "cancelled:preflight",
            RuleEvaluationOutcome::Cancelled,
        ));
    }
    build_report(context, evaluations, &admission)
}

fn ensure_not_cancelled(
    context: &RuleExecutionContext<'_>,
    rule_id: &str,
    scope_id: &str,
) -> RuleResult<()> {
    if context.is_cancelled() {
        return Err(
            RuleError::new(RuleErrorCode::RuleCancelled, "rule execution cancelled")
                .with_rule(rule_id)
                .with_scope(scope_id),
        );
    }
    Ok(())
}

fn build_report(
    context: &RuleExecutionContext<'_>,
    evaluations: Vec<RuleEvaluationRecord>,
    admission: &Admission,
) -> RuleResult<RuleExecutionReport> {
    let cancellation_report = !evaluations.is_empty()
        && evaluations
            .iter()
            .all(|evaluation| matches!(evaluation.outcome(), RuleEvaluationOutcome::Cancelled));
    if !cancellation_report {
        ensure_not_cancelled(context, "wow.rules", "report:preflight")?;
    }
    let usage = admission.usage();
    let registry_id = context.registry().registry_id();
    let context_id = context
        .project()
        .snapshot()
        .generation_context()
        .context_id();
    let project_snapshot_id = context.project().snapshot_id();
    let analyzer_snapshot_id = context.project().analyzer_snapshot_id();
    let reference_generation = context.reference().generation_id();
    let reference_view_digest = context.reference().self_digest();
    let fixture_policy_id = context.policy_id();
    let max_bytes = context.budget().max_serialized_output_bytes();
    let material = ReportMaterial {
        registry_id,
        context_id,
        project_snapshot_id,
        analyzer_snapshot_id,
        reference_generation,
        reference_view_digest,
        fixture_policy_id,
        evaluations: &evaluations,
        budget_usage: usage,
    };
    let expected_bytes = preflight_report(&material, max_bytes)?;
    admission.verify_report_plan(&material, expected_bytes)?;
    if !cancellation_report {
        ensure_not_cancelled(context, "wow.rules", "report:construction")?;
    }
    observe(Construction::Report);
    let report = RuleExecutionReport::build(
        registry_id,
        context_id,
        project_snapshot_id,
        analyzer_snapshot_id,
        reference_generation,
        reference_view_digest,
        fixture_policy_id,
        evaluations,
        usage,
    )?;
    verify_canonical_output(&report, expected_bytes, max_bytes)?;
    if !cancellation_report {
        ensure_not_cancelled(context, "wow.rules", "report:delivery")?;
    }
    Ok(report)
}

fn evaluate_api(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
) -> RuleResult<Vec<RuleEvaluationRecord>> {
    let mut output = Vec::new();
    let calls = context
        .project()
        .member_call_report()
        .calls()
        .iter()
        .map(|call| (call.reference_fact_id(), call))
        .collect::<BTreeMap<_, _>>();
    for file in context.project().file_manifest() {
        if !scope.contains(file.file_id()) {
            continue;
        }
        for fact in context.project().member_references_for_file(file.file_id()) {
            if context.is_cancelled() {
                admission.admit_evaluation()?;
                let cancelled_scope = format!("api:{}:cancelled", file.file_id());
                preflight_cancelled(
                    admission,
                    context,
                    descriptor.rule_id(),
                    descriptor.rule_version(),
                    &cancelled_scope,
                )?;
                observe(Construction::Evaluation);
                output.push(RuleEvaluationRecord::new(
                    descriptor.rule_id().clone(),
                    descriptor.rule_version().clone(),
                    &cancelled_scope,
                    RuleEvaluationOutcome::Cancelled,
                ));
                return Ok(output);
            }
            let Some(entity_lookup_key) = api_lookup_key(context, fact) else {
                continue;
            };
            if context.is_fixture_policy() && fact.resolution() == EmmyReferenceResolution::Resolved
            {
                continue;
            }
            admission.admit_evaluation()?;
            let scope_id = format!("api:{}:{}", file.file_id(), fact.fact_id());
            let call = calls.get(fact.fact_id()).copied();
            let fact_ids = api_fact_ids(fact, call);
            if fact.resolution() == EmmyReferenceResolution::Possible {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    admission,
                    &scope_id,
                    vec![RuleBlockerKind::AmbiguousReference],
                    descriptor.required_capabilities().to_vec(),
                    Vec::new(),
                    fact_ids,
                    None,
                )?);
                continue;
            }
            let gate = capability_gate(context, file, descriptor.required_capabilities())?;
            if !gate.blockers.is_empty() {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    admission,
                    &scope_id,
                    gate.blockers,
                    gate.blocking_capabilities,
                    gate.blocking_partitions,
                    fact_ids,
                    None,
                )?);
                continue;
            }
            let (lookup, found) = exact_lookup(
                context,
                admission,
                context.api_partition_id(),
                &entity_lookup_key,
            )?;
            match lookup.outcome() {
                RuleReferenceOutcome::Found => {
                    output.push(clean(
                        context,
                        descriptor,
                        admission,
                        CleanInput {
                            scope_id: &scope_id,
                            claim: RuleCleanClaimKind::ApiExistsForExactUse,
                            fact_ids,
                            lookup: &lookup,
                            coverage_ids: gate.coverage_ids,
                            guard: None,
                        },
                    )?);
                }
                RuleReferenceOutcome::AuthoritativeAbsent
                    if fact.resolution() == EmmyReferenceResolution::Unresolved =>
                {
                    output.push(api_finding(
                        context,
                        descriptor,
                        admission,
                        ApiFindingInput {
                            file,
                            fact,
                            call,
                            entity_lookup_key: &entity_lookup_key,
                            lookup,
                            coverage_ids: gate.coverage_ids,
                        },
                    )?);
                }
                RuleReferenceOutcome::AuthoritativeAbsent => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        vec![RuleBlockerKind::UnsupportedFactShape],
                        vec![capability("reference.symbol.exact_lookup")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                }
                RuleReferenceOutcome::Conflict => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        vec![RuleBlockerKind::ReferenceConflict],
                        vec![capability("reference.symbol.exact_lookup")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                }
                RuleReferenceOutcome::PartialCoverage
                | RuleReferenceOutcome::NotEvaluated
                | RuleReferenceOutcome::PartitionMissing => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        reference_blockers(lookup.outcome()),
                        vec![capability("reference.symbol.exact_lookup")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                }
            }
            if found.is_some_and(|record| record.kind() != ReferenceRecordKind::Api) {
                return Err(RuleError::new(
                    RuleErrorCode::RuleLookupOutcomeInvalid,
                    "API exact lookup returned a non-API record",
                )
                .with_rule(API_EXISTS_RULE)
                .with_scope(&scope_id));
            }
        }
    }
    evaluate_xml_api(context, scope, descriptor, admission, &mut output)?;
    Ok(output)
}

fn evaluate_xml_api(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
    output: &mut Vec<RuleEvaluationRecord>,
) -> RuleResult<()> {
    let Some(report) = context
        .project()
        .snapshot()
        .analyzer_binding()
        .xml_lua_analysis()
    else {
        return Ok(());
    };
    for unit in report.units() {
        if !scope.contains_xml_document(&unit.document) {
            continue;
        }
        validate_xml_api_unit(context, unit)?;
        let calls = unit
            .member_calls
            .iter()
            .map(|call| (call.reference_fact_id.as_ref(), call))
            .collect::<BTreeMap<_, _>>();
        for fact in &unit.member_references {
            if context.is_cancelled() {
                admission.admit_evaluation()?;
                let cancelled_scope = format!("api:xml:{}:cancelled", unit.unit_id);
                preflight_cancelled(
                    admission,
                    context,
                    descriptor.rule_id(),
                    descriptor.rule_version(),
                    &cancelled_scope,
                )?;
                observe(Construction::Evaluation);
                output.push(RuleEvaluationRecord::new(
                    descriptor.rule_id().clone(),
                    descriptor.rule_version().clone(),
                    &cancelled_scope,
                    RuleEvaluationOutcome::Cancelled,
                ));
                return Ok(());
            }
            let Some(entity_lookup_key) =
                api_lookup_key_parts(context, &fact.receiver, &fact.member)
            else {
                continue;
            };
            if context.is_fixture_policy() && fact.resolution == EmmyReferenceResolution::Resolved {
                continue;
            }
            admission.admit_evaluation()?;
            let scope_id = format!("api:xml:{}:{}", unit.unit_id, fact.fact_id);
            let call = calls.get(fact.fact_id.as_ref()).copied();
            let fact_ids = xml_api_fact_ids(fact, call);
            if fact.resolution == EmmyReferenceResolution::Possible {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    admission,
                    &scope_id,
                    vec![RuleBlockerKind::AmbiguousReference],
                    descriptor.required_capabilities().to_vec(),
                    Vec::new(),
                    fact_ids,
                    None,
                )?);
                continue;
            }
            let gate = xml_api_capability_gate(context, unit, descriptor.required_capabilities())?;
            if !gate.blockers.is_empty() {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    admission,
                    &scope_id,
                    gate.blockers,
                    gate.blocking_capabilities,
                    gate.blocking_partitions,
                    fact_ids,
                    None,
                )?);
                continue;
            }
            let (lookup, found) = exact_lookup(
                context,
                admission,
                context.api_partition_id(),
                &entity_lookup_key,
            )?;
            match lookup.outcome() {
                RuleReferenceOutcome::Found => {
                    output.push(clean(
                        context,
                        descriptor,
                        admission,
                        CleanInput {
                            scope_id: &scope_id,
                            claim: RuleCleanClaimKind::ApiExistsForExactUse,
                            fact_ids,
                            lookup: &lookup,
                            coverage_ids: gate.coverage_ids,
                            guard: None,
                        },
                    )?);
                }
                RuleReferenceOutcome::AuthoritativeAbsent
                    if fact.resolution == EmmyReferenceResolution::Unresolved =>
                {
                    output.push(xml_api_finding(
                        context,
                        descriptor,
                        admission,
                        XmlApiFindingInput {
                            unit,
                            fact,
                            call,
                            entity_lookup_key: &entity_lookup_key,
                            lookup,
                            coverage_ids: gate.coverage_ids,
                        },
                    )?);
                }
                RuleReferenceOutcome::AuthoritativeAbsent => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        vec![RuleBlockerKind::UnsupportedFactShape],
                        vec![capability("reference.symbol.exact_lookup")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                }
                RuleReferenceOutcome::Conflict => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        vec![RuleBlockerKind::ReferenceConflict],
                        vec![capability("reference.symbol.exact_lookup")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                }
                RuleReferenceOutcome::PartialCoverage
                | RuleReferenceOutcome::NotEvaluated
                | RuleReferenceOutcome::PartitionMissing => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        reference_blockers(lookup.outcome()),
                        vec![capability("reference.symbol.exact_lookup")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                }
            }
            if found.is_some_and(|record| record.kind() != ReferenceRecordKind::Api) {
                return Err(RuleError::new(
                    RuleErrorCode::RuleLookupOutcomeInvalid,
                    "API exact lookup returned a non-API record",
                )
                .with_rule(API_EXISTS_RULE)
                .with_scope(&scope_id));
            }
        }
    }
    Ok(())
}

fn validate_xml_api_unit(
    context: &RuleExecutionContext<'_>,
    unit: &XmlLuaUnitAnalysis,
) -> RuleResult<()> {
    if !unit.context.admits_static_source_association() {
        return Err(RuleError::new(
            RuleErrorCode::ApiExistsScopeInvalid,
            "XML Lua unit does not carry exact static script-site authority",
        )
        .with_rule(API_EXISTS_RULE)
        .with_scope(unit.unit_id.as_ref()));
    }
    let source = context
        .project()
        .xml_document_source(&unit.document)
        .map_err(|error| {
            RuleError::new(
                RuleErrorCode::ApiExistsScopeInvalid,
                format!("XML source artifact cannot be resolved: {error}"),
            )
            .with_rule(API_EXISTS_RULE)
            .with_scope(unit.unit_id.as_ref())
        })?
        .ok_or_else(|| {
            RuleError::new(
                RuleErrorCode::ApiExistsScopeInvalid,
                "XML Lua unit references a document outside the captured project generation",
            )
            .with_rule(API_EXISTS_RULE)
            .with_scope(unit.unit_id.as_ref())
        })?;
    if source.content_digest() != unit.document_digest {
        return Err(RuleError::new(
            RuleErrorCode::ApiExistsScopeInvalid,
            "XML Lua unit document digest differs from the captured source artifact",
        )
        .with_rule(API_EXISTS_RULE)
        .with_scope(unit.unit_id.as_ref()));
    }

    let mut references = BTreeMap::new();
    for fact in &unit.member_references {
        if references.insert(fact.fact_id.as_ref(), fact).is_some() {
            return Err(RuleError::new(
                RuleErrorCode::RuleFactReferenceGraphInvalid,
                "XML Lua unit contains duplicate member-reference fact identities",
            )
            .with_rule(API_EXISTS_RULE)
            .with_scope(unit.unit_id.as_ref()));
        }
        validate_xml_mapping(unit, &fact.receiver_source, source.byte_length(), true)?;
        validate_xml_mapping(unit, &fact.member_source, source.byte_length(), true)?;
        validate_xml_mapping(unit, &fact.reference_source, source.byte_length(), true)?;
    }
    let mut calls = BTreeMap::new();
    let mut call_references = BTreeMap::new();
    for call in &unit.member_calls {
        if calls.insert(call.fact_id.as_ref(), call).is_some()
            || call_references
                .insert(call.reference_fact_id.as_ref(), call.fact_id.as_ref())
                .is_some()
            || !references.contains_key(call.reference_fact_id.as_ref())
            || references
                .get(call.reference_fact_id.as_ref())
                .is_some_and(|reference| {
                    reference.emmy_fact_id.as_str() != call.emmy_reference_fact_id.as_str()
                })
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleFactReferenceGraphInvalid,
                "XML Lua call facts do not form a unique closed reference graph",
            )
            .with_rule(API_EXISTS_RULE)
            .with_scope(unit.unit_id.as_ref()));
        }
        validate_xml_mapping(unit, &call.callee_source, source.byte_length(), true)?;
        validate_xml_mapping(unit, &call.call_source, source.byte_length(), true)?;
    }
    if unit.semantic_state == XmlLuaSemanticState::NotEvaluatedNoInlineUnits
        && (!unit.member_references.is_empty() || !unit.member_calls.is_empty())
    {
        return Err(RuleError::new(
            RuleErrorCode::RuleFactReferenceGraphInvalid,
            "unevaluated XML Lua unit contains semantic facts",
        )
        .with_rule(API_EXISTS_RULE)
        .with_scope(unit.unit_id.as_ref()));
    }
    Ok(())
}

fn validate_xml_mapping(
    unit: &XmlLuaUnitAnalysis,
    mapping: &XmlLuaMappedSpan,
    document_bytes: u64,
    exact_nonempty: bool,
) -> RuleResult<()> {
    let valid_virtual = mapping.virtual_byte_start <= mapping.virtual_byte_end
        && mapping.virtual_byte_end <= unit.byte_length;
    let exact = !exact_nonempty
        || (mapping.mapping == XmlLuaDiagnosticMapping::ExactPieces
            && mapping.virtual_byte_start < mapping.virtual_byte_end);
    let valid_xml = !mapping.xml_spans.is_empty()
        && mapping.xml_spans.iter().all(|span| {
            span.byte_start <= span.byte_end
                && span.byte_end <= document_bytes
                && (!exact_nonempty || span.byte_start < span.byte_end)
        });
    if valid_virtual && exact && valid_xml {
        Ok(())
    } else {
        Err(RuleError::new(
            RuleErrorCode::RuleSourceHandleInvalid,
            "XML Lua fact does not retain an exact bounded source mapping",
        )
        .with_rule(API_EXISTS_RULE)
        .with_scope(unit.unit_id.as_ref()))
    }
}

fn api_lookup_key(
    context: &RuleExecutionContext<'_>,
    fact: &EmmyMemberReferenceFact,
) -> Option<String> {
    api_lookup_key_parts(context, fact.receiver(), fact.member())
}

fn api_lookup_key_parts(
    context: &RuleExecutionContext<'_>,
    receiver: &str,
    member: &str,
) -> Option<String> {
    if context.is_fixture_policy() {
        return (receiver == "C_E0Fixture").then(|| format!("function:{receiver}.{member}"));
    }
    let key = format!("function:{receiver}.{member}");
    let namespace_prefix = format!("function:{receiver}.");
    let partition = context
        .reference()
        .partitions()
        .iter()
        .find(|partition| partition.id() == context.api_partition_id())?;
    let known_namespace = partition
        .records()
        .iter()
        .any(|record| record.key() == key || record.key().starts_with(&namespace_prefix))
        || context.reference().conflicts().iter().any(|conflict| {
            conflict.partition_id() == context.api_partition_id()
                && (conflict.key() == key || conflict.key().starts_with(&namespace_prefix))
        });
    known_namespace.then_some(key)
}

fn evaluate_secret(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
) -> RuleResult<Vec<RuleEvaluationRecord>> {
    ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:provider")?;
    if !context.supports_secret_policy() {
        admission.admit_evaluation()?;
        return Ok(vec![not_evaluated(
            context,
            descriptor,
            admission,
            "secret:production-policy:restriction-facets-unavailable",
            vec![RuleBlockerKind::MissingRestrictionFacet],
            vec![capability("reference.restriction.facets")?],
            Vec::new(),
            Vec::new(),
            None,
        )?]);
    }
    if context
        .reference()
        .partitions()
        .iter()
        .all(|partition| partition.id() != context.secret_partition_id())
    {
        admission.admit_evaluation()?;
        return Ok(vec![not_evaluated(
            context,
            descriptor,
            admission,
            "secret:restriction-partition-unavailable",
            vec![RuleBlockerKind::ReferencePartitionMissing],
            vec![capability("reference.restriction.facets")?],
            Vec::new(),
            Vec::new(),
            None,
        )?]);
    }

    let mut output = Vec::new();
    let references = context
        .project()
        .member_call_report()
        .references()
        .iter()
        .map(|fact| (fact.fact_id(), fact))
        .collect::<BTreeMap<_, _>>();
    ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:references")?;
    let calls = context
        .project()
        .member_call_report()
        .calls()
        .iter()
        .map(|fact| (fact.fact_id(), fact))
        .collect::<BTreeMap<_, _>>();
    ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:calls")?;
    let flow = context.project().local_flow_report();
    let mut all_bindings = BTreeMap::new();
    for binding in flow.bindings() {
        ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:bindings")?;
        all_bindings.insert(binding.fact_id(), binding);
    }
    let uses_by_operation = group_uses(context, flow.uses())?;

    for binding in flow.bindings() {
        ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:binding")?;
        let (Some(receiver), Some(member)) =
            (binding.initializer_receiver(), binding.initializer_member())
        else {
            continue;
        };
        let producer_entity = format!("function:{receiver}.{member}");
        if context.is_fixture_policy() {
            if producer_entity != FIXTURE_SECRET_ENTITY {
                continue;
            }
        } else {
            let relevant = restriction_key_relevant(context, &producer_entity);
            ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:binding:relevance")?;
            if !relevant {
                continue;
            }
        }
        ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:binding")?;

        let file = context
            .project()
            .file_by_path(binding.path())
            .map_err(project_error)?
            .ok_or_else(|| {
                RuleError::new(
                    RuleErrorCode::RuleStaleInputForbidden,
                    "local binding path is absent from the project snapshot",
                )
            })?;
        if !scope.contains(file.file_id()) {
            continue;
        }
        let Some(reference_id) = binding.initializer_reference_fact_id() else {
            output.push(secret_missing_relation(
                context,
                descriptor,
                admission,
                file.file_id(),
                binding,
                "reference",
            )?);
            continue;
        };
        let Some(call_id) = binding.initializer_call_fact_id() else {
            output.push(secret_missing_relation(
                context,
                descriptor,
                admission,
                file.file_id(),
                binding,
                "call",
            )?);
            continue;
        };
        let Some(reference) = references.get(reference_id).copied() else {
            return Err(fact_graph_error(
                "binding references an unknown member fact",
            ));
        };
        let Some(call) = calls.get(call_id).copied() else {
            return Err(fact_graph_error("binding references an unknown call fact"));
        };
        if reference.resolution() != EmmyReferenceResolution::Resolved
            || reference.receiver() != receiver
            || reference.member() != member
            || call.reference_fact_id() != reference.fact_id()
            || call.path() != binding.path()
            || reference.path() != binding.path()
            || call.content_sha256() != binding.content_sha256()
            || reference.content_sha256() != binding.content_sha256()
        {
            return Err(fact_graph_error(
                "Secret producer binding/member/call identities are inconsistent",
            ));
        }
        let mut operations = Vec::new();
        for operation in flow.operations() {
            ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:operation-scan")?;
            if operation.path() == binding.path()
                && operation
                    .operand_binding_fact_ids()
                    .any(|id| id == binding.fact_id())
            {
                operations.push(operation);
            }
        }
        if operations.is_empty() {
            continue;
        }
        for operation in operations {
            ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:operation")?;
            admission.admit_evaluation()?;
            let scope_id = format!("secret:{}:{}", file.file_id(), operation.fact_id());
            let gate = capability_gate(context, file, descriptor.required_capabilities())?;
            let mut fact_ids = vec![
                binding.fact_id().into(),
                reference.fact_id().into(),
                call.fact_id().into(),
                operation.fact_id().into(),
            ];
            let mut uses = Vec::new();
            if let Some(operation_uses) = uses_by_operation.get(operation.fact_id()) {
                for fact in operation_uses {
                    ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:use")?;
                    if fact.binding_fact_id() == binding.fact_id() {
                        uses.push(*fact);
                    }
                }
            }
            for fact in &uses {
                ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:use")?;
                fact_ids.push(fact.fact_id().into());
            }
            if !gate.blockers.is_empty() {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    admission,
                    &scope_id,
                    gate.blockers,
                    gate.blocking_capabilities,
                    gate.blocking_partitions,
                    fact_ids,
                    None,
                )?);
                observe(Construction::SecretOperationComplete);
                continue;
            }
            if operation.kind() != EmmyOperationKind::Concatenation {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    admission,
                    &scope_id,
                    vec![RuleBlockerKind::UnsupportedFactShape],
                    vec![capability("emmy.fact.operations")?],
                    Vec::new(),
                    fact_ids,
                    None,
                )?);
                observe(Construction::SecretOperationComplete);
                continue;
            }
            if uses.is_empty() {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    admission,
                    &scope_id,
                    vec![RuleBlockerKind::MissingFactRelation],
                    vec![capability("emmy.fact.local_flow")?],
                    Vec::new(),
                    fact_ids,
                    None,
                )?);
                observe(Construction::SecretOperationComplete);
                continue;
            }
            ensure_not_cancelled(context, SECRET_LOCAL_RULE, &scope_id)?;
            let (lookup, found) = exact_lookup(
                context,
                admission,
                context.secret_partition_id(),
                &producer_entity,
            )?;
            ensure_not_cancelled(context, SECRET_LOCAL_RULE, &scope_id)?;
            match lookup.outcome() {
                RuleReferenceOutcome::Found => {
                    let record = found.ok_or_else(|| {
                        RuleError::new(
                            RuleErrorCode::RuleLookupOutcomeInvalid,
                            "found Secret lookup omitted its record",
                        )
                    })?;
                    match classify_secret_record(context, record)? {
                        SecretFacetDecision::NoRestriction => {
                            output.push(clean(
                                context,
                                descriptor,
                                admission,
                                CleanInput {
                                    scope_id: &scope_id,
                                    claim: RuleCleanClaimKind::SecretProducerHasNoMatchingFacet,
                                    fact_ids,
                                    lookup: &lookup,
                                    coverage_ids: gate.coverage_ids,
                                    guard: None,
                                },
                            )?);
                            observe(Construction::SecretOperationComplete);
                            continue;
                        }
                        SecretFacetDecision::Unsupported => {
                            output.push(not_evaluated(
                                context,
                                descriptor,
                                admission,
                                &scope_id,
                                vec![RuleBlockerKind::MissingRestrictionFacet],
                                vec![capability("reference.restriction.facets")?],
                                Vec::new(),
                                fact_ids,
                                Some(&lookup),
                            )?);
                            observe(Construction::SecretOperationComplete);
                            continue;
                        }
                        SecretFacetDecision::Restricted => {}
                    }

                    if !context.is_fixture_policy() {
                        ensure_not_cancelled(context, SECRET_LOCAL_RULE, &scope_id)?;
                        let (guard_lookup, guard_record) = exact_lookup(
                            context,
                            admission,
                            context.secret_partition_id(),
                            context.guard_entity(),
                        )?;
                        ensure_not_cancelled(context, SECRET_LOCAL_RULE, &scope_id)?;
                        let guard_supported = guard_record
                            .map(|record| classify_guard_record(context, record))
                            .transpose()?
                            .unwrap_or(false);
                        if guard_lookup.outcome() != RuleReferenceOutcome::Found || !guard_supported
                        {
                            let blockers = match guard_lookup.outcome() {
                                RuleReferenceOutcome::Conflict => {
                                    vec![RuleBlockerKind::ReferenceConflict]
                                }
                                RuleReferenceOutcome::PartialCoverage
                                | RuleReferenceOutcome::NotEvaluated
                                | RuleReferenceOutcome::PartitionMissing
                                | RuleReferenceOutcome::AuthoritativeAbsent => {
                                    vec![RuleBlockerKind::MissingRestrictionFacet]
                                }
                                RuleReferenceOutcome::Found => {
                                    vec![RuleBlockerKind::UnsupportedFactShape]
                                }
                            };
                            fact_ids.push(guard_lookup.lookup_id().into());
                            output.push(not_evaluated(
                                context,
                                descriptor,
                                admission,
                                &scope_id,
                                blockers,
                                vec![capability("reference.restriction.facets")?],
                                Vec::new(),
                                fact_ids,
                                Some(&guard_lookup),
                            )?);
                            observe(Construction::SecretOperationComplete);
                            continue;
                        }
                        fact_ids.push(guard_lookup.lookup_id().into());
                    }

                    let guard = classify_guard(
                        context.guard_callee(),
                        flow.guards(),
                        flow.control_flow(),
                        &all_bindings,
                        binding,
                        operation,
                    );
                    append_decisive_guard_facts(
                        &mut fact_ids,
                        flow.guards(),
                        flow.control_flow(),
                        binding,
                        operation,
                    );
                    if guard == RuleGuardClassification::DominatingExactValue {
                        output.push(clean(
                            context,
                            descriptor,
                            admission,
                            CleanInput {
                                scope_id: &scope_id,
                                claim: if context.is_fixture_policy() {
                                    RuleCleanClaimKind::SecretFixtureOperationGuardedForExactValueAndScope
                                } else {
                                    RuleCleanClaimKind::SecretProductionOperationGuardedForExactValueAndScope
                                },
                                fact_ids,
                                lookup: &lookup,
                                coverage_ids: gate.coverage_ids,
                                guard: Some(guard),
                            },
                        )?);
                        observe(Construction::SecretOperationComplete);
                    } else {
                        output.push(secret_finding(
                            context,
                            descriptor,
                            admission,
                            file,
                            binding,
                            reference,
                            call,
                            operation,
                            &producer_entity,
                            fact_ids,
                            &uses,
                            flow.guards(),
                            guard,
                            lookup,
                            gate.coverage_ids,
                        )?);
                        observe(Construction::SecretOperationComplete);
                    }
                }
                RuleReferenceOutcome::AuthoritativeAbsent => {
                    output.push(clean(
                        context,
                        descriptor,
                        admission,
                        CleanInput {
                            scope_id: &scope_id,
                            claim: RuleCleanClaimKind::SecretProducerHasNoMatchingFacet,
                            fact_ids,
                            lookup: &lookup,
                            coverage_ids: gate.coverage_ids,
                            guard: None,
                        },
                    )?);
                    observe(Construction::SecretOperationComplete);
                }
                RuleReferenceOutcome::Conflict => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        vec![RuleBlockerKind::ReferenceConflict],
                        vec![capability("reference.restriction.facets")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                    observe(Construction::SecretOperationComplete);
                }
                RuleReferenceOutcome::PartialCoverage
                | RuleReferenceOutcome::NotEvaluated
                | RuleReferenceOutcome::PartitionMissing => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        admission,
                        &scope_id,
                        reference_blockers(lookup.outcome()),
                        vec![capability("reference.restriction.facets")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                    observe(Construction::SecretOperationComplete);
                }
            }
        }
    }
    Ok(output)
}

fn restriction_key_relevant(context: &RuleExecutionContext<'_>, entity: &str) -> bool {
    matches!(
        context
            .reference()
            .lookup(context.secret_partition_id(), entity),
        LookupResult::Found(_) | LookupResult::Conflict(_)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecretFacetDecision {
    Restricted,
    NoRestriction,
    Unsupported,
}

fn classify_secret_record(
    context: &RuleExecutionContext<'_>,
    record: &ReferenceRecord,
) -> RuleResult<SecretFacetDecision> {
    if record.kind() != ReferenceRecordKind::Restriction {
        return Err(RuleError::new(
            RuleErrorCode::RuleLookupOutcomeInvalid,
            "Secret exact lookup returned a non-restriction record",
        )
        .with_rule(SECRET_LOCAL_RULE));
    }
    if record.payload() != context.secret_return_payload() {
        return Ok(SecretFacetDecision::Unsupported);
    }
    if record.restrictions().len() > 1 {
        return Ok(SecretFacetDecision::Unsupported);
    }
    let Some(facet) = record
        .restrictions()
        .iter()
        .find(|facet| facet.id() == "secret.return")
    else {
        return Ok(SecretFacetDecision::NoRestriction);
    };
    Ok(match facet.state() {
        RestrictionState::Restricted | RestrictionState::Forbidden => {
            SecretFacetDecision::Restricted
        }
        RestrictionState::Allowed => SecretFacetDecision::NoRestriction,
        RestrictionState::Unknown => SecretFacetDecision::Unsupported,
    })
}

fn classify_guard_record(
    context: &RuleExecutionContext<'_>,
    record: &ReferenceRecord,
) -> RuleResult<bool> {
    if record.kind() != ReferenceRecordKind::Restriction {
        return Err(RuleError::new(
            RuleErrorCode::RuleLookupOutcomeInvalid,
            "Secret guard lookup returned a non-restriction record",
        )
        .with_rule(SECRET_LOCAL_RULE));
    }
    Ok(
        record.payload() == context.guard_payload().unwrap_or_default()
            && record.restrictions().len() == 1
            && record.restrictions().iter().any(|facet| {
                facet.id() == "secret.predicate" && facet.state() == RestrictionState::Allowed
            }),
    )
}

fn classify_guard(
    accepted_callee: &str,
    guards: &[EmmyGuardFact],
    relations: &[wow_emmy::EmmyControlFlowFact],
    bindings: &BTreeMap<&str, &EmmyLocalBindingFact>,
    binding: &EmmyLocalBindingFact,
    operation: &EmmyOperationFact,
) -> RuleGuardClassification {
    for relation in relations {
        if relation.relation() == EmmyControlFlowRelationKind::Dominates
            && relation.binding_fact_id() == binding.fact_id()
            && relation.operation_fact_id() == operation.fact_id()
            && guards.iter().any(|guard| {
                guard.fact_id() == relation.guard_fact_id()
                    && guard.kind() == EmmyGuardKind::AccessSingle
                    && guard.callee() == accepted_callee
                    && guard.guarded_binding_fact_id() == binding.fact_id()
            })
        {
            return RuleGuardClassification::DominatingExactValue;
        }
    }
    let operation_span = operation.operation_span();
    let same_binding = guards
        .iter()
        .filter(|guard| {
            guard.kind() == EmmyGuardKind::AccessSingle
                && guard.callee() == accepted_callee
                && guard.guarded_binding_fact_id() == binding.fact_id()
        })
        .collect::<Vec<_>>();
    if same_binding.iter().any(|guard| {
        guard
            .guard_span()
            .byte_start()
            .zip(operation_span.byte_end())
            .is_some_and(|(guard_start, operation_end)| guard_start >= operation_end)
    }) {
        return RuleGuardClassification::AfterUse;
    }
    if !same_binding.is_empty() {
        return RuleGuardClassification::NonDominating;
    }
    if guards.iter().any(|guard| {
        bindings
            .get(guard.guarded_binding_fact_id())
            .is_some_and(|other| other.path() == operation.path())
            && span_contains(guard.guarded_block_span(), operation_span)
    }) {
        RuleGuardClassification::DifferentValue
    } else {
        RuleGuardClassification::Absent
    }
}

fn span_contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer
        .byte_start()
        .zip(outer.byte_end())
        .zip(inner.byte_start().zip(inner.byte_end()))
        .is_some_and(|((outer_start, outer_end), (inner_start, inner_end))| {
            outer_start <= inner_start && inner_end <= outer_end
        })
}

fn api_finding(
    context: &RuleExecutionContext<'_>,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
    input: ApiFindingInput<'_>,
) -> RuleResult<RuleEvaluationRecord> {
    let ApiFindingInput {
        file,
        fact,
        call,
        entity_lookup_key,
        lookup,
        coverage_ids,
    } = input;
    // The provider admitted this evaluation before lookup or fact retention.
    admission.admit_finding()?;
    let mut input_fact_ids = api_fact_ids(fact, call);
    input_fact_ids.sort();
    input_fact_ids.dedup();
    let entity = wow_core::EntityKey::new(
        "function",
        &format!("{}.{}", fact.receiver(), fact.member()),
    )
    .map_err(core_construction)?;
    let mut plans = Vec::new();
    admit_source_plan(
        admission,
        &mut plans,
        project_source_plan(
            context,
            file.relative_path().as_str(),
            file.content_digest(),
            fact.member_span(),
            Some(&entity),
        ),
    )?;
    if let Some(call) = call {
        admit_source_plan(
            admission,
            &mut plans,
            project_source_plan(
                context,
                file.relative_path().as_str(),
                file.content_digest(),
                call.call_span(),
                None,
            ),
        )?;
    }
    let reference_origin = reference_origin(context);
    admit_source_plan(
        admission,
        &mut plans,
        reference_source_plan(context, &reference_origin)?,
    )?;
    let scope_id = format!("api:{}:{}", file.file_id(), fact.fact_id());
    let arguments = api_argument_plans(
        context,
        entity_lookup_key,
        fact.receiver(),
        fact.member(),
        call.is_some(),
    );
    preflight_finding(
        admission,
        context,
        &FindingMaterial {
            scope_id: &scope_id,
            descriptor,
            subject: &entity,
            sources: &plans,
            arguments: &arguments,
            fact_ids: &input_fact_ids,
            coverage_ids: &coverage_ids,
            lookup: &lookup,
            guard: None,
        },
    )?;
    let (primary, related, source_handles, reference_handle_id) =
        materialize_sources(context, plans)?;
    let evidence = evidence_bundle(context, &source_handles, reference_handle_id)?;
    let evidence_ids = evidence
        .iter()
        .map(EvidenceRecord::evidence_id)
        .collect::<Vec<_>>();
    let root_material = (
        descriptor.rule_id(),
        descriptor.rule_version(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        fact.fact_id(),
        lookup.lookup_id(),
        primary.handle_id(),
        entity_lookup_key,
    );
    admission.preflight_identity("wow-core/root-cause/e0-1", &root_material)?;
    observe(Construction::Root);
    let root = derive_root_cause_key(&root_material).map_err(core_construction)?;
    let draft = FindingDraft::new(
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        descriptor.semantic_category().clone(),
        descriptor.technical_severity(),
        descriptor.rollout_policy(),
        primary.handle_id(),
        CoverageStatus::Complete,
    )
    .subject_entity_key(entity)
    .related_source_handle_ids(related.iter().map(SourceHandle::handle_id).collect())
    .evidence_ids(evidence_ids)
    .required_capability_ids(descriptor.required_capabilities().to_vec())
    .message_arguments(materialize_arguments(&arguments)?)
    .map_err(core_construction)?
    .root_causes(Some(root), None)
    .remediation(
        Remediation::new(RemediationClass::PlanOnly, None, None).map_err(core_construction)?,
    );
    observe(Construction::Finding);
    let finding = draft
        .bind(
            context
                .project()
                .snapshot()
                .generation_context()
                .context_id(),
            &source_handles,
            &evidence,
        )
        .map_err(core_construction)?;
    let usage = RuleBudgetUsage {
        evaluations: 1,
        findings: 1,
        source_handles: usize_to_u64(source_handles.len())?,
        evidence_records: usize_to_u64(evidence.len())?,
    };
    let result = RuleFindingSet::build(
        context.policy_id(),
        RuleFindingSetInput {
            input_fact_ids,
            findings: vec![finding],
            source_handles,
            evidence_records: evidence,
            coverage_ids,
            reference_lookup: lookup,
            guard_classification: None,
            budget_usage: usage,
        },
    )?;
    observe(Construction::Evaluation);
    Ok(RuleEvaluationRecord::new(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        &scope_id,
        RuleEvaluationOutcome::Findings {
            result: Box::new(result),
        },
    ))
}

fn xml_api_finding(
    context: &RuleExecutionContext<'_>,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
    input: XmlApiFindingInput<'_>,
) -> RuleResult<RuleEvaluationRecord> {
    let XmlApiFindingInput {
        unit,
        fact,
        call,
        entity_lookup_key,
        lookup,
        coverage_ids,
    } = input;
    // The provider admitted this evaluation before lookup or fact retention.
    admission.admit_finding()?;
    let mut input_fact_ids = xml_api_fact_ids(fact, call);
    input_fact_ids.sort();
    input_fact_ids.dedup();
    let entity = EntityKey::new("function", &format!("{}.{}", fact.receiver, fact.member))
        .map_err(core_construction)?;
    let mut plans = Vec::new();
    plan_xml_sources(
        context,
        admission,
        &mut plans,
        unit,
        &fact.member_source,
        Some(&entity),
    )?;
    if let Some(call) = call {
        plan_xml_sources(
            context,
            admission,
            &mut plans,
            unit,
            &call.call_source,
            None,
        )?;
    }
    let reference_origin = reference_origin(context);
    admit_source_plan(
        admission,
        &mut plans,
        reference_source_plan(context, &reference_origin)?,
    )?;
    let scope_id = format!("api:xml:{}:{}", unit.unit_id, fact.fact_id);
    let arguments = api_argument_plans(
        context,
        entity_lookup_key,
        &fact.receiver,
        &fact.member,
        call.is_some(),
    );
    preflight_finding(
        admission,
        context,
        &FindingMaterial {
            scope_id: &scope_id,
            descriptor,
            subject: &entity,
            sources: &plans,
            arguments: &arguments,
            fact_ids: &input_fact_ids,
            coverage_ids: &coverage_ids,
            lookup: &lookup,
            guard: None,
        },
    )?;
    let (primary, related, source_handles, reference_handle_id) =
        materialize_sources(context, plans)?;
    let evidence = evidence_bundle(context, &source_handles, reference_handle_id)?;
    let evidence_ids = evidence
        .iter()
        .map(EvidenceRecord::evidence_id)
        .collect::<Vec<_>>();
    let root_material = (
        descriptor.rule_id(),
        descriptor.rule_version(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        unit.context.context_id(),
        unit.unit_id.as_ref(),
        fact.fact_id.as_ref(),
        lookup.lookup_id(),
        primary.handle_id(),
        entity_lookup_key,
    );
    admission.preflight_identity("wow-core/root-cause/e0-1", &root_material)?;
    observe(Construction::Root);
    let root = derive_root_cause_key(&root_material).map_err(core_construction)?;
    let draft = FindingDraft::new(
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        descriptor.semantic_category().clone(),
        descriptor.technical_severity(),
        descriptor.rollout_policy(),
        primary.handle_id(),
        CoverageStatus::Complete,
    )
    .subject_entity_key(entity)
    .related_source_handle_ids(related.iter().map(SourceHandle::handle_id).collect())
    .evidence_ids(evidence_ids)
    .required_capability_ids(descriptor.required_capabilities().to_vec())
    .message_arguments(materialize_arguments(&arguments)?)
    .map_err(core_construction)?
    .root_causes(Some(root), None)
    .remediation(
        Remediation::new(RemediationClass::PlanOnly, None, None).map_err(core_construction)?,
    );
    observe(Construction::Finding);
    let finding = draft
        .bind(
            context
                .project()
                .snapshot()
                .generation_context()
                .context_id(),
            &source_handles,
            &evidence,
        )
        .map_err(core_construction)?;
    let usage = RuleBudgetUsage {
        evaluations: 1,
        findings: 1,
        source_handles: usize_to_u64(source_handles.len())?,
        evidence_records: usize_to_u64(evidence.len())?,
    };
    let result = RuleFindingSet::build(
        context.policy_id(),
        RuleFindingSetInput {
            input_fact_ids,
            findings: vec![finding],
            source_handles,
            evidence_records: evidence,
            coverage_ids,
            reference_lookup: lookup,
            guard_classification: None,
            budget_usage: usage,
        },
    )?;
    observe(Construction::Evaluation);
    Ok(RuleEvaluationRecord::new(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        &scope_id,
        RuleEvaluationOutcome::Findings {
            result: Box::new(result),
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn secret_finding(
    context: &RuleExecutionContext<'_>,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
    file: &ProjectFileRecord,
    binding: &EmmyLocalBindingFact,
    reference: &EmmyMemberReferenceFact,
    call: &EmmyMemberCallFact,
    operation: &EmmyOperationFact,
    producer_entity: &str,
    mut input_fact_ids: Vec<Box<str>>,
    uses: &[&wow_emmy::EmmyLocalUseFact],
    guards: &[EmmyGuardFact],
    guard: RuleGuardClassification,
    lookup: RuleReferenceLookupRecord,
    coverage_ids: Vec<CoverageId>,
) -> RuleResult<RuleEvaluationRecord> {
    // The provider admitted this evaluation before lookup or fact retention.
    admission.admit_finding()?;
    let entity_name = producer_entity.strip_prefix("function:").ok_or_else(|| {
        RuleError::new(
            RuleErrorCode::RuleFindingInputInvalid,
            "Secret producer entity is not a canonical function key",
        )
    })?;
    let entity = wow_core::EntityKey::new("function", entity_name).map_err(core_construction)?;
    input_fact_ids.sort();
    input_fact_ids.dedup();
    let mut plans = Vec::new();
    admit_source_plan(
        admission,
        &mut plans,
        project_source_plan(
            context,
            file.relative_path().as_str(),
            file.content_digest(),
            operation.operation_span(),
            Some(&entity),
        ),
    )?;
    for span in [
        reference.reference_span(),
        call.call_span(),
        binding.declaration_span(),
    ] {
        admit_source_plan(
            admission,
            &mut plans,
            project_source_plan(
                context,
                file.relative_path().as_str(),
                file.content_digest(),
                span,
                None,
            ),
        )?;
    }
    for use_fact in uses {
        admit_source_plan(
            admission,
            &mut plans,
            project_source_plan(
                context,
                file.relative_path().as_str(),
                file.content_digest(),
                use_fact.use_span(),
                None,
            ),
        )?;
    }
    if let Some(guard_fact) = representative_guard(guards, binding, operation, guard) {
        admit_source_plan(
            admission,
            &mut plans,
            project_source_plan(
                context,
                file.relative_path().as_str(),
                file.content_digest(),
                guard_fact.guard_span(),
                None,
            ),
        )?;
    }
    let reference_origin = reference_origin(context);
    admit_source_plan(
        admission,
        &mut plans,
        reference_source_plan(context, &reference_origin)?,
    )?;
    let scope_id = format!("secret:{}:{}", file.file_id(), operation.fact_id());
    let arguments = [
        ArgumentPlan::new("facet", MessageArgumentKind::Identifier, "secret.return"),
        ArgumentPlan::new(
            if context.is_fixture_policy() {
                "fixture_policy"
            } else {
                "rule_policy"
            },
            MessageArgumentKind::Identifier,
            context.policy_id(),
        ),
        ArgumentPlan::new("guard_state", MessageArgumentKind::Text, guard_name(guard)),
        ArgumentPlan::new(
            "operation",
            MessageArgumentKind::Identifier,
            "concatenation",
        ),
        ArgumentPlan::new("producer", MessageArgumentKind::Identifier, producer_entity),
        ArgumentPlan::new(
            "remediation_plan",
            MessageArgumentKind::Identifier,
            "restructure-secret-capable-value-use",
        ),
        ArgumentPlan::new("return_position", MessageArgumentKind::Integer, "1"),
    ];
    preflight_finding(
        admission,
        context,
        &FindingMaterial {
            scope_id: &scope_id,
            descriptor,
            subject: &entity,
            sources: &plans,
            arguments: &arguments,
            fact_ids: &input_fact_ids,
            coverage_ids: &coverage_ids,
            lookup: &lookup,
            guard: Some(guard),
        },
    )?;
    let (primary, related, source_handles, reference_handle_id) =
        materialize_sources(context, plans)?;
    let evidence = evidence_bundle(context, &source_handles, reference_handle_id)?;
    let evidence_ids = evidence
        .iter()
        .map(EvidenceRecord::evidence_id)
        .collect::<Vec<_>>();
    let root_material = (
        descriptor.rule_id(),
        descriptor.rule_version(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        binding.fact_id(),
        operation.fact_id(),
        lookup.lookup_id(),
        context.policy_id(),
        context.policy_digest(),
        guard,
        primary.handle_id(),
    );
    admission.preflight_identity("wow-core/root-cause/e0-1", &root_material)?;
    observe(Construction::Root);
    let root = derive_root_cause_key(&root_material).map_err(core_construction)?;
    let draft = FindingDraft::new(
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        descriptor.semantic_category().clone(),
        Severity::Error,
        RolloutPolicy::Advisory,
        primary.handle_id(),
        CoverageStatus::Complete,
    )
    .subject_entity_key(entity)
    .related_source_handle_ids(related.iter().map(SourceHandle::handle_id).collect())
    .evidence_ids(evidence_ids)
    .required_capability_ids(descriptor.required_capabilities().to_vec())
    .message_arguments(materialize_arguments(&arguments)?)
    .map_err(core_construction)?
    .root_causes(Some(root), None)
    .remediation(
        Remediation::new(RemediationClass::PlanOnly, None, None).map_err(core_construction)?,
    );
    observe(Construction::Finding);
    let finding = draft
        .bind(
            context
                .project()
                .snapshot()
                .generation_context()
                .context_id(),
            &source_handles,
            &evidence,
        )
        .map_err(core_construction)?;
    let usage = RuleBudgetUsage {
        evaluations: 1,
        findings: 1,
        source_handles: usize_to_u64(source_handles.len())?,
        evidence_records: usize_to_u64(evidence.len())?,
    };
    let result = RuleFindingSet::build(
        context.policy_id(),
        RuleFindingSetInput {
            input_fact_ids,
            findings: vec![finding],
            source_handles,
            evidence_records: evidence,
            coverage_ids,
            reference_lookup: lookup,
            guard_classification: Some(guard),
            budget_usage: usage,
        },
    )?;
    observe(Construction::Evaluation);
    Ok(RuleEvaluationRecord::new(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        &scope_id,
        RuleEvaluationOutcome::Findings {
            result: Box::new(result),
        },
    ))
}

fn append_decisive_guard_facts(
    fact_ids: &mut Vec<Box<str>>,
    guards: &[EmmyGuardFact],
    relations: &[wow_emmy::EmmyControlFlowFact],
    binding: &EmmyLocalBindingFact,
    operation: &EmmyOperationFact,
) {
    let operation_span = operation.operation_span();
    fact_ids.extend(
        guards
            .iter()
            .filter(|guard| {
                guard.guarded_binding_fact_id() == binding.fact_id()
                    || span_contains(guard.guarded_block_span(), operation_span)
            })
            .map(|guard| Box::<str>::from(guard.fact_id())),
    );
    fact_ids.extend(
        relations
            .iter()
            .filter(|relation| relation.operation_fact_id() == operation.fact_id())
            .map(|relation| Box::<str>::from(relation.fact_id())),
    );
}

fn representative_guard<'a>(
    guards: &'a [EmmyGuardFact],
    binding: &EmmyLocalBindingFact,
    operation: &EmmyOperationFact,
    classification: RuleGuardClassification,
) -> Option<&'a EmmyGuardFact> {
    match classification {
        RuleGuardClassification::Absent => None,
        RuleGuardClassification::DifferentValue => guards.iter().find(|guard| {
            guard.guarded_binding_fact_id() != binding.fact_id()
                && span_contains(guard.guarded_block_span(), operation.operation_span())
        }),
        _ => guards
            .iter()
            .find(|guard| guard.guarded_binding_fact_id() == binding.fact_id()),
    }
}

fn clean(
    context: &RuleExecutionContext<'_>,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
    input: CleanInput<'_>,
) -> RuleResult<RuleEvaluationRecord> {
    let CleanInput {
        scope_id,
        claim,
        mut fact_ids,
        lookup,
        mut coverage_ids,
        guard,
    } = input;
    fact_ids.sort();
    fact_ids.dedup();
    coverage_ids.sort_unstable();
    coverage_ids.dedup();
    preflight_clean(
        admission,
        context,
        descriptor,
        scope_id,
        claim,
        &fact_ids,
        lookup,
        &coverage_ids,
        guard,
    )?;
    let record = CleanEvaluationRecord::build(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        context.policy_id(),
        scope_id,
        claim,
        fact_ids,
        lookup.lookup_id(),
        descriptor.required_capabilities().to_vec(),
        coverage_ids,
        guard,
        RuleBudgetUsage {
            evaluations: 1,
            ..RuleBudgetUsage::default()
        },
    )?;
    observe(Construction::Clean);
    observe(Construction::Evaluation);
    Ok(RuleEvaluationRecord::new(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        scope_id,
        RuleEvaluationOutcome::EvaluatedClean { record },
    ))
}

#[allow(clippy::too_many_arguments)]
fn not_evaluated(
    context: &RuleExecutionContext<'_>,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
    scope_id: &str,
    mut blockers: Vec<RuleBlockerKind>,
    mut blocking_capabilities: Vec<CapabilityId>,
    blocking_partitions: Vec<BlockingPartitionRef>,
    mut fact_ids: Vec<Box<str>>,
    lookup: Option<&RuleReferenceLookupRecord>,
) -> RuleResult<RuleEvaluationRecord> {
    if blocking_capabilities.is_empty() {
        blocking_capabilities.push(
            descriptor
                .required_capabilities()
                .first()
                .cloned()
                .ok_or_else(|| registry_error("descriptor has no capability requirement"))?,
        );
    }
    blockers.sort();
    blockers.dedup();
    fact_ids.sort();
    fact_ids.dedup();
    preflight_not_evaluated(
        admission,
        context,
        descriptor,
        scope_id,
        &blockers,
        &blocking_capabilities,
        &blocking_partitions,
        &fact_ids,
        lookup,
    )?;
    let core = NotEvaluatedRecord::new(
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        producer_id()?,
        rule_version()?,
        "rule",
        scope_id,
        MessageCode::from_str("rule_required_capability_unavailable").map_err(core_construction)?,
        blocking_capabilities,
        blocking_partitions,
        Vec::new(),
    )
    .map_err(core_construction)?;
    let detail = RuleNotEvaluatedDetail::build(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        scope_id,
        blockers,
        core,
        fact_ids,
        lookup.map(RuleReferenceLookupRecord::lookup_id),
    )?;
    observe(Construction::Evaluation);
    Ok(RuleEvaluationRecord::new(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        scope_id,
        RuleEvaluationOutcome::NotEvaluated {
            detail: Box::new(detail),
        },
    ))
}

fn secret_missing_relation(
    context: &RuleExecutionContext<'_>,
    descriptor: &crate::RuleDescriptor,
    admission: &mut Admission,
    file_id: &ProjectFileId,
    binding: &EmmyLocalBindingFact,
    relation: &str,
) -> RuleResult<RuleEvaluationRecord> {
    admission.admit_evaluation()?;
    let scope_id = format!("secret:{file_id}:{}:{relation}", binding.fact_id());
    not_evaluated(
        context,
        descriptor,
        admission,
        &scope_id,
        vec![RuleBlockerKind::MissingFactRelation],
        vec![capability("emmy.fact.local_bindings")?],
        Vec::new(),
        vec![binding.fact_id().into()],
        None,
    )
}

#[derive(Default)]
struct CapabilityGate {
    blockers: Vec<RuleBlockerKind>,
    blocking_capabilities: Vec<CapabilityId>,
    blocking_partitions: Vec<BlockingPartitionRef>,
    coverage_ids: Vec<CoverageId>,
}

fn capability_gate(
    context: &RuleExecutionContext<'_>,
    file: &ProjectFileRecord,
    required: &[CapabilityId],
) -> RuleResult<CapabilityGate> {
    let mut gate = CapabilityGate::default();
    for capability in required {
        let name = capability.as_str();
        if name.starts_with("reference.") {
            continue;
        }
        if name.starts_with("project.") {
            let records = context
                .project()
                .project_coverage_records()
                .iter()
                .filter(|record| record.capability_id() == capability)
                .collect::<Vec<_>>();
            if records.is_empty() {
                gate.blockers.push(RuleBlockerKind::MissingCapability);
                gate.blocking_capabilities.push(capability.clone());
            } else {
                for record in records {
                    gate.coverage_ids.push(record.coverage_id());
                    if record.status() != CoverageStatus::Complete
                        || !record.conflict_ids().is_empty()
                        || !record.truncation_refs().is_empty()
                    {
                        gate.blockers.push(RuleBlockerKind::FailedCapability);
                        gate.blocking_capabilities.push(capability.clone());
                        gate.blocking_partitions
                            .push(BlockingPartitionRef::from_record(record));
                    }
                }
            }
            continue;
        }
        if name.starts_with("emmy.") {
            let records = context
                .project()
                .snapshot()
                .analyzer_binding()
                .capability_records()
                .iter()
                .filter(|record| {
                    record.capability_id() == capability
                        && match record.scope() {
                            ProjectAnalyzerCapabilityScope::Workspace => true,
                            ProjectAnalyzerCapabilityScope::File => {
                                record.subject_id() == file.file_id().as_str()
                            }
                        }
                })
                .collect::<Vec<_>>();
            if records.is_empty() {
                gate.blockers.push(RuleBlockerKind::MissingCapability);
                gate.blocking_capabilities.push(capability.clone());
            } else if records.iter().any(|record| {
                record.state() != ProjectAnalyzerCapabilityState::Complete
                    || record.parse_error_count() != 0
            }) {
                gate.blockers.push(RuleBlockerKind::FailedCapability);
                gate.blocking_capabilities.push(capability.clone());
            }
        }
    }
    if let Some(record) = context
        .project()
        .project_coverage_records()
        .iter()
        .find(|record| {
            record.capability_id().as_str() == "project.analyzer.facts.available"
                && record.partition_id().scope() == "project.file"
                && record.partition_id().key() == Some(file.file_id().as_str())
        })
    {
        gate.coverage_ids.push(record.coverage_id());
        if record.status() != CoverageStatus::Complete {
            gate.blockers.push(RuleBlockerKind::FailedCapability);
            gate.blocking_capabilities
                .push(capability("project.analyzer.facts.available")?);
            gate.blocking_partitions
                .push(BlockingPartitionRef::from_record(record));
        }
    } else {
        gate.blockers.push(RuleBlockerKind::MissingCapability);
        gate.blocking_capabilities
            .push(capability("project.analyzer.facts.available")?);
    }
    finalize_capability_gate(&mut gate);
    Ok(gate)
}

fn xml_api_capability_gate(
    context: &RuleExecutionContext<'_>,
    unit: &XmlLuaUnitAnalysis,
    required: &[CapabilityId],
) -> RuleResult<CapabilityGate> {
    let mut gate = CapabilityGate::default();
    let facts_complete = xml_unit_facts_complete(unit);
    for capability in required {
        let name = capability.as_str();
        if name.starts_with("reference.") {
            continue;
        }
        if name.starts_with("project.") {
            let records = context
                .project()
                .project_coverage_records()
                .iter()
                .filter(|record| record.capability_id() == capability)
                .collect::<Vec<_>>();
            if records.is_empty() {
                gate.blockers.push(RuleBlockerKind::MissingCapability);
                gate.blocking_capabilities.push(capability.clone());
            } else {
                for record in records {
                    gate.coverage_ids.push(record.coverage_id());
                    if record.status() != CoverageStatus::Complete
                        || !record.conflict_ids().is_empty()
                        || !record.truncation_refs().is_empty()
                    {
                        gate.blockers.push(RuleBlockerKind::FailedCapability);
                        gate.blocking_capabilities.push(capability.clone());
                        gate.blocking_partitions
                            .push(BlockingPartitionRef::from_record(record));
                    }
                }
            }
            continue;
        }
        if !name.starts_with("emmy.") {
            continue;
        }
        let complete = match name {
            "emmy.file.parsed" | "emmy.fact.references" | "emmy.fact.calls" => facts_complete,
            "emmy.source_coordinates.exact" => {
                unit.member_references.iter().all(|fact| {
                    fact.receiver_source.mapping == XmlLuaDiagnosticMapping::ExactPieces
                        && fact.member_source.mapping == XmlLuaDiagnosticMapping::ExactPieces
                        && fact.reference_source.mapping == XmlLuaDiagnosticMapping::ExactPieces
                }) && unit.member_calls.iter().all(|call| {
                    call.callee_source.mapping == XmlLuaDiagnosticMapping::ExactPieces
                        && call.call_source.mapping == XmlLuaDiagnosticMapping::ExactPieces
                })
            }
            _ => {
                let records = context
                    .project()
                    .snapshot()
                    .analyzer_binding()
                    .capability_records()
                    .iter()
                    .filter(|record| {
                        record.capability_id() == capability
                            && record.scope() == ProjectAnalyzerCapabilityScope::Workspace
                    })
                    .collect::<Vec<_>>();
                if records.is_empty() {
                    gate.blockers.push(RuleBlockerKind::MissingCapability);
                    gate.blocking_capabilities.push(capability.clone());
                    continue;
                }
                records.iter().all(|record| {
                    record.state() == ProjectAnalyzerCapabilityState::Complete
                        && record.parse_error_count() == 0
                })
            }
        };
        if !complete {
            gate.blockers.push(RuleBlockerKind::FailedCapability);
            gate.blocking_capabilities.push(capability.clone());
        }
    }
    if let Some(record) = context
        .project()
        .project_coverage_records()
        .iter()
        .find(|record| {
            record.capability_id().as_str() == "project.analyzer.facts.available"
                && record.partition_id().scope() == "project.xml_lua_unit"
                && record.partition_id().key() == Some(unit.unit_id.as_ref())
        })
    {
        gate.coverage_ids.push(record.coverage_id());
        if record.status() != CoverageStatus::Complete {
            gate.blockers.push(RuleBlockerKind::FailedCapability);
            gate.blocking_capabilities
                .push(capability("project.analyzer.facts.available")?);
            gate.blocking_partitions
                .push(BlockingPartitionRef::from_record(record));
        }
    } else {
        gate.blockers.push(RuleBlockerKind::MissingCapability);
        gate.blocking_capabilities
            .push(capability("project.analyzer.facts.available")?);
    }
    finalize_capability_gate(&mut gate);
    Ok(gate)
}

fn xml_unit_facts_complete(unit: &XmlLuaUnitAnalysis) -> bool {
    unit.semantic_state == XmlLuaSemanticState::Complete
        && unit.diagnostics.is_empty()
        && unit.semantic_diagnostics.iter().all(|diagnostic| {
            !matches!(
                diagnostic.kind,
                EmmySyntaxDiagnosticKind::LuaSyntax | EmmySyntaxDiagnosticKind::DocumentationSyntax
            )
        })
}

fn finalize_capability_gate(gate: &mut CapabilityGate) {
    gate.blockers.sort();
    gate.blockers.dedup();
    gate.blocking_capabilities.sort();
    gate.blocking_capabilities.dedup();
    gate.blocking_partitions.sort();
    gate.blocking_partitions.dedup();
    gate.coverage_ids.sort_unstable();
    gate.coverage_ids.dedup();
}

fn exact_lookup<'a>(
    context: &'a RuleExecutionContext<'_>,
    admission: &Admission,
    partition: &str,
    key: &str,
) -> RuleResult<(RuleReferenceLookupRecord, Option<&'a ReferenceRecord>)> {
    ensure_not_cancelled(context, "wow.rules", "reference:lookup")?;
    let result = context.reference().lookup(partition, key);
    let outcome = match &result {
        LookupResult::Found(_) => RuleReferenceOutcome::Found,
        LookupResult::AuthoritativeAbsence => RuleReferenceOutcome::AuthoritativeAbsent,
        LookupResult::Conflict(_) => RuleReferenceOutcome::Conflict,
        LookupResult::Unknown(LookupUnknownReason::PartitionMissing) => {
            RuleReferenceOutcome::PartitionMissing
        }
        LookupResult::Unknown(LookupUnknownReason::PartialCoverage) => {
            RuleReferenceOutcome::PartialCoverage
        }
        LookupResult::Unknown(LookupUnknownReason::NotEvaluated) => {
            RuleReferenceOutcome::NotEvaluated
        }
    };
    preflight_lookup(admission, context, partition, key, outcome)?;
    match &result {
        LookupResult::Found(record) => admission.preflight_value(record)?,
        LookupResult::Conflict(conflict) => {
            admission.preflight_identity("wow-rules/reference-conflict/e0-e/1", conflict)?
        }
        _ => {}
    }
    ensure_not_cancelled(context, "wow.rules", "reference:lookup-record")?;
    observe(Construction::Lookup);
    match result {
        LookupResult::Found(record) => Ok((
            RuleReferenceLookupRecord::new(
                partition,
                key,
                context.reference().generation_id(),
                context.reference().self_digest(),
                RuleReferenceOutcome::Found,
                Some(record.digest().map_err(reference_error)?),
                None,
            )?,
            Some(record),
        )),
        LookupResult::AuthoritativeAbsence => Ok((
            RuleReferenceLookupRecord::new(
                partition,
                key,
                context.reference().generation_id(),
                context.reference().self_digest(),
                RuleReferenceOutcome::AuthoritativeAbsent,
                None,
                None,
            )?,
            None,
        )),
        LookupResult::Conflict(conflict) => Ok((
            RuleReferenceLookupRecord::new(
                partition,
                key,
                context.reference().generation_id(),
                context.reference().self_digest(),
                RuleReferenceOutcome::Conflict,
                None,
                Some(
                    canonical_id(
                        "reference-conflict:sha256:",
                        "wow-rules/reference-conflict/e0-e/1",
                        conflict,
                    )?
                    .to_string(),
                ),
            )?,
            None,
        )),
        LookupResult::Unknown(reason) => {
            let outcome = match reason {
                LookupUnknownReason::PartitionMissing => RuleReferenceOutcome::PartitionMissing,
                LookupUnknownReason::PartialCoverage => RuleReferenceOutcome::PartialCoverage,
                LookupUnknownReason::NotEvaluated => RuleReferenceOutcome::NotEvaluated,
            };
            Ok((
                RuleReferenceLookupRecord::new(
                    partition,
                    key,
                    context.reference().generation_id(),
                    context.reference().self_digest(),
                    outcome,
                    None,
                    None,
                )?,
                None,
            ))
        }
    }
}

fn reference_blockers(outcome: RuleReferenceOutcome) -> Vec<RuleBlockerKind> {
    match outcome {
        RuleReferenceOutcome::PartialCoverage => vec![RuleBlockerKind::PartialCoverage],
        RuleReferenceOutcome::PartitionMissing => vec![RuleBlockerKind::ReferencePartitionMissing],
        RuleReferenceOutcome::NotEvaluated => vec![RuleBlockerKind::MissingCapability],
        RuleReferenceOutcome::Conflict => vec![RuleBlockerKind::ReferenceConflict],
        RuleReferenceOutcome::Found | RuleReferenceOutcome::AuthoritativeAbsent => Vec::new(),
    }
}

fn project_source_plan<'a>(
    context: &'a RuleExecutionContext<'_>,
    path: &'a str,
    content_digest: ContentDigest<SourceContent>,
    span: SourceSpan,
    entity: Option<&'a EntityKey>,
) -> SourcePlan<'a> {
    let origin = context
        .project()
        .snapshot()
        .source_registry()
        .source_origin();
    let origin_kind = match origin.origin_kind() {
        wow_project::ProjectSourceOriginKind::FixtureProject => SourceOriginKind::Fixture,
        wow_project::ProjectSourceOriginKind::RepositoryProject => {
            SourceOriginKind::GeneratedArtifact
        }
        wow_project::ProjectSourceOriginKind::BlizzardUiPlatformSource => {
            // The published registry was admitted by Project. Its profile is the
            // same closed source-class input used by Project::source_handle.
            if context
                .project()
                .configuration()
                .platform_packages()
                .is_some_and(|packages| {
                    packages.source().profile().source_class()
                        == wow_project::platform_source::PlatformSourceClass::SyntheticFixture
                })
            {
                SourceOriginKind::Fixture
            } else {
                SourceOriginKind::GeneratedArtifact
            }
        }
    };
    SourcePlan {
        origin_kind,
        origin_id: origin.origin_id().as_str(),
        revision: origin.revision_identity(),
        reference_generation: Some(context.project().configuration().reference_generation()),
        project_generation: Some(context.project().project_generation()),
        path,
        span,
        content_digest,
        entity_key: entity,
    }
}

fn plan_xml_sources<'a>(
    context: &'a RuleExecutionContext<'_>,
    admission: &mut Admission,
    plans: &mut Vec<SourcePlan<'a>>,
    unit: &'a XmlLuaUnitAnalysis,
    mapping: &XmlLuaMappedSpan,
    entity: Option<&'a EntityKey>,
) -> RuleResult<()> {
    let source = context
        .project()
        .xml_document_source(&unit.document)
        .map_err(|error| {
            RuleError::new(
                RuleErrorCode::RuleSourceHandleInvalid,
                format!("XML source artifact cannot be resolved: {error}"),
            )
            .with_rule(API_EXISTS_RULE)
            .with_scope(unit.unit_id.as_ref())
        })?
        .ok_or_else(|| {
            RuleError::new(
                RuleErrorCode::RuleSourceHandleInvalid,
                "XML source artifact is not present in the project generation",
            )
            .with_rule(API_EXISTS_RULE)
            .with_scope(unit.unit_id.as_ref())
        })?;
    validate_xml_mapping(unit, mapping, source.byte_length(), true)?;
    for (index, piece) in mapping.xml_spans.iter().enumerate() {
        let span =
            SourceSpan::byte_range(piece.byte_start, piece.byte_end).map_err(core_construction)?;
        let handle_entity = if index == 0 { entity } else { None };
        admit_source_plan(
            admission,
            plans,
            project_source_plan(
                context,
                &unit.document,
                source.content_digest(),
                span,
                handle_entity,
            ),
        )?;
    }
    if mapping.xml_spans.is_empty() {
        return Err(RuleError::new(
            RuleErrorCode::RuleSourceHandleInvalid,
            "XML source mapping did not produce a primary source handle",
        )
        .with_rule(API_EXISTS_RULE)
        .with_scope(unit.unit_id.as_ref()));
    }
    Ok(())
}

fn reference_origin(context: &RuleExecutionContext<'_>) -> String {
    if context.is_fixture_policy() {
        "reference-fixture:e0-e".to_owned()
    } else {
        format!("native-reference:{}", context.policy_profile_id())
    }
}

fn reference_source_plan<'a>(
    context: &'a RuleExecutionContext<'_>,
    origin_id: &'a str,
) -> RuleResult<SourcePlan<'a>> {
    Ok(SourcePlan {
        origin_kind: if context.is_fixture_policy() {
            SourceOriginKind::Fixture
        } else {
            SourceOriginKind::GeneratedArtifact
        },
        origin_id,
        revision: context.reference().generation_id(),
        reference_generation: Some(context.project().configuration().reference_generation()),
        project_generation: None,
        path: if context.is_fixture_policy() {
            "reference/fixture-view.json"
        } else {
            "reference/native-view.json"
        },
        span: SourceSpan::whole_file(),
        content_digest: context
            .reference()
            .self_digest()
            .parse()
            .map_err(core_construction)?,
        entity_key: None,
    })
}

type MaterializedSources = (
    SourceHandle,
    Vec<SourceHandle>,
    Vec<SourceHandle>,
    wow_core::StableHandleId,
);
fn materialize_sources(
    context: &RuleExecutionContext<'_>,
    plans: Vec<SourcePlan<'_>>,
) -> RuleResult<MaterializedSources> {
    let mut handles = Vec::with_capacity(plans.len());
    for plan in plans {
        observe(Construction::Handle);
        let handle = if plan.project_generation.is_some() {
            context
                .project()
                .source_handle(plan.path, plan.span, plan.entity_key.cloned())
                .map_err(|error| {
                    RuleError::new(
                        RuleErrorCode::RuleSourceHandleInvalid,
                        format!("project source handle construction failed: {error}"),
                    )
                })?
        } else {
            reference_view_handle(context)?
        };
        handles.push(handle);
    }
    let primary = handles.first().ok_or_else(budget_error)?.clone();
    let reference_id = handles.last().ok_or_else(budget_error)?.handle_id();
    let mut related = handles.iter().skip(1).cloned().collect::<Vec<_>>();
    canonicalize_sources(&mut related);
    canonicalize_sources(&mut handles);
    Ok((primary, related, handles, reference_id))
}

fn api_argument_plans<'a>(
    context: &'a RuleExecutionContext<'_>,
    entity: &'a str,
    receiver: &'a str,
    member: &'a str,
    call: bool,
) -> [ArgumentPlan<'a>; 7] {
    [
        ArgumentPlan::new(
            "authority",
            MessageArgumentKind::Text,
            "authoritative_absent",
        ),
        ArgumentPlan::new("entity", MessageArgumentKind::Identifier, entity),
        ArgumentPlan::new("member", MessageArgumentKind::Identifier, member),
        ArgumentPlan::new(
            "profile",
            MessageArgumentKind::Identifier,
            context
                .project()
                .configuration()
                .selected_profile()
                .profile_id()
                .as_str(),
        ),
        ArgumentPlan::new("receiver", MessageArgumentKind::Identifier, receiver),
        ArgumentPlan::new(
            "remediation_plan",
            MessageArgumentKind::Identifier,
            "verify-current-api-contract",
        ),
        ArgumentPlan::new(
            "use_kind",
            MessageArgumentKind::Text,
            if call {
                "direct_member_call"
            } else {
                "member_reference"
            },
        ),
    ]
}

fn materialize_arguments(plans: &[ArgumentPlan<'_>]) -> RuleResult<Vec<MessageArgument>> {
    plans
        .iter()
        .map(|plan| {
            MessageArgument::new(plan.name, plan.kind, plan.value, true).map_err(core_construction)
        })
        .collect()
}

fn reference_view_handle(context: &RuleExecutionContext<'_>) -> RuleResult<SourceHandle> {
    let digest = ContentDigest::<SourceContent>::from_str(context.reference().self_digest())
        .map_err(core_construction)?;
    let (origin_kind, origin_id, path) = if context.is_fixture_policy() {
        (
            SourceOriginKind::Fixture,
            "reference-fixture:e0-e".to_owned(),
            "reference/fixture-view.json",
        )
    } else {
        (
            SourceOriginKind::GeneratedArtifact,
            format!("native-reference:{}", context.policy_profile_id()),
            "reference/native-view.json",
        )
    };
    SourceHandleBuilder::new(
        origin_kind,
        origin_id,
        context.reference().generation_id(),
        path,
        SourceSpan::whole_file(),
        digest,
    )
    .reference_generation(context.project().configuration().reference_generation())
    .build()
    .map_err(|error| {
        RuleError::new(
            RuleErrorCode::RuleSourceHandleInvalid,
            format!("reference view handle construction failed: {error}"),
        )
    })
}

fn evidence_bundle(
    context: &RuleExecutionContext<'_>,
    source_handles: &[SourceHandle],
    reference_handle_id: wow_core::StableHandleId,
) -> RuleResult<Vec<EvidenceRecord>> {
    // All three records and their full output were admitted from borrowed plans.
    let context_id = context
        .project()
        .snapshot()
        .generation_context()
        .context_id();
    let project_ids = source_handles
        .iter()
        .filter(|handle| handle.handle_id() != reference_handle_id)
        .map(SourceHandle::handle_id)
        .collect::<Vec<_>>();
    observe(Construction::Evidence);
    let project = derive_evidence(
        context_id,
        ProvenanceClass::ProjectSource,
        EvidenceConfidence::Proven,
        ClaimScope::ProjectFact,
        producer_id()?,
        rule_version()?,
        project_ids,
        Vec::new(),
        Vec::new(),
    )
    .map_err(core_construction)?;
    observe(Construction::Evidence);
    let reference = derive_evidence(
        context_id,
        ProvenanceClass::PlatformSource,
        EvidenceConfidence::Proven,
        ClaimScope::PlatformContract,
        producer_id()?,
        rule_version()?,
        vec![reference_handle_id],
        Vec::new(),
        Vec::new(),
    )
    .map_err(core_construction)?;
    observe(Construction::Evidence);
    let derivation = derive_evidence(
        context_id,
        ProvenanceClass::ExternalImplementation,
        EvidenceConfidence::Derived,
        ClaimScope::ProjectFact,
        producer_id()?,
        rule_version()?,
        Vec::new(),
        Vec::new(),
        vec![project.evidence_id(), reference.evidence_id()],
    )
    .map_err(core_construction)?;
    let mut records = vec![project, reference, derivation];
    records.sort_by_key(EvidenceRecord::evidence_id);
    validate_evidence_derivation_graph(&records).map_err(core_construction)?;
    Ok(records)
}

fn canonicalize_sources(sources: &mut Vec<SourceHandle>) {
    sources.sort_by_key(SourceHandle::handle_id);
    sources.dedup_by_key(|handle| handle.handle_id());
}

fn group_uses<'a>(
    context: &RuleExecutionContext<'_>,
    uses: &'a [wow_emmy::EmmyLocalUseFact],
) -> RuleResult<BTreeMap<&'a str, Vec<&'a wow_emmy::EmmyLocalUseFact>>> {
    let mut grouped = BTreeMap::<&'a str, Vec<&'a wow_emmy::EmmyLocalUseFact>>::new();
    for use_fact in uses {
        ensure_not_cancelled(context, SECRET_LOCAL_RULE, "secret:use-index")?;
        grouped
            .entry(use_fact.operation_fact_id())
            .or_default()
            .push(use_fact);
    }
    Ok(grouped)
}

fn api_fact_ids(
    fact: &EmmyMemberReferenceFact,
    call: Option<&EmmyMemberCallFact>,
) -> Vec<Box<str>> {
    let mut ids = vec![fact.fact_id().into()];
    if let Some(call) = call {
        ids.push(call.fact_id().into());
    }
    ids
}

fn xml_api_fact_ids(
    fact: &XmlLuaMemberReference,
    call: Option<&XmlLuaMemberCall>,
) -> Vec<Box<str>> {
    let mut ids = vec![fact.fact_id.clone()];
    if let Some(call) = call {
        ids.push(call.fact_id.clone());
    }
    ids
}

const fn guard_name(value: RuleGuardClassification) -> &'static str {
    match value {
        RuleGuardClassification::Absent => "absent",
        RuleGuardClassification::DominatingExactValue => "dominating_exact_value",
        RuleGuardClassification::AfterUse => "after_use",
        RuleGuardClassification::DifferentValue => "different_value",
        RuleGuardClassification::NonDominating => "non_dominating",
    }
}

fn usize_to_u64(value: usize) -> RuleResult<u64> {
    u64::try_from(value).map_err(|_| budget_error())
}

fn producer_id() -> RuleResult<ProducerId> {
    PRODUCER_ID.parse().map_err(core_construction)
}

fn rule_version() -> RuleResult<ToolVersion> {
    PRODUCER_VERSION.parse().map_err(core_construction)
}

fn capability(value: &str) -> RuleResult<CapabilityId> {
    value.parse().map_err(core_construction)
}

fn budget_error() -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleExecutionBudgetExceeded,
        "rule execution exceeded its deterministic budget",
    )
}

fn registry_error(message: &str) -> RuleError {
    RuleError::new(RuleErrorCode::RuleRegistryInvalid, message)
}

fn fact_graph_error(message: &str) -> RuleError {
    RuleError::new(RuleErrorCode::RuleFactReferenceGraphInvalid, message)
        .with_rule(SECRET_LOCAL_RULE)
}

fn core_construction(error: wow_core::CoreError) -> RuleError {
    RuleError::new(
        RuleErrorCode::CoreConstructionFailed,
        format!("core record construction failed: {error}"),
    )
}

fn project_error(error: wow_project::ProjectError) -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleStaleInputForbidden,
        format!("project lookup failed: {error}"),
    )
}

fn reference_error(error: wow_reference::ReferenceViewError) -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleLookupOutcomeInvalid,
        format!("reference record validation failed: {error}"),
    )
}

#[cfg(test)]
mod budget_tests {
    use super::{
        CONSTRUCTIONS, CancellationHookGuard, Construction, ConstructionCounts,
        cancel_on_construction, cancellation_snapshot,
    };
    use crate as wow_rules;

    // Reuse the existing real native fixture, without adding source members.
    include!("../tests/e0_rules.rs");

    #[test]
    fn construction_boundaries_refuse_before_the_next_owner() -> TestResult {
        let project = project()?;
        let reference = reference(
            &project,
            ReferenceCoverageStatus::Complete,
            ReferenceCoverageStatus::Complete,
            true,
            false,
        )?;
        let cancelled = AtomicBool::new(false);
        let api_file = project
            .file_manifest()
            .iter()
            .find(|file| file.relative_path().as_str() == "main/missing-api.lua")
            .ok_or("API fixture file")?;
        let api_scope = RuleScope::files(vec![api_file.file_id().clone()]);
        let reset = || CONSTRUCTIONS.with(|cell| cell.set(ConstructionCounts::default()));
        let observed = || CONSTRUCTIONS.with(std::cell::Cell::get);

        reset();
        let first = run_with_budget(
            &project,
            &reference,
            &cancelled,
            RuleExecutionBudget::fixture_e0()?,
            &api_scope,
        )?;
        let first_counts = observed();
        assert_eq!(first_counts.evaluations, 1);
        assert_eq!(first_counts.lookups, 1);
        assert_eq!(
            first_counts.handles,
            usize::try_from(first.budget_usage().source_handles)?
        );
        assert_eq!(first_counts.evidence, 3);
        assert_eq!(first_counts.findings, 1);
        assert_eq!(first_counts.reports, 1);
        let first_bytes = wow_core::canonical_json_bytes(&first)?;
        let exact_bytes = u64::try_from(first_bytes.len())?;

        let all = run_with_budget(
            &project,
            &reference,
            &cancelled,
            RuleExecutionBudget::fixture_e0()?,
            &RuleScope::all(),
        )?;
        let (retained_handles, retained_evidence) =
            all.evaluations()
                .iter()
                .fold((0, 0), |(handles, evidence), evaluation| {
                    if let RuleEvaluationOutcome::Findings { result } = evaluation.outcome() {
                        (
                            handles + result.source_handles().len(),
                            evidence + result.evidence_records().len(),
                        )
                    } else {
                        (handles, evidence)
                    }
                });
        assert_eq!(
            all.budget_usage().source_handles,
            u64::try_from(retained_handles)?
        );
        assert_eq!(
            all.budget_usage().evidence_records,
            u64::try_from(retained_evidence)?
        );
        // Dedup happens on complete plans: no duplicate handle is constructed
        // and then discarded, including Secret's overlapping local-use spans.
        assert_eq!(observed().handles, first_counts.handles + retained_handles);
        let all_bytes = u64::try_from(wow_core::canonical_json_bytes(&all)?.len())?;
        reset();
        let limit_one = RuleExecutionBudget::new(1, 256, 4_096, 4_096, all_bytes)?;
        let error = run_with_budget(
            &project,
            &reference,
            &cancelled,
            limit_one,
            &RuleScope::all(),
        )
        .err()
        .ok_or("evaluation refusal")?;
        assert_eq!(
            error.code(),
            wow_rules::RuleErrorCode::RuleExecutionBudgetExceeded
        );
        // API evaluation one completes; Secret cannot create its next lookup,
        // handles, evidence, finding, evaluation or final report.
        assert_eq!(
            observed(),
            ConstructionCounts {
                reports: 0,
                ..first_counts
            }
        );

        for budget in [
            RuleExecutionBudget::new(
                1_024,
                256,
                first.budget_usage().source_handles - 1,
                4_096,
                exact_bytes,
            )?,
            RuleExecutionBudget::new(1_024, 256, 4_096, 2, exact_bytes)?,
        ] {
            reset();
            let error = run_with_budget(&project, &reference, &cancelled, budget, &api_scope)
                .err()
                .ok_or("retention refusal")?;
            assert_eq!(
                error.code(),
                wow_rules::RuleErrorCode::RuleExecutionBudgetExceeded
            );
            assert_eq!(
                observed(),
                ConstructionCounts {
                    lookups: 1,
                    ..ConstructionCounts::default()
                }
            );
        }

        reset();
        let short = RuleExecutionBudget::new(1_024, 256, 4_096, 4_096, exact_bytes - 1)?;
        let error = run_with_budget(&project, &reference, &cancelled, short, &api_scope)
            .err()
            .ok_or("byte refusal")?;
        assert_eq!(
            error.code(),
            wow_rules::RuleErrorCode::RuleOutputBudgetExceeded
        );
        // Full borrowed output preflight refuses before SourceHandleBuilder,
        // evidence derivation, root/fingerprint binding and report Core calls.
        assert_eq!(
            observed(),
            ConstructionCounts {
                lookups: 1,
                ..ConstructionCounts::default()
            }
        );

        reset();
        let tiny = RuleExecutionBudget::new(1_024, 256, 4_096, 4_096, 1)?;
        assert_eq!(
            run_with_budget(&project, &reference, &cancelled, tiny, &api_scope)
                .err()
                .map(|error| error.code()),
            Some(wow_rules::RuleErrorCode::RuleOutputBudgetExceeded)
        );
        assert_eq!(observed(), ConstructionCounts::default());

        reset();
        let exact = RuleExecutionBudget::new(1_024, 256, 4_096, 4_096, exact_bytes)?;
        let admitted = run_with_budget(&project, &reference, &cancelled, exact, &api_scope)?;
        assert_eq!(admitted, first);
        assert_eq!(wow_core::canonical_json_bytes(&admitted)?, first_bytes);
        assert_eq!(observed(), first_counts);
        Ok(())
    }

    #[test]
    fn mid_request_cancellation_stops_secret_after_first_operation() -> TestResult {
        let project = project()?;
        let reference = reference(
            &project,
            ReferenceCoverageStatus::Complete,
            ReferenceCoverageStatus::Complete,
            true,
            false,
        )?;
        let budget = RuleExecutionBudget::fixture_e0()?;
        let reset = || CONSTRUCTIONS.with(|cell| cell.set(ConstructionCounts::default()));
        let observed = || CONSTRUCTIONS.with(std::cell::Cell::get);

        let handoff_stop = std::sync::Arc::new(AtomicBool::new(false));
        reset();
        {
            let _hook: CancellationHookGuard = cancel_on_construction(
                Construction::ApiProviderComplete,
                1,
                std::sync::Arc::clone(&handoff_stop),
            );
            let error = run_with_budget(
                &project,
                &reference,
                handoff_stop.as_ref(),
                budget,
                &RuleScope::all(),
            )
            .err()
            .ok_or("API-to-Secret handoff cancellation")?;
            assert_eq!(error.code(), wow_rules::RuleErrorCode::RuleCancelled);
            assert!(handoff_stop.load(std::sync::atomic::Ordering::Relaxed));
            let counts = observed();
            assert_eq!(
                counts,
                cancellation_snapshot().ok_or("handoff stop snapshot")?
            );
            assert_eq!(counts.api_provider_completions, 1);
            assert_eq!(counts.lookups, 1);
            assert_eq!(counts.secret_operations, 0);
            assert_eq!(counts.reports, 0);
        }

        let secret_stop = std::sync::Arc::new(AtomicBool::new(false));
        let secret_budget = RuleExecutionBudget::fixture_e0()?;
        reset();
        {
            let _hook: CancellationHookGuard = cancel_on_construction(
                Construction::SecretOperationComplete,
                1,
                std::sync::Arc::clone(&secret_stop),
            );
            let error = run_with_budget(
                &project,
                &reference,
                secret_stop.as_ref(),
                secret_budget,
                &RuleScope::all(),
            )
            .err()
            .ok_or("Secret operation cancellation")?;
            assert_eq!(error.code(), wow_rules::RuleErrorCode::RuleCancelled);
            assert!(secret_stop.load(std::sync::atomic::Ordering::Relaxed));
            let counts = observed();
            // The hook snapshots the exact first-operation boundary when it
            // sets stop. Equality proves no later lookup or Core result
            // constructor ran, including a clean record or final report.
            assert_eq!(
                counts,
                cancellation_snapshot().ok_or("Secret stop snapshot")?
            );
            assert_eq!(counts.api_provider_completions, 1);
            assert_eq!(counts.lookups, 2);
            assert_eq!(counts.secret_operations, 1);
            assert_eq!(counts.reports, 0);
        }
        Ok(())
    }
}
