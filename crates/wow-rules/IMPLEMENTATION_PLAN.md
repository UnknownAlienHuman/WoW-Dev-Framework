# `wow-rules` E0-E implementation plan

**Status:** ordered handoff plan for a future coding agent. This documentation change adds no Rust code.

## Phase 0 — prerequisites

1. Confirm E0-A `wow-core` implementation and canonical vectors are merged.
2. Confirm E0-B `wow-reference` exact lookup/facet fixtures and coverage/authority semantics are implemented/frozen.
3. Confirm E0-C `wow-emmy` exact pin/probe, analyzer facts, source coordinates, and generic diagnostic mapping are implemented/frozen.
4. Confirm E0-D `wow-project` coherent `ProjectSnapshot`/view and generation/source registry are implemented/frozen.
5. Read all files listed in [`AGENTS.md`](AGENTS.md).
6. Confirm no competing provider/rule registry implementation exists.
7. Confirm `wow-graph` remains inactive for E0-E.

**Gate:** no provider code begins while prerequisite public seams/fixture IDs remain draft/null.

## Phase 1 — crate skeleton

Create the smallest crate with internal responsibilities such as:

```text
descriptor
registry
context
capability_gate
execution
outcome
finding
root_cause
remediation
api_exists
secret_local
fixture
error
```

Rules:

- direct framework dependencies exactly `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`;
- no graph/store/search/service/transport/IO/source-edit module;
- no empty modules for later rule families;
- providers pure over immutable context;
- no async runtime/background executor unless contract changes.

**Gate:** crate compiles with only two active rule implementations and no placeholder success paths.

## Phase 2 — rule descriptor and registry

Implement:

```text
validate_rule_descriptor
register_rule_descriptor
build_rule_registry
validate_rule_registry
```

Freeze descriptors:

```text
wow.api.exists@1
wow.secret.local_operation@1
severity error
rollout advisory
remediation plan_only
```

Run `RULES-REG-*`.

**Gate:** exact registry identity/digest and inactive later rules rejected.

## Phase 3 — execution context and scope

Implement:

```text
validate_rule_execution_context
canonicalize_rule_scope
select_applicable_rules
```

Validate profile/reference/project/analyzer/registry/fixture policy/source/coverage/budgets.

Run `RULES-CTX-*`, applicable selection cases.

**Gate:** no cross-generation/profile/stale context reaches providers.

## Phase 4 — universal capability gate

Implement exact partition selection and blocker construction:

```text
resolve_rule_requirements
select_required_partitions
evaluate_rule_capabilities
build_rule_not_evaluated
validate_rule_not_evaluated
```

Run `RULES-GATE-*`.

**Gate:** every partial/failed/conflict/truncation/stale case blocks correctly; empty inputs never clean.

## Phase 5 — output primitives

Implement:

```text
RuleEvaluationOutcome
CleanEvaluationRecord
RuleFailure
RuleRootCauseKey
CausalRelationHint
Remediation
RuleExecutionCoverageReport
RuleExecutionReport
```

Build validators and deterministic identity/order.

Run `RULES-OUT-*`, `RULES-EVID-*`, `RULES-ROOT-*`, `RULES-REM-*` independent of provider algorithms.

**Gate:** outcome exclusivity, evidence/source separation, plan-only remediation, and canonical identity pass.

## Phase 6 — API input assembly and gate

Implement:

```text
is_api_exists_scope_applicable
assemble_api_exists_input
build_api_exists_exact_query
evaluate_api_exists_capabilities
classify_api_exists_lookup_outcome
```

Use exact E0-B query only.

Run `API-001..014` up to classification without findings.

**Gate:** unresolved analyzer fact alone never becomes absence; only authoritative exact miss selects finding branch.

## Phase 7 — API finding/clean/NotEvaluated

Implement:

```text
build_api_exists_finding
build_api_exists_clean_record
build_api_exists_not_evaluated
build_api_generic_causal_hint
validate_api_exists_outcome
```

Run all `API-FIND-*`, `API-CAUSE-*`, seams/mutations.

**Gate:** exact project span, independent authority evidence, no absent source handle/replacement/edit/folding.

## Phase 8 — Secret input assembly and facet gate

Implement:

```text
is_secret_local_scope_applicable
assemble_secret_local_input
resolve_secret_producer_and_return_slot
evaluate_secret_local_capabilities
classify_secret_facet_lookup
trace_supported_local_value_flow
classify_secret_operation
```

Run `SECRET-001`, facet cases, producer/flow unavailable cases without final outcome construction.

**Gate:** exact producer/return facet/value/use/operation; annotation/name alone insufficient.

## Phase 9 — Guard/control-flow classification

Implement:

```text
classify_applicable_guard
classify_guard_dominance
```

Use fixture policy `wow-rules-e0-fixture-policy/1` only.

Classifications:

```text
dominating_exact_value
absent
after_use
different_value
non_dominating
unknown_or_unsupported
```

Run `SECRET-GUARD-*`, shadow/copy/conversion mutations.

**Gate:** only exact-value proven dominance reaches clean; uncertain flow NotEvaluated.

## Phase 10 — Secret finding/clean/NotEvaluated

Implement:

```text
build_secret_local_finding
build_secret_local_clean_record
build_secret_local_not_evaluated
validate_secret_local_outcome
```

Run all `SECRET-*`, `SECRET-FACET-*`, `SECRET-OUT-*`, seams/mutations.

**Gate:** all supported unsafe/guard variants correct, no declassification/runtime/generalization/edit.

## Phase 11 — provider execution and aggregation

Implement:

```text
run_rule
validate_rule_outcome
aggregate_rule_outcomes
canonicalize_rule_execution_report
derive_rule_execution_report_id
```

Run `RULES-EXEC-*`, deterministic ordering, duplicate scopes, budgets/cancellation.

**Gate:** no hidden execution order, no late result, no final stream folding.

## Phase 12 — fixture and checksum freeze

Before or with first implementation commit:

1. import frozen E0-B profile/reference/query/facet/coverage/evidence IDs;
2. import frozen E0-C analyzer facts/source/generic finding IDs;
3. import frozen E0-D project generation/snapshot/source registry IDs;
4. derive exact RuleRegistry/fixture policy/context/evaluation/finding/root/remediation IDs;
5. canonicalize all E0-E examples;
6. write actual SHA-256 member/bundle digests;
7. update `CONTRACT.json` and manifest implementation state;
8. reject null prerequisite/output/digest fields after activation.

Expected files are immutable test vectors; tests do not rewrite them.

## Phase 13 — false-blocking/evaluation report

Run closed corpus and report at minimum:

```text
scope count
Findings/Clean/NotEvaluated/Failed/Cancelled counts per rule
expected versus observed
false finding count
false clean count
missing NotEvaluated count
capability/conflict blocker distribution
canonical output digest
```

E0 rollout remains advisory. Promotion to blocking requires later representative corpus/decision.

## Phase 14 — deterministic/security/mutation review

Vary:

```text
provider/scope/fact/coverage/conflict/lookup/generic finding order
worker scheduling
temporary root
message wording
budget boundaries
cancellation points
```

Deliberately break:

```text
authority decision requirement
project/reference evidence separation
exact source span validation
facet requirement
guard value identity
guard dominance
copy/conversion rule
outcome exclusivity
root cause identity
remediation tier
```

All mutations must fail for exact structured reasons.

## Phase 15 — public seam review

Consumers:

- `wow-service`: registry/executor/report/outcomes; final folding/envelope/status;
- `apps/wow`: only through service;
- no direct source mutation consumer.

Review:

- only two active rules;
- no graph edge;
- no IO/network/process/editor/client/source mutation;
- no search/replacement/autofix;
- no project/reference selection/retry;
- no runtime claim;
- no raw upstream types;
- no hidden coverage/NotEvaluated suppression.

Run `RULES-SEAM-*`, deferred/security cases.

## Phase 16 — completion report

Report:

```text
crate dependencies/public API
rule descriptors/registry identity
fixture policy/context identities
prerequisite E0-B/C/D IDs consumed
capability requirements and blockers
API rule outcome matrix
Secret rule outcome matrix
evidence/root cause/remediation structures
all applicable test IDs/results
false-blocking/evaluation counts
security/no-IO/no-mutation/no-edit checks
canonical byte/digest determinism
known NotEvaluated/deferred cases
```

## Forbidden shortcuts

Do not:

- run provider before capability/context gate;
- infer absence from analyzer unresolved/empty result;
- create a local negative-authority shortcut;
- ignore conflict/truncation/stale records;
- use project source as platform evidence;
- fabricate absent source handles;
- infer replacement/edit from similarity;
- infer Secret from annotation/name;
- accept guard by variable name or after use;
- guess dominance;
- declassify through copy/conversion/serialization/pcall;
- claim runtime/general safety;
- emit any E0 source edit;
- suppress/fold global findings;
- add another rule family/graph dependency;
- change fixtures merely to make implementation easier.

## Completion boundary

E0-E ends at deterministic provider outcomes for exactly two rules. Final service orchestration, cross-rule/root-cause folding, result envelope, CLI, and exit policy remain E0-F.
