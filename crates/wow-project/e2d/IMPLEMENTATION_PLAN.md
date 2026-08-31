# E2-D integrated publication implementation plan

**Status:** normative order; code not started.

## Phase 0 — freeze prerequisites

Implement and freeze E0-A, E1-A, E2-A, E2-B, and E2-C. Freeze E2-D store profile and registered bundle contracts. Populate all required fixture pins before Rust code.

## Phase 1 — request/base/coherence types

Implement publication request, base/head resolution, candidate eligibility, coherence tuple, and stale/mixed input rejection.

Tests: `PROJECT-E2D-REQ-*`, `PROJECT-E2D-COH-*`.

## Phase 2 — project logical plan

Implement deterministic project record/write manifest and expected effects from one exact E2-C candidate. No store access.

Tests: `PROJECT-E2D-PLAN-*`.

## Phase 3 — graph plan handoff

Build exact graph replacement request; validate graph plan/proposal mappings/conflicts/coverage without rewriting.

Tests: `PROJECT-E2D-GRAPH-*`.

## Phase 4 — publication bundle

Merge registered project/graph plans, objects, expected manifests, validations, budgets, and cancellation into one canonical bundle.

Tests: `PROJECT-E2D-BUNDLE-*`.

## Phase 5 — store build integration

Invoke E2-D store transaction/seal/open. Validate exact result and classify failure/inactive state.

Tests: `PROJECT-E2D-STORE-*`.

## Phase 6 — post-open validation and snapshots

Open ProjectView/GraphView from exact sealed store, run golden validation, derive coherence/snapshot manifests under the selected noncyclic strategy.

Tests: `PROJECT-E2D-VALID-*`, `PROJECT-E2D-SNAPSHOT-*`.

## Phase 7 — head CAS and published view

Build one head, perform exact CAS, resolve ambiguous result, acquire normal reader lease, and verify coherent consumer open.

Tests: `PROJECT-E2D-HEAD-*`, `PROJECT-E2D-VIEW-*`.

## Phase 8 — recovery/LKG

Implement sealed inactive adoption input, target failure/LKG reporting, current corruption reporting, and no relabel/fallback.

Tests: `PROJECT-E2D-RECOVERY-*`, `PROJECT-E2D-LKG-*`.

## Phase 9 — full fault/determinism matrix

Run process/IO/cancel/disk/store/domain/CAS faults at every phase; 1/2/N/shuffled input deterministic logical output; old reader stability; retention integration.

## Phase 10 — freeze

Populate implementation commits, profiles, candidate/plans/bundle/store/snapshot/head/view/failure/recovery IDs and all SHA-256 values. Tests never rewrite fixtures automatically.

## Deferred

- service/UI/CLI operations;
- E3 context/skeleton/Project Map;
- E4 search/lineage/impact;
- runtime/client validation;
- remote/multi-host publication;
- CI/release automation.
