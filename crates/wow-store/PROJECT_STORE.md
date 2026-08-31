# ProjectStore E2-D routing and selected physical model

**Status:** normative E2-D design selection; implementation has not started.

The pre-E2 document intentionally left the physical model open. E2-D now selects the smallest design that satisfies incremental publication, immutable historical readers, graph partition replacement, and bounded storage growth:

```text
profile ID: project-store-wal-manifested-partitions-v1

one SQLite database per ProjectStore epoch
journal mode: WAL after executable profile probe
one writer owner
immutable content-addressed partition versions
complete generation membership maps
two-stage inactive-build then validated activation
snapshot-bound readers
explicit checkpoint, backup, retention, and GC
```

Canonical details are under [`e2/`](e2/README.md). The earlier deferred boundary remains available in Git history at the commit immediately before E2-D.

## Why this model

A database file per project generation makes frequent one-file updates copy or rebuild too much state. Full duplicated generation-keyed rows make storage proportional to all retained generations. Recursive base-plus-delta chains make read cost, corruption recovery, and retention reasoning depend on unbounded ancestry.

The selected model stores each logical partition version once and stores, for every generation, a complete ordered mapping from partition key to immutable partition-version ID. Unchanged partitions are reused without recursive lookup; changed partitions create new immutable versions.

## Publication shape

```text
validated ProjectIndexCandidate
-> wow-graph GraphPublicationPlan
-> project/graph registered logical operation plans
-> ProjectStore generation build transaction
-> committed PublishedInactive generation
-> exact read-back and golden validation
-> current-generation compare-and-swap transaction
-> coherent current ProjectSnapshot + GraphSnapshot + StoreGeneration
```

A reader observes the old publication set or the new publication set, never a mixture.

## Ownership

`wow-project` owns source/project generations, TOC/XML/load/analyzer/recognizer semantics, invalidation, ProjectSnapshot, and publication-bundle construction.

`wow-graph` owns graph registries, semantic keys/assertions, partition replacement, GraphGeneration, GraphSnapshot, conflicts, coverage, and graph validation.

`wow-store` owns SQLite/WAL/transactions, physical partition storage, generation membership, one-writer enforcement, read snapshots, current-record CAS, durability, checkpoint, backup, recovery, retention, and GC.

## Hard stops

- no domain decisions in `wow-store`;
- no raw SQL or connection handle outside `wow-store`;
- no in-place mutation of a sealed partition version or published generation;
- no recursive delta-chain read model;
- no pointer activation before read-back validation;
- no stale-base rebase or last-known-good relabel;
- no external-process reader/writer support in v1 without a new lease/locking contract;
- no schema/runtime-profile upgrade in place when it changes epoch compatibility;
- no benchmark claim before executable implementation;
- no Rust or Cargo activation during this documentation phase.
