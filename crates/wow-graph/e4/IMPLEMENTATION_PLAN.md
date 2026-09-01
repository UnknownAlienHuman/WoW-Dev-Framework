# E4-B lineage implementation plan

**Status:** normative implementation order. Rust implementation has not started.

E4-B does not begin merely because this documentation exists. It requires the implemented and frozen generation-local graph/project/reference/search foundation plus exact paired-generation inputs and accepted quantitative gates.

## Phase 0 — prerequisite verification and freeze

Required before the first E4-B Rust commit:

```text
implemented/frozen wow-core
implemented/frozen wow-store E2-D
implemented/frozen wow-graph E2-A
implemented/frozen wow-project E2-C and E3-A
implemented/frozen wow-reference E1-B
implemented/frozen wow-search E4-A
exact project/reference/search producer and read-port catalogs
exact relation/change/migration/impact registries and proof/review profiles
selected and probed lineage-store physical/runtime profile
synthetic and reviewed paired real corpora with ground truth
accepted candidate/authority/change/migration/impact/performance thresholds
all fixture/member/bundle SHA-256 values
```

Missing evidence keeps `implementation_state = not-started`.

## Phase 1 — graph E4 module activation

Extend only `wow-graph` with the minimum E4-B modules. Do not create separate lineage/search/service/application crates by convenience.

Existing direct framework dependencies remain:

```text
wow-core
wow-store
```

Do not add direct dependencies from `wow-graph` to `wow-project`, `wow-reference`, `wow-search` or `wow-service`. Their producer inputs arrive as typed contract records coordinated above the graph crate.

## Phase 2 — profile and exact universe types

Implement:

- `LineageUniverseSet` and exact generation bindings;
- relation/change/migration/impact registries;
- producer/reviewer authority classes;
- proof-ceiling arithmetic;
- capability/coverage/conflict/value-state primitives;
- canonical IDs/order/serialization;
- errors.

Run `LIN-PROF-*`, `LIN-ID-*`, core security and determinism tests.

## Phase 3 — producer input validation

Implement immutable schemas/adapters for:

```text
project_stable_identity
project_source_fingerprint
project_structural_change
reference_explicit_transition
reference_deprecation_or_replacement
search_lineage_candidate
review_decision
```

No producer algorithm is reimplemented inside `wow-graph`. Validate exact scope, evidence, coverage, ceiling and partition ownership.

Run `LIN-PROD-*`.

## Phase 4 — bounded candidate generation

Implement reviewed staged blocking, pair generation, proposal IDs, pair budgets, skipped/truncated records and deterministic bipartite candidate graph.

Run `LIN-CAND-*` and resource/cancellation cases. Prove no unrestricted all-pairs path exists.

## Phase 5 — ambiguity components

Implement connected component partitioning, one-to-one/one-to-many/many-to-one/many-to-many/before-only/after-only shapes, competing assignment retention and explicit copy/split/merge ambiguity.

Run `LIN-AMB-*`. No automatic assignment or promotion.

## Phase 6 — deterministic proof and review engine

Implement:

- relation-specific sufficient-evidence rules;
- effective ceiling calculation;
- review attestation/profile validation;
- accept/reject/defer/conflict/supersede states;
- accepted assertion construction;
- conflict/multiplicity/exclusivity checks;
- complete explanation closure.

Run `LIN-REV-*` and relevant security mutations.

## Phase 7 — typed change and absence classification

Implement value states, exact field/relation comparison, compound ChangeRecords, before-only/after-only records and negative-authority gates.

Run `LIN-CHG-*` and `LIN-ABS-*`.

Removal/introduction code must be developed only after partial/truncation/conflict hard negatives exist.

## Phase 8 — replacement and migration

Implement explicit replacement/deprecation semantics, migration candidates, closed typed recipe steps, preconditions/constraints/postconditions/validation requirements and advisory tiers.

Run `LIN-MIG-*`.

No source edits, arbitrary code transforms or runtime-success claims.

## Phase 9 — logical persistence model

Freeze the exact physical placement/profile after executable benchmark/probe. Implement graph-owned logical schemas and registered store operations for:

- registries/universe sets;
- input/proposal/component/review/assertion/conflict partitions;
- change/absence/migration records;
- manifests/indexes;
- exact query snapshots and retention references.

Use `wow-store` only; no raw SQL API.

Run `LIN-STORE-*` and inactive publication/recovery cases.

## Phase 10 — immutable publication and reads

Implement complete target membership, one-writer inactive build, fresh read-back validation, sealing, exact readers, idempotency, retention/GC closure and golden queries.

Run `LIN-PUB-*`, `LIN-QUERY-*`, logical/physical determinism and corruption/recovery tests.

## Phase 11 — bounded lineage queries

Implement exact comparison, trace and explanation operations with whole-record pagination and exact snapshot-bound continuation.

Candidate/Possible inclusion remains opt-in and labeled. Multi-hop paths remain paths.

## Phase 12 — bounded static impact

Implement exact change-root planning, relation/direction profiles, direct/transitive paths, confidence caps, cross-universe bridge validation, stopping states, continuation and explicit nonclaims.

Run `LIN-IMP-*` on synthetic and real paired corpora.

## Phase 13 — evaluation and anti-overfitting

Run every frozen corpus with:

```text
1/2/N workers
random entity/producer/proposal/review/graph/store order
cold/warm cache
multiple physical store layouts
irrelevant repository/owner/path/popularity mutations
decisive identity/transition/coverage/relation mutations
```

Required reports:

- candidate-pair/component recall and work reduction;
- accepted lineage precision/recall by proof class;
- zero false authority counts;
- copy/move/split/merge ambiguity behavior;
- removal/introduction honesty;
- typed change accuracy;
- migration candidate/recipe boundary;
- impact target/path accuracy and no runtime overclaim;
- latency/CPU/memory/store/pair/fanout budgets;
- security/privacy/license/cancellation/recovery;
- canonical determinism.

Any hard authority/security/determinism failure blocks E4-B regardless of recall.

## Phase 14 — E4-C seam freeze

After E4-B passes, freeze typed service-facing ports for:

```text
exact LineageUniverseSet acquisition
producer partition submission
review decision submission
lineage compare/trace/explain
change/migration/impact operations
retention/continuation lifecycle
canonical result envelopes
```

Do not implement symbolic current resolution or CLI in `wow-graph`.

## Deferred

```text
E4-C service and apps/wow orchestration
E5 named calibration packs
E6 external/Codebase Memory candidates
E7 LSP/MCP/release/publishing
runtime validation and automatic edit application
```

## Completion report

```text
exact prerequisite implementation and fixture pins
activated wow-graph modules/dependencies
relation/profile/store/runtime IDs
producer ports and partition schemas
candidate blocking/component metrics
proof/review/conflict results
change/removal/introduction results
replacement/migration candidate/recipe results
static-impact roots/paths/coverage/budgets
publication/read/retention/recovery results
corpora/evaluation/performance/security/determinism gates
pass/fail/skipped/NotEvaluated commands and evidence
known E4-C/runtime/edit deferrals
```
