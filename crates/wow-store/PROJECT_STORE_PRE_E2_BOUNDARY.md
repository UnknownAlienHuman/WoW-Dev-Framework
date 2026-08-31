# Mutable ProjectStore — deferred E2 contract

**Status:** normative future boundary; implementation is explicitly inactive in E1-A.

This file prevents E1 storage work from making choices that would block the later mutable project-generation store. It is not permission to add WAL/project schemas now.

## 1. Purpose

ProjectStore will persist rebuildable project index/graph/generation data while source files and ProjectGeneration domain identity remain owned by `wow-project`.

It differs from ReferenceStore:

```text
ReferenceStore
    immutable, sealed, read-only, one published ReferenceGeneration

ProjectStore
    mutable owned store, one writer, WAL/read snapshots, atomic ProjectStoreGeneration publication
```

## 2. Activation gate

ProjectStore may activate only in E2 after:

- E1-A store foundation works;
- `wow-project` E2 persistent logical schema/record/operation contract exists;
- `wow-graph` typed storage contract exists where required;
- migration/invalidation/publication fixtures are defined;
- workload/latency/size benchmarks justify persistence layout;
- `crates/MANIFEST.json` and dependency graph/workstream status updated.

E1-A code must return typed `operation_not_implemented_for_milestone` for ProjectStore open/write/checkpoint APIs.

## 3. Ownership boundary

`wow-project` owns:

```text
ProjectGenerationId and source truth
workspace/file/TOC/XML/analyzer fact semantics
incremental invalidation plan
which logical partitions belong to a generation
published ProjectSnapshot/View semantics
```

`wow-graph` owns graph entity/relation semantics when activated.

`wow-store` owns:

```text
SQLite connection/profile/WAL lifecycle
schema/migration ledger
one-writer transaction boundaries
physical persistence and read snapshots
atomic store-generation metadata publication
checkpoint/backup/retention/integrity
content objects
```

Store does not decide invalidation or graph meaning.

## 4. One writer

One owner/actor serializes all writes for a ProjectStore.

Rules:

- no second independent writer connection;
- write request references exact base published ProjectStoreGeneration and target ProjectGeneration;
- stale-base write rejected;
- no optimistic silent merge;
- writer applies registered operation plan in one publication transaction/staged partition protocol;
- cancellation/failure does not advance current store generation;
- readers never observe partial target generation.

## 5. WAL profile

Future SQLite runtime profile must pin/probe:

```text
journal_mode=WAL
synchronous policy
busy timeout/locking
wal_autocheckpoint or explicit checkpoint
read snapshot behavior
writer/read concurrency
crash recovery
-WAL/-SHM lifecycle
backup/copy semantics
```

WAL files are owned mutable runtime state, not release artifacts. No WAL setting leaks to ReferenceStore.

## 6. Project store generation

```text
ProjectStoreGeneration
    StoreId / StoreGenerationId
    exact ProjectGenerationId
    base store generation
    schema registry/bundle versions
    applied invalidation/write plan ID/digest
    logical partition manifest/digests
    object reference set
    transaction/publication report
    integrity/coverage state
```

Current pointer advances only after transaction/metadata/validation completes.

## 7. Generation publication model

Possible implementation models to evaluate before E2:

### Versioned rows in one WAL database

- rows keyed by generation/partition;
- atomic current-generation metadata update in transaction;
- readers select exact generation;
- retention/GC deletes old generation rows later.

### Database file per project-store generation

- immutable snapshot-like publication;
- higher write/copy cost;
- simpler historical readers.

### Hybrid base + generation delta

- more complexity; must prove measured need.

No model is accepted here. E2 benchmarks/contracts choose the smallest correct approach. Do not assume a file-per-generation or row-version model prematurely in E1-A code.

## 8. Incremental writes

`wow-project` supplies exact affected partitions and replacement semantics.

Store validates:

- plan/base/target identities;
- every operation in registered catalog;
- complete replacement/deletion/upsert set per partition;
- reference/object accounting;
- transaction budgets;
- no cross-generation row leakage;
- validation checks before current pointer advance.

Store does not compute dependencies/affected partitions.

## 9. Read snapshots

A ProjectStore read view:

- binds exact StoreGenerationId/ProjectGenerationId;
- starts one consistent SQLite read transaction/snapshot;
- does not switch when writer publishes newer generation;
- respects retention/lease;
- exposes registered prepared reads only;
- carries exact schema/runtime profile/context;
- no raw connection/application SQL.

## 10. Last-known-good

If target ProjectGeneration persistence fails:

- previous published store generation remains current/last-known-good;
- failed target retains target ProjectGeneration ID and failure record;
- no relabel/substitution;
- source/analyzer in-memory candidate and persisted current stay distinguishable;
- higher service decides whether/how to expose old snapshot for explicit request.

## 11. Checkpoints

Checkpoint policy must be explicit and benchmarked:

- automatic vs explicit;
- mode and trigger thresholds;
- cancellation/busy behavior;
- size/durability implications;
- reader lease interaction;
- failure does not corrupt/advance generation;
- no unbounded WAL growth.

Checkpoint metrics/timing noncanonical; logical state/digests canonical.

## 12. Backup/rebuild

ProjectStore is rebuildable from source/project generation inputs.

- backup is operational convenience, not platform/source authority;
- corruption recovery may discard/rebuild owned derived store after preserving diagnostics/evidence;
- no mutation of untrusted external DB into owned state;
- backup/restore validates schema/generation/object manifests;
- restore does not relabel generation.

## 13. Retention and GC

Retain:

```text
current published
last-known-good as policy requires
active reader leases
generations requested for comparison/debugging
generations referenced by service/task evidence as configured
```

Delete only after complete row/object/reference/lease accounting. No age-only deletion.

## 14. Required future operations

```text
create_or_open_project_store
validate_project_store_runtime_profile
plan_project_store_update
begin_project_store_write
apply_partition_replacement_plan
publish_project_store_generation
acquire_project_store_read_snapshot
checkpoint_project_store
backup_or_rebuild_project_store
retain_release_project_store_generation
garbage_collect_project_store_generations
```

All return typed deferred state in E1-A.

## 15. Required future tests

- one writer/stale-base rejection;
- read snapshot remains old while new generation publishes;
- write failure/cancel no current advance;
- cross-generation row leakage mutation;
- incremental replacement deletes stale partition rows;
- WAL crash/recovery/checkpoint/reader interactions;
- last-known-good not relabeled;
- exact generation open/retention;
- schema migration with existing derived data;
- corruption rebuild vs backup restore;
- object reference/GC safety;
- workload latency/DB/WAL size benchmark;
- deterministic logical generation manifests.

## 16. E1-A hard stops

- no ProjectStore Cargo module/connection/WAL code;
- no placeholder success/read empty database;
- no assumed physical generation model;
- no project/graph domain tables in standard metadata schema;
- no ReferenceStore WAL policy reuse;
- no filesystem watcher/source invalidation logic;
- no multi-writer connection;
- no old generation substitution/relabel.
