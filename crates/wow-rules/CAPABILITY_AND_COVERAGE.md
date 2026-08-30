# Rule capability, coverage, conflict, and authority gating

**Status:** normative E0-E availability contract.

A rule runs only when its exact prerequisite facts, lookups, coverage partitions, generation context, and conflict state are usable. This document defines the gate. Providers may not weaken it locally.

## 1. Independent axes

Keep separate:

```text
capability exists
coverage status of the selected partition
fact/query result exists
conflict state
truncation/budget state
generation freshness/coherence
profile applicability
rule semantic support
```

`Complete` source ingestion alone does not resolve a semantic conflict. A positive fact can exist under broader partial coverage. A missing fact does not prove absence.

## 2. Gate result

```text
RuleCapabilityGate
    Runnable
    NotEvaluated(blockers)
    ContextError(errors)
    Cancelled
```

### Runnable

Every requirement for the exact scope is satisfied.

### NotEvaluated

Context identities are coherent, but evidence/capability/coverage/semantic support is insufficient.

### ContextError

Profile/reference/project/analyzer generations, views, source handles, or registry identity conflict. Do not report this as ordinary unavailable evidence.

## 3. Requirement model

```text
RuleCapabilityRequirement
    capability_id
    partition selector
    accepted coverage statuses
    required fact/query kind
    conflict policy
    truncation policy
    generation freshness requirement
    required for evaluation/clean/finding branch
```

Partition selectors resolve to exact records before execution.

## 4. Universal context requirements

Both E0 rules require:

```text
one valid ProfileIdentity
one matching ReferenceGenerationId
one matching ProjectGenerationId
one valid ReferenceView for that profile/reference
one valid ProjectView/AnalyzerSnapshot for that project generation
current project source handles/facts
valid rule registry/policy identity
valid execution budget
```

Mismatch -> context error, not `NotEvaluated`.

## 5. `wow.api.exists@1` requirements

### Project/analyzer

```text
project.generation.coherent
project.source.registry.complete
project.analyzer.snapshot.available
emmy.library.loaded
emmy.file.parsed for selected file
emmy.fact.references for selected file/use
emmy.fact.calls when call relation is required
emmy.source_coordinates.exact
```

Required facts:

- one direct Main-project unresolved member reference;
- optional/direct call fact tied to that reference;
- exact canonical entity query key and project source span;
- no ambiguity/dynamic status for E0 scope.

### Reference

```text
reference.fixture.profile.valid
reference.symbol.exact_lookup
reference.source_handle.resolve as needed for related evidence
```

Exact selected partition:

```text
reference.fixture.apidoc.system:C_E0Fixture
```

Finding branch additionally requires:

```text
exact lookup outcome = authoritative_absent
negative-authority decision = authoritative
coverage Complete for exact system/kind domain
no unresolved conflict/truncation/stale context
```

Clean branch requires:

```text
exact lookup outcome = found
lookup capability usable
```

### Blockers -> NotEvaluated

- `absent_without_authority`;
- reference lookup conflict;
- partial/failed/unknown exact partition;
- missing declared reference input;
- unresolved unknown field blocking exact lookup;
- annotation library failure preventing exact project reference fact;
- unresolved fact is ambiguous/dynamic rather than E0 direct member;
- output/evaluation budget prevents complete selected-use evaluation;
- rule/profile scope unsupported.

Profile/generation mismatch remains context error.

## 6. `wow.secret.local_operation@1` requirements

### Project/analyzer

```text
project.generation.coherent
project.source.registry.complete
project.analyzer.snapshot.available
emmy.library.loaded
emmy.file.parsed
emmy.fact.references
emmy.fact.calls
emmy.fact.local_bindings
emmy.fact.local_flow
emmy.fact.operations
emmy.fact.guards
emmy.fact.control_flow
emmy.source_coordinates.exact
```

Required facts:

- resolved producer member/call `C_E0Fixture.SecretText`;
- exact return-position/local-binding relation;
- exact local use and concatenation operation;
- containing function/scope;
- guard/control-flow facts sufficient to classify absent/dominating/after-use/different-value for E0.

### Reference

```text
reference.fixture.profile.valid
reference.symbol.exact_lookup
reference.restriction.facets
reference.source_handle.resolve
```

Exact selected partition:

```text
reference.fixture.restriction:C_E0Fixture.SecretText
```

Required facet:

```text
kind = secret.return
subject = function:C_E0Fixture.SecretText
target = return_position:1
applicability = unconditional_fixture
outcome = found
coverage = Complete
no conflict
```

### Fixture semantics

```text
rule fixture policy = wow-rules-e0-fixture-policy/1
accepted guard kind = access_single
recognized fixture callee = canaccessvalue
supported operation = concatenation
scope = function_local
```

### Finding branch

All requirements available and one of:

```text
no applicable guard for exact value
accepted guard occurs after operation
accepted guard targets another value
accepted guard does not dominate operation
copy/conversion flow still reaches operation without accepted dominating guard
```

### Clean branch

All requirements available and:

```text
accepted guard targets exact value
accepted guard/control-flow relation proves dominance over exact operation
operation belongs to supported scope/kind
```

### Blockers -> NotEvaluated

- facet lookup unavailable/partial/conflict/none-nonauthoritatively;
- facet conditional/runtime semantics unsupported by E0 fixture;
- producer/member/call unresolved/ambiguous/dynamic;
- binding/value flow unknown;
- operation kind unsupported;
- control-flow/dominance capability missing/partial;
- guard fact ambiguous;
- interprocedural/dynamic callback/container flow outside E0;
- source span invalid/stale;
- budget/truncation prevents complete scope evaluation.

## 7. Positive facts under partial broader coverage

A known exact project/reference fact may remain usable even if an unrelated broader partition is partial. The gate chooses the narrow partitions required by the rule.

Examples:

- `KnownApi` exact reference fact may be found while the system partition is partial; API existence for that exact use can be clean because presence is proven.
- absence under the same partial system partition is not authoritative.
- Secret facet can remain complete even if unrelated symbol inventory coverage is partial, provided producer entity and facet identity/context are valid.

The gate must record surrounding partial coverage as relevant warnings when it affects interpretation, without overblocking unrelated exact facts.

## 8. Conflict policy

### API rule

Any conflict affecting exact entity presence/kind/query domain blocks absence finding. A conflict irrelevant to the exact presence dimension may be retained as supplemental context without blocking only when the contract proves independence.

### Secret rule

Any conflict affecting producer identity, return slot, facet kind/applicability, accepted guard semantics, or required analyzer flow blocks evaluation.

No provider selects a first/last conflict winner.

## 9. Truncation and budgets

If selected scope input/result is truncated:

- no `EvaluatedClean`;
- no absence finding unless the decisive exact query/authority was completed independently and the rule scope itself is complete;
- normally return `NotEvaluated` with exact budget/truncation blockers;
- partial findings require an explicitly partial report and are not E0 default.

## 10. Staleness

Reject or block:

- source handle content digest mismatch;
- analyzer fact from old snapshot/project generation;
- reference lookup from another reference generation/profile;
- rule fixture policy for another profile/version;
- retained last-known-good project snapshot substituted for requested target generation.

Stale/mixed context is not fixed by retrying another implicit current snapshot inside the rule crate.

## 11. NotEvaluated construction

```text
NotEvaluatedRecord
    rule ID/version
    generation context
    exact attempted scope
    missing capabilities
    blocking coverage records
    conflict IDs
    stale/mismatch details where context remains representable
    unsupported semantic/fact kinds
    budget/truncation blockers
    structured next evidence requirements
```

Examples of next evidence requirements:

```text
load a complete exact reference partition
resolve annotation-library failure
provide exact control-flow dominance facts
select the correct project/reference generation
run a runtime scenario (later production cases only)
```

No speculative source edit is included.

## 12. EvaluatedClean construction

Requires:

- all rule requirements satisfied;
- exact scope enumerated/examined;
- decisive fact/query IDs recorded;
- no finding condition;
- complete budget for scope;
- no hidden blocker;
- narrow rule-specific clean claim.

`EvaluatedClean` does not imply other rule families passed.

## 13. Rule execution coverage

The execution report records:

```text
requested/selected rules
scopes discovered/examined
scopes Findings/Clean/NotEvaluated/Failed
capability checks and selected partitions
lookups performed
facts consumed
conflicts/blockers
budget usage/truncation
```

It does not rewrite producer coverage.

## 14. Required operations

```text
resolve_rule_requirements
select_required_partitions
validate_rule_context_coherence
evaluate_api_exists_capability_gate
evaluate_secret_local_capability_gate
build_rule_not_evaluated
validate_rule_not_evaluated
build_clean_evaluation_record
validate_clean_evaluation_record
build_rule_execution_coverage_report
```

## 15. Mutation tests

Tests deliberately attempt to:

- infer absence from unresolved analyzer fact;
- infer authority from all-global-summary-complete while exact partition partial/conflicted;
- treat empty facts under failed capability as clean;
- ignore conflict IDs;
- reuse stale source/facts/lookups;
- accept guard without dominance;
- accept different-value guard;
- accept copy/conversion as declassification;
- return clean after budget truncation;
- downgrade context mismatch to ordinary NotEvaluated.

Every mutation must fail for the intended structured reason.
