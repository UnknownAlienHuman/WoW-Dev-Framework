# E2-C candidate boundary and E2-D coherent publication handoff

**Status:** normative cross-crate boundary, updated for the selected E2-D ProjectStore profile.

## E2-C output remains unchanged

E2-C produces an immutable validated `ProjectIndexCandidate` with `persistent_publication_state = NotPublishedE2C`. Candidate construction remains independent of SQLite layout, WAL state, row IDs, current pointers, checkpoints, backups, retention, and garbage collection.

A candidate contains exact source, root, universe, package, TOC, XML, load, Lua-unit, analyzer, recognizer, graph-proposal, invalidation, coverage, conflict, truncation, and canonical manifest identities. It is not a published project snapshot.

## Selected E2-D store contract

The persistence side is defined by [`../../wow-store/e2/README.md`](../../wow-store/e2/README.md) and uses:

```text
one owned SQLite WAL database per ProjectStore epoch
one writer owner
immutable content-addressed partition versions
complete generation membership maps
PublishedInactive -> read-back validation -> CAS activation
snapshot-bound readers and process-local generation leases
```

The physical profile does not enter E2-C candidate identity.

## Project publication bundle

`wow-project` assembles the domain-facing handoff:

```text
ProjectPublicationBundle
    validated ProjectIndexCandidate ID/digest
    exact expected current ProjectPublicationSet/StoreGeneration: optional
    target ProjectGenerationId and ProjectSnapshot candidate
    exact AnalyzerSnapshot and source/TOC/XML/load/recognizer manifests
    project-owned logical partition replacement plan
    exact GraphPublicationPlan from wow-graph
    graph registry and validated producer proposal partitions
    project/graph schema, operation, and validation bundle IDs
    expected logical counts/digests and golden reads
    object-reference manifest
    publication capability policy
    budgets/cancellation
```

The bundle contains typed logical records and registered operation invocations only. It contains no raw SQL, SQLite table name, connection, transaction callback, PRAGMA, filesystem path, or row ID.

## Ownership

`wow-project` owns:

- ProjectGeneration and ProjectSnapshot semantics;
- source/project/analyzer/recognizer coherence;
- invalidation and logical partition ownership;
- construction and validation of the publication bundle;
- interpretation of project-domain validation results.

`wow-graph` owns:

- GraphGeneration and GraphSnapshot semantics;
- graph registry, assertions, conflicts, coverage, indexes, and graph validation;
- construction of the exact graph publication plan.

`wow-store` owns:

- the ProjectStore epoch and selected physical profile;
- SQLite/WAL/transaction/one-writer behavior;
- partition-version materialization and generation membership;
- inactive generation commit, exact read snapshots, CAS activation, durability, checkpoint, backup, recovery, retention, and GC.

No crate silently implements another owner's semantics.

## Noncyclic identity order

```text
E2-C candidate and project semantic manifest
+ wow-graph GraphGenerationId / GraphSnapshotId
-> ProjectSnapshotId

ProjectSnapshotId
+ GraphSnapshotId
+ AnalyzerSnapshotId
+ project/graph logical partition manifests
-> ProjectPublicationSetId

ProjectPublicationSetId
-> ProjectStoreGenerationId
-> inactive validation report
-> CurrentPublicationRecord
```

`ProjectSnapshotId` is derived before `ProjectPublicationSetId`; the publication set then binds the already-stable project, graph, analyzer, and logical-partition identities. None of `ProjectGenerationId`, `GraphGenerationId`, `ProjectSnapshotId`, `GraphSnapshotId`, or `ProjectPublicationSetId` includes `ProjectStoreGenerationId`. The store generation binds the complete publication set afterward.

## Two-stage publication

### Stage 1 — build inactive

The writer validates the exact base, materializes/reuses immutable partition versions, writes a complete target generation membership map and semantic manifests, runs in-transaction invariants, and commits the target as `PublishedInactive`.

Current remains unchanged.

### Stage 2 — validate and activate

A fresh exact read snapshot opens the inactive target and runs project, graph, store, object, stale-removal, cross-generation-leakage, and golden-query validation. Only a successful report may enter a separate activation transaction that compare-and-swaps the single current publication record against the exact expected base.

If current changed after the inactive build, activation fails as stale. There is no silent rebase or merge.

## Atomic reader contract

A current read resolves one `CurrentPublicationRecord` and acquires one exact SQLite read transaction plus a generation lease. Every project and graph read is scoped through that record and the target generation membership map.

A reader observes either:

```text
old StoreGeneration + old ProjectSnapshot + old GraphSnapshot + old AnalyzerSnapshot
```

or:

```text
new StoreGeneration + new ProjectSnapshot + new GraphSnapshot + new AnalyzerSnapshot
```

Never a mixed set. Existing readers remain on the old SQLite snapshot after activation; new readers observe the new current record.

## Failure and last-known-good

- E2-C candidate failure produces no publication bundle.
- Store build failure or cancellation leaves current unchanged.
- A committed inactive generation is recoverable but not current.
- Failed validation quarantines or retains the inactive target for inspection according to policy.
- CAS failure leaves the validated target inactive and requires exact re-evaluation against current; no implicit activation.
- Current, last-known-good, failed-target, validated-inactive, and rollback-candidate identities remain distinct.
- No old generation is relabeled as the failed target or merged with target inputs.

## E2-D output consumed by `wow-project`

```text
PublishedProjectState
    exact CurrentPublicationRecord
    ProjectStoreReadSnapshot
    ProjectSnapshot/View identity
    GraphSnapshot/View identity
    AnalyzerSnapshot identity
    profile/reference generation
    project/graph/store capability and validation records
```

`wow-project` exposes domain views, not raw storage handles.

## Persistent determinism

E2-C retains responsibility for logical candidate determinism. E2-D separately proves:

- logical partition/version and membership determinism;
- ProjectPublicationSet and StoreGeneration determinism;
- exact project/graph query equivalence;
- physical SQLite/WAL/checkpoint/backup byte classification.

Physical page or WAL differences cannot change semantic IDs or query results.

## E2-D entry gate

No Rust implementation begins until E2-A, E2-B, E2-C, and E1-A implementations and fixtures exist; the exact SQLite binding/runtime/platform adapter is probed; the selected physical profile passes the frozen synthetic and pinned real-addon benchmark corpus; project/graph/store schema-operation-validation bundles exist; and all publication, crash, reader, retention, and checksum vectors are frozen.
