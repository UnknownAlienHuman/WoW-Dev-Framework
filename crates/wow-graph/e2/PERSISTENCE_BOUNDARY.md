# `wow-graph` / `wow-store` persistence boundary

**Status:** normative logical schema and registered-operation contract; physical ProjectStore model remains owned by `wow-store`.

## Ownership

`wow-graph` owns:

- logical graph records and invariants;
- required indexes/query access paths;
- registered write/read/validation operation definitions;
- partition replacement and snapshot semantics;
- expected logical manifests/counts/digests.

`wow-store` owns:

- SQLite/runtime/WAL/connection/transaction lifecycle;
- schema/migration execution;
- one-writer enforcement;
- read snapshots/leases;
- durability/checkpoint/backup/recovery/GC;
- physical row/object storage.

## Logical record families

```text
graph_registry_manifest
graph_entity_assertion
graph_relation_assertion
graph_partition_manifest
graph_conflict
graph_coverage
graph_snapshot_manifest
graph_generation_current/history metadata
```

Derived/materialized entity/relation indexes may exist for performance only when deterministically rebuildable from assertions and validated against them.

## Required index capabilities

- entity key exact lookup;
- relation source/kind/target and reverse traversal;
- assertion by producer partition;
- evidence/conflict/coverage refs;
- scope/profile/generation filtering;
- axis relation lookup;
- deterministic ordered pagination;
- stale partition deletion/replacement.

Index choice is physical implementation detail; public semantics are not tied to SQL schema names.

## Registered write operations

```text
insert_registry_bundle
replace_graph_partitions
insert_entity_assertions
insert_relation_assertions
replace_conflicts_and_coverage
publish_graph_snapshot_manifest
retain_or_remove_graph_generation
```

Actual plan is phase-ordered and atomic through store. No application-provided SQL.

## Registered reads

```text
open_graph_snapshot
entity_assertions_by_key
relation_assertions_by_key
neighbors_ordered
axis_members_ordered
partition_manifest
conflicts_and_coverage
snapshot_manifest
```

Complex bounded paths may use graph-owned in-memory projection over registered reads. Only the requested neighborhood is loaded.

## ProjectStore activation

E2 chooses the smallest correct physical model after benchmark/fixture validation. This contract does not force row-versioned vs file-per-generation storage. Whatever model is selected must provide:

- exact snapshot binding;
- atomic partition publication;
- stale-base rejection;
- no cross-generation leakage;
- one writer;
- reader stability;
- deterministic logical manifest;
- safe retention/GC.

## Validation

After store publication:

- reopen exact snapshot;
- compare partition/assertion/conflict/coverage manifests;
- run golden exact/neighbor/axis/path queries;
- validate no stale producer assertion;
- validate reverse indexes and endpoint closure;
- reject physical success if logical graph validation fails.

## No raw interfaces

No raw connection, SQL string, table name, PRAGMA, transaction callback, or SQLite row ID crosses the graph public API.
