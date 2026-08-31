# E2-D ProjectStore publication protocol

**Status:** normative two-stage generation publication and activation protocol.

## Inputs

```text
ProjectStorePublicationRequest
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

## Preflight

Before any write:

- validate request, IDs, digests, profiles, schemas, catalogs, and budgets;
- validate candidate and graph plan are immutable and mutually coherent;
- verify current record equals expected base;
- verify target does not collide with incompatible existing content;
- calculate new, reused, and removed partition-version sets;
- validate complete target membership and object closure;
- acquire the only writer owner and finite SQLite write lock.

Failure here performs no database mutation.

## Phase A — partition materialization

For each new partition version in deterministic order:

1. validate owner plan and exact key/version/digest;
2. execute registered insert operations;
3. validate row counts, digests, and reference closure;
4. mark partition sealed;
5. retain/reuse an equivalent existing sealed version only after full equivalence validation.

Reused partitions are not rewritten.

V1 permits one bounded transaction per partition version followed by a final generation transaction. A sealed but unreferenced partition is inert and GC-eligible; no partial partition can be marked sealed.

## Phase B — inactive generation transaction

```text
BEGIN IMMEDIATE
recheck epoch/current/base identities
insert ProjectPublicationSet and semantic manifests
insert complete generation membership
insert exact object references
insert ProjectStoreGeneration state=PublishedInactive
run in-transaction generic and owner validation checks
commit
```

The current record remains unchanged.

On rollback or cancel, no inactive generation is visible. Previously sealed unreferenced partition versions may remain and are later GC candidates.

## Phase C — read-back validation

Open a fresh exact read transaction against the target inactive generation, not the current pointer.

Validate:

```text
schema/runtime/epoch/generation manifests
complete membership
partition versions and domain rows
project source/TOC/XML/load/analyzer/recognizer records
graph assertions/conflicts/coverage/index closure
object references and payload manifests
cross-generation leakage sentinels
ProjectSnapshot and GraphSnapshot identities
golden project reads
golden graph exact/neighbor/axis/path queries
capability/coverage/partial state
```

Produce `InactiveGenerationValidationReport`. Validation is read-only and nonrepairing.

## Phase D — activation transaction

```text
BEGIN IMMEDIATE
read current record
require exact expected base digest and IDs
require target state=PublishedInactive or ValidatedInactive
require exact successful validation report
require target epoch/schema/runtime/profile compatibility
insert publication/activation history
mark target Active and prior current Superseded
compare-and-swap CurrentPublicationRecord
commit
```

The CAS condition is explicit; zero or multiple updated rows fail.

## Phase E — post-activation open

Open a normal current read snapshot and verify it resolves exactly to the activated IDs. This is an operational confirmation, not permission to rewrite history.

If confirmation fails, report degraded/corrupt current state. Do not silently point back. A reviewed rollback may CAS to a retained validated generation.

## Existing target idempotency

- equivalent sealed partition version: reuse;
- equivalent published-inactive generation: revalidate and continue;
- equivalent already-current generation: explicit `AlreadyCurrent`;
- same ID with different content: corruption/collision, quarantine;
- target built against another base: no activation.

## Crash states

```text
before/during inactive transaction
    old current; transaction rollback

after inactive commit, before validation
    old current; recoverable inactive target

after validation, before activation
    old current; validated inactive target; retry with stale-base check

during activation transaction
    old or new current atomically

after activation commit
    new current; old generation retained by policy
```

## Cancellation

No background continuation. During noninterruptible SQLite commit/atomic calls, observe and classify the exact result before returning.

## No multi-database commit

All project and graph rows for one publication set live in the same epoch database. Content objects may be written first because unreferenced verified objects are inert; the DB never references an unverified object.
