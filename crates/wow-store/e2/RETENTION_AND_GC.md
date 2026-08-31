# E2-D retention and garbage collection

**Status:** normative generation/object lifetime contract.

## Principle

Deletion is a proof obligation. A generation or object is removable only when it is absent from the complete explicit root and transitive reference closure for one validated registry epoch and remains absent after the pre-delete recheck.

## Retention root kinds

```text
Current
LastKnownGood
ActiveReaderLease
EvidenceOrSourceHandle
DebugPin
RollbackPin
UserPin
RecoverySubject
QuarantineHold
PublicationOrIdempotencyInProgress
BackupOrExportHold: future/profile-specific
```

Each root names exact publication/generation/image/object IDs and an owner/reason/policy.

## Current root

Every current `PublicationSetId` protects:

```text
publication-set manifest and receipt
StoreGenerationId / StoreImageId and all generation members
ProjectGenerationId / ProjectSnapshotId
GraphGenerationId / GraphSnapshotId
all referenced objects and required manifests/attestations
```

## Last-known-good root

LKG is explicit, not “previous by sequence.” It identifies a prior publication set that passed the configured gate and is retained for fallback/debugging. Replacing current does not automatically delete or relabel LKG.

Policy may retain:

```text
current
one explicit LKG
N explicit rollback points
failed/sealed candidate for bounded diagnosis
```

The exact policy is versioned.

## Reader roots

Active reader leases protect exact publication/generation/image/object closure. A lease expiry/removal only removes that one root; GC still evaluates all other roots.

Stale process leases are classified/reaped through a frozen lease-recovery policy, not by arbitrary wall-clock guess.

## Evidence/source roots

A diagnostic, finding, audit, source map, test corpus, or retained result may need a generation/object to resolve evidence. The owning manifest registers an exact retention root or durable externalized object guarantee.

Never delete a generation merely because it is not current if public evidence handles still reference it.

## Debug/user/rollback pins

Pins are bounded, attributable, listable, and explicitly removable. They cannot contain arbitrary paths or wildcard all future objects. Canonical pin identity references exact immutable subjects.

## Recovery and quarantine roots

Unresolved recovery/quarantine state protects required bytes/manifests until the explicit recovery/retention policy permits deletion. Quarantine does not mean immediately disposable.

## Publication-in-progress roots

Active build/idempotency operations protect their staging directory and supplied/staged objects. On terminal failure/cancel, protection transitions to cleanup/recovery policy.

## Reference graph

The GC planner builds:

```text
roots
-> PublicationSetManifest
-> store generation/image/manifests
-> project/graph snapshot/manifests
-> domain/store attestations/reports required by retention policy
-> object reference set
-> objects
```

Registry operational history can retain receipts/IDs after heavy generation bytes are removed only if its policy/schema represents a tombstoned unavailable payload honestly.

## GC plan

```text
validate registry/store profile and acquire mutation serialization
-> snapshot registry epoch
-> enumerate and validate all roots
-> traverse exact references
-> classify protected subjects
-> enumerate generation/object inventory safely
-> classify candidates, unknowns, corruption, and orphans
-> compute expected counts/bytes
-> emit dry-run ProjectStoreGcPlan
-> validate policy and budgets
```

Unknown or inconsistent references block deletion for affected scope.

## Candidate classes

```text
SupersededUnrootedGeneration
FailedOrCancelledStaging
ValidSealedInactiveUnrooted
OrphanContentAddressedObject
CompletedCleanupTemp
ExpiredOperationalLeaseRecord
QuarantineEligibleForExplicitDeletion
UnknownOrInconsistentHold
```

Age can prioritize candidates after classification but cannot be the sole class criterion.

## Pre-delete recheck

Immediately before destructive work:

- acquire the required registry/store mutation lock;
- verify registry epoch/current pointers unchanged or rebuild plan;
- re-evaluate active leases and pins;
- verify no publication/recovery operation adopted the subject;
- verify candidate generation/image/object IDs and checksums/path safety;
- mark deletion intent durably in registry;
- commit mark before file deletion according to profile.

A stale plan is rejected, not rebased silently.

## Deletion order

Recommended:

1. cleanup operation-local temp files;
2. delete unrooted generation directories/images after handles closed;
3. update generation tombstone/GC completion state;
4. recompute or use validated post-generation object reference closure;
5. delete unreferenced objects;
6. finalize GC report/plan state.

Never delete shared objects before every referencing generation is gone.

## Windows behavior

On Windows a generation may remain undeletable while any process holds it. The profile must:

- use reader leases and closed handles;
- classify sharing violations as retryable operational state, not logical corruption;
- avoid rename/delete loops;
- preserve deletion intent and recheck roots on retry;
- never revoke a valid reader only to satisfy GC.

## Interrupted GC

Durable states:

```text
Planned
Marked
DeletingGenerations
DeletingObjects
ValidatingPostState
Completed
Cancelled
Failed
RecoveryRequired
```

Recovery reconciles registry marks with actual members. Missing already-marked candidate can be accepted only after validating it was not rooted and no unexpected partial directory/reference remains.

## Orphan handling

### Staging orphans

Require no active idempotency/recovery root. Validate safe path and ownership before deletion.

### Generation directory absent from registry

Validate manifests; classify as recoverable sealed generation or corrupt orphan. Do not delete until operation lineage and root closure checked.

### Registry generation missing directory

Integrity incident, not an ordinary GC candidate.

### Object absent from registry references

Hash/path/metadata validate; delete only after complete generation/object-reference scan for the epoch.

### Registry reference to missing object

Integrity incident that blocks normal activation/read and object GC.

## Retention policy changes

A policy change is versioned and audited. Tightening retention triggers a new plan; it cannot retroactively claim prior evidence availability. Broadening retention cannot resurrect deleted bytes and must report unavailable historical subjects.

## Reports

```text
ProjectStoreRetentionReport
    profile/epoch/root counts and digests
    protected publication/generation/image/object sets
    candidate classes/counts/bytes
    unknown/inconsistent holds
    dry-run versus executed differences
    sharing/retry/failure state
    post-GC closure and integrity results
```

Canonical reports exclude absolute paths and wall-clock timing; operational supplements may include safe timing.

## Required tests

- current/LKG/reader/evidence/debug/user/recovery roots;
- one object referenced by multiple generations;
- pointer changes after plan and before mark;
- lease acquired concurrently with GC;
- stale/crashed lease recovery;
- generation deletion then object closure;
- Windows sharing violation;
- crash at every mark/delete/finalize stage;
- orphan generation valid versus corrupt;
- missing object/reference integrity incident;
- policy change;
- age-only deletion mutation fails;
- private path not leaked;
- randomized inventory order gives same plan.

## Hard stops

- no GC during unclassified registry corruption;
- no wildcard/path-based root semantics;
- no object deletion before generation reference closure;
- no deletion from stale plan;
- no current/LKG/lease/evidence/pin removal;
- no age-only proof;
- no silent ignore of sharing violation or missing member;
- no background scheduled GC in E2-D.
