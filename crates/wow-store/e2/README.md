# `wow-store` E2-D ProjectStore and coherent publication contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-store/e2-d/projectstore-coherent-publication`

**Selected physical profile:** `project-store-wal-manifested-partitions-v1`

## Mission

Persist one validated E2-C project index candidate and one E2-A graph publication plan as a coherent, queryable, crash-classified project publication without mixing project, analyzer, recognizer, graph, or store generations.

```text
ProjectIndexCandidate
+ GraphPublicationPlan
+ project/graph schema-operation-validation bundles
+ exact base current publication
+ ProjectStore runtime/physical profile
-> validate identities, schemas, partitions, objects, budgets, and stale base
-> materialize/reuse immutable partition versions
-> write a complete target generation membership map
-> write ProjectSnapshot and GraphSnapshot semantic manifests
-> commit target as PublishedInactive
-> reopen an exact read snapshot and run project/graph/store golden validation
-> atomically compare-and-swap the current publication record
-> expose one coherent read view
```

## Selected model

```text
one owned SQLite file per ProjectStore epoch
WAL mode and explicit effective PRAGMAs
one writer owner/actor
immutable partition versions keyed by logical content identity
full membership map per ProjectStore generation
no recursive parent/delta traversal
target generation committed inactive before activation
activation in a separate CAS transaction after read-back validation
old read transactions remain stable
partition/object reclamation only after retained-generation and lease closure
```

## Why a store epoch exists

`ProjectStoreEpochId` binds the physical model, SQLite runtime profile, schema bundle set, canonicalization contract, and security limits. Normal project updates stay within one epoch. An incompatible schema/runtime/physical-profile change builds a new epoch database and switches an outer registry record only after complete validation. It does not rewrite the active epoch in place.

## Direct dependency

```text
wow-core
```

Cross-component consumers/producers:

```text
wow-project E2-C candidate and publication bundle
wow-graph E2-A graph publication plan and validation catalog
wow-store E1-A schema/object/publication foundation
```

`wow-store` does not import their crates or interpret their records.

## E2-D package

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`PHYSICAL_MODEL.md`](PHYSICAL_MODEL.md)
5. [`SCHEMA_COMPOSITION.md`](SCHEMA_COMPOSITION.md)
6. [`PROJECT_GRAPH_BINDING.md`](PROJECT_GRAPH_BINDING.md)
7. [`PUBLICATION_PROTOCOL.md`](PUBLICATION_PROTOCOL.md)
8. [`READ_SNAPSHOTS_AND_LEASES.md`](READ_SNAPSHOTS_AND_LEASES.md)
9. [`WAL_CHECKPOINT_AND_CONCURRENCY.md`](WAL_CHECKPOINT_AND_CONCURRENCY.md)
10. [`RECOVERY_BACKUP_RETENTION_GC.md`](RECOVERY_BACKUP_RETENTION_GC.md)
11. [`BENCHMARK_AND_PROFILE_FREEZE.md`](BENCHMARK_AND_PROFILE_FREEZE.md)
12. [`SECURITY_AND_BUDGETS.md`](SECURITY_AND_BUDGETS.md)
13. [`ERROR_MODEL.md`](ERROR_MODEL.md)
14. [`TEST_MATRIX.md`](TEST_MATRIX.md)
15. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
16. [`CONTRACT.json`](CONTRACT.json)
17. [`examples/`](examples/README.md)

## Public E2-D store operations

```text
validate_project_store_runtime_profile
validate_project_store_schema_set
create_project_store_epoch
open_project_store_epoch
acquire_project_store_writer
plan_partition_version_materialization
build_inactive_project_store_generation
open_project_store_generation_snapshot
validate_inactive_project_store_generation
activate_project_store_generation
open_current_project_store_snapshot
checkpoint_project_store
backup_project_store
recover_project_store
retain_project_store_generations
garbage_collect_project_store
build_and_activate_project_store_epoch
```

The project-facing orchestration operation remains owned by `wow-project`, not by `wow-store`.

## Completion gate

E2-D code is complete only when the selected profile passes executable SQLite/binding/platform probes and benchmark gates; one-file, TOC, XML, recognizer, and graph-registry updates publish exact coherent generations; inactive generation validation precedes activation; stale base and concurrent writer attempts fail deterministically; old readers retain the old snapshot; every crash/cancel point yields old-current, new-current, or recoverable inactive state; no cross-generation row/query leakage exists; checkpoints cannot invalidate readers; retention and GC preserve every leased/current/last-known-good/evidence-referenced generation and object; logical output is deterministic; and no raw SQL/domain semantics leak through the store seam.
