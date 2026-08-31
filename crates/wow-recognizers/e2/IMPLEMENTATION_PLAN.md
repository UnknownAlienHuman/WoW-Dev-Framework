# E2-B implementation plan

**Status:** normative sequence; implementation not started.

## Phase 0 — prerequisites and freeze

Before any Rust commit:

- implement/freeze `wow-core` E0-A;
- implement/freeze exact `wow-emmy` fact schema/pin/probe;
- implement/freeze `wow-graph` E2-A registry/proposal seam;
- freeze future `wow-project` TOC/XML fact adapter profile needed by active fixture rules;
- freeze core pack/rules, graph registry bundle, fact schema profile, fixtures, evaluation thresholds, output partitions, and SHA-256 values;
- update manifest/workstream implementation state.

No code may bind to documentation-only draft type names without an accompanying contract revision.

## Phase 1 — pack/value/schema validation

Implement:

```text
bounded canonical JSON pack parser
pack/rule/clause/capture/output types
fact/graph schema compatibility validation
clause/capture/output DAG validation
resource-bound validation
canonical semantic pack/rule IDs/digests
```

Tests: `RECOG-CONFIG-*`, `RECOG-SCHEMA-*`, pack security mutations.

## Phase 2 — typed fact bundle adapters

Implement recognizer-owned fact envelope and adapters for frozen `wow-emmy` facts. Add synthetic TOC/XML/project adapter fixtures only; real parsing remains E2-C.

Tests: `RECOG-INPUT-*`.

## Phase 3 — matcher core

Implement deterministic selectors, joins, exact predicates, scope predicates, exists/not-exists, ordering/control-flow supplied relations, captures, ambiguity, budgets, cancellation, and canonical match identity.

No graph publication or source parsing.

Tests: `RECOG-MATCH-*`, determinism/security.

## Phase 4 — proposal builder and graph validation seam

Implement proposed entity/relation assertions, explanation/coverage/output partition manifests, and exact `wow-graph` registry/proposal validation calls.

Tests: `RECOG-OUT-*`, initial partition fixtures.

## Phase 5 — active core rule families

Implement in this order:

1. TOC package/file/dependency/LOD/SavedVariables;
2. XML template/object/inheritance/script;
3. CreateFrame/CreateFromMixins/Mixin;
4. native frame events and EventRegistry native bridge;
5. custom TriggerEvent/RegisterCallback producer-consumer;
6. CVar callback;
7. SetScript/HookScript/hooksecurefunc;
8. LibStub/new/embed;
9. SavedVariables literal state paths.

Each rule is merged only with its complete positive/near-negative/partial/dynamic/mutation set.

## Phase 6 — producer partition/replacement model

Implement deterministic `RecognizerOutputPartition`, version compatibility, shadow/default/disable behavior, empty replacement on disable, and integration fixtures proving stale output removal without foreign producer mutation.

Actual graph publication remains caller/graph owned.

Tests: `RECOG-PART-*`, `RECOG-COV-*`.

## Phase 7 — evaluation and mutation harness

Implement corpus manifest, labels, mutations, precision/recall/unknown accounting, graph proposal validation reporting, resource/determinism metrics, and promotion decision against a frozen evaluation profile.

Tests: `RECOG-MUT-*`.

## Phase 8 — E2-C handoff

Document/freeze the caller seam for full `wow-project`:

```text
project parses/publishes exact TOC/XML facts
project assembles RecognizerFactBundle per generation/partition
recognizers return output partitions
project submits proposals to graph
project publishes coherent ProjectSnapshot/GraphSnapshot
```

No reverse dependency or hidden project orchestration in recognizers.

## Phase 9 — byte freeze

Populate all null implementation commits, schema/profile/registry IDs, rule/match/proposal/output/evaluation IDs, expected bytes, and member/bundle SHA-256 values. Tests verify committed fixtures and never rewrite them automatically.

## Deferred

- named calibration packs and actual broad repository corpus activation;
- framework-specific module/lifecycle/plugin/style/element roles;
- Secret guard/sink recognizers and diagnostics;
- complete Blizzard UI graph/source skeletons;
- lineage/impact/search/semantic candidates/CBM/runtime;
- CI or release automation.
