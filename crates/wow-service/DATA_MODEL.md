# `wow-service` E0-F data model

**Status:** normative semantic model for `status` and `check` orchestration.

Concrete Rust layout may differ. Ownership, coherence, outcome, folding, and canonicalization semantics may not.

## 1. Object graph

```text
ServiceConfiguration
├── ServiceComponentRegistry
├── ProjectSelectionPolicy
├── OperationBudgetPolicy
├── DeferredOperationRegistry
└── OutputContract

StatusRequest
    -> ServiceStatusResult

CheckRequest
    -> acquire ServiceContextLease
       ├── ReferenceView
       ├── ProjectSnapshot / ProjectView
       ├── AnalyzerSnapshot identity
       ├── RuleRegistry
       └── capability/coverage/conflict registries
    -> GenericFindingSet
    -> RuleExecutionReport
    -> RawCheckData
    -> FindingPresentationGraph
    -> ServiceSemanticStatus
    -> CheckResultEnvelope
```

## 2. Service configuration

```text
ServiceConfiguration
    service_id
    service_schema_version
    tool_versions
    configured_project_id
    configured_profile_identity
    configured_reference_generation
    reference_provider_identity
    project_provider_identity
    analyzer_contract/pin/probe/config identity
    rule_registry_identity
    project_generation_selection_policy
    operation_budget_policy
    cancellation_policy
    output_contract_version
    deferred_operation_registry
```

### Invariants

- exact non-floating identities after implementation freeze;
- configured project/profile/reference/analyzer/rules agree;
- no later component activation;
- budgets/output schema explicit;
- no local path/token/editor/client implicit state;
- deterministic canonical configuration digest.

## 3. Component registry

```text
ServiceComponentRegistry
    registry_id
    component_records[]
    canonical_digest
```

```text
ServiceComponentRecord
    component_id
    crate_or_application
    contract_id
    implementation_identity
    health_state
    selected_snapshot_or_generation_ids[]
    capability_summary_ids[]
    last_known_good_ids[]
    failed_target_ids[]
    deferred_features[]
```

Health state:

```text
Ready
Degraded
Failed
Unavailable
Deferred
```

`Ready` means the component can serve its declared operation contract. It does not mean diagnostics/tests/runtime passed.

## 4. Project generation selector

```text
ProjectGenerationSelector
    Exact(ProjectGenerationId)
    CurrentPublished(ProjectId)
```

Canonical result never uses `CurrentPublished` as generation identity. Selection metadata may record the selector, while `GenerationContext` records the resolved exact generation.

## 5. Status request

```text
StatusRequest
    service_id
    project_id: optional (uses configured exact project when omitted by explicit E0 policy)
    detail_level: summary | capabilities
    output_budget
```

Status has no source scope and runs no diagnostics/rules.

## 6. Status result

```text
ServiceStatusResult
    result_id
    service_configuration_id
    configured_project/profile/reference identities
    current_project_snapshot/generation
    analyzer pin/probe/config/snapshot identities
    rule registry and rule descriptors/rollout
    component records/health
    capability summaries and important coverage blockers
    last-known-good and failed target records
    deferred operations/capabilities
    operation budget policy
    output schema/tool versions
    warnings
    canonical_digest
```

No field named `pass`, `clean`, or equivalent check result unless referring to an external explicitly identified run record (not part of E0 status).

## 7. Check request

```text
CheckRequest
    request_id
    service_id
    project_id
    project_generation_selector
    scope
    requested_rule_ids or E0 default policy
    include_generic_findings: true
    rollout_filter
    presentation_mode: raw_and_roots
    operation_budget
    cancellation token/state
```

E0 scope:

```text
AllProjectFiles
FileIds(ProjectFileId[])
SourceScopes(exact file/function/fact IDs from closed fixture)
```

No path glob/fuzzy search/filesystem scan.

## 8. Service context lease

```text
ServiceContextLease
    lease_id
    service_configuration_id
    selection_metadata
    generation_context
    profile_identity
    reference_view_identity
    project_snapshot_identity
    project_view_identity
    analyzer_snapshot_identity
    analyzer pin/probe/config identity
    rule_registry_identity
    capability/coverage/conflict registry identities
    acquired_scope
    budget/cancellation state
    canonical_digest
```

### Invariants

- one exact profile/reference/project generation;
- project/analyzer/reference identities agree;
- rule fixture policy agrees;
- source/facts/findings/coverage records belong to context;
- immutable for request duration;
- no later current pointer reread/switch;
- no mutable lower-layer handles in public result.

## 9. Generic finding set

```text
GenericFindingSet
    set_id
    generation_context
    project/analyzer snapshot identities
    selected scope
    raw_generic_finding_ids[]
    related analyzer coverage IDs[]
    blockers/warnings[]
    budget usage
    canonical_digest
```

The service obtains it through ProjectView. It does not recreate or normalize diagnostics.

## 10. Rule execution input/output

Service passes the coherent context/scope to `wow-rules` and receives:

```text
RuleExecutionReport
    selected/skipped rules
    Findings/Clean/NotEvaluated/Failed/Cancelled outcomes
    root-cause/causal hints
    rule execution coverage
    budget usage
```

Service validates the report against the lease before using it.

## 11. Raw check data

```text
RawCheckData
    generation_context
    generic_finding_set
    rule_execution_report
    raw_findings[]
    clean_evaluation_records[]
    not_evaluated_records[]
    rule_failures[]
    component warnings/failures[]
    capability/coverage/conflict references[]
    combined budget usage
```

`raw_findings` is canonical union of generic findings and rule findings, deduplicated only by exact core finding identity.

No presentation folding changes it.

## 12. Finding presentation graph

```text
FindingPresentationGraph
    graph_id
    generation_context
    raw_finding_ids[]
    display_root_ids[]
    nodes[]
    edges[]
    orphan_or_independent_ids[]
    blocker_relations[]
    canonical_digest
```

```text
PresentationNode
    node_id
    record_kind: finding | not_evaluated | component_failure | warning
    referenced_record_id
    primary_parent_id: optional
    display_order_key
```

```text
PresentationEdge
    edge_id
    from_node_id
    to_node_id
    relation_kind
    evidence IDs[]
    confidence
    is_primary_parent
```

E0 relation kinds:

```text
causes_or_explains
blocked_by
exact_duplicate_of
related_competing_cause
```

Graph must be acyclic. Raw record IDs remain independently available.

## 13. Semantic status

```text
ServiceSemanticStatus
    clean
    findings
    partial
    failed
    cancelled
```

Derivation inputs:

```text
context coherence
mandatory component failures
cancellation
requested-scope NotEvaluated/degradable failures/truncation
raw finding count
complete budget/scope proof
```

Status precedence defined in `RESULT_ENVELOPE.md`.

## 14. Check result envelope

```text
CheckResultEnvelope
    envelope_schema_version
    result_id
    operation = check
    request identity/selector metadata
    service semantic status
    exact GenerationContext
    component snapshot identities
    selected scope
    component/capability summary
    raw findings
    presentation graph
    clean evaluation records
    NotEvaluated records
    rule failures and coherent partial warnings
    rule execution coverage
    deferred operation/capability records
    budget/truncation state
    producer/tool/schema versions
    canonical digest
```

If mandatory context/result construction fails, service returns `ServiceFailureResult`, not a malformed check envelope.

## 15. Failure result

```text
ServiceFailureResult
    result_id
    operation
    error code
    request/configuration/selection identities when valid
    observed context/component identities
    capability/partition/conflict IDs
    last-known-good/failed target IDs separately
    recovery class
    canonical digest
```

No raw private source/path/token.

## 16. Cancellation result

```text
ServiceCancelledResult
    result_id
    operation
    request identity
    selected exact context if acquisition completed
    cancellation phase
    completed-but-unpublished work summary
    canonical digest
```

No check envelope or late result after cancellation.

## 17. Deferred operation record

```text
DeferredOperationRecord
    operation_id
    state = Deferred
    first_milestone
    error_code = operation_not_implemented_for_milestone
    required inactive component/capability contracts[]
```

Status lists these. Invoking them returns a typed failure result.

## 18. Check scope

```text
CheckScope
    all project files
    explicit file IDs
    explicit analyzer fact/function/use IDs
```

Scope validation uses ProjectView; service never reads paths/source itself.

Canonical scope order:

```text
file ID
source/fact scope key
```

## 19. Root-cause selection record

```text
RootCauseSelectionRecord
    child record ID
    candidate parent relations[]
    selected primary parent relation: optional
    selection rule code
    competing parent relations retained[]
```

Selection policy is deterministic and evidence-authority based, never message-length/order.

## 20. Budget aggregation

```text
ServiceBudgetUsage
    context acquisition
    generic finding collection
    rule execution
    presentation graph
    serialization/output
    total
```

A stage cannot exceed its/allocation/total without typed partial/failure. Budget fields in canonical output are deterministic counts/bytes/work units, not wall-clock timings.

## 21. Canonical ordering

Within check envelope:

1. raw findings by core canonical finding order;
2. clean evaluations by rule/scope/evaluation ID;
3. NotEvaluated by rule/scope/blocker/ID;
4. failures/warnings by component/code/partition/ID;
5. presentation roots by referenced record canonical order;
6. edges by from/to/relation/evidence/ID;
7. deferred records by operation/capability ID;
8. capabilities/coverage by existing core order.

## 22. Canonical identity exclusions

Exclude:

```text
wall-clock start/end/duration
temporary path
process/thread/worker ID
memory address
stdout/stderr formatting
localized/human message prose when structured fields unchanged
current-selector token after exact resolution (retained only as nonidentity metadata when desired)
```

## 23. CLI projection types

Service exposes no CLI exit code, stdout, stderr, or argument parser type. `apps/wow` maps:

```text
ServiceStatusResult
CheckResultEnvelope
ServiceFailureResult
ServiceCancelledResult
DeferredOperation failure
```

into CLI behavior.

## 24. Fixture identities

Closed case sets:

```text
wow-service-e0-status-v1
wow-service-e0-check-clean-v1
wow-service-e0-check-findings-v1
wow-service-e0-check-partial-v1
wow-service-e0-check-failure-v1
wow-service-e0-check-cancelled-v1
wow-service-e0-root-folding-v1
wow-cli-e0-v1
```

Exact IDs/digests freeze before implementation.
