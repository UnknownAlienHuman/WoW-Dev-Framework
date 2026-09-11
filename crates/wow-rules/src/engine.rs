use std::collections::BTreeMap;
use std::str::FromStr;

use wow_core::{
    BlockingPartitionRef, CapabilityId, ClaimScope, ContentDigest, CoverageId, CoverageStatus,
    EvidenceConfidence, EvidenceRecord, FindingDraft, MessageArgument, MessageArgumentKind,
    MessageCode, NotEvaluatedRecord, ProducerId, ProvenanceClass, Remediation, RemediationClass,
    RolloutPolicy, Severity, SourceContent, SourceHandle, SourceHandleBuilder, SourceOriginKind,
    SourceSpan, ToolVersion, canonical_json_bytes, derive_evidence, derive_root_cause_key,
    validate_evidence_derivation_graph,
};
use wow_emmy::{
    EmmyControlFlowRelationKind, EmmyGuardFact, EmmyGuardKind, EmmyLocalBindingFact,
    EmmyMemberCallFact, EmmyMemberReferenceFact, EmmyOperationFact, EmmyOperationKind,
    EmmyReferenceResolution,
};
use wow_project::{
    ProjectAnalyzerCapabilityScope, ProjectAnalyzerCapabilityState, ProjectFileId,
    ProjectFileRecord,
};
use wow_reference::{
    LookupResult, LookupUnknownReason, ReferenceRecord, ReferenceRecordKind, RestrictionState,
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

const API_PARTITION: &str = "reference.fixture.apidoc.system:C_E0Fixture";
const SECRET_PARTITION: &str = "reference.fixture.restriction:C_E0Fixture.SecretText";
const SECRET_ENTITY: &str = "function:C_E0Fixture.SecretText";
const SECRET_PAYLOAD: &str = "return_position:1;applicability:unconditional_fixture";
const PRODUCER_ID: &str = "wow.rules";
const PRODUCER_VERSION: &str = "1.0.0";

struct ApiFindingInput<'a> {
    file: &'a ProjectFileRecord,
    fact: &'a EmmyMemberReferenceFact,
    call: Option<&'a EmmyMemberCallFact>,
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

/// Executes the closed E0-E registry synchronously over one immutable context.
pub fn execute_e0(
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

    let mut evaluations = evaluate_api(context, scope, api)?;
    evaluations.extend(evaluate_secret(context, scope, secret)?);
    let usage = summarize_usage(&evaluations)?;
    enforce_budget(context, usage)?;
    let report = RuleExecutionReport::build(
        context.registry().registry_id(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        context.project().snapshot_id(),
        context.project().analyzer_snapshot_id(),
        context.reference().generation_id(),
        context.reference().self_digest(),
        context.fixture_policy().policy_id(),
        evaluations,
        usage,
    )?;
    if u64::try_from(
        canonical_json_bytes(&report)
            .map_err(core_construction)?
            .len(),
    )
    .unwrap_or(u64::MAX)
        > context.budget().max_serialized_output_bytes()
    {
        return Err(RuleError::new(
            RuleErrorCode::RuleOutputBudgetExceeded,
            "serialized rule report exceeds the configured output budget",
        ));
    }
    Ok(report)
}

fn cancelled_report(context: &RuleExecutionContext<'_>) -> RuleResult<RuleExecutionReport> {
    let version = rule_version()?;
    let evaluations = [API_EXISTS_RULE, SECRET_LOCAL_RULE]
        .into_iter()
        .map(|rule| {
            Ok(RuleEvaluationRecord::new(
                rule.parse().map_err(core_construction)?,
                version.clone(),
                "cancelled:preflight",
                RuleEvaluationOutcome::Cancelled,
            ))
        })
        .collect::<RuleResult<Vec<_>>>()?;
    RuleExecutionReport::build(
        context.registry().registry_id(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        context.project().snapshot_id(),
        context.project().analyzer_snapshot_id(),
        context.reference().generation_id(),
        context.reference().self_digest(),
        context.fixture_policy().policy_id(),
        evaluations,
        RuleBudgetUsage {
            evaluations: 2,
            ..RuleBudgetUsage::default()
        },
    )
}

fn evaluate_api(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
    descriptor: &crate::RuleDescriptor,
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
                output.push(RuleEvaluationRecord::new(
                    descriptor.rule_id().clone(),
                    descriptor.rule_version().clone(),
                    &format!("api:{}:cancelled", file.file_id()),
                    RuleEvaluationOutcome::Cancelled,
                ));
                return Ok(output);
            }
            if fact.receiver() != "C_E0Fixture"
                || fact.resolution() == EmmyReferenceResolution::Resolved
            {
                continue;
            }
            let entity_lookup_key = format!("function:{}.{}", fact.receiver(), fact.member());
            let scope_id = format!("api:{}:{}", file.file_id(), fact.fact_id());
            let call = calls.get(fact.fact_id()).copied();
            let fact_ids = api_fact_ids(fact, call);
            if fact.resolution() == EmmyReferenceResolution::Possible {
                output.push(not_evaluated(
                    context,
                    descriptor,
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
                    &scope_id,
                    gate.blockers,
                    gate.blocking_capabilities,
                    gate.blocking_partitions,
                    fact_ids,
                    None,
                )?);
                continue;
            }
            let (lookup, found) = exact_lookup(context, API_PARTITION, &entity_lookup_key)?;
            match lookup.outcome() {
                RuleReferenceOutcome::Found => {
                    output.push(clean(
                        context,
                        descriptor,
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
    Ok(output)
}

fn evaluate_secret(
    context: &RuleExecutionContext<'_>,
    scope: &RuleScope,
    descriptor: &crate::RuleDescriptor,
) -> RuleResult<Vec<RuleEvaluationRecord>> {
    let mut output = Vec::new();
    let references = context
        .project()
        .member_call_report()
        .references()
        .iter()
        .map(|fact| (fact.fact_id(), fact))
        .collect::<BTreeMap<_, _>>();
    let calls = context
        .project()
        .member_call_report()
        .calls()
        .iter()
        .map(|fact| (fact.fact_id(), fact))
        .collect::<BTreeMap<_, _>>();
    let flow = context.project().local_flow_report();
    let all_bindings = flow
        .bindings()
        .iter()
        .map(|binding| (binding.fact_id(), binding))
        .collect::<BTreeMap<_, _>>();
    let uses_by_operation = group_uses(flow.uses());

    for binding in flow.bindings() {
        if binding.initializer_receiver() != Some("C_E0Fixture")
            || binding.initializer_member() != Some("SecretText")
        {
            continue;
        }
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
            || reference.receiver() != "C_E0Fixture"
            || reference.member() != "SecretText"
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
        let operations = flow
            .operations()
            .iter()
            .filter(|operation| {
                operation.path() == binding.path()
                    && operation
                        .operand_binding_fact_ids()
                        .any(|id| id == binding.fact_id())
            })
            .collect::<Vec<_>>();
        if operations.is_empty() {
            continue;
        }
        for operation in operations {
            let scope_id = format!("secret:{}:{}", file.file_id(), operation.fact_id());
            let gate = capability_gate(context, file, descriptor.required_capabilities())?;
            let mut fact_ids = vec![
                binding.fact_id().into(),
                reference.fact_id().into(),
                call.fact_id().into(),
                operation.fact_id().into(),
            ];
            let uses = uses_by_operation
                .get(operation.fact_id())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|fact| fact.binding_fact_id() == binding.fact_id())
                .collect::<Vec<_>>();
            fact_ids.extend(uses.iter().map(|fact| fact.fact_id().into()));
            if !gate.blockers.is_empty() {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    &scope_id,
                    gate.blockers,
                    gate.blocking_capabilities,
                    gate.blocking_partitions,
                    fact_ids,
                    None,
                )?);
                continue;
            }
            if operation.kind() != EmmyOperationKind::Concatenation {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    &scope_id,
                    vec![RuleBlockerKind::UnsupportedFactShape],
                    vec![capability("emmy.fact.operations")?],
                    Vec::new(),
                    fact_ids,
                    None,
                )?);
                continue;
            }
            if uses.is_empty() {
                output.push(not_evaluated(
                    context,
                    descriptor,
                    &scope_id,
                    vec![RuleBlockerKind::MissingFactRelation],
                    vec![capability("emmy.fact.local_flow")?],
                    Vec::new(),
                    fact_ids,
                    None,
                )?);
                continue;
            }
            let (lookup, found) = exact_lookup(context, SECRET_PARTITION, SECRET_ENTITY)?;
            match lookup.outcome() {
                RuleReferenceOutcome::Found => {
                    let record = found.ok_or_else(|| {
                        RuleError::new(
                            RuleErrorCode::RuleLookupOutcomeInvalid,
                            "found Secret lookup omitted its record",
                        )
                    })?;
                    match classify_secret_record(record)? {
                        SecretFacetDecision::NoRestriction => {
                            output.push(clean(
                                context,
                                descriptor,
                                CleanInput {
                                    scope_id: &scope_id,
                                    claim: RuleCleanClaimKind::SecretProducerHasNoMatchingFacet,
                                    fact_ids,
                                    lookup: &lookup,
                                    coverage_ids: gate.coverage_ids,
                                    guard: None,
                                },
                            )?);
                            continue;
                        }
                        SecretFacetDecision::Unsupported => {
                            output.push(not_evaluated(
                                context,
                                descriptor,
                                &scope_id,
                                vec![RuleBlockerKind::MissingRestrictionFacet],
                                vec![capability("reference.restriction.facets")?],
                                Vec::new(),
                                fact_ids,
                                Some(&lookup),
                            )?);
                            continue;
                        }
                        SecretFacetDecision::Restricted => {}
                    }
                    let guard = classify_guard(
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
                            CleanInput {
                                scope_id: &scope_id,
                                claim: RuleCleanClaimKind::SecretFixtureOperationGuardedForExactValueAndScope,
                                fact_ids,
                                lookup: &lookup,
                                coverage_ids: gate.coverage_ids,
                                guard: Some(guard),
                            },
                        )?);
                    } else {
                        output.push(secret_finding(
                            context,
                            descriptor,
                            file,
                            binding,
                            reference,
                            call,
                            operation,
                            fact_ids,
                            &uses,
                            flow.guards(),
                            guard,
                            lookup,
                            gate.coverage_ids,
                        )?);
                    }
                }
                RuleReferenceOutcome::AuthoritativeAbsent => {
                    output.push(clean(
                        context,
                        descriptor,
                        CleanInput {
                            scope_id: &scope_id,
                            claim: RuleCleanClaimKind::SecretProducerHasNoMatchingFacet,
                            fact_ids,
                            lookup: &lookup,
                            coverage_ids: gate.coverage_ids,
                            guard: None,
                        },
                    )?);
                }
                RuleReferenceOutcome::Conflict => {
                    output.push(not_evaluated(
                        context,
                        descriptor,
                        &scope_id,
                        vec![RuleBlockerKind::ReferenceConflict],
                        vec![capability("reference.restriction.facets")?],
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
                        &scope_id,
                        reference_blockers(lookup.outcome()),
                        vec![capability("reference.restriction.facets")?],
                        Vec::new(),
                        fact_ids,
                        Some(&lookup),
                    )?);
                }
            }
        }
    }
    Ok(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecretFacetDecision {
    Restricted,
    NoRestriction,
    Unsupported,
}

fn classify_secret_record(record: &ReferenceRecord) -> RuleResult<SecretFacetDecision> {
    if record.kind() != ReferenceRecordKind::Restriction {
        return Err(RuleError::new(
            RuleErrorCode::RuleLookupOutcomeInvalid,
            "Secret exact lookup returned a non-restriction record",
        )
        .with_rule(SECRET_LOCAL_RULE));
    }
    if record.payload() != SECRET_PAYLOAD {
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

fn classify_guard(
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
    let input_fact_ids = api_fact_ids(fact, call);
    let entity = wow_core::EntityKey::new(
        "function",
        &format!("{}.{}", fact.receiver(), fact.member()),
    )
    .map_err(core_construction)?;
    let primary = project_handle(context, file, fact.member_span(), Some(entity.clone()))?;
    let mut related = Vec::new();
    if let Some(call) = call {
        related.push(project_handle(context, file, call.call_span(), None)?);
    }
    let reference_handle = reference_view_handle(context)?;
    related.push(reference_handle.clone());
    let mut source_handles = vec![primary.clone()];
    source_handles.extend(related.iter().cloned());
    canonicalize_sources(&mut source_handles);
    let evidence = evidence_bundle(context, &source_handles, reference_handle.handle_id())?;
    let evidence_ids = evidence
        .iter()
        .map(EvidenceRecord::evidence_id)
        .collect::<Vec<_>>();
    let root = derive_root_cause_key(&(
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
    ))
    .map_err(core_construction)?;
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
    .message_arguments(vec![
        argument("authority", "authoritative_absent")?,
        identifier_argument("entity", entity_lookup_key)?,
        identifier_argument("member", fact.member())?,
        identifier_argument(
            "profile",
            context
                .project()
                .configuration()
                .selected_profile()
                .profile_id()
                .as_str(),
        )?,
        identifier_argument("receiver", fact.receiver())?,
        identifier_argument("remediation_plan", "verify-current-api-contract")?,
        argument(
            "use_kind",
            if call.is_some() {
                "direct_member_call"
            } else {
                "member_reference"
            },
        )?,
    ])
    .map_err(core_construction)?
    .root_causes(Some(root), None)
    .remediation(
        Remediation::new(RemediationClass::PlanOnly, None, None).map_err(core_construction)?,
    );
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
    let scope_id = format!("api:{}:{}", file.file_id(), fact.fact_id());
    let usage = RuleBudgetUsage {
        evaluations: 1,
        findings: 1,
        source_handles: usize_to_u64(source_handles.len())?,
        evidence_records: usize_to_u64(evidence.len())?,
    };
    let result = RuleFindingSet::build(
        context.fixture_policy().policy_id(),
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
    file: &ProjectFileRecord,
    binding: &EmmyLocalBindingFact,
    reference: &EmmyMemberReferenceFact,
    call: &EmmyMemberCallFact,
    operation: &EmmyOperationFact,
    input_fact_ids: Vec<Box<str>>,
    uses: &[&wow_emmy::EmmyLocalUseFact],
    guards: &[EmmyGuardFact],
    guard: RuleGuardClassification,
    lookup: RuleReferenceLookupRecord,
    coverage_ids: Vec<CoverageId>,
) -> RuleResult<RuleEvaluationRecord> {
    let entity = wow_core::EntityKey::new("function", "C_E0Fixture.SecretText")
        .map_err(core_construction)?;
    let primary = project_handle(
        context,
        file,
        operation.operation_span(),
        Some(entity.clone()),
    )?;
    let mut related = vec![
        project_handle(context, file, reference.reference_span(), None)?,
        project_handle(context, file, call.call_span(), None)?,
        project_handle(context, file, binding.declaration_span(), None)?,
    ];
    for use_fact in uses {
        related.push(project_handle(context, file, use_fact.use_span(), None)?);
    }
    if let Some(guard_fact) = representative_guard(guards, binding, operation, guard) {
        related.push(project_handle(
            context,
            file,
            guard_fact.guard_span(),
            None,
        )?);
    }
    let reference_handle = reference_view_handle(context)?;
    related.push(reference_handle.clone());
    let mut source_handles = vec![primary.clone()];
    source_handles.extend(related.iter().cloned());
    canonicalize_sources(&mut source_handles);
    let evidence = evidence_bundle(context, &source_handles, reference_handle.handle_id())?;
    let evidence_ids = evidence
        .iter()
        .map(EvidenceRecord::evidence_id)
        .collect::<Vec<_>>();
    let root = derive_root_cause_key(&(
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
        context.fixture_policy().policy_id(),
        context.fixture_policy().policy_digest(),
        guard,
        primary.handle_id(),
    ))
    .map_err(core_construction)?;
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
    .message_arguments(vec![
        identifier_argument("facet", "secret.return")?,
        identifier_argument("fixture_policy", context.fixture_policy().policy_id())?,
        argument("guard_state", guard_name(guard))?,
        identifier_argument("operation", "concatenation")?,
        identifier_argument("producer", SECRET_ENTITY)?,
        identifier_argument("remediation_plan", "restructure-secret-capable-value-use")?,
        MessageArgument::new("return_position", MessageArgumentKind::Integer, "1", true)
            .map_err(core_construction)?,
    ])
    .map_err(core_construction)?
    .root_causes(Some(root), None)
    .remediation(
        Remediation::new(RemediationClass::PlanOnly, None, None).map_err(core_construction)?,
    );
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
    let scope_id = format!("secret:{}:{}", file.file_id(), operation.fact_id());
    let usage = RuleBudgetUsage {
        evaluations: 1,
        findings: 1,
        source_handles: usize_to_u64(source_handles.len())?,
        evidence_records: usize_to_u64(evidence.len())?,
    };
    let result = RuleFindingSet::build(
        context.fixture_policy().policy_id(),
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
    input: CleanInput<'_>,
) -> RuleResult<RuleEvaluationRecord> {
    let CleanInput {
        scope_id,
        claim,
        fact_ids,
        lookup,
        coverage_ids,
        guard,
    } = input;
    let record = CleanEvaluationRecord::build(
        descriptor.rule_id().clone(),
        descriptor.rule_version().clone(),
        context
            .project()
            .snapshot()
            .generation_context()
            .context_id(),
        context.fixture_policy().policy_id(),
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
    scope_id: &str,
    blockers: Vec<RuleBlockerKind>,
    mut blocking_capabilities: Vec<CapabilityId>,
    blocking_partitions: Vec<BlockingPartitionRef>,
    fact_ids: Vec<Box<str>>,
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
    file_id: &ProjectFileId,
    binding: &EmmyLocalBindingFact,
    relation: &str,
) -> RuleResult<RuleEvaluationRecord> {
    let scope_id = format!("secret:{file_id}:{}:{relation}", binding.fact_id());
    not_evaluated(
        context,
        descriptor,
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
    gate.blockers.sort();
    gate.blockers.dedup();
    gate.blocking_capabilities.sort();
    gate.blocking_capabilities.dedup();
    gate.blocking_partitions.sort();
    gate.blocking_partitions.dedup();
    gate.coverage_ids.sort_unstable();
    gate.coverage_ids.dedup();
    Ok(gate)
}

fn exact_lookup<'a>(
    context: &'a RuleExecutionContext<'_>,
    partition: &str,
    key: &str,
) -> RuleResult<(RuleReferenceLookupRecord, Option<&'a ReferenceRecord>)> {
    let result = context.reference().lookup(partition, key);
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

fn project_handle(
    context: &RuleExecutionContext<'_>,
    file: &ProjectFileRecord,
    span: SourceSpan,
    entity: Option<wow_core::EntityKey>,
) -> RuleResult<SourceHandle> {
    let base = file.source_handle_base();
    let mut builder = SourceHandleBuilder::new(
        base.origin_kind(),
        base.origin_id(),
        base.revision(),
        base.path().as_str(),
        span,
        *base.content_digest(),
    )
    .project_generation(context.project().project_generation());
    if let Some(entity) = entity {
        builder = builder.entity_key(entity);
    }
    builder.build().map_err(|error| {
        RuleError::new(
            RuleErrorCode::RuleSourceHandleInvalid,
            format!("project source handle construction failed: {error}"),
        )
    })
}

fn reference_view_handle(context: &RuleExecutionContext<'_>) -> RuleResult<SourceHandle> {
    let digest = ContentDigest::<SourceContent>::from_str(context.reference().self_digest())
        .map_err(core_construction)?;
    SourceHandleBuilder::new(
        SourceOriginKind::Fixture,
        "reference-fixture:e0-e",
        context.reference().generation_id(),
        "reference/fixture-view.json",
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
    uses: &'a [wow_emmy::EmmyLocalUseFact],
) -> BTreeMap<&'a str, Vec<&'a wow_emmy::EmmyLocalUseFact>> {
    let mut grouped = BTreeMap::<&'a str, Vec<&'a wow_emmy::EmmyLocalUseFact>>::new();
    for use_fact in uses {
        grouped
            .entry(use_fact.operation_fact_id())
            .or_default()
            .push(use_fact);
    }
    grouped
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

fn argument(name: &str, value: &str) -> RuleResult<MessageArgument> {
    MessageArgument::new(name, MessageArgumentKind::Text, value, true).map_err(core_construction)
}

fn identifier_argument(name: &str, value: &str) -> RuleResult<MessageArgument> {
    MessageArgument::new(name, MessageArgumentKind::Identifier, value, true)
        .map_err(core_construction)
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

fn summarize_usage(evaluations: &[RuleEvaluationRecord]) -> RuleResult<RuleBudgetUsage> {
    let mut usage = RuleBudgetUsage {
        evaluations: usize_to_u64(evaluations.len())?,
        ..RuleBudgetUsage::default()
    };
    for evaluation in evaluations {
        if let RuleEvaluationOutcome::Findings { result } = evaluation.outcome() {
            usage.findings = usage
                .findings
                .checked_add(usize_to_u64(result.findings().len())?)
                .ok_or_else(budget_error)?;
            usage.source_handles = usage
                .source_handles
                .checked_add(usize_to_u64(result.source_handles().len())?)
                .ok_or_else(budget_error)?;
            usage.evidence_records = usage
                .evidence_records
                .checked_add(usize_to_u64(result.evidence_records().len())?)
                .ok_or_else(budget_error)?;
        }
    }
    Ok(usage)
}

fn enforce_budget(context: &RuleExecutionContext<'_>, usage: RuleBudgetUsage) -> RuleResult<()> {
    let budget = context.budget();
    if usage.evaluations > budget.max_evaluations()
        || usage.findings > budget.max_findings()
        || usage.source_handles > budget.max_source_handles()
        || usage.evidence_records > budget.max_evidence_records()
    {
        Err(budget_error())
    } else {
        Ok(())
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
