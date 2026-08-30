# Rule provider execution

**Status:** normative E0-E provider registry and execution pipeline.

## 1. Registry

```text
RuleRegistry
    registry_id
    registry_schema_version
    descriptors[]
    enabled_rule_versions
    canonical_digest
```

E0 registry contains exactly:

```text
wow.api.exists@1
wow.secret.local_operation@1
```

Registration rejects:

- duplicate RuleId/version;
- same RuleId with incompatible active versions;
- descriptor missing capabilities/fixtures/budgets/rollout;
- rule outside active milestone marked enabled;
- hidden dependency not present in descriptor/contract;
- graph-dependent rule while graph edge inactive.

## 2. Execution request

```text
RuleExecutionRequest
    requested_rule_ids[] or policy selector
    execution_context
    scope
    rollout_filter
    budget
    cancellation
```

The request does not contain source bytes or mutable component handles. It references immutable views/facts/records.

## 3. Pipeline

```text
validate registry
-> validate execution context coherence
-> canonicalize requested rules/scope
-> select descriptors applicable to profile/scope/policy
-> evaluate required capabilities/partitions/conflicts/staleness
-> produce NotEvaluated for blocked rules/scopes
-> assemble rule-specific normalized inputs
-> run pure provider under budget/cancellation
-> validate provider outcome
-> construct findings/clean records/failures
-> attach rule execution coverage/report
-> canonical sort and derive execution report identity
```

No provider runs before context/capability validation.

## 4. Applicable-rule selection

```text
select_applicable_rules(registry, request)
```

Filters by:

- requested/enabled RuleId/version;
- milestone/rollout policy;
- selected profile/flavor applicability;
- project source scope and fact kinds;
- requested file/span/entity scope;
- rule fixture policy in E0;
- cancellation/budget preflight.

A rule skipped as nonapplicable is reported separately from `NotEvaluated`.

## 5. Execution ordering

Providers have no semantic dependence on execution order.

Canonical scheduling/order is by:

```text
RuleId
rule version
scope file/source identity
```

Parallel execution may be introduced later only if outputs remain identical and providers stay pure. E0 may execute synchronously.

## 6. Rule-specific input assembly

The execution layer constructs typed inputs only after required facts/lookups validate.

### API rule

```text
unresolved Main member/reference fact
optional direct call fact
project SourceHandle
exact EntityKey
ReferenceView exact lookup outcome
reference coverage/conflicts/authority decision
generic symptom IDs sharing exact source/fact relation
```

### Secret rule

```text
producer/member/call fact
return-position relation
local binding/value flow
exact local use + operation
applicable guard/control-flow facts
exact restriction-facet lookup
fixture guard-semantics policy
coverage/conflicts
```

Input assembly itself can yield `NotEvaluated` or context failure. It cannot invent missing facts.

## 7. Capability preflight

```text
evaluate_rule_capabilities(descriptor, context, scope)
```

Returns:

```text
Runnable
NotEvaluated(blockers)
ContextError
Cancelled
```

The preflight checks exact selected partitions, not only global capability summaries.

## 8. Provider contract

Conceptual pure interface:

```text
RuleProvider::evaluate(
    descriptor,
    validated RuleSpecificInput,
    immutable RuleExecutionContext,
    budget/cancellation read view
) -> RuleEvaluationOutcome
```

Providers cannot:

- mutate context/views;
- perform IO/process/network/editor/client operations;
- invoke another provider for hidden dependency;
- change registry/policy;
- allocate unbounded output;
- catch context mismatch and retry another generation/profile;
- return raw/unvalidated findings.

## 9. Outcome validation

### Findings

Validate:

- at least one finding;
- each belongs to exact rule/version/context;
- primary project source handle valid/current;
- evidence/coverage/conflict IDs resolve;
- no duplicate canonical finding identity;
- remediation allowed by descriptor;
- output budget observed;
- no clean/NotEvaluated for same exact evaluation scope.

### EvaluatedClean

Validate:

- all required capabilities usable;
- exact scope examined;
- decisive facts/lookups recorded;
- no finding condition matched;
- no hidden conflict/truncation/staleness;
- budget complete;
- clean claim stays rule-specific.

### NotEvaluated

Validate:

- at least one exact blocker;
- blocker belongs to required capability/partition/context;
- no speculative finding/clean claim;
- attempted scope/rule recorded;
- next evidence request is structured and non-mutating.

### Failed

Validate implementation/contract failure code and no partial finding/clean output.

### Cancelled

No late outcome publication after cancellation.

## 10. Per-use evaluation

E0 providers evaluate one canonical use/operation scope at a time, then aggregate deterministically.

Benefits:

- exact source location;
- clear capability/fact blockers;
- deterministic deduplication;
- no one bad use hiding unrelated uses;
- service can group root causes later.

A file-level request expands to canonical use scopes using supplied facts; it does not scan source independently.

## 11. Duplicate observations

Equivalent analyzer observations may refer to one source use.

Deduplicate using structured:

```text
rule/version/context
primary source handle/span
subject/entity key
operation/use kind
canonical decisive fact IDs
```

Do not deduplicate two distinct source spans merely because messages/subjects match.

## 12. Root-cause hints

Providers may emit:

```text
RuleRootCauseKey
CausalRelationHint[]
```

API example conditions for generic symptom relation:

- generic finding and API project fact share exact file/content/span/reference fact;
- same project generation/analyzer snapshot;
- generic category represents the same unresolved member condition;
- API reference exact absence is authoritative.

Otherwise no causal hint.

Final folding, display priority, and raw finding retention remain service-owned.

## 13. Budgets

Descriptor/request define:

```text
max scopes
max facts per scope
max reference lookups
max findings
max related evidence
max output bytes
optional deterministic work units
```

Budget exceedance before complete evaluation yields `NotEvaluated`/Failed according to contract, never a clean result. Partial findings may be returned only if the execution report explicitly marks truncation and service policy accepts them; E0 prefers whole-scope outcomes.

## 14. Cancellation

- checked between canonical scopes and bounded internal steps;
- provider state is request-local;
- no mutation rollback required because providers are pure;
- completed scopes may be retained only in an explicitly partial execution report; E0 default is Cancelled without publishing incomplete rule set;
- no background continuation.

## 15. Rollout policy

E0 descriptors:

```text
technical severity: error
rollout: advisory
```

Selection policy may include shadow/advisory/blocking filters, but the provider does not decide overall command exit status. Service/application policy owns that later.

## 16. Execution report

```text
RuleExecutionReport
    report_id
    registry_id
    execution_context_id
    requested/selected/skipped rule IDs
    canonical scopes
    outcomes
    capability checks
    budget usage
    cancellation/truncation state
    causal hints
    canonical digest
```

No wall-clock timing in canonical identity; timing may be supplemental evaluation telemetry.

## 17. Required operations

```text
register_rule_descriptor
validate_rule_descriptor
build_rule_registry
validate_rule_registry
select_applicable_rules
validate_rule_execution_context
canonicalize_rule_scope
evaluate_rule_capabilities
assemble_api_exists_input
assemble_secret_local_input
run_rule
validate_rule_outcome
aggregate_rule_outcomes
canonicalize_rule_execution_report
derive_rule_execution_report_id
```

## 18. Hard stops

- no provider execution before capability/context validation;
- no hidden provider ordering dependency;
- no source parsing/scanning;
- no local coverage shortcut;
- no empty findings == clean;
- no message-based dedup/root cause;
- no final stream folding;
- no IO/mutation/edit;
- no late background result after cancellation;
- no unbounded scopes/output;
- no inactive rule family in E0 registry.
