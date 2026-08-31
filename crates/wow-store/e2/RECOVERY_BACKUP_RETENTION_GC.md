# Recovery, backup, retention, and garbage collection

**Status:** normative E2-D operational-state and deletion-proof contract for the selected WAL/partition-version profile.

## Recovery principle

Recovery observes and classifies exact durable state before acting. Caller errors, response loss, process death, WAL presence, object/file age, or a missing in-memory lease are not sufficient outcome evidence.

Detailed operation retry rules are in [`IDEMPOTENCY_AND_RECOVERY.md`](IDEMPOTENCY_AND_RECOVERY.md).

## Startup recovery scan

Under one writer/recovery owner:

1. validate the outer epoch registry;
2. open the selected epoch with the exact recovery runtime profile;
3. inspect main database, WAL, SHM, and SQLite recovery result;
4. validate schema/runtime/physical/epoch manifests;
5. read and validate current publication closure;
6. enumerate durable operation records and building/inactive/validated/failed/quarantined generations;
7. validate partition seals, complete membership, domain rows/indexes, and object/reference closure;
8. reconcile checkpoint, backup, retention, lease, GC, and cleanup records;
9. classify actions without changing current implicitly.

No arbitrary external database, directory, or content object is adopted.

## Recoverable inactive generation

A target may resume validation/activation only when exact epoch, schema, runtime, physical, operation/request digest, publication set, membership, partition, object, owner contract, expected base, and validation inputs still match.

If current advanced, the target remains stale inactive. Recovery does not rebase it.

## Current corruption

- report explicit degraded/corrupt state;
- stop new normal leases according to policy;
- preserve current/operation/validation evidence;
- do not auto-repair rows, recompute IDs, or silently select last-known-good;
- allow explicit rollback to a separately revalidated retained publication through a new CAS transaction;
- rebuild from exact source/profile/tool inputs when trust cannot be restored.

## Quarantine

Quarantine blocks activation and normal reads for an affected generation/partition/object while preserving exact manifests and evidence. Quarantine does not mutate semantic IDs and is not immediate permission to delete. Current changes only through explicit validated activation/rollback.

## Backup

Use the selected SQLite online backup/snapshot API or another executable-probed mechanism that includes committed WAL state. A backup manifest records:

```text
exact epoch and current publication
retained generation set included
schema/runtime/physical profile
main backup payload digest/length
object reference closure
backup operation ID/request digest
validation report
original semantic identities
```

Never copy the main database alone while committed WAL frames may be absent from it. Backup is operational derived-state protection, not source authority.

## Restore

Restore creates a candidate epoch or private recovery path. It validates bytes, SQLite integrity, schemas, current/generations, complete membership, partition/object closure, project/graph golden reads, and original identities before any outer registry activation.

Restore cannot relabel generations, fabricate missing domain attestations, or merge restored and live partitions.

## Retention roots

```text
Current
LastKnownGood
ActiveReaderLease
ValidatedInactiveAwaitingActivation
PublicationOrIdempotencyInProgress
EvidenceOrSourceHandle
DebugPin
RollbackPin
UserPin
RecoverySubject
QuarantineHold
BackupOrExportHold
PolicyRetained
```

Every root is exact, attributable, bounded, listable, and removable through policy. Wildcard future roots and arbitrary path roots are forbidden.

## Root closure

A retained publication protects:

```text
CurrentPublicationRecord / activation history required by policy
ProjectPublicationSet
ProjectStoreGeneration and complete membership
Project/Graph/Analyzer snapshots and required manifests
all referenced partition versions and owner rows/indexes
validation, coverage, conflict, and evidence records required by policy
object reference sets and content objects
operation/recovery records required to interpret the state
```

A generation is not removable merely because it is not current.

## GC planning

```text
acquire writer and lease-admission guard
validate epoch/current/schema/profile
snapshot operation/current/lease/pin/retention roots
traverse complete generation -> partition -> object closure
classify protected, eligible, unknown, corrupt, and orphan subjects
compute deterministic counts/bytes and owner delete requirements
emit a dry-run ProjectStoreGcPlan with base root snapshot digest
release without deletion
```

Unknown or inconsistent state blocks deletion for the affected scope.

## Pre-delete recheck

Immediately before destructive work:

- reacquire writer and lease-admission guard;
- require the exact GC base root/current/operation snapshot digest;
- re-evaluate readers, pins, current, inactive activation, recovery, backup, and operation state;
- verify candidate IDs/digests and owner delete catalogs;
- reject a stale plan rather than silently rebase;
- record deletion intent durably where the profile requires resumable GC.

## Generation GC transaction

Within one bounded writer transaction:

1. remove only explicitly eligible generation membership/publication rows;
2. retain current and every root;
3. compute partition versions unreachable from the remaining complete membership maps;
4. invoke registered owner partition-delete operations for unreachable versions;
5. validate indexes, foreign keys, reference closure, and current invariants;
6. commit;
7. derive content-object deletion candidates from the committed reference state.

Failure/cancellation preserves current and yields an exact recoverable GC state.

## Partition and object GC

A partition version deletes only when no retained membership, evidence/debug/recovery/operation pin, lease, or owner reference reaches it and all owner rows/indexes delete atomically.

An object deletes only after authoritative committed reference accounting, a final root recheck, digest/path validation, and no active object handle. Shared objects require explicit global ownership; path coincidence is not evidence.

## Epoch GC

An old epoch database and its sidecars delete only when:

- it is not selected by the outer registry;
- no reader/backup/recovery/rollback/evidence pin exists;
- no operation can resume against it;
- its generation/partition/object closure is resolved;
- all handles are closed;
- the path is root-confined and exact;
- the epoch deletion plan is revalidated immediately before deletion.

## Windows sharing behavior

Windows may deny deletion/rename of an old epoch, backup, or object while a process holds it. The implementation:

- classifies sharing violation as retryable operational state;
- preserves deletion intent and exact candidate identity;
- closes owned handles;
- rechecks all roots/leases on every retry;
- uses finite retry/backoff policy;
- never spins, force-closes another valid reader, or labels logical data corrupt solely because deletion is blocked.

## Interrupted GC

Durable states:

```text
Planned
Marked
DeletingGenerationMembership
DeletingPartitionVersions
DeletingObjects
ValidatingPostState
Completed
Cancelled
Failed
RecoveryRequired
```

Recovery reconciles committed database state, object inventory, and deletion records before continuing. It never assumes a missing object/file was safely deleted without reference validation.

## Orphans and integrity incidents

```text
sealed unreferenced partition
    validate; reusable or GC candidate

unsealed/partial partition rows after recovery
    must not be visible as sealed; cleanup/quarantine through exact transaction state

unreferenced valid content object
    GC candidate after complete DB reference scan

registry/current reference to missing generation/partition/object
    integrity incident; blocks normal activation/read and destructive GC

unknown external SQLite/object/path
    never adopted
```

## Policy changes

Retention policy changes are versioned and audited. Tightening creates a new GC plan. Broadening cannot resurrect deleted bytes and reports unavailable historical subjects.

## Reports

Canonical reports include exact epoch/profile/root snapshot, protected and eligible generation/partition/object sets, unknown holds, dry-run/executed differences, sharing/retry/failure state, and post-GC closure. They exclude private paths, arbitrary wall-clock data, raw SQL, and source payloads.

## Hard stops

- no age-only deletion;
- no GC during unresolved current/schema/reference corruption;
- no deletion from a stale plan;
- no current/LKG/reader/evidence/recovery/operation root removal;
- no object deletion before committed partition/generation closure;
- no backup by copying main DB without required WAL state;
- no silent last-known-good substitution;
- no unbounded retry or scheduled background GC in E2-D;
- no destructive action after recovery budget exhaustion.
