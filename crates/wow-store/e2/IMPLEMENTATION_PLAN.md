# E2-D implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — prerequisite, consolidation, and freeze

- merge/freeze E0/E1 store foundation;
- merge/freeze E2-A graph, E2-B recognizers, E2-C project candidate;
- verify current routing contains only `project-store-wal-manifested-partitions-v1`;
- prove deleted generation-image documents/fixtures are absent from current normative routes;
- pin SQLite binding/library/platform adapter;
- run runtime/open/WAL/locking/checkpoint/backup/Windows-sharing probes;
- freeze profiles, schemas, catalogs, operation/idempotency model, fixtures, benchmarks, and checksums;
- update implementation state only after every required value exists.

## Phase 1 — epoch/runtime profile

Implement `ProjectStoreId`, `ProjectStoreEpochId`, physical/runtime configuration, outer registry, path/security validation, and open/create profile. No domain tables yet.

## Phase 2 — schema composition

Register and validate store metadata, immutable partition metadata, project, and graph bundles plus operation/validation catalogs. Prove no direct domain dependency or raw SQL seam.

## Phase 3 — durable operation and idempotency records

Implement operation ID/request digest, monotonic operation states, attempt history, target/base binding, same-ID/different-digest rejection, and exact existing-state classification. No project/graph rows yet.

## Phase 4 — immutable partition versions

Implement partition identity, materialization, seal/equivalence reuse, row/manifest/object validation, collision quarantine, and inert unreferenced partition recovery.

## Phase 5 — complete generation membership

Implement full target membership, publication set, store generation identity, object references, and no-delta resolution.

## Phase 6 — inactive publication

Implement one writer, stale-base preflight, partition build, inactive generation transaction, cancellation/failure classification, and operation-state transitions.

## Phase 7 — read snapshots and validation

Implement exact target/current readers, shared lease-admission/GC guard, process-local leases, registered owner reads, semantic continuation, cross-generation sentinels, and project/graph golden validation.

## Phase 8 — activation CAS and response-loss recovery

Implement validated-inactive state, second stale-base check, publication/activation history, current-record CAS, durable receipt, same-operation retry, old/new reader tests, and explicit rollback.

## Phase 9 — WAL/checkpoint and pressure

Implement finite busy policy, effective profile checks, checkpoints, long-reader reporting, WAL ceilings, reader admission behavior, and crash tests.

## Phase 10 — startup recovery, quarantine, and backup

Implement durable-state classification, inactive resume, current corruption handling, response-loss reconciliation, quarantine, online backup, restore-as-candidate-epoch, and rebuild.

## Phase 11 — retention and GC

Implement exact roots/pins/leases/operation holds, dry-run/root snapshot, immediate stale-plan recheck, generation removal, orphan partition detection, owner delete catalogs, object candidates, Windows sharing behavior, epoch GC, interrupted-GC recovery, and cancellation tests.

## Phase 12 — benchmark confirmation

Run all corpora including pinned `UnknownAlienHuman/roth-ui`; verify baseline and incremental thresholds, one-file updates do not copy/rewrite the full database, readers/checkpoints/retention stay bounded, and the selected profile remains accepted. Record baseline/last-known-good without changing the model silently.

## Phase 13 — integration

Implement `wow-project` E2-D publication orchestration using public `wow-graph` and `wow-store` seams. End-to-end fixture:

```text
E2-C candidate
-> graph plan
-> operation record
-> partition materialization/reuse
-> complete membership
-> inactive store generation
-> fresh validation
-> activation CAS
-> response-loss retry
-> exact project/graph reads and semantic continuation
-> one-file update
-> old/new readers
-> rollback/recovery/backup/GC
```

## Deferred

- external multi-process readers/writers;
- distributed/server database;
- runtime WoW/SavedVariables/log persistence;
- E3 context/skeletons;
- E4 search/lineage;
- release automation/CI.
