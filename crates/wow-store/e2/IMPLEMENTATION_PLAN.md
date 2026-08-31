# E2-D store implementation plan

**Status:** normative order; code not started.

## Phase 0 — prerequisite freeze

- implement/freeze E0-A and E1-A;
- choose/pin SQLite/runtime/library and supported filesystem matrix;
- freeze physical profile, registered bundle interfaces, durability, and error schema;
- freeze all examples/checksums before first Rust commit.

## Phase 1 — profile/root capability

Implement profile validation, owned-root adapter contract, filesystem probes, path/link safety, and internal member IDs.

Tests: `STORE-E2-PROFILE-*`, `STORE-E2-SEC-*`.

## Phase 2 — registered bundles

Implement schema/operation/read/validation registries, payload validation, plan DAG/order, and no-raw-SQL architecture tests.

Tests: `STORE-E2-BUNDLE-*`, `STORE-E2-OP-*`.

## Phase 3 — staging transaction

Implement one writer, staging generation, object staging, registered operation execution, rollback, budgets, cancellation, and transaction validation.

Tests: `STORE-E2-TXN-*`, fault cases through commit.

## Phase 4 — seal/open

Implement checkpoint/close/durability, noncyclic manifests, atomic materialization, checksums, read-only open, and independent golden validation.

Tests: `STORE-E2-SEAL-*`, `STORE-E2-OPEN-*`.

## Phase 5 — registry head and leases

Implement opaque typed head record CAS, exact generation leases, reader consistency, lease limits, and no-fallback exact reads.

Tests: `STORE-E2-HEAD-*`, `STORE-E2-LEASE-*`.

## Phase 6 — recovery

Implement owned inventory classification, abandoned staging handling, sealed inactive revalidation/adoption input, quarantine, and current corruption reporting. No in-place repair.

Tests: `STORE-E2-RECOVERY-*`, crash matrix.

## Phase 7 — retention and GC

Implement roots, mark traversal, stale-plan precondition check, generation sweep, object second-pass reachability, cancellation, and post-GC validation.

Tests: `STORE-E2-GC-*`.

## Phase 8 — integrated fixture

Consume the frozen E2-D `wow-project` publication bundle and E2-A graph logical operations. Prove old-or-new across crash/cancel/CAS conflicts and stable old reader leases.

## Phase 9 — freeze

Populate all implementation/runtime/profile/generation/artifact/head/lease/recovery/GC IDs and SHA-256 values. Tests verify committed bytes and never rewrite fixtures automatically.

## Deferred

- row-versioned single-database ProjectStore;
- remote/multi-host store or consensus;
- online in-place migration/compaction;
- search/FTS physical profile;
- release distribution;
- CI.
