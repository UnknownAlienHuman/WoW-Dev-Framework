# E2-D recovery and idempotency

**Status:** normative crash, retry, quarantine, and explicit recovery contract.

## Recovery principle

Recovery observes and classifies exact durable state before acting. File presence, operation age, or a missing response is not enough to decide whether publication failed or succeeded.

## Recovery inputs

```text
registry identity/schema/integrity
publication/idempotency/generation/receipt records
current pointers and publication-set manifests
generation/staging/quarantine directory inventory
member/checksum/image/logical manifests
object inventory/reference sets
active/stale reader and retention roots
exact selected physical/runtime/recovery profile
```

No arbitrary external directory or database is adopted.

## Recovery point classes

```text
NoDurableOperation
PlannedOnly
StagingObjectsPartial
StagingDatabasePartial
TransactionCommittedInStaging
GenerationDirectoryPublishedUnregistered
SealedInactiveRegistered
StoreValidatedAwaitingDomainAttestations
DomainValidatedAwaitingActivation
ActivationCommittedReceiptAvailable
ActivationCommittedReceiptMissingOrResponseLost
ActivationTransactionNotCommitted
RegistryPointerGenerationMismatch
CorruptOrIdentityMismatchedGeneration
GcInterrupted
CleanupInterrupted
UnknownInconsistentState
```

## Idempotency state machine

```text
Planned
-> Building
-> Sealed
-> StoreValidated
-> DomainValidated
-> Activated

or
Failed | Cancelled | Quarantined | CleanupPending
```

Transitions are monotonic except explicit recovery classification records; durable state is never rewritten to imply an earlier step did not occur.

## Retry rules

### Same operation ID and request digest

- `Planned/Building`: resume only if the profile supports exact validated resumption; otherwise clean/quarantine and deterministically restart under the same logical plan with a new attempt ID.
- `Sealed/StoreValidated/DomainValidated`: continue only after revalidating exact generation/image/manifests.
- `Activated`: return the existing exact receipt; no rebuild or pointer mutation.
- `Failed/Cancelled`: retry only under explicit retry/recovery policy; prior state remains recorded.
- `Quarantined`: never resume as authoritative; rebuild from inputs under a new attempt/operation policy.

### Same operation ID, different digest

Always reject `project_store_idempotency_key_conflict`.

### Same logical target, different operation ID

Classify using existing generation/publication manifests. It may be `NoChange`, exact already sealed, exact already active, or a distinct physical rebuild comparison; never blindly overwrite.

## Staging recovery

### Partial objects

Verify temp/final object bytes. Complete valid content-addressed objects may remain; incomplete temp files are cleanup candidates. No generation authority exists.

### Partial database or uncommitted transaction

Never present as generation. Delete/quarantine according to profile after ensuring no writer/lease uses it.

### Committed database before seal

Reopen under recovery profile, verify transaction/logical state and checkpoint preconditions. Either complete normal seal pipeline or quarantine. Do not mutate logical rows to repair mismatches.

## Published-directory recovery

A generation directory can exist without complete registry state after crash.

Required checks:

```text
safe exact path/layout
all fixed members present and no unexpected sidecars
member/image/checksum/logical manifests valid
SQLite read-only integrity/schema checks
StoreGenerationId and StoreImageId match path/manifests/database
object reference closure
operation/idempotency request linkage
```

If valid, register only as `SealedInactive`/recovery subject. It cannot become current without normal domain attestations and activation CAS.

If invalid, quarantine.

## Activation recovery

### Registry says predecessor current

Target activation did not commit. Target remains sealed inactive; caller may retry exact activation after revalidation/CAS.

### Registry says target current and receipt exists

Return existing receipt.

### Registry says target current but response/receipt finalization uncertain

Reconstruct/validate receipt from committed publication-set/idempotency records under an explicit recovery operation. Do not roll back automatically merely because the caller saw an error.

### Pointer references missing or corrupt target

Integrity incident:

- block new normal leases for target;
- preserve registry evidence;
- create recovery/quarantine record;
- validate predecessor/rollback candidates independently;
- require explicit CAS rollback/republish decision;
- never silently rewrite pointer during ordinary open.

## Registry recovery

- validate SQLite/open/schema/application versions;
- use SQLite-native recovery only under a separately reviewed operational procedure, never as an automatic semantic repair;
- restore from a validated registry backup only with exact generation/publication reconciliation;
- compare every current/pin/lease/reference to generation/object inventory;
- unresolved ambiguity blocks ordinary publication and GC.

A rebuilt registry cannot invent domain attestations or current history from directory mtimes.

## Corruption classes

```text
registry corruption
member missing/unexpected
image checksum mismatch
logical manifest mismatch
SQLite integrity/foreign-key/schema failure
object missing/corrupt
publication pointer closure mismatch
domain attestation mismatch
permissions/path ownership violation
```

Each yields exact scope and recovery class. No in-place change to sealed bytes.

## Quarantine

Quarantine action:

1. create durable recovery record;
2. prevent new leases/activation;
3. close handles;
4. move/copy under exact security profile if safe and atomic, or mark in place blocked;
5. preserve member/manifests/checksums needed for investigation;
6. remove from current only through explicit validated rollback/repair transaction;
7. later delete only through retention/GC policy.

Quarantined state is never a normal query source.

## Last-known-good and rollback

Last-known-good is an explicit retention root over a prior validated publication set. Recovery can propose it as rollback target but cannot:

- relabel it as the failed target;
- combine it with target source/analyzer/graph records;
- activate it without expected-current CAS and integrity validation;
- claim failed target success.

## Recovery cancellation

Recovery itself is explicit, bounded, cancellable, and nonbackground. Cancellation preserves the last durable classified state and returns exact pending actions. It cannot leave current changed unless the activation/rollback transaction already committed, in which case the committed state is reported.

## Recovery budgets

Bound:

```text
registry rows/generations/staging/quarantine entries
members/objects and bytes hashed
SQLite integrity/read queries
time/memory/open handles
recovery records/report output
```

Budget exhaustion yields partial/NotEvaluated recovery and blocks destructive action or activation.

## Crash injection matrix

Inject termination/power-loss simulation:

```text
after idempotency record insert
during each object stream/rename
after database create/schema/write batches
before/after generation transaction commit
during checkpoint/close/checksum/manifest writes
during generation directory rename/fsync
before/after read-only store validation
after each domain attestation record
before each activation registry statement/commit
after activation commit before response
through lease creation/release
through GC mark/delete/finalize
through cleanup/quarantine move
```

For every point assert one classified recoverable state and old-or-new pointer semantics.

## Hard stops

- no state inferred from mtime/age alone;
- no sealed database repair in place;
- no arbitrary directory adoption;
- no response loss treated automatically as activation failure;
- no automatic rollback or pointer rewrite on ordinary open;
- no destructive cleanup before root/lease/idempotency reconciliation;
- no recovery process/network/source execution;
- no fabricated receipt/attestation/domain success.
