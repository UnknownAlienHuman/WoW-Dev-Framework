# E2-A implementation plan

**Status:** normative order; implementation not started.

## Phase 0 — prerequisites and freeze

- implement/freeze `wow-core`;
- implement/freeze E1 store foundation and selected ProjectStore physical profile;
- freeze E2 graph registry bundle and examples;
- freeze synthetic producer partitions, snapshot, queries, and checksums;
- update crate manifest/workstream activation before Rust code.

## Phase 1 — registry/value types

Implement bounded typed values and registry validation. No storage.

Tests: `GRAPH-REG-*`, security schema mutations.

## Phase 2 — semantic keys and assertions

Implement scope, EntityKey/RelationKey, assertion IDs, evidence/coverage validation, conflict inputs, canonical ordering.

Tests: `GRAPH-ID-*`, `GRAPH-ASSERT-*`.

## Phase 3 — in-memory partition engine

Implement validation, replacement planning, derived views/conflicts/coverage, and snapshot manifest over synthetic fixtures.

Tests: `GRAPH-PART-*`, determinism.

## Phase 4 — axes and bounded queries

Implement exact entity, neighbors, axes, paths, subgraph, explain, cursor, budgets, cancellation.

Tests: `GRAPH-AXIS-*`, `GRAPH-QUERY-*`, `GRAPH-SEC-*`.

## Phase 5 — logical store schema/operations

Freeze graph logical schema/operation/validation bundle and integrate through `wow-store`. No raw SQL public seam.

Tests: `GRAPH-STORE-*`, crash/failure/cancellation/publication.

## Phase 6 — immutable snapshot publication

Implement one-writer replacement, store generation binding, reopen/golden validation, retention/last-known-good behavior.

## Phase 7 — producer handoff fixtures

Use synthetic normalized project/reference and recognizer-shaped assertions only. Full project/recognizer implementations remain E2-B/E2-C.

## Phase 8 — freeze bytes

Populate all null prerequisite, registry, partition, snapshot, query, store, and SHA-256 values. Tests verify committed fixtures and never rewrite them.

## Deferred

- real TOC/XML/project producer integration;
- named framework recognizers;
- full Blizzard UI graph/source skeletons;
- lineage/impact/search/CBM/runtime;
- service/application operations beyond internal integration fixtures;
- CI.
