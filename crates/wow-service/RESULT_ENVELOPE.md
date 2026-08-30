# Service result envelopes

**Status:** normative E0-F public operation output and semantic-status contract.

Service results are transport-independent, versioned, evidence-bearing, and canonically serializable. Applications project them; they do not redefine semantics.

## 1. Result family

```text
ServiceOperationResult
    Status(ServiceStatusResult)
    Check(CheckResultEnvelope)
    Failure(ServiceFailureResult)
    Cancelled(ServiceCancelledResult)
```

Deferred operation invocation returns `Failure` with `operation_not_implemented_for_milestone`.

## 2. Check result envelope

```text
CheckResultEnvelope
    envelope_schema_version
    result_id
    operation = check
    request_id
    service_configuration_id
    service_context_lease_id
    selector metadata
    ServiceSemanticStatus
    exact GenerationContext
    component_snapshot_identities
    selected scope
    component health/capability summary
    raw findings
    presentation graph
    clean evaluation records
    NotEvaluated records
    rule failures included in coherent partial result
    warnings
    rule execution coverage/report references
    deferred operations/capabilities
    budget/truncation state
    producer/tool/schema versions
    canonical digest
```

Every ID/reference resolves within the result's exact context or an explicitly registered prerequisite artifact.

## 3. Service semantic status

```text
clean
findings
partial
failed
cancelled
```

### Derivation precedence

```text
1. failed
2. cancelled
3. partial
4. findings
5. clean
```

### `failed`

Use when no coherent operation result can be published:

- invalid request/configuration;
- context/generation/profile mismatch;
- mandatory component/registry failure;
- invalid generic/rule report;
- invalid presentation graph;
- unresolved envelope references/digest;
- security/policy violation;
- result-status derivation contradiction.

Normally represented as `ServiceFailureResult`, not a malformed check envelope.

### `cancelled`

Operation cancelled before publication. Normally represented as `ServiceCancelledResult`. No late check envelope.

### `partial`

A coherent useful check result exists, but at least one requested-scope lane is incomplete:

- rule `NotEvaluated`;
- degradable analyzer/reference/project/rule capability failure;
- explicit truncation/budget incompleteness;
- coherent partial generic/rule collection;
- rule failure explicitly allowed as partial by policy (E0 normally treats provider contract failure as failed, not partial).

Findings may coexist. They remain in raw findings.

### `findings`

Every requested-scope lane completed, no blockers/truncation, and raw findings are nonempty.

Advisory rollout findings still yield semantic `findings`.

### `clean`

Every requested-scope lane completed, no blockers/failures/truncation, raw findings empty, and rule/generic scopes explicitly report clean/nonapplicable completion.

Empty arrays alone are insufficient.

## 4. Status operation result

`ServiceStatusResult` uses a different operation-state field:

```text
available
partial
failed
cancelled
```

Do not reuse check semantic `clean/findings` for status.

## 5. Component snapshot identities

Include:

```text
ReferenceView/ReferenceGeneration/ProfileIdentity
ProjectSnapshot/ProjectGeneration/ProjectView
AnalyzerSnapshot/pin/probe/configuration
RuleRegistry/fixture policy
Core/service/output contract versions
```

No raw mutable handle/session/actor/database ID exposed unless it is a stable public identity in the owning contract.

## 6. Raw findings

```text
raw_findings[]
```

Rules:

- contains all generic + WoW findings unchanged;
- canonical exact-identity dedup only;
- source/evidence/context validated;
- count is raw finding count, not root count;
- presentation parent/child state is not embedded by mutating findings;
- no source body/raw Secret-capable value added by service.

## 7. Clean evaluations

```text
clean_evaluations[]
```

Retain rule ID/version, scope, decisive inputs, coverage/budget, and narrow clean claim.

Do not convert them into generic “project safe” flags.

## 8. NotEvaluated

```text
not_evaluated[]
```

Retain exact rule/scope/blockers/coverage/conflicts/staleness/unsupported semantics. Result status becomes `partial` when any requested scope is NotEvaluated.

Deferred capabilities unrelated to requested check scope remain in deferred records and do not alone force partial.

## 9. Presentation graph

Include complete `FindingPresentationGraph` plus summary counts:

```text
raw_problem_record_count
display_root_count
child_relation_count
independent_root_count
```

Counts do not alter semantic status beyond graph validation.

## 10. Warnings and partial failures

Warnings carry structured:

```text
code
component/operation
context IDs
capability/partition/conflict IDs
arguments
severity/impact classification
```

A coherent partial result may include component lane warnings. Mandatory failure returns `ServiceFailureResult` instead.

## 11. Deferred operations/capabilities

Check/status envelopes list active milestone deferred records for discoverability. They are not executed and do not count as requested-scope blockers unless the user explicitly requested one of them.

## 12. Budget and truncation

```text
ServiceBudgetUsage
TruncationRecord[]
```

Include deterministic counts/bytes/work units by stage. Wall-clock timing is supplemental/noncanonical.

Any truncation affecting requested completeness prevents `clean` and normally yields `partial`.

## 13. Selector metadata

Record:

```text
requested selector: Exact | CurrentPublished
requested exact ID/project ID
resolved exact ProjectGenerationId
```

Canonical result identity uses the resolved exact generation. Whether selector token participates as supplemental request identity is explicit; it cannot replace the exact generation.

## 14. Canonical result ID and digest

Use E0-A core canonicalization/domain separation.

Conceptual domains:

```text
wow-service:status-result:e0-f:1
wow-service:check-result:e0-f:1
wow-service:failure-result:e0-f:1
wow-service:cancelled-result:e0-f:1
wow-service:presentation-graph:e0-f:1
```

Result ID/digest covers all semantic fields/references/order. It excludes volatile telemetry/prose.

## 15. Canonical ordering

```text
component identities by component ID
capability summaries by capability/partition/ID
raw findings by core total order
clean evaluations by rule/scope/ID
NotEvaluated by rule/scope/blockers/ID
warnings/failures by component/code/partition/ID
presentation roots by referenced-record order
presentation edges by from/to/relation/evidence/ID
deferred records by operation/capability ID
budget records by stage ID
```

Transport JSON object keys use core canonical JSON order.

## 16. Identity exclusions

Exclude from canonical identity/digest:

```text
wall-clock timestamp/duration
process/thread/worker/session memory address
temporary checkout path
stdout/stderr terminal state/color
localized/human summary prose
input return/discovery order
UI expansion/collapse state
```

If a supposedly excluded field changes semantic output, fix the model.

## 17. Failure result

```text
ServiceFailureResult
    schema version
    result ID/digest
    operation/request/config IDs
    error code/structured arguments
    valid observed component/context identities
    capability/partition/conflict blockers
    failed target and last-known-good identities separately
    recovery class
    producer/tool versions
```

No clean/findings/partial status. Do not embed an incomplete check envelope.

## 18. Cancellation result

```text
ServiceCancelledResult
    schema version
    result ID/digest
    operation/request ID
    exact selected context if acquired
    cancellation phase
    completed-but-unpublished stage summary
    producer/tool versions
```

No partial raw findings unless a future explicit partial-cancellation contract is accepted. E0 default publishes no check result.

## 19. Validation

```text
validate_service_operation_result
validate_check_result_envelope
validate_service_status_result
validate_service_failure_result
validate_service_cancelled_result
validate_service_semantic_status
validate_result_reference_closure
validate_raw_finding_preservation
validate_presentation_graph_binding
validate_budget_and_truncation
canonicalize_service_result
derive_service_result_id
```

## 20. Required mutations

Reject:

- semantic `clean` with findings/NotEvaluated/truncation;
- semantic `findings` with NotEvaluated;
- raw finding omitted because folded;
- root count substituted for finding count;
- finding/evaluation from another generation;
- unresolved evidence/source/coverage ID;
- current selector without resolved exact generation;
- last-known-good relabeled target;
- invalid/cyclic presentation graph;
- message/timestamp/temp path changes digest;
- output ordering based on upstream return order;
- deferred requested operation returning empty success;
- invalid digest/reference closure.

## 21. E0 fixture result types

```text
status-result.json
check-clean-result.json
check-findings-result.json
check-partial-result.json
check-context-error.json
check-cancelled-result.json
```

Fields/IDs remain null only while implementation is not-started; freeze before first Rust commit.

## 22. Hard stops

- no malformed check envelope for failure;
- no clean from empty arrays;
- no folding mutation/deletion;
- no advisory findings treated as clean;
- no partial hidden by findings;
- no volatile identity fields;
- no transport/CLI fields in service semantic contract;
- no local/private/raw Secret data leakage;
- no unresolved cross-context references.
