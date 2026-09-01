# E4-A miss classification, result sets, pagination, and continuation

**Status:** normative.

## Miss evaluation

Search distinguishes retrieval outcome from entity authority.

```text
ExactFound
ExactNotFoundWithAuthority
ExactNotFoundPartial
NoCandidatesUnderExecutedLanes
CandidateOnly
LaneUnavailable
ConflictBlocked
Truncated
Cancelled
Failed
```

## Exact authoritative miss

`ExactNotFoundWithAuthority` is allowed only when all are true:

1. query class is an exact key/name/alias/member lookup with frozen comparison semantics;
2. every required shard is exact, validated, and bound;
3. relevant owner enumeration and document/field/index coverage are Complete;
4. alias/member target-resolution coverage is Complete when relevant;
5. lane/query execution is Complete and untruncated;
6. no unresolved conflict affects the key/value/scope;
7. owner negative-authority rules permit the conclusion;
8. no privacy filter hides relevant documents;
9. result validation passes.

A text/fuzzy/shape/graph no-hit can never satisfy this by itself.

## Partial and candidate-only outcomes

- `ExactNotFoundPartial`: exact query ran but some decisive coverage/conflict/privacy/index state prevents authority.
- `NoCandidatesUnderExecutedLanes`: approximate lanes completed with no candidates; nonauthoritative.
- `CandidateOnly`: one or more candidates exist, but none is an exact band satisfying the request.
- `LaneUnavailable`: required lane/index/profile missing.
- `ConflictBlocked`: conflict prevents a decisive exact answer.
- `Truncated`: candidate enumeration, ranking, or output stopped at an explicit bound.
- `Cancelled` and `Failed` remain distinct.

## Result-set materialization

Before returning page one, E4-A creates an immutable logical `SearchResultSetManifest` containing:

- exact shards/universe/query/profiles;
- complete ordered retained candidate IDs and rank tuples;
- lane-result manifests;
- candidate-count and coverage state;
- miss/conflict/omission/budget state;
- validation digest.

This can be stored as a bounded content-addressed object through a store seam or carried by a higher layer. Physical storage is not owned by search semantics.

## Candidate cap versus page size

These are different:

```text
candidate enumeration/ranking cap
    limits which candidates enter the result-set manifest

page size
    selects a whole-candidate slice from that immutable manifest
```

A small page does not make the result truncated. A candidate cap does.

## Pagination

Pages contain whole candidates. Mandatory candidate identity, match class, rank tuple, origin/coverage/conflict state, and requested explanation cannot be split or omitted silently.

Stable ordering comes entirely from the result-set manifest.

## Continuation cursor

```text
SearchContinuation
    exact SearchUniverseSet and shard IDs
    normalized query and profile IDs
    SearchResultSetManifestId/digest
    next whole-candidate index or stable key
    page profile
    cumulative request/result/detail/snippet budgets
    prior lane/truncation/omission state
    retention requirements
    integrity digest
```

No mutable current pointer.

## Continuation rules

- reacquire the exact retained shards/result manifest;
- verify all digests/profiles;
- never rerun against current;
- never reset cumulative budgets;
- never change lane, confidence, privacy, ranking, snippet, detail, or page semantics;
- preserve all previous omissions/truncation;
- return no duplicate/missing candidate across pages;
- expired/GCed/corrupt inputs fail exactly;
- a retry with a new query/profile is a new request/result identity.

## Retention admission

A higher service layer must secure retention for every shard and retained result object before advertising continuation. E4-A reports required roots/receipts but does not resolve current or manage service leases.

## Replay option

A profile may allow deterministic replay instead of persisting the ordered candidate manifest only when:

- all exact immutable shards remain retained;
- query/lane/ranking/profile implementations are exact and pinned;
- replay produces the same manifest digest before returning the next page;
- total replay resource limits are bounded;
- any mismatch fails rather than continuing from a different result.

Default contract prefers the explicit result-set manifest.

## No-change and empty page

- Exact retry of the same completed request produces the same result-set ID.
- A continuation past the final candidate returns a typed end state, not a new authoritative miss.
- Empty first page with candidates hidden by page/budget policy is invalid.
- Empty result and exact authoritative miss remain distinct.

## Cancellation and broken continuation

Cancellation during page/detail rendering does not alter the result-set manifest. It returns cancellation/partial output only under an explicit profile and never marks the chain complete.

Tampered, cross-generation, cross-profile, cross-privacy, or cross-request cursors are rejected.

## Determinism

Pages and cursor identities remain stable across workers, storage layout, cache hit/miss, and owner activation after binding.

## Tests

- exact authoritative and partial miss;
- approximate empty result;
- candidate-only;
- required lane unavailable;
- conflict-blocked;
- candidate-cap truncation versus page size;
- 1/N/all pages with no duplicates/gaps;
- exact replay;
- current advances between pages;
- shard/result GC;
- tampered cursor;
- budget reset mutation;
- privacy/profile change mutation;
- cancellation and final-page behavior.
