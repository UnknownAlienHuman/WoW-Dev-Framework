# AGENTS.md — `wow-store` E2-D

## Primary ownership

Implement the ProjectStore physical model, generic schema composition, one-writer WAL lifecycle, immutable partition versions, generation membership, inactive build, read-back validation support, current-record CAS, read snapshots, checkpoint, backup, recovery, retention, and GC.

Do not implement project, graph, analyzer, recognizer, rule, search, or service semantics.

## Mandatory reading

Read repository/crate instructions, all E1-A store contracts, the complete E2-D package, E2-C `PUBLICATION_BOUNDARY.md`, E2-A graph persistence/publication contracts, and current KB routing.

## Before code

1. Verify exact implementations and fixture digests for `wow-core`, E1-A `wow-store`, E2-A `wow-graph`, E2-B `wow-recognizers`, and E2-C `wow-project`.
2. Pin the Rust SQLite binding, SQLite library, compile options, platform/filesystem adapter, and effective PRAGMA profile through executable probes.
3. Freeze the physical profile, schema set, operation catalogs, validation catalogs, benchmark corpus, crash vectors, and checksums.
4. Confirm every null in E2-D fixtures required by the freeze gate is populated.
5. State whether a change affects epoch compatibility, generation identity, logical partition identity, runtime-only behavior, or observability only.

## Writer rules

- Exactly one store writer owner in v1.
- Acquire a finite lock; no spin/retry loop.
- Require exact current epoch and base publication IDs.
- Never silently rebase or merge against a newer current generation.
- Build target rows as immutable partition versions.
- Commit target generation as inactive.
- Activate only after exact read-back validation and a second stale-base check.
- Do not expose a target as current before CAS commit.
- No write after partition-version seal or generation completion.

## Reader rules

- Start one SQLite read transaction before reading the current record.
- Bind the returned view to exact epoch, store generation, project generation, graph generation, and snapshot IDs.
- Never switch generations mid-view.
- Use registered prepared reads only.
- No external multi-process reader contract in v1.
- Release the in-process generation lease explicitly.

## Domain boundary

- Schema and operation bundles are repository-owned compile-time inputs from domain owners.
- Store validates IDs, digests, parameters, cardinality, and transaction phase.
- Store never interprets API, event, frame, source, graph, finding, or coverage meaning.
- No raw SQL, DDL, table name, connection, statement, PRAGMA, or row ID escapes.

## Failure discipline

- Old current stays current until successful activation.
- Inactive generation is recoverable or GC-eligible, never current by inference.
- Post-activation corruption/degradation is explicit; rollback is a new validated CAS operation.
- Last-known-good retains original IDs.
- Cancellation creates no background work.
- Missing executable probe or benchmark is `skipped`/blocking, never pass.

## Completion report

```text
repository/ref and E2-D profile
SQLite/binding/platform pins
schema/operation/validation bundle IDs
base and target epoch/store/project/graph/snapshot IDs
partitions reused/materialized/removed
inactive build and read-back validation result
activation CAS result
WAL/checkpoint/read-lease state
crash/recovery/retention/GC tests
logical and physical determinism classifications
skipped probes/benchmarks and unresolved risks
```
