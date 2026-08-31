# E2-D implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — prerequisite and freeze

- merge/freeze E0/E1 store foundation;
- merge/freeze E2-A graph, E2-B recognizers, E2-C project candidate;
- pin SQLite binding/library/platform adapter;
- run runtime/open/WAL/locking/checkpoint probes;
- freeze profiles, schemas, catalogs, fixtures, benchmarks, and checksums;
- update implementation state only after every required value exists.

## Phase 1 — epoch/runtime profile

Implement `ProjectStoreId`, `ProjectStoreEpochId`, physical/runtime configuration, outer registry, path/security validation, and open/create profile. No domain tables yet.

## Phase 2 — schema composition

Register and validate store metadata, project, and graph bundles plus operation/validation catalogs. Prove no direct domain dependency or raw SQL seam.

## Phase 3 — immutable partition versions

Implement partition identity, materialization, seal/equivalence reuse, row/manifest/object validation, and collision quarantine.

## Phase 4 — complete generation membership

Implement full target membership, publication set, store generation identity, and no-delta resolution.

## Phase 5 — inactive publication

Implement one writer, stale-base preflight, partition build, inactive transaction, cancellation/failure/idempotency.

## Phase 6 — read snapshots and validation

Implement exact target/current readers, process-local leases, owner reads, cross-generation sentinels, and project/graph golden validation.

## Phase 7 — activation CAS

Implement validated-inactive state, second stale-base check, publication history, current-record CAS, old/new reader tests, and explicit rollback.

## Phase 8 — WAL/checkpoint and pressure

Implement finite busy policy, effective profile checks, checkpoints, long-reader reporting, WAL ceilings, and crash tests.

## Phase 9 — recovery/backup

Implement startup classification, inactive resume, current corruption handling, online backup, restore-as-candidate-epoch, and rebuild.

## Phase 10 — retention/GC

Implement pins/leases, generation removal, orphan partition detection, owner delete catalogs, object candidates, epoch GC, and cancellation tests.

## Phase 11 — benchmark confirmation

Run all corpora including pinned `UnknownAlienHuman/roth-ui`, verify selected profile thresholds, and record baseline/last-known-good.

## Phase 12 — integration

Implement `wow-project` E2-D publication orchestration using public `wow-graph` and `wow-store` seams. End-to-end fixture:

```text
E2-C candidate
-> graph plan
-> inactive store generation
-> validation
-> activation
-> exact project/graph reads
-> one-file update
-> old/new readers
-> rollback/recovery/GC
```

## Deferred

- external multi-process readers/writers;
- distributed/server database;
- runtime WoW/SavedVariables/log persistence;
- E3 context/skeletons;
- E4 search/lineage;
- release automation/CI.
