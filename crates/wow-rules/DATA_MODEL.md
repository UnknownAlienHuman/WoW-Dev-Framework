# `wow-rules` E0-E data model

**Status:** normative semantic model for the two-rule E0 slice.

Concrete Rust layout may differ. Type ownership, evidence separation, capability behavior, and identity invariants may not.

## 1. Object graph

```text
RuleRegistry
└── RuleDescriptor[]

RuleExecutionRequest
├── RuleId/version selection
├── RuleExecutionContext
│   ├── GenerationContext
│   ├── ReferenceView
│   ├── ProjectView
│   ├── AnalyzerSnapshot/fact-set identities
│   ├── project/reference/core coverage and conflicts
│   ├── RuleFixturePolicy (E0 only)
│   ├── BudgetSpec/Usage
│   └── cancellation
└── RuleScope

validated request/context
    -> capability evaluation
    -> provider-specific input selection/join
    -> RuleEvaluationOutcome
       ├── Findings
       ├── EvaluatedClean
       ├── NotEvaluated
       ├── Failed
       └── Cancelled
```

## 2. Rule descriptor

```text
RuleDescriptor
    rule_id: RuleId
    rule_version: u32
    semantic_category: String
    technical_severity: Severity
    rollout_policy: shadow | advisory | blocking
    profile_applicability
    source_scope
    required_capabilities: RuleCapabilityRequirement[]
    required_input_kinds[]
    execution_budget
    output_budget
    remediation_tiers[]
    root_cause_policy
    fixture_case_set_ids[]
    implementation_milestone
```

E0 descriptors:

```text
wow.api.exists@1
wow.secret.local_operation@1
```

### Invariants

- stable unique RuleId/version pair;
- no duplicate/incompatible active descriptor;
- only E0 rules active;
- rollout explicit and independent of severity;
- required capabilities precise enough to block unavailable cases;
- fixture/evaluation sets present;
- no hidden IO/dependency declaration.

## 3. Capability requirement

```text
RuleCapabilityRequirement
    capability_id
    partition_selector
    accepted_statuses
    conflict_policy
    truncation_policy
    source_owner
    required_for: evaluation | clean | specific_finding
```

E0 accepted status is generally `Complete`. A positive project fact may exist under broader Partial coverage, but a rule may require exact fact/flow capability Complete for the selected use scope.

## 4. Rule execution context

```text
RuleExecutionContext
    context_id
    generation_context
    profile_identity
    reference_view_identity
    project_snapshot_identity
    project_view
    analyzer_snapshot_identity
    active_rule_registry_identity
    rule_fixture_policy_identity
    coverage_record_ids[]
    conflict_record_ids[]
    budget_spec
    cancellation_state
```

The context references immutable views/records. It contains no mutable actor/session handle.

### Context invariants

- one profile/reference/project generation;
- analyzer facts/findings/source handles match project snapshot;
- reference view matches profile/reference generation;
- coverage/conflict IDs resolve in context;
- fixture policy matches fixture profile and rule version;
- budget valid;
- no stale/foreign source/evidence.

## 5. Rule scope

```text
RuleScope
    file_ids[]
    source_span_filter: optional
    fact_ids[]: optional
    entity_keys[]: optional
```

E0 default scopes are selected direct fixture facts/use sites, not an unbounded repository scan.

The rule reports the exact examined scope in every outcome.

## 6. Rule fixture policy

```text
RuleFixturePolicy
    policy_id
    profile_id
    reference_generation
    rule_versions
    accepted_guard_semantics[]
    supported_operation_kinds[]
    supported_producer_facets[]
    scope_limits
```

E0 policy:

```text
policy_id: wow-rules-e0-fixture-policy/1
accepted guard: access_single for canaccessvalue structural fact
supported operation: concatenation
supported facet: secret.return / return_position 1 / unconditional_fixture
scope: one function-local flow slice
```

This policy is fixture-only, not release platform authority.

## 7. Rule evaluation status

```text
RuleEvaluationStatus
    Findings
    EvaluatedClean
    NotEvaluated
    Failed
    Cancelled
```

## 8. Clean evaluation record

```text
CleanEvaluationRecord
    evaluation_id
    rule_id/version
    generation_context
    examined_scope
    input_fact_ids[]
    reference_lookup_ids[]
    capability_coverage_ids[]
    conflict_ids[]
    producer/version
    budget_usage
    clean_claim_kind
    canonical_digest
```

`clean_claim_kind` is rule-specific and narrow:

```text
api_exists_for_exact_use
secret_operation_has_accepted_dominating_guard_for_fixture_flow
ordinary_or_nonmatching_operation_outside-finding condition (only when fully evaluated)
```

A clean record is not a general safety certificate.

## 9. NotEvaluated record

Use `wow-core` `NotEvaluatedRecord` semantics plus rule-owned structured details:

```text
RuleNotEvaluatedDetail
    rule_id/version
    generation_context
    attempted_scope
    missing_capability_ids[]
    blocking_coverage_ids[]
    conflict_ids[]
    stale_or_mismatch_reasons[]
    unsupported_fact_or_semantics[]
    budget_or_truncation_blockers[]
    recommended_next_evidence (structured, non-editing)
```

No finding or clean outcome accompanies the same scope/rule evaluation.

## 10. Rule failure

```text
RuleFailure
    rule_id/version
    context/evaluation ID
    error code
    source/fact/query IDs when safe
    capability/partition IDs
    retry class
```

`Failed` means rule implementation/contract execution failed despite apparently available prerequisites. It is distinct from unavailable evidence (`NotEvaluated`).

## 11. Finding input bundle

```text
RuleFindingInput
    rule_id/version
    generation_context
    primary project source handle
    project evidence IDs[]
    reference evidence or authority input IDs[]
    rule derivation input IDs[]
    structured message arguments
    technical severity
    rollout policy
    root_cause_key
    remediation classification
    capability/coverage IDs[]
    conflict IDs[]
```

The bundle is validated before constructing the `wow-core Finding`.

## 12. Root-cause key

```text
RuleRootCauseKey
    rule_id/version
    generation_context ID
    primary project source handle/span identity
    canonical subject/entity key
    decisive project fact IDs[]
    decisive reference lookup/facet/authority IDs[]
    rule-specific operation/slot kind
```

Rendered message text is excluded.

## 13. Causal relation hint

```text
CausalRelationHint
    cause_finding_or_root_key
    symptom_finding_id
    relation_kind
    evidence_fact_ids[]
    confidence
```

E0 API example:

```text
wow.api.exists finding
    causes_or_explains
same-source generic unresolved-member finding
```

Only emit when exact source/fact IDs establish the relation. Service decides final folding.

## 14. Remediation classification

```text
Remediation
    tier: exact_edit | validated_recipe | plan_only | candidate_only
    code
    structured_steps[]
    required_post_checks[]
    required_runtime_scenarios[]
    edit: optional (none in E0)
```

E0 permits only `plan_only`.

## 15. Provider execution report

```text
RuleExecutionReport
    registry identity
    context ID
    selected rule IDs/versions
    skipped nonapplicable rules[]
    outcomes[]
    examined files/facts/queries
    capability checks[]
    budget usage
    canonical order/digest
```

This report is consumed by service orchestration. It does not constitute the final result envelope.

## 16. API rule input model

```text
ApiExistsInput
    project_file_id
    project_source_handle
    unresolved_member_reference_fact_id
    call_fact_id: optional
    exact_entity_key
    exact_reference_lookup_result
    reference_coverage_ids[]
    reference_conflict_ids[]
    same-source generic finding IDs[]
```

### Valid decisive states

```text
reference found
reference authoritative_absent
reference absent_without_authority
reference conflict
```

## 17. Secret-local input model

```text
SecretLocalInput
    project_file_id
    containing_function_key
    producer_reference_fact_id
    producer_call_fact_id
    producer_entity_key
    producer_return_position
    local_binding/value key
    local_use_fact_id
    operation_fact_id
    operation_kind
    guard_fact_ids[]
    control_flow_relation_ids[]
    exact_restriction_facet_lookup
    rule_fixture_policy
    project/reference coverage IDs[]
    conflicts[]
```

## 18. Rule evaluation IDs

```text
RuleEvaluationId
    derived from:
        rule ID/version
        generation context ID
        examined scope
        decisive fact/query/facet/coverage/conflict IDs
        fixture policy ID
        rule execution schema version
```

Excludes timestamp, discovery order, rendered messages, memory addresses, and worker scheduling.

## 19. API finding model

```text
ApiExistsFindingArguments
    exact_entity_key
    reference_profile_id
    reference_generation
    use_kind
    member_spelling
    authority_decision
```

Primary source: unresolved member/reference project span.

Related evidence:

- project member/call fact/source;
- reference query/coverage/authority decision;
- optional generic symptom fact/finding relation.

No absent reference source handle or replacement.

## 20. Secret finding model

```text
SecretLocalFindingArguments
    producer_entity_key
    producer_return_position
    restriction_facet_kind
    operation_kind
    guard_state: absent | after_use | different_value | non_dominating
    scope_kind: function_local
```

Primary source: exact operation span.

Related evidence:

- producer call/binding/use/operation source facts;
- facet raw/source/evidence/coverage;
- guard/control-flow facts when present;
- fixture rule policy.

No runtime/global safety claim.

## 21. Capability coverage ownership

Rules consume existing core/reference/project/analyzer coverage. They may emit a rule-execution coverage/evaluation record describing examined scope and result.

They do not rewrite upstream coverage statuses.

## 22. Canonical ordering

Within one execution report:

1. rule ID/version;
2. outcome status order defined by contract;
3. primary project source handle/path/span;
4. canonical subject/entity key;
5. operation/finding category;
6. evaluation/finding ID.

Input fact/coverage/conflict order is canonicalized before identity/output.

## 23. Source/provenance limits

`wow-rules` may construct derived evidence/finding records only from validated inputs. It cannot introduce new platform/project source observations.

Project and reference source handles remain separately typed/originated.

## 24. Fixture IDs

Closed E0 case sets:

```text
wow-rules-e0-registry-v1
wow-rules-e0-api-exists-v1
wow-rules-e0-secret-local-v1
wow-rules-e0-execution-v1
```

Exact IDs/fingerprints/checksums freeze before implementation.
