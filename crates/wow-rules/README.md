# `wow-rules` implementation contract

**Status:** E0-active contract scaffold; no Rust code yet.

## Mission

`wow-rules` implements capability-declared World of Warcraft diagnostic providers over exact reference facts, normalized Emmy facts, project snapshots, and graph views. It emits evidence-bearing findings and remediation classifications without performing persistence, transport, or hidden source discovery.

## Owned responsibilities

- rule/provider registry and versioned descriptors;
- rule capability requirements;
- deterministic execution over immutable check contexts;
- rule-specific findings, related evidence, and root-cause keys;
- API, event, widget, TOC/load, Secret/restriction, overlay/hook, and project consistency diagnostics;
- remediation tier classification;
- shadow/evaluation/default rollout policy metadata;
- per-rule fixtures and false-blocking measurement;
- rule documentation and stable IDs.

## Explicit non-responsibilities

`wow-rules` does not:

- parse or index source;
- mutate Emmy/project/graph/reference state;
- open files, databases, networks, processes, editors, or the WoW client;
- rank general search results;
- infer current profile/build;
- call external repositories or Codebase Memory;
- execute autofixes selected by fuzzy/semantic similarity;
- suppress upstream analyzer diagnostics to hide missing capabilities;
- claim runtime confirmation.

## Provider contract

Each provider declares:

```text
stable rule ID and rule version
default rollout = shadow | advisory | blocking
required capabilities
accepted entity/file scopes
input fact kinds
finding and root-cause kinds
remediation tier(s)
profile/flavor applicability
budget and cancellation behavior
fixture/evaluation set
```

Providers read one immutable check context and append structured findings. They cannot mutate shared state or depend on execution order unless a root-cause pipeline explicitly defines that order.

## Required operations

| Operation | Required behavior |
|---|---|
| `register_rule_descriptor` | Reject duplicate/incompatible IDs and record version/capabilities/rollout. |
| `select_applicable_rules` | Filter by profile, scope, enabled policy, and available fact families. |
| `evaluate_rule_capabilities` | Return runnable or `NotEvaluated` with exact missing/failed partitions. |
| `run_rule` | Execute one deterministic provider under budget/cancellation. |
| `normalize_rule_finding` | Bind source/evidence/generation/coverage/root-cause/remediation metadata. |
| `group_root_cause_keys` | Emit deterministic causal grouping keys; final stream folding is orchestrated by service. |
| `classify_remediation` | Return `exact_edit`, `validated_recipe`, `plan_only`, or `candidate_only`. |
| `build_rule_coverage_report` | Report files/entities checked, skipped scopes, capability gaps, and truncation. |
| `evaluate_rule_corpus` | Produce false-positive/false-blocking/NotEvaluated/coverage metrics. |

## Initial rule families

Planned rule IDs include:

```text
wow.api.exists
wow.api.deprecated
wow.api.arguments
wow.event.exists_payload
wow.widget.method
wow.toc.reachable
wow.load.use_before_load
wow.secret.local_operation
wow.secret.unsafe_log
wow.overlay.direct_blizzard_override
wow.framework.duplicate_registration
```

No rule is implemented merely because it appears in this list. Each activates at the milestone where required facts and fixtures exist.

## E0 rule: `wow.api.exists`

### Purpose

Detect a direct resolved/global/member API reference that is absent from the selected fixture profile when the relevant reference and semantic partitions are complete.

### Required inputs

```text
one selected fixture profile/reference generation
normalized exact reference lookup
resolved source reference/member fact and span
project/analyzer generation
complete/partial capability state
```

### Algorithm contract

1. Ignore unresolved syntax that the generic analyzer already reports as a root cause unless the rule has an exact API candidate.
2. Normalize the API identity according to the reference contract; do not use fuzzy matching.
3. Query the exact active profile.
4. If found, emit no absence finding; optional migration/deprecation work belongs to later rules.
5. If absent under complete relevant coverage, emit one `wow.api.exists` finding.
6. If coverage is partial/failed/unknown, emit `NotEvaluated`, not an error or clean pass.
7. Attach reference lookup evidence/coverage and exact source span.
8. Do not propose a replacement in E0.

### Required E0 cases

- known valid API: clean;
- known absent API: one finding;
- absent under partial reference coverage: `NotEvaluated`;
- unresolved non-API local/global: no duplicate WoW finding;
- profile mismatch: root generation/profile error, not API absence;
- deterministic duplicate reference uses follow documented per-use/per-symbol policy.

## E0 rule: `wow.secret.local_operation`

### Purpose

Detect one direct local operation on a value whose selected profile contract marks it Secret/inaccessible for the fixture case, without pretending to solve full runtime or interprocedural secrecy.

### Required inputs

```text
exact producer API/facet fact
normalized local expression/use facts
control-flow/guard facts supplied by wow-emmy
selected profile/generation
facet and predicate capability coverage
```

### E0 supported pattern

The fixture must select one explicit producer and one direct operation, such as comparison, concatenation, arithmetic, branch use, or unsafe logging. The exact operation is fixed by the golden fixture and documented in the test.

### Algorithm contract

1. Trace only within the current function/local fact slice supported by the analyzer adapter.
2. Bind the producer result to the exact restriction facet from the selected profile.
3. Check whether an approved access guard dominates the operation for that exact value.
4. Emit a finding only for the supported direct operation without a valid dominating guard.
5. Copies/conversions/`pcall`/serialization do not declassify the value.
6. Unknown facet/predicate/control-flow capability returns `NotEvaluated`.
7. Runtime-dependent conditional secrecy not represented by the fixture remains `NotEvaluated` or outside E0 scope.
8. Remediation is `plan_only` unless an exact mechanically proven guard insertion precondition is defined later.

### Required E0 cases

- direct unsafe operation: finding;
- correctly dominating access guard: clean;
- guard after use: finding;
- different value guarded: finding;
- unknown facet or missing control-flow fact: `NotEvaluated`;
- conversion/copy false scrub: finding when supported by fixture;
- ordinary non-secret value: clean;
- no permanent spell whitelist.

## General Secret/restriction rules

All later Secret rules must follow the current external KB security model:

- gate before use;
- distinguish `issecretvalue` from access predicates;
- treat scrubbing as lossy nil substitution, not recovery;
- avoid private/forbidden managed-object side channels;
- keep runtime/build/context-specific state out of permanent static whitelists;
- never fabricate combat/runtime safety.

## Remediation tiers

```text
exact_edit
    exact proven source precondition and deterministic edit

validated_recipe
    structured transformation requiring post-check and possibly runtime smoke test

plan_only
    evidence-backed steps; no automatic mutation

candidate_only
    investigation options without sufficient proof
```

A `Candidate` fact cannot authorize `exact_edit`.

## Rollout policy

- New rule families start `shadow` unless enforcing a proven invariant with established corpus.
- Promotion requires capability/coverage behavior, positive/negative/partial fixtures, deterministic output, false-blocking measurement, and documented remediation.
- Blocking policy is separate from technical severity.
- An upstream analyzer diagnostic change does not silently change WoW rule rollout.

## Required tests

### E0

- all cases listed for the two E0 rules;
- same profile/project generation on every finding;
- deterministic order and root-cause key;
- rule capability gap produces `NotEvaluated`;
- clean file stays clean;
- target path execution assertion.

### Later rules

- profile/flavor matrix;
- false-positive launch corpus;
- duplicate/downstream root-cause behavior;
- exact remediation precondition invalidation after source digest change;
- budget/cancellation;
- rule version partition identity;
- runtime-required case remains explicit.

## Documentation sources

- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/SECRET_VALUES_AND_RESTRICTIONS.md`](../../docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [Current WoW KB agent rules](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md)
- [Current Secret/taint guidance](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_security.md)
- [Current event/callback rules](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_EventPatterns.md)
- [Current hook decision tree](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_HookDecisionTree.md)
- [Current subsystem router](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_SubsystemRouter.md)

## Definition of done

E0 rules are complete when the golden fixture produces one generic analyzer finding, one exact API absence finding, and one direct Secret-local finding under a single coherent generation; clean/partial cases behave correctly; and no rule performs hidden IO, fuzzy replacement, or runtime overclaiming.
