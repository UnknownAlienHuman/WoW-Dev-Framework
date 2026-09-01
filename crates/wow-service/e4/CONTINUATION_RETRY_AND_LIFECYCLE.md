# E4-C continuation, retry, idempotency, and resource lifecycle

**Status:** normative.

## Operation classes

```text
pure exact reads
    search_query/search_explain/search_select
    lineage_compare/trace/explain
    migration_candidates/validate
    impact_plan/run/explain

continuations
    search_continue
    impact_continue
    delegated E3-C context continuation

idempotent artifact writes
    search_index_build
    lineage_build
    lineage_review_apply

read-only validations
    search_index_validate
    lineage_validate
    lineage_review_validate
    migration_validate
```

## Resource lifecycle

Every operation has an explicit resource plan:

```text
validate outer request
-> resolve symbolic selectors when allowed
-> acquire exact owner views and leases in canonical order
-> validate cross-owner compatibility
-> acquire exact shard/lineage/artifact views
-> execute owner operation(s)
-> validate owner result
-> admit retention/idempotency receipts when required
-> close/release resources in reverse order
-> finalize canonical public envelope
```

Public success is forbidden before mandatory close/release completes.

## Closure failure

If operation work succeeds but a mandatory close fails:

- do not return `Complete`, `Partial`, `CandidateOnly`, `Truncated`, or `NoChange` as success;
- return structured service failure with the already-created owner artifact/result identity for recovery/audit;
- never rerun the owner operation automatically;
- preserve any durable idempotency receipt;
- do not expose raw lease/store handles.

Best-effort secondary diagnostic cleanup may continue only synchronously inside the operation's bounded cleanup phase; no detached cleanup task.

## Exact read retry

A pure exact request may be retried only with the same exact inputs/profiles. Equivalent retries must yield identical semantic bytes unless an explicitly noncanonical operational report differs.

Owner `busy`/transient behavior uses the frozen finite retry profile. Retry never changes generation, shard, snapshot, proof profile, privacy profile or budget.

## Current-alias retry

When symbolic current changes during stable acquisition:

```text
close entire attempt
-> repeat the complete bounded acquisition protocol
```

If the accepted exact current set differs from a previous completed request, the new request receives new exact selector and result identities. No prior result is relabeled.

## Idempotent build/review operations

Canonical key:

```text
OperationId + CanonicalRequestDigest
```

Durable states:

```text
Planned
AcquiringInputs
ProducingPartitionsOrDocuments
BuildingInactiveArtifact
PublishedInactive
Validating
Validated
Cataloging
Cataloged
RetentionAdmitted
Returning
Completed
Failed
Cancelled
Quarantined
```

Rules:

- same operation ID + same digest resumes or returns exact recorded state;
- same operation ID + different digest is rejected;
- response loss after publication/cataloging returns the recorded exact receipt;
- retry does not create a second shard/snapshot/catalog record;
- prior valid artifacts remain unchanged on failure;
- cancelled/failed intermediate state cannot be reported complete.

## Search continuation

Binds:

```text
exact SearchUniverseSet and shard IDs
normalized query and profiles
immutable result-set manifest
lane cursors/frontiers
last stable rank key
selected/omitted manifests
prior miss/truncation state
cumulative candidate/query/output/time budgets
retention receipt set
```

No current resolution, hidden query broadening, profile change, budget reset or shard rebuild.

## Impact continuation

Binds:

```text
exact LineageGraphSnapshot
exact target GraphSnapshots
exact impact plan/root/profile
visited/frontier/path/result manifests
prior coverage/conflict/omission/truncation state
cumulative traversal/output/time budgets
retention receipt set
```

No lineage recomputation or target graph refresh.

## Search-to-context lifecycle

`search_context` must retain the exact search result/shards and exact owner publication long enough to validate selection and acquire the same exact E3-C context universe. It may share an already-open compatible owner lease only through an explicit owner-port capability; otherwise it reopens the exact retained generation, not current.

The `SearchSelectionReceipt` is finalized before context invocation but the public combined outcome is finalized only after context and all resources close.

## Review lifecycle

A review envelope is validated at use time against its exact authorization profile, key/revocation/expiry state and graph proposal/component state. A previously valid review may become unauthorized before application; application must revalidate.

A successful apply records the exact base snapshot, decision set and new snapshot. It never modifies the review envelope or base snapshot.

## Cancellation

Cancellation is checked during selector collection, acquisition, owner calls, candidate/partition generation, review validation, publication, validation, traversal, retention and rendering/serialization.

On cancellation:

- signal owner operations through typed cancellation;
- stop at owner-defined safe points;
- close all resources synchronously;
- preserve durable incomplete/artifact states under original IDs;
- do not advertise continuation unless its exact retention contract explicitly supports the returned partial state;
- do not continue in background;
- never return complete.

## Retention and GC races

Continuation/result handles require retention admission before publication. Retention ports must close the race between artifact resolution and GC eligibility using the owner contract. Service cannot emulate leases with timestamps or local memory.

## Canonical versus operational data

Canonical result identity may include exact operation/idempotency/retention receipt IDs where semantically required. It excludes wall-clock durations, process IDs, worker IDs, retry sleep, terminal state and host paths.

## Validation

Test every lifecycle stage for:

- owner success then close failure;
- response loss before/after publication, validation, catalog and return;
- same/different idempotency digest;
- current churn and retry exhaustion;
- stale/missing retained shard or lineage snapshot;
- GC admission race;
- cancellation at each phase;
- no double owner invocation or duplicate artifact;
- no public success before closure.
