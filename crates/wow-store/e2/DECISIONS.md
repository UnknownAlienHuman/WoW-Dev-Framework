# E2-D ProjectStore decisions

**Status:** normative.

## PSTORE-001 — One owned database per epoch

A ProjectStore epoch is one owned SQLite database plus its WAL/SHM runtime state. Normal project generations do not create whole database copies.

## PSTORE-002 — WAL is selected for the mutable path

The profile requires WAL only after an executable SQLite/binding/platform probe confirms effective behavior. ReferenceStore remains immutable and does not inherit this policy.

## PSTORE-003 — One writer owner in v1

All generation, activation, checkpoint, retention, recovery, and GC writes serialize through one owner. Cross-process cooperative writers are unsupported.

## PSTORE-004 — Immutable content-addressed partition versions

A logical partition version is inserted once, sealed, and never edited. Equivalent canonical logical payloads reuse the same version after full validation.

## PSTORE-005 — Full generation membership, no recursive delta chain

Each ProjectStore generation stores a complete ordered map from logical partition key to partition-version ID. Reads never walk a parent chain.

## PSTORE-006 — Semantic generations precede storage identity

`ProjectGenerationId`, `GraphGenerationId`, `ProjectSnapshotId`, and `GraphSnapshotId` are derived by domain contracts before store generation identity. Store identity cannot alter domain semantics.

## PSTORE-007 — Noncyclic identity DAG

```text
domain inputs
-> project/graph partition versions and semantic snapshot IDs
-> ProjectPublicationSetId
-> complete membership/object manifests
-> ProjectStoreGenerationId
-> validation report
-> current publication record
```

No object hashes a structure that contains its own ID.

## PSTORE-008 — Two-stage publication

Target generation is committed as `PublishedInactive`, reopened and validated, then activated by a separate compare-and-swap transaction.

## PSTORE-009 — Current record is the atomic coherence boundary

One record names exact epoch, store generation, publication set, project generation/snapshot, graph generation/snapshot, analyzer snapshot, and profile/reference identities. Readers acquire it inside one read transaction.

## PSTORE-010 — Old readers remain stable

A reader keeps one SQLite snapshot and generation lease. Activation does not change its view.

## PSTORE-011 — Project and graph payloads share the same store epoch

Project and graph logical partitions needed for one published project view live in the same SQLite epoch so activation does not require cross-database atomic commit.

## PSTORE-012 — Schema composition remains owner-separated

`wow-store` owns generic metadata/partition/generation records. `wow-project` and `wow-graph` own their logical schemas and operation/validation catalogs.

## PSTORE-013 — No raw SQL public seam

Only registered static prepared operations execute. User/source/MCP/application data never supplies SQL, identifiers, DDL, PRAGMAs, or callbacks.

## PSTORE-014 — Incompatible schema/profile changes create a new epoch

A breaking schema, physical-model, SQLite-profile, or canonicalization change builds and validates a new epoch database. It is not migrated into the active epoch in place by default.

## PSTORE-015 — Exact stale-base rejection twice

The base current record is checked before inactive build planning and again during activation. No silent rebase.

## PSTORE-016 — Validation precedes activation

Store, project, graph, membership, object, stale-removal, cross-generation-leakage, and golden-query validation of the inactive generation must pass before current can reference it.

## PSTORE-017 — Post-activation failure does not rewrite history

A discovered defect marks the current publication degraded/failed through a new record. Rollback, when allowed, is a new CAS activation of an already validated retained generation.

## PSTORE-018 — Checkpoint is operational

WAL frame count, checkpoint timing, and sidecar bytes do not enter logical generation identity. Checkpoint failure does not relabel semantic state.

## PSTORE-019 — Reader leases are process-local in v1

External processes do not acquire supported ProjectStore readers. GC combines in-process leases, current/retention records, operation state, and SQLite activity conservatively.

## PSTORE-020 — Retention is reference-based, not age-only

Current, last-known-good, leased, evidence/debug-pinned, recovery/quarantine, validated-inactive, operation-in-progress, backup, and policy-retained generations cannot be collected.

## PSTORE-021 — GC is generation/partition/object closed

A partition version or object deletes only after no retained generation membership/reference/lease/operation can reach it and domain validation catalogs approve deletion.

## PSTORE-022 — Backup is not source authority

Backup/restore is an operational copy of rebuildable derived state. Restore preserves original identities and passes full validation before activation.

## PSTORE-023 — Logical determinism is mandatory; physical byte identity is classified

Equivalent logical inputs must yield identical semantic IDs, membership, rows, reports, and query results. SQLite file/WAL bytes are reported separately and are not assumed reproducible.

## PSTORE-024 — Benchmarks can reject the selected profile only through contract revision

Implementation may not silently switch to whole-file generations, duplicated rows, recursive deltas, another database, or a server. Failed gates require a reviewed contract change.

## PSTORE-025 — No direct negative authority

Row absence is not project/reference/graph negative authority. Domain coverage and conflict contracts remain required.

## PSTORE-026 — No runtime WoW data

SavedVariables contents, combat/event payloads, secret-capable values, logs, and client state are outside E2-D.

## PSTORE-027 — Operation idempotency is durable and digest-bound

Every mutating operation has an operation ID plus canonical request digest. Same ID/different digest is rejected. Response loss is reconciled from durable operation, validation, activation, and current records.

## PSTORE-028 — Generation-image publication is rejected for interactive indexing

A complete SQLite file per normal project generation is not the selected model because small edits would copy or rebuild whole database images and rely on platform-specific copy/delete behavior. Its useful atomicity and recovery requirements remain incorporated into the WAL model.

## PSTORE-029 — Recovery classifies before acting

Caller errors, process death, WAL presence, file age, and missing responses do not determine outcome. Recovery observes durable state first; ambiguous state blocks activation, cleanup, and GC.

## PSTORE-030 — Continuation is semantic and generation-bound

Pagination/continuation binds exact publication/store generation, query catalog, normalized parameters, ordering version, last semantic key, and integrity digest. Physical row/page/scan position is forbidden.

## PSTORE-031 — Windows sharing violations are operational, not semantic

Open-handle delete/rename failures are classified and retried only after lease/root re-evaluation. They do not justify revoking a valid reader, spinning, or labeling data corrupt.

## PSTORE-032 — Removed alternatives are preserved by history, not current routing

Superseded generation-image documents remain in Git history and PR #13. Current crate routing contains one physical architecture only.
