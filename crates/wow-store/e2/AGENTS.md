# AGENTS.md — `wow-store` E2-D

## Primary ownership

Implement the selected ProjectStore physical model, generic schema composition, one-writer WAL lifecycle, immutable partition versions, complete generation membership, durable operation/idempotency state, inactive build, read-back validation support, current-record CAS, read snapshots and semantic continuations, checkpoint, backup, recovery, retention, and GC.

Do not implement project, graph, analyzer, recognizer, rule, search, or service semantics.

## Mandatory reading

Read repository/crate instructions, all E1-A store contracts, the complete E2-D v2 package, E2-C `PUBLICATION_BOUNDARY.md`, E2-A graph persistence/publication contracts, and current KB routing.

The only selected E2-D physical profile is:

```text
project-store-wal-manifested-partitions-v1
```

The whole-SQLite-file-per-generation design from historical PR #13 is rejected current guidance. Read [`REJECTED_ALTERNATIVES.md`](REJECTED_ALTERNATIVES.md); do not recover deleted image-specific documents from Git history as implementation instructions.

## Before code

1. Verify exact implementations and fixture digests for `wow-core`, E1-A `wow-store`, E2-A `wow-graph`, E2-B `wow-recognizers`, and E2-C `wow-project`.
2. Pin the Rust SQLite binding, SQLite library, compile options, platform/filesystem adapter, and effective PRAGMA profile through executable probes.
3. Freeze the physical profile, schema set, operation catalogs, validation catalogs, operation/idempotency model, reader/continuation profile, benchmark corpus, crash vectors, and checksums.
4. Confirm every null in E2-D fixtures required by the freeze gate is populated.
5. State whether a change affects epoch compatibility, generation identity, operation identity, logical partition identity, runtime-only behavior, or observability only.
6. Run architecture routing checks proving no superseded generation-image path is current.

## Writer rules

- Exactly one store writer owner in v1.
- Acquire a finite lock; no spin/retry loop.
- Require exact current epoch and base publication IDs.
- Never silently rebase or merge against a newer current generation.
- Build target rows as immutable partition versions.
- Write a complete target membership map; no recursive generation ancestry.
- Commit target generation as inactive.
- Activate only after exact fresh read-back validation and a second stale-base check.
- Do not expose a target as current before CAS commit.
- No write after partition-version seal or published generation completion.
- Every mutating operation uses a durable operation ID plus canonical request digest.
- Same operation ID/different digest is a hard conflict.
- Reconcile response loss from durable state before retrying anything.

## Reader rules

- Acquire the shared lease-admission/GC guard.
- Start one SQLite read transaction before reading current.
- Register the process-local generation lease before releasing admission.
- Bind the returned view to exact epoch, publication set, store generation, project generation, graph generation, analyzer snapshot, and snapshot IDs.
- Never switch generations mid-view or continuation.
- Use registered prepared reads only.
- Continuations use semantic keys and exact generation/query/parameter/order identity, never row/page/WAL position.
- No external multi-process reader contract in v1.
- Release provisional and active generation leases on all exits.

## Domain boundary

- Schema and operation bundles are repository-owned compile-time inputs from domain owners.
- Store validates IDs, digests, parameters, cardinality, transaction phase, and logical manifests.
- Store never interprets API, event, frame, source, graph, finding, or coverage meaning.
- No raw SQL, DDL, table name, connection, statement, PRAGMA, or row ID escapes.
- Storage row absence never creates domain negative authority.

## Recovery and deletion discipline

- Observe durable state before classifying caller failure, cancellation, process death, or response loss.
- Old current stays current until successful activation.
- Inactive generation is recoverable, stale, quarantined, or GC-eligible; never current by inference.
- Post-activation corruption/degradation is explicit; rollback is a new validated CAS operation.
- Last-known-good retains original IDs.
- GC requires an exact current/root/lease/operation snapshot and immediate pre-delete recheck.
- Unknown closure means no deletion.
- Windows sharing violations are finite retryable operational states, not corruption.
- Cancellation creates no background work.
- Missing executable probe, benchmark, or mutation suite is `skipped`/blocking, never pass.

## Required tests

Run both:

- [`TEST_MATRIX.md`](TEST_MATRIX.md)
- [`IDEMPOTENCY_AND_CONSOLIDATION_TESTS.md`](IDEMPOTENCY_AND_CONSOLIDATION_TESTS.md)

A test implementation that omits response-loss, cursor, lease/GC race, Windows sharing, or architecture-consolidation mutations is incomplete.

## Completion report

```text
repository/ref and E2-D contract revision/profile
SQLite/binding/platform pins
schema/operation/validation bundle IDs
operation ID/request digest and durable operation-state result
base and target epoch/store/publication/project/graph/analyzer/snapshot IDs
partitions reused/materialized/removed
inactive build and fresh read-back validation result
activation CAS and response-loss/idempotency result
reader/continuation/lease-admission state
WAL/checkpoint/Windows sharing state
crash/recovery/backup/retention/GC tests
logical and physical determinism classifications
architecture-consolidation result
skipped probes/benchmarks and unresolved risks
```
