# ProjectStore idempotency, retry, and recovery

**Status:** normative E2-D operation-state contract for the selected WAL/partition-version profile.

## Principle

A missing response, process termination, lock error, or file presence is not proof that an operation failed. Recovery first observes exact durable SQLite and object-store state, classifies it, and only then resumes, retries, quarantines, or reports the result.

No retry may rebuild or activate blindly.

## Operation identity

Every mutating operation uses:

```text
ProjectStoreOperationId
    caller-supplied or orchestrator-derived stable opaque ID

ProjectStoreOperationRequestDigest
    canonical digest of:
        operation kind
        exact epoch/store/base current record
        target publication set and store generation
        partition/object plans
        schema/operation/validation catalogs
        runtime/physical/durability/budget profiles
```

The operation ID itself does not alter semantic generation identity.

## Durable operation record

```text
ProjectStoreOperationRecord
    operation ID
    request digest
    attempt IDs/history
    exact base and target IDs
    state
    committed partition versions
    inactive generation and validation IDs
    activation receipt: optional
    failure/cancellation/recovery records
    canonical digest
```

States are monotonic:

```text
Planned
-> MaterializingPartitions
-> BuildingInactiveGeneration
-> PublishedInactive
-> ValidatingInactive
-> ValidatedInactive
-> Activating
-> Activated

or
Failed | Cancelled | Quarantined | CleanupPending | RecoveryRequired
```

A later recovery record may classify what happened; it does not rewrite prior durable facts.

## Idempotency rules

### Same operation ID and same request digest

- `Planned`: begin or resume according to exact policy.
- `MaterializingPartitions`: validate every existing sealed partition version, continue missing work, and never rewrite sealed content.
- `PublishedInactive`: reopen exact target and resume read-back validation.
- `ValidatedInactive`: recheck the exact expected current record, then attempt activation.
- `Activated`: return the existing exact activation receipt after closure validation.
- `Failed` or `Cancelled`: retry only when the recorded recovery class permits it; preserve the prior attempt.
- `Quarantined`: never resume the quarantined subject as authoritative; rebuild under a new target/attempt policy.

### Same operation ID and different request digest

Always reject `project_store_idempotency_key_conflict`.

### Same logical target and different operation ID

Inspect exact partition, generation, publication-set, validation, and current records. The result may be:

```text
NoChange
ExactPartitionsReusable
ExactInactiveGenerationReusableAfterValidation
AlreadyCurrent
StaleTargetAgainstDifferentBase
IncompatibleCollision
```

It is never permission to overwrite or relabel existing content.

## Response-loss recovery

If activation commits but the caller loses the response:

1. retry with the same operation ID and digest;
2. read the durable operation/current/activation records;
3. verify the current record and receipt bind the exact target;
4. return the existing receipt;
5. do not rebuild, reactivate, roll back, or create another semantic publication.

An error observed by the caller after a noninterruptible SQLite commit is classified only after reading durable state.

## Recovery inventory

Startup/recovery examines, under the single writer/recovery owner:

```text
outer epoch registry and selected epoch
SQLite main database, WAL, and SHM state
schema/runtime/physical profile manifests
current publication record
operation and activation records
Building/PublishedInactive/ValidatedInactive/Failed/Quarantined generations
sealed and unsealed partition versions
complete generation membership maps
object manifests and reference closure
reader leases and retention pins
checkpoint/backup/GC records
```

No arbitrary external database, path, or directory is adopted.

## Recovery classes

```text
NoDurableOperation
PlannedOnly
PartitionTransactionRolledBack
SealedUnreferencedPartitions
InactiveGenerationTransactionRolledBack
PublishedInactiveAwaitingValidation
ValidatedInactiveAwaitingActivation
ActivationNotCommitted
ActivationCommittedReceiptAvailable
ActivationCommittedResponseLost
CurrentRecordOrGenerationMismatch
PartitionOrObjectIntegrityMismatch
WalRecoveryRequired
CheckpointInterrupted
BackupOrRestoreIncomplete
GcInterrupted
UnknownInconsistentState
```

## Recovering partition materialization

- uncommitted partition rows disappear through SQLite rollback/recovery;
- sealed unreferenced partition versions are inert and may be reused only after full identity/row/reference validation;
- a same-ID/different-content partition is corruption and is quarantined logically; it is never overwritten;
- partial rows cannot be marked sealed;
- recovery never edits a sealed partition version in place.

## Recovering inactive generations

A `PublishedInactive` target may resume only when all exact epoch, schema, runtime, physical, base, publication-set, membership, partition, object, and owner validation inputs still match.

If the current record has advanced beyond the expected base, the target remains inactive/stale. Recovery cannot silently rebase or merge it.

## Activation recovery

```text
current still predecessor
    activation did not commit; exact validated target may retry CAS

current is target and receipt exists
    return existing receipt

current is target and caller response/receipt finalization was interrupted
    reconstruct only from committed operation/current/history records

current references missing or inconsistent generation/publication set
    integrity incident; block normal open, record recovery state, require explicit validated action
```

Ordinary open never silently selects last-known-good or rewrites current.

## WAL and checkpoint recovery

Use only the frozen SQLite/binding/platform recovery behavior. Validate effective journal mode, schema, current record, membership, partition/object closure, and logical queries after SQLite recovery.

Do not infer durability beyond the measured profile. A process-kill test is not a power-loss guarantee.

## Quarantine

Quarantine is logical state inside the owned epoch plus protected diagnostic/object references. It:

1. blocks new activation and normal leases for the subject;
2. preserves exact IDs, manifests, reports, and relevant objects;
3. records why the state is not trusted;
4. leaves current unchanged unless an explicit validated rollback CAS is requested;
5. deletes only through retention/GC policy.

No path move is required for correctness; any physical quarantine copy/export is a separate bounded operation.

## Last-known-good and rollback

Last-known-good is an explicit retained publication set, not “the previous row.” Recovery may propose it but cannot:

- relabel it as the failed target;
- mix it with target project/analyzer/graph records;
- activate it without exact integrity validation and expected-current CAS;
- report the failed target successful.

Rollback creates a new activation/history record selecting an existing exact validated set. The set's immutable identity does not change.

## Recovery cancellation and budgets

Recovery is synchronous, bounded, cancellable, and nonbackground. Budgets cover rows/generations/partitions/objects/bytes hashed, SQLite checks, open handles, time, memory, and report output.

Budget exhaustion yields partial/NotEvaluated recovery and blocks activation, cleanup, GC, or destructive repair for the affected scope.

## Required crash injection

Inject termination before/after:

```text
operation record insert
partition transaction begin/commit/seal record
generation membership transaction begin/commit
read-back validation start/report commit
activation CAS statements/commit
response/receipt return
checkpoint start/end
backup snapshot/finalization
lease acquire/release
GC plan/mark/partition delete/object delete/finalization
```

Every point must classify to old current, new current, or a recoverable/quarantined inactive state with no mixed publication.

## Hard stops

- no state inferred from mtime, age, WAL size, or caller error alone;
- no same operation ID with different request digest;
- no sealed partition mutation;
- no response loss treated automatically as activation failure;
- no automatic rollback or last-known-good substitution;
- no cleanup before operation, lease, pin, membership, and object closure reconciliation;
- no unbounded retry/spin;
- no background continuation after cancellation;
- no fabricated receipt, validation, or domain success.
