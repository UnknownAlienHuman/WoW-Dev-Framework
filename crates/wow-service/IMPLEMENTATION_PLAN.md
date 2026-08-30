# `wow-service` E0-F implementation plan

**Status:** ordered handoff plan for a future coding agent. This contract change adds no Rust code.

## Phase 0 — prerequisites

1. Confirm E0-A `wow-core` implementation/canonical vectors are merged.
2. Confirm E0-B ReferenceView/profile/coverage fixtures are implemented/frozen.
3. Confirm E0-C analyzer pin/probe/snapshot/facts/generic diagnostic fixtures are implemented/frozen.
4. Confirm E0-D ProjectSnapshot/source registry/update fixtures are implemented/frozen.
5. Confirm E0-E RuleRegistry/outcome/finding fixtures are implemented/frozen.
6. Read every file listed in [`AGENTS.md`](AGENTS.md).
7. Confirm no competing service/CLI orchestration exists.
8. Confirm later crates/components remain inactive.

**Gate:** no service/CLI code begins while prerequisite public seams or result vectors remain draft/null.

## Phase 1 — minimal crate skeleton

Create only responsibilities such as:

```text
configuration
component_registry
selection
context
status
check
raw_aggregation
presentation
semantic_status
envelope
budget
cancellation
deferred
error
fixture
```

Rules:

- direct framework dependencies exactly core/reference/emmy/project/rules;
- no parser/store/graph/search/CBM/context-builder/transport modules;
- no generic future operation trait/dispatcher;
- no empty success paths;
- E0 can remain synchronous/in-process;
- public API uses normalized framework types only.

**Gate:** crate compiles with only status/check and typed deferred failures.

## Phase 2 — service configuration and component registry

Implement:

```text
validate_service_configuration
build_service_component_registry
validate_service_component_registry
build_deferred_operation_registry
validate_deferred_operation_registry
canonicalize_service_configuration
```

Run `SERVICE-CONFIG-*`.

**Gate:** exact component/contract/implementation identities, minimal dependency set, deterministic configuration/registry digest.

## Phase 3 — project generation selector and context acquisition

Implement:

```text
validate_service_context_request
resolve_project_generation_selector
acquire_project_snapshot
acquire_reference_view_for_project
validate_analyzer_binding_for_context
acquire_rule_registry
acquire_capability_registries
validate_service_context_coherence
canonicalize_service_scope
build_service_context_lease
validate_service_context_lease
```

Run `SERVICE-CTX-*`.

**Gate:** one exact immutable lease, current selector resolved once, no fallback/last-known-good substitution/switch.

## Phase 4 — status operation

Implement:

```text
validate_status_request
collect_service_component_records
classify_component_health
collect_status_capabilities
collect_last_known_good_and_failed_targets
collect_deferred_operation_records
build_status_warnings
assemble_service_status_result
validate_service_status_result
canonicalize_service_status_result
```

Run `STATUS-*`.

**Gate:** exact identities/capabilities/deferred state with no check/test/runtime/clean inference.

## Phase 5 — check request and scope

Implement:

```text
validate_check_request
resolve_check_scope
```

Support only closed exact scope kinds. Validate budgets and cancellation before expensive component reads.

Run request/scope portions of `CHECK-*`, `SERVICE-BUDGET-*`.

**Gate:** no path glob/fuzzy/filesystem scope and deterministic selected scope.

## Phase 6 — generic finding collection

Implement through ProjectView only:

```text
collect_generic_finding_set
validate_generic_finding_set
```

Run `CHECK-GENERIC-*`.

**Gate:** exact generic findings/coverage, no service recomputation/mutation, failed capability not empty clean.

## Phase 7 — rule execution handoff

Implement:

```text
build_rule_execution_request
invoke wow-rules executor
validate_rule_execution_report_for_context
```

Run `CHECK-RULES-*`.

**Gate:** all Findings/Clean/NotEvaluated/Failed/Cancelled retained; no weaker retry or local rule algorithm.

## Phase 8 — raw check aggregation

Implement:

```text
assemble_raw_check_data
validate_raw_check_data
canonical_exact_finding_union
```

Run `CHECK-RAW-*`.

**Gate:** every generic/WoW finding unchanged, exact identity dedup only, all outcomes/blockers preserved.

## Phase 9 — presentation graph

Implement:

```text
build_presentation_nodes
validate_causal_hint
build_component_blocker_relations
build_exact_duplicate_relations
select_primary_presentation_parents
retain_competing_relations
validate_presentation_graph_references
detect_presentation_cycles
derive_display_roots
canonicalize_presentation_graph
derive_presentation_graph_id
```

Run all `FOLD-*` mutations.

**Gate:** acyclic one-primary-parent forest/projection, exact structured relations, raw preservation, deterministic graph.

## Phase 10 — semantic status

Implement:

```text
derive_service_semantic_status
validate_service_semantic_status
```

Precedence:

```text
failed > cancelled > partial > findings > clean
```

Run `CHECK-CLEAN-*`, `CHECK-FIND-*`, `CHECK-PARTIAL-*`, `CHECK-FAIL-*`, `STATUS-PREC-*`.

**Gate:** no empty-clean/advisory-clean/findings-with-blockers misclassification.

## Phase 11 — result envelopes

Implement:

```text
assemble_check_result_envelope
validate_check_result_envelope
assemble_service_failure_result
assemble_service_cancelled_result
validate_service_operation_result
validate_result_reference_closure
validate_raw_finding_preservation
validate_presentation_graph_binding
validate_budget_and_truncation
canonicalize_service_result
derive_service_result_id
```

Run `ENVELOPE-*`.

**Gate:** one closed canonical result family with exact references/IDs/digests and no volatile identity.

## Phase 12 — budgets and cancellation

Implement explicit stage aggregation/propagation:

```text
context
scope
generic
rules
aggregation
presentation
serialization
```

Run `SERVICE-BUDGET-*`, `SERVICE-CANCEL-*` across every cancellation phase.

**Gate:** no clean under incompleteness, no silent record dropping, no late/background publication.

## Phase 13 — deferred operation surface

Implement typed failures/status records for every deferred E0 operation. Do not add stubs returning empty data.

Run `SERVICE-DEFER-*`.

**Gate:** only status/check callable.

## Phase 14 — `apps/wow` CLI projection

Implement only after service result family is stable.

CLI owns:

```text
argument parsing
status/check command selection
project/generation/scope/rule/format options
service request construction
canonical JSON stdout
noncanonical text projection
stderr policy
exit-code mapping
```

CLI depends only on `wow-service` plus transport/serialization libraries, not lower framework crates.

Run `CLI-*` and `apps/wow` contract tests.

**Gate:** same service result serialized exactly; no domain logic/lower bypass.

## Phase 15 — fixture/result/checksum freeze

Before or with first implementation commit:

1. import frozen E0-A–E implementation and fixture bundle IDs;
2. freeze service configuration/component registry/context lease IDs;
3. freeze status result;
4. freeze clean/findings/partial/failure/cancelled check results;
5. freeze raw finding/outcome counts and IDs;
6. freeze presentation graph roots/edges/selection records;
7. freeze service statuses and CLI exit mappings;
8. canonicalize all E0-F service/CLI examples;
9. write member/bundle SHA-256 values;
10. update `CONTRACT.json`, apps contract, and manifest implementation states;
11. reject null fields after activation.

Tests verify fixtures; they never rewrite expected outputs automatically.

## Phase 16 — deterministic/security/mutation review

Vary:

```text
component/capability/scope/generic/rule/finding/outcome/edge return order
worker scheduling
temporary root
human message/text rendering
current project pointer after lease acquisition
```

Deliberately break:

```text
context equality/fallback
status false validation claim
raw finding preservation
causal relation evidence
cycle/primary-parent rules
semantic status precedence
result reference closure/digest
budget/cancellation publication
CLI lower-crate bypass/exit mapping
```

All mutations must fail for exact structured reasons.

## Phase 17 — end-to-end golden runs

Run:

```text
status healthy/degraded/failed-target
a clean exact-scope check
a full findings check
a partial broken-capability check
a context failure
a cancellation
a deferred command
JSON and text CLI projections
```

Verify byte-identical canonical JSON under repetitions/permutations.

## Phase 18 — completion report

Report:

```text
active dependencies/public API
component and context identities
status field/health/deferred results
check raw finding/rule outcome counts
presentation graph roots/children/raw preservation
semantic status results
failure/cancellation/budget behavior
canonical result IDs/digests
CLI commands/formats/exit codes/stdout/stderr behavior
all test IDs/commands: pass | fail | skipped
security/no-source-execution/no-editor-mutation/no-lower-bypass checks
known deferred operations/capabilities
```

## Forbidden shortcuts

Do not:

- select/retry another snapshot/profile mid-request;
- use last-known-good for another required generation;
- infer status pass/clean from component readiness;
- reimplement generic diagnostics or WoW rules;
- drop raw findings while folding;
- fold/dedup by message text;
- derive status from root count or advisory rollout;
- call partial clean;
- publish after cancellation;
- silently truncate/drop records;
- activate later crates/operations;
- implement source edits/search/replacement/runtime behavior;
- let CLI import lower crates or alter semantic records;
- change fixtures merely to match an easier implementation.

## Completion boundary

E0-F ends with `status`, `check`, one canonical result family, and one thin CLI. No LSP/MCP, daemon, search, graph, context/skeleton, patch impact, indexing, runtime review, source edit, release automation, or CI is implemented here.
