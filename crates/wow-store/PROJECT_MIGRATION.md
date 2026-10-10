# Cross-epoch ProjectStore migration

**Status:** executable physical v1/v2 to v3 migration, stopping at a validated
inactive target epoch. The scoped checks below pass; full W16/E2 acceptance remains
open.

Migration moves a physical v1 or v2 snapshot into a new physical v3 root and stops
there. The target has no current publication and no root registry, so it stays inert
until a separate cross-epoch activation contract exists. Same-epoch physical
selection is [PROJECT_REGISTRY](PROJECT_REGISTRY.md); restoring a held instance from
an explicit backup is [PROJECT_QUARANTINE_RESTORE](PROJECT_QUARANTINE_RESTORE.md).
The implementation lives in [migration.rs](src/project/migration.rs),
[migration/model.rs](src/project/migration/model.rs) and
[migration/plan.rs](src/project/migration/plan.rs).

## Source and target

The input is a `VerifiedBackup`, never a live store. `MigrationCandidate::stage`
calls `source.verify` and requires the source epoch profile to be
`PHYSICAL_PROFILE` or `RETAINED_PHYSICAL_PROFILE`; `MigrationIntent::validate`
enforces the same source profiles against a `GC_PHYSICAL_PROFILE` target. The
catalog travels unchanged: stage passes the source owner and catalog into
`Database::create_inactive_with_gc`, and the intent requires equal owner and catalog
between the source and target epochs. Because the catalog decides which member
schemas are admitted, an unchanged catalog is what lets source partitions be
re-admitted unchanged.

`create_inactive_with_gc` creates the native v3 database and epoch manifests, leaving the root
without `project-store-registry.json`, and `ensure_unselected` re-checks that
absence at every phase boundary. `open_inactive` is the matching reopen path and
refuses any root that has a selector. `ProjectStore::create_with_gc` publishes a
selector, so it is deliberately not the constructor here.

Payloads travel byte-for-byte. `plan::request` reads every member through the held
source read and reuses the returned `PartitionRecord`, so the bytes are the source's
own canonical bytes and each `PartitionVersionId` is reproduced exactly.

## Write order and interruption

The durable intent precedes partition and publication writes, not every write.
`stage` first creates the target root, its writer lock, epoch directory and SQLite
database, writes both the root and epoch-directory `epoch-manifest.json`, and then
restores the source snapshot under `source-archive`. Only after that does it persist
`migration-intent.json` and prepare publications.

An interruption before the intent exists can leave incomplete root metadata,
an empty target database or a partial source archive on disk. The migration is not
resumable through `MigrationCandidate::open`, which requires the intent file to
exist and to equal a recomputed intent byte for byte. The root is not silently
adopted either: `create_inactive_with_gc` creates its directory and database with
`create_new`, so re-staging onto the same path fails, and `open` refuses any root
that already has a selector. A pre-intent partial root is retained for inspection;
this API cannot resume or delete it.

`source-archive` is operational evidence of what this migration consumed: the
frozen, independently verifiable input from which identity comparisons are derived.
It is not authority for current Blizzard facts or domain semantics, and
revalidating patch-sensitive claims against current source remains a separate
obligation.

## Identity and aliases

Every changed identifier is content-derived, so the new target epoch yields new
generation and validation IDs by construction, while partition versions stay identical
because `PartitionVersionId::derive` consumes only the record profile, key, schema
and payload digest.

`plan::build` walks every source generation and builds one `PublicationRequest` per
generation, recording a `MigrationMapping` of source generation, target generation,
derived operation ID and request digest. Operation IDs derive from the migration
operation and the source generation identity, so aliases are reproducible rather than
caller-chosen.

Several source generations can normalize to one target generation. `plan::build`
deduplicates by target generation and rejects disagreement in request digest for the
same target, and the intent rejects two source generations mapping to one target
under a different operation or digest. The single representative set used by prepare,
owner checks and inventory is `plan::representatives`.

## Target current

No target current publication exists and none is created. `prepare_all` rejects a
non-`None` target current, inventory rejects any current publication and any
publication history. Staged operations are `PublishedInactive`; completed
operations are `ValidatedInactive`, with no activation. No `CurrentRecordId` is
manufactured.

The source current is recorded as evidence instead. `MigrationCurrentMapping` holds
the original source `CurrentPublication` together with the target generation and
`ValidationId` reached through that source generation's mapping, and the intent
requires the source current's generation to appear in the mappings and its epoch to
be the source epoch. That pair is unactivated evidence, not an activation capability.

## Owner validation

Validation is compiled-owner only. `finish` re-reads each supplied generation,
recomputes `ValidationRecord` against the target epoch's catalog checks, and rejects
epoch mismatch, digest mismatch or duplicates, then requires the capability set to
equal the representative target generations exactly. The service driver builds those
checks by replaying every target generation through `AcquiredProjectPair::read` and
`ReadSnapshot::owner_validation` with the graph and publication storage checks.
`ValidatedMigration` is only produced by `finish`, so a serialized record cannot
stand in for owner verdicts.

## Final record preflight

A durable final record is admitted only as canonical complete closure. When
`migration-record.json` exists, inventory requires it to equal the record recomputed
from the freshly captured target state, so no stale, partial or substituted record
passes. The record is reconciled as evidence of a completed target, never as
permission to recreate missing effects or owners.

That preflight is not an owner verdict. A resumed migration still needs new compiled
checks for every target generation: the durable record cannot replace
`owner_validation`, and a resumed run rebuilds and verifies them again before
finishing.

## Reconciliation and failure

`MigrationCandidate::open` reconciles one frozen request. It re-reads the intent,
requires the same operation ID, source snapshot digest and derived source-archive
operation, reopens the source backup, rebuilds the plan and requires equality, then
admits every existing effect before declaring any remaining write. `write_receipt`
reconciles identical bytes and re-syncs; conflicting bytes conflict.

Cancellation and failure leave artifacts in place. There is no blind retry and no
cleanup: progress resumes only through `open` under the identical original request,
and the receipt carries an unknown acknowledgment, so response loss is an unknown
outcome rather than proof of failure.

## Not implemented

Cross-epoch activation, mapped target retention roots, payload upgrade, source
truncation, GC eligibility of migrated data, target-side quarantine, scheduled or
background migration, and selective or partial migration are out of scope.
Interruption inside writes, power loss, old-runtime transformation and broader
platform/deletion faults remain NotEvaluated.

## Verified scope (2026-10-09, Windows)

Four store regressions cover v1 aliases and original history, v2 pins and exact
reopening, canceled/omitted owner checks, corrupt seals, missing target manifests,
substituted final records and foreign Prepared work rejected without logical SQL
mutation. One native service regression checks both Project/Graph pairs, unchanged
semantic IDs, old Current/readers and identical completion after fresh replay.

Repository policy, fmt, workspace check, strict Clippy, tests (906 passed,
1 ignored, 107 targets), strict rustdoc and workspace build passed. The ignored
external-consumer gate, source/runtime gates and full W16/E2 acceptance remain open.
