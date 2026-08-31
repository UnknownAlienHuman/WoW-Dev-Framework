# E2-D ProjectStore decisions

**Status:** normative.

## STORE-E2-001 — File-per-generation is the first ProjectStore profile

One immutable SQLite file plus manifests per store generation is selected for E2-D. It minimizes reader/writer coupling, simplifies rollback and crash classification, and makes mixed-generation exposure structurally harder.

## STORE-E2-002 — Row-versioned single-database storage is deferred

It may be evaluated later, but cannot replace this profile without a new physical-profile version and equivalent crash, migration, reader, retention, and determinism evidence.

## STORE-E2-003 — Store remains domain-neutral

Project and graph crates own logical schemas and invariants. Store validates registered bundles and expected manifests without interpreting domain meaning.

## STORE-E2-004 — One writer and one generation transaction

All project, graph, coverage, conflict, manifest, and object-reference writes for a publication occur in one staging generation transaction.

## STORE-E2-005 — Sealing precedes head publication

A staging database is committed, checkpointed, closed, materialized, checksummed, and reopened read-only before any current head may advance.

## STORE-E2-006 — One coherent head

The registry stores one project publication head that binds store, project, graph, analyzer, recognizer, profile, and reference identities. Separate mutable project and graph current pointers are forbidden.

## STORE-E2-007 — Compare-and-swap prevents stale overwrite

Head update includes an expected prior head/generation. Conflict returns typed failure; no silent rebase or last-write-wins.

## STORE-E2-008 — Logical and physical identity are separate

Logical generation identity excludes SQLite row order, page layout, WAL history, host, time, and temp path. Physical artifact identity records exact sealed bytes/profile/checksums.

## STORE-E2-009 — Published generations are immutable

No in-place migrations, repair, compaction, checksum rewrite, or content mutation.

## STORE-E2-010 — Registered operations only

No raw SQL, table names, PRAGMAs, connection handles, or arbitrary transaction callbacks cross the public API.

## STORE-E2-011 — Staging WAL is not published state

WAL may be used under the selected build profile. A published generation has passed checkpoint/close/seal rules and cannot depend on a mutable staging WAL.

## STORE-E2-012 — Read views bind one lease and generation

A view resolves one head, acquires one generation lease, and never follows head changes mid-operation.

## STORE-E2-013 — Leases are retention roots

Active reader leases prevent generation and referenced-object collection. Lease expiry/reclamation requires explicit heartbeat/owner-death policy, not wall-clock guess alone.

## STORE-E2-014 — Recovery is classification, not blind repair

Staging, sealed inactive, current, leased, corrupt, orphan, and quarantined artifacts are classified from manifests/checksums/registry. Recovery never edits a sealed generation.

## STORE-E2-015 — Sealed inactive adoption requires exact revalidation

A generation published physically but not headed may be adopted only when the exact publication bundle, expected base/head, domain validation reports, and head CAS conditions still hold.

## STORE-E2-016 — Last-known-good keeps original identity

It may remain current or be retained, but cannot be relabeled as a failed target.

## STORE-E2-017 — Retention is root-based

Current, last-known-good, pinned, leased, evidence, recovery, and quarantine roots are marked; all references are traversed before sweep.

## STORE-E2-018 — Object GC follows generation manifests

An object is eligible only when no retained generation/reference/evidence manifest reaches it.

## STORE-E2-019 — Compaction/repacking creates a new artifact

Logical identities may remain equal only when canonical logical manifests are equal; the physical profile/artifact identity changes explicitly.

## STORE-E2-020 — Validation cannot repair

Validation reports exact defects and blocks publication. It never creates rows, rebuilds indexes, updates checksums, or drops unknown records.

## STORE-E2-021 — Durability is profile-bound

Sync, journal, checkpoint, page, locking, timeout, and atomic-replace assumptions are explicit profile inputs and tested on supported filesystems.

## STORE-E2-022 — Cancellation has phase-specific semantics

Before sealing: no published target. After sealed materialization but before head: inactive generation. After successful head CAS: publication is complete and cancellation cannot relabel it as aborted.

## STORE-E2-023 — No age-only deletion

Time is supplemental policy data, not proof that a generation/object is unreachable.

## STORE-E2-024 — No network or remote database

E2-D is local owned-storage persistence. Remote synchronization and multi-host consensus are out of scope.
