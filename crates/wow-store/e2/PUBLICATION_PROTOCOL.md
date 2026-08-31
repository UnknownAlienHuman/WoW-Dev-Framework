# E2-D ProjectStore publication protocol

**Status:** normative two-stage generation publication and activation protocol for `project-store-wal-manifested-partitions-v1`.

## Inputs

```text
ProjectStorePublicationRequest
    operation ID and canonical request digest
    exact StoreId/EpochId/runtime/physical/schema profiles
    exact base CurrentPublicationRecord or explicit no-base
    ProjectPublicationSet
    project and graph partition materialization plans
    complete target membership manifest
    object write/reference plan
    owner validation and golden-read plan
    publication capability policy
    budgets/cancellation/durability requirement
```

No `force`, floating `current`, silent rebase, raw SQL, or storage callback exists.

## Preflight

Before any domain write:

- validate operation ID/digest and existing durable operation state;
- validate request, IDs, digests, profiles, schemas, catalogs, and budgets;
- validate candidate and graph plan are immutable and mutually coherent;
- verify current record equals expected base;
- verify target does not collide with incompatible existing content;
- calculate new, reused, and removed partition-version sets;
- validate complete target membership and object closure;
- acquire the only writer owner and finite SQLite write lock.

A preflight failure performs no project/graph database mutation. The durable operation record may retain the classified failure.

## Phase A — partition materialization

For each new partition version in deterministic order:

1. validate owner plan and exact key/version/digest;
2. execute registered insert operations in a bounded transaction;
3. validate row counts, logical digests, indexes, and reference closure;
4. mark the partition version sealed in that transaction;
5. retain/reuse an equivalent existing sealed version only after full equivalence validation.

Reused partitions are not rewritten. A sealed but unreferenced partition is inert and later GC-eligible. No partial partition can be marked sealed, and a same-ID/different-content partition is corruption.

After each committed partition, update the operation record without changing semantic target identity.

## Phase B — inactive generation transaction

```text
BEGIN IMMEDIATE
recheck epoch/current/base identities
insert or verify ProjectPublicationSet and semantic manifests
insert complete generation membership
insert exact object references
insert ProjectStoreGeneration state=PublishedInactive
run in-transaction generic and owner invariants
update operation state=PublishedInactive
commit
```

The current record remains unchanged.

On rollback or cancellation, no inactive generation is visible. Previously sealed unreferenced partition versions and verified unreferenced objects may remain inert and become GC candidates.

## Phase C — fresh read-back validation

Open a new exact SQLite read transaction against the target inactive generation, never through current.

Validate:

```text
schema/runtime/epoch/generation manifests
complete membership and partition seal state
partition versions, domain rows, and indexes
project source/TOC/XML/load/analyzer/recognizer records
graph assertions/conflicts/coverage/index closure
object references and payload manifests
removed-input and stale-partition absence
cross-generation leakage sentinels
ProjectSnapshot and GraphSnapshot identities
golden project reads
golden graph exact/neighbor/axis/path queries
capability/coverage/partial state
```

Produce an immutable `InactiveGenerationValidationReport`. Validation is read-only and nonrepairing. A partial, truncated, cancelled, or failed report cannot authorize activation.

## Phase D — activation transaction

```text
BEGIN IMMEDIATE
read current record
require exact expected base digest and IDs
require target state=PublishedInactive or ValidatedInactive
require exact successful validation report
require exact operation ID/request digest/target closure
require target epoch/schema/runtime/profile compatibility
insert publication and activation history
mark target Active and prior current Superseded
compare-and-swap CurrentPublicationRecord
insert activation receipt
update operation state=Activated
commit
```

The CAS predicate is explicit; zero or multiple affected rows fail. Project, graph, analyzer, and store do not have separately advancing current pointers.

## Initial publication

Initial activation explicitly expects no current record. Concurrent initial publishers may build inert targets, but only one exact CAS wins. The other remains inactive/stale and cannot silently rebase.

## Replacement publication

Expected predecessor must equal current during activation. A stale publisher cannot overwrite a newer publication even when its target is otherwise valid.

## Existing-target idempotency

```text
equivalent sealed partition version
    validate and reuse

equivalent PublishedInactive generation
    reopen, revalidate, and continue

equivalent ValidatedInactive generation
    recheck expected current and retry activation

equivalent already-current generation with same operation ID/digest
    return existing activation receipt

same operation ID with different digest
    reject project_store_idempotency_key_conflict

same semantic target under another operation
    classify NoChange/AlreadyCurrent/reusable/stale/collision; never overwrite

same ID with different content
    corruption/collision; quarantine

target built against another base
    no activation
```

## Response loss

If activation commits but the response is lost, retry with the same operation ID and digest. Durable current/history/operation/receipt records determine the outcome. The implementation returns the existing exact receipt and does not rebuild, reactivate, or roll back.

A cancellation or error observed during a noninterruptible commit is reported only after the committed state is observed.

## Partial candidate policy

A `PartialCandidate` may activate only when:

- domain publication policy explicitly permits every incomplete scope;
- store/schema/integrity/identity/membership/security validation is complete;
- project and graph reports retain exact blockers, conflicts, truncation, and `NotEvaluated` state;
- current record and consumer view label the publication partial.

Storage never decides which WoW capability is safe to degrade and never upgrades coverage.

## Crash states

```text
before or during partition transaction
    old current; rollback or inert sealed/unsealed recovery subject

after partition commit, before generation commit
    old current; sealed unreferenced partitions may exist

during inactive generation transaction
    old current; target absent after rollback

after inactive commit, before validation
    old current; recoverable PublishedInactive target

after validation, before activation
    old current; ValidatedInactive target; retry only if base remains exact

during activation transaction
    old or new current atomically

after activation commit, before response
    new current; same-operation retry returns existing receipt
```

## Phase E — post-activation open

Open a normal current read snapshot and verify it resolves exactly to the activated IDs. This is operational confirmation, not permission to rewrite history.

If confirmation fails, report degraded/corrupt current state. Do not silently point back. A reviewed rollback may CAS to a retained validated generation.

## Rollback

Rollback is an explicit activation request naming exact current, exact retained validated target, reason/policy, and validation state. It creates a new activation/history record; it does not mutate or relabel the retained publication set.

## Cancellation

No background continuation. Cancellation before commit rolls back the active transaction. During SQLite commit/atomic calls, observe and classify durable state before returning. Cancellation after activation commit cannot undo committed current state.

## No multi-database commit

All project and graph rows for one publication set live in the same epoch database. Content-addressed objects may be written first because verified unreferenced objects are inert; the database never references an unverified object.
