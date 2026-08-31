# `wow-graph` / `wow-store` E2 persistence boundary

**Status:** normative logical graph boundary, updated for the selected E2-D ProjectStore profile.

## Selected persistence profile

The physical ProjectStore contract is [`../../wow-store/e2/README.md`](../../wow-store/e2/README.md):

```text
one SQLite WAL database per compatible ProjectStore epoch
immutable content-addressed partition versions
complete generation membership maps
PublishedInactive -> exact read-back validation -> CAS activation
```

`wow-graph` remains independent of physical SQLite layout and depends on `wow-store` only through typed registered schema, operation, publication-plan, and snapshot-read seams.

## Graph ownership

`wow-graph` owns:

- versioned graph entity, relation, attribute, and axis registries;
- semantic entity/relation keys;
- immutable entity/relation assertions and producer partitions;
- graph conflicts, coverage, derived views, and required query indexes;
- `GraphGenerationId`, `GraphSnapshotManifest`, and graph capability semantics;
- graph logical schema/operation/validation bundles;
- graph publication planning and exact post-write golden validation.

It does not own SQLite/WAL/transactions, physical row placement, current-pointer mechanics, checkpoints, backups, leases, retention, or GC.

## Store ownership

`wow-store` owns:

- ProjectStore epoch/runtime/physical profile;
- one-writer acquisition and finite busy policy;
- schema composition and migration execution;
- immutable partition-version materialization;
- complete store-generation membership maps;
- inactive generation transaction and state;
- snapshot-bound reads and generation leases;
- CAS activation of the coherent current publication record;
- durability, WAL checkpoint, backup, recovery, retention, and GC.

Store never interprets graph fields or turns database absence into graph authority.

## Graph publication plan

```text
GraphPublicationPlan
    exact graph registry bundle and contract versions
    exact base GraphGeneration/producer partition manifests: optional
    target ProjectGeneration/Profile/Reference context
    ordered producer partition replacement batches
    expected semantic keys/assertions/conflicts/coverage/views/index manifests
    GraphGenerationId and GraphSnapshotManifest candidate
    graph schema/operation/validation bundle IDs
    expected exact/neighbor/axis/path query vectors
    object-reference manifest
    budgets/cancellation
```

The plan contains registered logical operations and typed records only. No raw SQL or storage callback.

## Graph partition versions

The selected store profile may place graph assertion, conflict, coverage, and derived-index rows in immutable partition versions. Their logical keys and payload digests remain graph-owned. Their physical storage, membership, reuse, and reclamation remain store-owned.

A rule/producer update replaces exactly the graph producer partition named by the graph plan. Assertions from other producer partitions remain in target membership. A partial, failed, truncated, or cancelled producer result cannot be materialized as a complete partition.

## Graph identity and store identity

`GraphGenerationId` is derived before store publication from graph semantics and exact producer partition manifests. It does not include `ProjectStoreGenerationId`.

`ProjectStoreGenerationId` later binds the complete ProjectPublicationSet, including the exact GraphGeneration/GraphSnapshot, membership, schema, and objects. This direction prevents identity cycles.

## Validation before activation

A committed inactive target is not graph-current. Through a fresh exact store read snapshot, `wow-graph` validates:

- registry and generation identity;
- assertion, endpoint, evidence, coverage, conflict, and derivation closure;
- no stale producer assertion after replacement;
- reverse/axis/index closure;
- no cross-generation or cross-universe leakage;
- exact entity, neighbor, axis, bounded-path, and explanation golden vectors;
- deterministic counts and logical digests;
- honest partial/NotEvaluated/absence behavior.

Failure blocks activation and remains explicit. Store or project cannot repair a graph failure by dropping assertions, weakening keys, or relabeling coverage.

## Read boundary

A graph view opens only from an exact coherent ProjectStore read snapshot/current publication set. All registered graph reads are scoped by the exact target generation membership map.

Graph public APIs expose graph keys, assertions, evidence, conflicts, coverage, and bounded results. They never expose:

```text
SQLite connection/transaction/cursor
SQL/table/index/PRAGMA names
physical row ID/page/WAL position
mutable partition or current record
store root or private path
```

## Retention and GC

`wow-graph` supplies logical reachability/reference validation for graph records and producer partitions. `wow-store` computes physical generation/partition/object reachability across current, last-known-good, leases, evidence/debug pins, and rollback/recovery policy.

A graph partition version is deleted only when no retained store generation references it and the graph-owned delete/validation catalog confirms closure. No age-only deletion.

## Hard stops

- no graph publication outside the coherent ProjectPublicationSet;
- no final GraphGeneration from E2-C candidate alone;
- no raw SQL or store handle in graph APIs;
- no direct current-pointer update by `wow-graph`;
- no graph query spanning store generations implicitly;
- no path result silently persisted as an edge;
- no candidate/possible evidence upgraded by persistence;
- no physical storage nondeterminism in semantic graph identity.
